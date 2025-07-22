use std::collections::HashMap;
use std::collections::VecDeque;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::RwLock;

use futures::FutureExt;
use futures::StreamExt;
use futures::stream::FuturesUnordered;

use crate::context::Context;
use crate::task::Input;
use crate::task::Task;

mod dag;
use dag::BuildDagError;
use dag::Dag;
use dag::Edge;
use dag::NodeData;

#[derive(Clone)]
pub struct Engine<I, D> {
    dag: Dag<I>,
    tasks: Arc<HashMap<I, Arc<dyn Task<I, D> + Send + Sync>>>,
}

impl<I, D> Engine<I, D> {
    pub fn new() -> Self {
        Self {
            dag: Dag::new(),
            tasks: Arc::new(HashMap::new()),
        }
    }

    pub fn builder() -> EngineBuilder<I, D> {
        EngineBuilder::new()
    }
}

impl<I, D> Default for Engine<I, D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, I, D> Engine<I, D>
where
    I: Clone + Eq + Hash + Send + 'a,
    D: Clone + Send + Sync + 'a,
{
    pub async fn run(&self, context: Context<'a, I, Option<D>>) {
        let graph = self.dag.graph();
        let mut in_degrees: HashMap<_, _> = graph
            .iter()
            .map(|(node, NodeData { in_neighbors, .. })| (node, in_neighbors.len()))
            .collect();

        let mut queue: VecDeque<_> = in_degrees
            .iter()
            .flat_map(|(&node, &in_degree)| if in_degree > 0 { None } else { Some(node) })
            .collect();

        while let Some(node) = queue.pop_front() {
            if let Some(task) = self.tasks.get(node).cloned() {
                let input = graph[node]
                    .in_neighbors
                    .iter()
                    .flat_map(|in_neighbor| {
                        context
                            .get(in_neighbor)
                            .map(|data| Input::new(in_neighbor.clone(), data))
                    })
                    .collect();

                context.set(
                    node.clone(),
                    async move { task.run(input).await }.boxed().shared(),
                );
            }

            for out_neighbor in &graph[node].out_neighbors {
                let in_degree = in_degrees.get_mut(out_neighbor).unwrap();
                *in_degree -= 1;

                if *in_degree == 0 {
                    queue.push_back(out_neighbor);
                }
            }
        }

        graph
            .iter()
            .flat_map(|(node, _)| context.get(node))
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;
    }
}

pub type BoxTask<I, D> = Box<dyn Task<I, D> + Send + Sync>;

#[derive(Clone)]
pub struct EngineBuilder<I, D> {
    tasks: Arc<RwLock<HashMap<I, BoxTask<I, D>>>>,
}

impl<I, D> EngineBuilder<I, D> {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<I, D> Default for EngineBuilder<I, D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I, D> EngineBuilder<I, D>
where
    I: Eq + Hash,
{
    pub fn exists_task(&self, task: &dyn Task<I, D>) -> bool {
        self.exists_task_by_id(task.id())
    }

    pub fn exists_task_by_id(&self, id: &I) -> bool {
        self.tasks.read().unwrap().contains_key(id)
    }

    pub fn remove_task(&self, task: &dyn Task<I, D>) -> &Self {
        self.remove_task_by_id(task.id())
    }

    pub fn remove_task_by_id(&self, id: &I) -> &Self {
        self.tasks.write().unwrap().remove(id);
        self
    }
}

impl<I, D> EngineBuilder<I, D>
where
    I: Clone + Eq + Hash,
{
    pub fn add_task(&self, task: BoxTask<I, D>) -> &Self {
        self.tasks.write().unwrap().insert(task.id().clone(), task);
        self
    }

    pub fn build(self) -> Result<Engine<I, D>, BuildEngineError> {
        let tasks = Arc::into_inner(self.tasks).unwrap().into_inner().unwrap();
        let mut builder = Dag::builder();

        for id in tasks.keys().cloned() {
            builder.add_node(id);
        }

        for (id, task) in &tasks {
            for dependency in task.dependencies().iter().cloned() {
                builder.add_edge(Edge::new(dependency, id.clone()));
            }
        }

        Ok(Engine {
            dag: builder.build().map_err(EngineErrorKind::DagBuildFailed)?,
            tasks: Arc::new(
                tasks
                    .into_iter()
                    .map(|(id, task)| (id, task.into()))
                    .collect(),
            ),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error(transparent)]
pub struct BuildEngineError(#[from] EngineErrorKind);

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
enum EngineErrorKind {
    #[error("failed to build DAG")]
    DagBuildFailed(#[from] BuildDagError),
}

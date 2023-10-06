use std::collections::HashMap;
use std::collections::VecDeque;
use std::error::Error;
use std::hash::Hash;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Dag<N> {
    graph: Arc<HashMap<N, NodeData<N>>>,
}

impl<N> Dag<N> {
    pub fn new() -> Self {
        Self {
            graph: Arc::new(HashMap::new()),
        }
    }

    pub fn builder() -> DagBuilder<N> {
        DagBuilder::new()
    }
}

impl<N> Default for Dag<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N> Dag<N>
where
    N: Clone + Eq + Hash,
{
    pub fn graph(&self) -> Arc<HashMap<N, NodeData<N>>> {
        self.graph.clone()
    }
}

#[derive(Clone, Debug)]
pub struct NodeData<N> {
    pub in_neighbors: Vec<N>,
    pub out_neighbors: Vec<N>,
}

impl<N> NodeData<N> {
    pub fn new() -> Self {
        Self::from(Vec::new(), Vec::new())
    }

    pub fn from(in_neighbors: Vec<N>, out_neighbors: Vec<N>) -> Self {
        Self {
            in_neighbors,
            out_neighbors,
        }
    }
}

impl<N> Default for NodeData<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct DagBuilder<N> {
    graph: HashMap<N, NodeData<N>>,
}

impl<N> DagBuilder<N> {
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
        }
    }
}

impl<N> Default for DagBuilder<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N> DagBuilder<N>
where
    N: Eq + Hash,
{
    pub fn add_node(&mut self, node: N) -> &mut Self {
        self.graph.entry(node).or_default();
        self
    }
}

impl<N> DagBuilder<N>
where
    N: Clone + Eq + Hash,
{
    pub fn add_edge(&mut self, Edge { from, to }: Edge<N>) -> &mut Self {
        if from == to {
            return self;
        }

        let graph = &mut self.graph;
        if graph
            .get(&from)
            .is_some_and(|NodeData { out_neighbors, .. }| out_neighbors.contains(&to))
        {
            return self;
        }

        if !graph.contains_key(&from) {
            graph.insert(from.clone(), NodeData::new());
        }

        graph.get_mut(&from).unwrap().out_neighbors.push(to.clone());
        graph.entry(to).or_default().in_neighbors.push(from);
        self
    }
}

impl<N> DagBuilder<N>
where
    N: Eq + Hash,
{
    pub fn build(self) -> Result<Dag<N>, Box<dyn Error>> {
        let graph = self.graph;
        let mut in_degrees: HashMap<_, _> = graph
            .iter()
            .map(|(node, NodeData { in_neighbors, .. })| (node, in_neighbors.len()))
            .collect();

        let mut queue: VecDeque<_> = in_degrees
            .iter()
            .flat_map(|(&node, &in_degree)| if in_degree > 0 { None } else { Some(node) })
            .collect();

        while let Some(node) = queue.pop_front() {
            for out_neighbor in &graph[node].out_neighbors {
                let in_degree = in_degrees.get_mut(out_neighbor).unwrap();
                *in_degree -= 1;

                if *in_degree == 0 {
                    queue.push_back(out_neighbor);
                }
            }
        }

        if in_degrees.values().any(|&in_degree| in_degree > 0) {
            Err("cycle detected")?
        }

        Ok(Dag {
            graph: Arc::new(graph),
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Edge<N> {
    pub from: N,
    pub to: N,
}

impl<N> Edge<N> {
    pub fn new(from: N, to: N) -> Self {
        Self { from, to }
    }
}

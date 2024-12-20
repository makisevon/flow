use std::error::Error;

use async_trait::async_trait;
use flow::engine::Engine;
use flow::task::Input;
use flow::task::Task;

fn main() {
    let builder = Engine::builder();
    builder
        .add_task(Box::new(Void::from(1, vec![3])))
        .add_task(Box::new(Void::from(2, vec![1])))
        .add_task(Box::new(Void::from(3, vec![2])));

    let err = unsafe { builder.build().unwrap_err_unchecked() };

    assert_eq!(format!("{err}"), "failed to build DAG");
    assert_eq!(
        format!("{}", err.source().unwrap()),
        "cycle detected in directed graph"
    );
}

struct Void {
    id: usize,
    dependencies: Vec<usize>,
}

impl Void {
    fn from(id: usize, dependencies: Vec<usize>) -> Self {
        Self { id, dependencies }
    }
}

#[async_trait]
impl Task<usize, ()> for Void {
    fn id(&self) -> &usize {
        &self.id
    }

    fn dependencies(&self) -> &[usize] {
        &self.dependencies
    }

    async fn run(&self, _: Vec<Input<'_, usize, ()>>) -> Option<()> {
        None
    }
}

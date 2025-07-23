use std::collections::HashMap;
use std::iter;
use std::time::Duration;
use std::time::Instant;

use dag_flow::context::Context;
use dag_flow::engine::Engine;
use dag_flow::task::Input;
use dag_flow::task::Task;
use futures::StreamExt;
use futures::executor;
use futures::stream::FuturesUnordered;
use futures_timer::Delay;

const NUMBERS: &[u64] = &[1, 2, 3];

fn main() {
    let builder = Engine::builder();
    for &number in NUMBERS {
        builder.add_task(Square::from(number));
    }
    builder.add_task(Sum::from(NUMBERS.into()));

    let engine = builder.build().unwrap();
    let context = Context::new();

    let now = Instant::now();
    executor::block_on(engine.run(context.clone()));

    if let Some(&max) = NUMBERS.iter().max() {
        assert_eq!(now.elapsed().as_secs(), max);
    }

    let ids: Vec<_> = NUMBERS
        .iter()
        .copied()
        .map(Square::id)
        .chain(iter::once(Sum::id()))
        .collect();

    let data: HashMap<_, _> = executor::block_on(
        ids.iter()
            .map(|id| {
                let data = context.get(id).unwrap();
                async move { (id, data.await.unwrap()) }
            })
            .collect::<FuturesUnordered<_>>()
            .collect(),
    );

    for (id, &number) in ids.iter().zip(NUMBERS) {
        assert_eq!(data[id], number.pow(2));
    }
}

struct Square {
    id: String,
    number: u64,
}

impl Square {
    fn id(number: u64) -> String {
        format!("square-{number}")
    }

    fn from(number: u64) -> Self {
        Self {
            id: Self::id(number),
            number,
        }
    }
}

impl Task<String, u64> for Square {
    fn id(&self) -> String {
        self.id.clone()
    }

    async fn run(&self, _: HashMap<String, Input<'_, u64>>) -> Option<u64> {
        Delay::new(Duration::from_secs(self.number)).await;
        Some(self.number.pow(2))
    }
}

struct Sum {
    id: String,
    dependencies: Vec<String>,
    numbers: Vec<u64>,
}

impl Sum {
    fn id() -> String {
        "sum".into()
    }

    fn from(numbers: Vec<u64>) -> Self {
        Self {
            id: Self::id(),
            dependencies: numbers.iter().copied().map(Square::id).collect(),
            numbers,
        }
    }
}

impl Task<String, u64> for Sum {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn dependencies(&self) -> Vec<String> {
        self.dependencies.clone()
    }

    async fn run(&self, inputs: HashMap<String, Input<'_, u64>>) -> Option<u64> {
        self.numbers
            .iter()
            .enumerate()
            .map(|(index, _)| inputs[&self.dependencies[index]].clone())
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .sum()
    }
}

use std::collections::HashMap;
use std::iter;
use std::time::Duration;
use std::time::Instant;

use async_trait::async_trait;
use flow::context::Context;
use flow::engine::Engine;
use flow::task::Input;
use flow::task::Task;
use futures::StreamExt;
use futures::executor;
use futures::stream::FuturesUnordered;
use futures_timer::Delay;

const NUMBERS: &[u64] = &[1, 2, 3];

fn main() {
    let builder = Engine::builder();
    for &number in NUMBERS {
        builder.add_task(Box::new(Square::from(number)));
    }
    builder.add_task(Box::new(Sum::from(NUMBERS.into())));

    let engine = builder.build().unwrap();
    let context = Context::new();

    let now = Instant::now();
    executor::block_on(engine.run(context.clone()));
    assert_eq!(now.elapsed().as_secs(), 3);

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

#[async_trait]
impl Task<String, u64> for Square {
    fn id(&self) -> &String {
        &self.id
    }

    async fn run(&self, _: Vec<Input<'_, String, u64>>) -> Option<u64> {
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

#[async_trait]
impl Task<String, u64> for Sum {
    fn id(&self) -> &String {
        &self.id
    }

    fn dependencies(&self) -> &[String] {
        &self.dependencies
    }

    async fn run(&self, input: Vec<Input<'_, String, u64>>) -> Option<u64> {
        let (index_max, max) = self
            .numbers
            .iter()
            .copied()
            .enumerate()
            .max_by(|(index_x, x), (index_y, y)| (x, index_y).cmp(&(y, index_x)))
            .unwrap_or_default();

        let input: HashMap<_, _> = input
            .into_iter()
            .map(|Input { id, data }| (id, data))
            .collect();
        let dependencies = &self.dependencies;

        self.numbers
            .iter()
            .enumerate()
            .flat_map(|(index, &number)| {
                if index == index_max {
                    return None;
                }

                let data = input[&dependencies[index]].clone();
                Some(async move {
                    Delay::new(Duration::from_secs(max - number)).await;
                    data.await
                })
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .chain(iter::once(input[&dependencies[index_max]].clone().await))
            .sum()
    }
}

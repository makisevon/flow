use std::collections::HashMap;
use std::iter;
use std::time::Duration;
use std::time::Instant;

use async_trait::async_trait;
use flow::context::Context;
use flow::engine::Engine;
use flow::task::Input;
use flow::task::Task;
use futures::executor;
use futures::stream::FuturesOrdered;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use futures_timer::Delay;

const NUMBERS: &[u64] = &[1, 2, 3];

fn main() {
    let builder = Engine::builder();
    for &number in NUMBERS {
        builder.add_task(Box::new(Square::new(number)));
    }
    builder.add_task(Box::new(Sum::from(NUMBERS)));

    let engine = builder.build().unwrap();
    let context = Context::new();

    let now = Instant::now();
    executor::block_on(engine.run(context.clone()));

    println!("elapsed = {:?}", now.elapsed());
    println!(
        "results = {:?}",
        executor::block_on(async {
            NUMBERS
                .iter()
                .map(|&number| Square::id(number))
                .chain(iter::once(Sum::id()))
                .map(|id| async {
                    let data = context.get(&id).unwrap();
                    (id, data.await)
                })
                .collect::<FuturesOrdered<_>>()
                .collect::<Vec<_>>()
                .await
        })
    );
}

struct Square {
    number: u64,
}

impl Square {
    fn id(number: u64) -> String {
        format!("square-{number}")
    }

    fn new(number: u64) -> Self {
        Self { number }
    }
}

#[async_trait]
impl Task<String, u64> for Square {
    fn id(&self) -> String {
        Self::id(self.number)
    }

    async fn run(&self, _: Vec<Input<'_, String, u64>>) -> Option<u64> {
        Delay::new(Duration::from_secs(self.number)).await;
        Some(self.number.pow(2))
    }
}

struct Sum {
    numbers: Vec<u64>,
}

impl Sum {
    fn id() -> String {
        "sum".into()
    }

    fn from(numbers: &[u64]) -> Self {
        Self {
            numbers: numbers.into(),
        }
    }
}

#[async_trait]
impl Task<String, u64> for Sum {
    fn id(&self) -> String {
        Self::id()
    }

    fn dependencies(&self) -> Vec<String> {
        self.numbers.iter().copied().map(Square::id).collect()
    }

    async fn run(&self, input: Vec<Input<'_, String, u64>>) -> Option<u64> {
        let max_number = self.numbers.iter().max().copied().unwrap_or_default();
        let input: HashMap<_, _> = input
            .into_iter()
            .map(|Input { id, data }| (id, data))
            .collect();

        self.numbers
            .iter()
            .flat_map(|&number| {
                if number == max_number {
                    return None;
                }

                let data = input[&Square::id(number)].clone();
                Some(async move {
                    Delay::new(Duration::from_secs(max_number - number)).await;
                    data.await
                })
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .chain(iter::once(input[&Square::id(max_number)].clone().await))
            .sum()
    }
}

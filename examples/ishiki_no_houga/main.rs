use std::collections::HashMap;
use std::time::Instant;

use flow::context::Context;
use flow::engine::Engine;
use futures::executor;
use futures::stream::FuturesUnordered;
use futures::StreamExt;

mod tasks;
use tasks::daikichiyama::Daikichiyama;
use tasks::daikichiyama::Observatory;
use tasks::kousaka_reina::KousakaReina;
use tasks::kousaka_reina::Trumpet;
use tasks::oumae_kumiko::Euphonium;
use tasks::oumae_kumiko::OumaeKumiko;

fn main() {
    let builder = Engine::builder();
    builder
        .add_task(Box::new(OumaeKumiko::new()))
        .add_task(Box::new(KousakaReina::new()))
        .add_task(Box::new(Daikichiyama::new()));

    let engine = builder.build().unwrap();
    let context = Context::new();

    let now = Instant::now();
    executor::block_on(engine.run(context.clone()));
    println!("elapsed = {:?}", now.elapsed());

    let oumae_kumiko = OumaeKumiko::id();
    let kousaka_reina = KousakaReina::id();
    let daikichiyama = Daikichiyama::id();

    let data: HashMap<_, _> = executor::block_on(
        vec![&oumae_kumiko, &kousaka_reina, &daikichiyama]
            .into_iter()
            .map(|id| {
                let data = context.get(id).unwrap();
                async move { (id, data.await.unwrap()) }
            })
            .collect::<FuturesUnordered<_>>()
            .collect(),
    );

    println!(
        "{oumae_kumiko} = {:?}",
        data[&oumae_kumiko].clone().downcast::<Euphonium>().unwrap()
    );

    println!(
        "{kousaka_reina} = {:?}",
        data[&kousaka_reina].clone().downcast::<Trumpet>().unwrap()
    );

    println!(
        "{daikichiyama} = {:?}",
        data[&daikichiyama]
            .clone()
            .downcast::<Observatory>()
            .unwrap()
    );
}

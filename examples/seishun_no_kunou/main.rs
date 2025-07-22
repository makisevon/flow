use std::collections::HashMap;
use std::time::Instant;

use flow::context::Context;
use flow::engine::Engine;
use futures::StreamExt;
use futures::executor;
use futures::stream::FuturesUnordered;

mod tasks;
use tasks::oumae_kumiko::OumaeKumiko;
use tasks::uji_bashi::UjiBashi;

fn main() {
    let builder = Engine::builder();
    builder
        .add_task(Box::new(OumaeKumiko::new()))
        .add_task(Box::new(UjiBashi::new()));

    let engine = builder.build().unwrap();
    let context = Context::new();

    let now = Instant::now();
    executor::block_on(engine.run(context.clone()));
    assert_eq!(now.elapsed().as_secs(), 1);

    let oumae_kumiko = OumaeKumiko::id();
    let uji_bashi = UjiBashi::id();

    let data: HashMap<_, _> = executor::block_on(
        vec![&oumae_kumiko, &uji_bashi]
            .into_iter()
            .map(|id| {
                let data = context.get(id).unwrap();
                async move { (id, data.await.unwrap()) }
            })
            .collect::<FuturesUnordered<_>>()
            .collect(),
    );

    assert_eq!(
        format!("{}", data[&oumae_kumiko].clone().oumae_kumiko().unwrap()),
        "Umaku Naritai"
    );

    assert_eq!(
        format!("{}", data[&uji_bashi].clone().uji_bashi().unwrap()),
        "Jigoku no Orphee"
    );
}

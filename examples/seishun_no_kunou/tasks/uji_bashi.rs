use std::fmt;
use std::time::Duration;

use async_trait::async_trait;
use flow::task::Input;
use flow::task::Task;
use futures_timer::Delay;
use std::collections::HashMap;

use super::data::Data;
use super::oumae_kumiko::OumaeKumiko;

pub struct UjiBashi {
    id: String,
    dependencies: Vec<String>,
}

impl UjiBashi {
    pub fn id() -> String {
        "uji-bashi".into()
    }

    pub fn new() -> Self {
        Self {
            id: Self::id(),
            dependencies: vec![OumaeKumiko::id()],
        }
    }
}

#[async_trait]
impl Task<String, Data> for UjiBashi {
    fn id(&self) -> &String {
        &self.id
    }

    fn dependencies(&self) -> &[String] {
        &self.dependencies
    }

    async fn run(&self, input: Vec<Input<'_, String, Data>>) -> Option<Data> {
        Delay::new(Duration::from_secs(1)).await;

        let input: HashMap<_, _> = input
            .into_iter()
            .map(|Input { id, data }| (id, data))
            .collect();

        let _run = input[&OumaeKumiko::id()]
            .clone()
            .await
            .unwrap()
            .oumae_kumiko()
            .unwrap();

        Some(Data::UjiBashi(Cry::new()))
    }
}

#[derive(Clone)]
pub struct Cry(String);

impl Cry {
    pub fn new() -> Self {
        Self("Jigoku no Orphee".into())
    }
}

impl fmt::Display for Cry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

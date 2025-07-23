use std::fmt;
use std::time::Duration;

use dag_flow::task::Input;
use dag_flow::task::Task;
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

impl Task<String, Data> for UjiBashi {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn dependencies(&self) -> Vec<String> {
        self.dependencies.clone()
    }

    async fn run(&self, inputs: HashMap<String, Input<'_, Data>>) -> Option<Data> {
        Delay::new(Duration::from_secs(1)).await;

        let _run = inputs[&OumaeKumiko::id()]
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

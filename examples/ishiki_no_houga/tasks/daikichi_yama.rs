use core::fmt;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use dag_flow::task::Input;
use dag_flow::task::Task;

use super::kousaka_reina::KousakaReina;
use super::kousaka_reina::Trumpet;
use super::oumae_kumiko::Euphonium;
use super::oumae_kumiko::OumaeKumiko;

pub struct DaikichiYama {
    id: String,
    dependencies: Vec<String>,
}

impl DaikichiYama {
    pub fn id() -> String {
        "daikichi-yama".into()
    }

    pub fn new() -> Self {
        Self {
            id: Self::id(),
            dependencies: vec![OumaeKumiko::id(), KousakaReina::id()],
        }
    }
}

impl Task<String, Arc<dyn Any + Send + Sync>> for DaikichiYama {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn dependencies(&self) -> Vec<String> {
        self.dependencies.clone()
    }

    async fn run(
        &self,
        inputs: HashMap<String, Input<'_, Arc<dyn Any + Send + Sync>>>,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        let _euphonium: Arc<Euphonium> = inputs[&OumaeKumiko::id()]
            .clone()
            .await
            .unwrap()
            .downcast()
            .unwrap();

        let _trumpet: Arc<Trumpet> = inputs[&KousakaReina::id()]
            .clone()
            .await
            .unwrap()
            .downcast()
            .unwrap();

        Some(Arc::new(Observatory::new()))
    }
}

pub struct Observatory(String);

impl Observatory {
    fn new() -> Self {
        Self("Ai wo Mitsuketa Basho".into())
    }
}

impl fmt::Display for Observatory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

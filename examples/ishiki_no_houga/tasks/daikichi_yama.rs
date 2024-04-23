use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use flow::task::Input;
use flow::task::Task;
use futures_timer::Delay;

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

#[async_trait]
impl Task<String, Arc<dyn Any + Send + Sync>> for DaikichiYama {
    fn id(&self) -> &String {
        &self.id
    }

    fn dependencies(&self) -> &[String] {
        &self.dependencies
    }

    async fn run(
        &self,
        input: Vec<Input<'_, String, Arc<dyn Any + Send + Sync>>>,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        Delay::new(Duration::from_secs(1)).await;

        let input: HashMap<_, _> = input
            .into_iter()
            .map(|Input { id, data }| (id, data))
            .collect();

        let _euphonium: Arc<Euphonium> = input[&OumaeKumiko::id()]
            .clone()
            .await
            .unwrap()
            .downcast()
            .unwrap();

        let _trumpet: Arc<Trumpet> = input[&KousakaReina::id()]
            .clone()
            .await
            .unwrap()
            .downcast()
            .unwrap();

        Some(Arc::new(Observatory::new()))
    }
}

#[derive(Debug)]
pub struct Observatory(String);

impl Observatory {
    fn new() -> Self {
        Self("Ai wo Mitsuketa Basho".into())
    }
}

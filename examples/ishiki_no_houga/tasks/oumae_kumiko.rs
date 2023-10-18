use std::any::Any;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use flow::task::Input;
use flow::task::Task;
use futures_timer::Delay;

pub struct OumaeKumiko {
    id: String,
}

impl OumaeKumiko {
    pub fn id() -> String {
        "oumae-kumiko".into()
    }

    pub fn new() -> Self {
        Self { id: Self::id() }
    }
}

#[async_trait]
impl Task<String, Arc<dyn Any + Send + Sync>> for OumaeKumiko {
    fn id(&self) -> &String {
        &self.id
    }

    async fn run(
        &self,
        _: Vec<Input<'_, String, Arc<dyn Any + Send + Sync>>>,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        Delay::new(Duration::from_secs(1)).await;
        Some(Arc::new(Euphonium::new()))
    }
}

#[derive(Debug)]
pub struct Euphonium(());

impl Euphonium {
    fn new() -> Self {
        Self(())
    }
}

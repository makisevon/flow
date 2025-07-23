use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use dag_flow::task::Input;
use dag_flow::task::Task;
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

impl Task<String, Arc<dyn Any + Send + Sync>> for OumaeKumiko {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn is_auto(&self) -> bool {
        false
    }

    async fn run(
        &self,
        _: HashMap<String, Input<'_, Arc<dyn Any + Send + Sync>>>,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        Delay::new(Duration::from_secs(1)).await;
        Some(Arc::new(Euphonium::new()))
    }
}

pub struct Euphonium(());

impl Euphonium {
    fn new() -> Self {
        Self(())
    }
}

impl fmt::Display for Euphonium {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Euphonium")
    }
}

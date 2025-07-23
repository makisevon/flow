use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use dag_flow::task::Input;
use dag_flow::task::Task;
use futures_timer::Delay;

pub struct KousakaReina {
    id: String,
}

impl KousakaReina {
    pub fn id() -> String {
        "kousaka-reina".into()
    }

    pub fn new() -> Self {
        Self { id: Self::id() }
    }
}

impl Task<String, Arc<dyn Any + Send + Sync>> for KousakaReina {
    fn id(&self) -> String {
        self.id.clone()
    }

    async fn run(
        &self,
        _: HashMap<String, Input<'_, Arc<dyn Any + Send + Sync>>>,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        Delay::new(Duration::from_secs(1)).await;
        Some(Arc::new(Trumpet::new()))
    }
}

pub struct Trumpet(());

impl Trumpet {
    fn new() -> Self {
        Self(())
    }
}

impl fmt::Display for Trumpet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Trumpet")
    }
}

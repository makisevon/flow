use std::fmt;
use std::time::Duration;

use dag_flow::task::Input;
use dag_flow::task::Task;
use futures_timer::Delay;

use super::data::Data;

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

impl Task<String, Data> for OumaeKumiko {
    fn id(&self) -> &String {
        &self.id
    }

    async fn run(&self, _: Vec<Input<'_, String, Data>>) -> Option<Data> {
        Delay::new(Duration::from_secs(1)).await;
        Some(Data::OumaeKumiko(Run::new()))
    }
}

#[derive(Clone)]
pub struct Run(String);

impl Run {
    fn new() -> Self {
        Self("Umaku Naritai".into())
    }
}

impl fmt::Display for Run {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

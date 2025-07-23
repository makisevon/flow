use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::RwLock;

use futures::future::BoxFuture;
use futures::future::Shared;

pub type Value<'a, T> = Shared<BoxFuture<'a, T>>;

#[derive(Clone, Debug)]
pub struct Context<'a, K, V> {
    context: Arc<RwLock<HashMap<K, Value<'a, V>>>>,
}

impl<K, V> Context<'_, K, V> {
    pub fn new() -> Self {
        Self {
            context: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<K, V> Default for Context<'_, K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, K, V> Context<'a, K, V>
where
    K: Eq + Hash,
{
    pub fn get(&self, key: &K) -> Option<Value<'a, V>> {
        self.context.read().unwrap().get(key).cloned()
    }

    pub fn set(&self, key: K, value: Value<'a, V>) -> Option<Value<'a, V>> {
        self.context.write().unwrap().insert(key, value)
    }
}

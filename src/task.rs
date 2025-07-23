use std::collections::HashMap;

use crate::context::Value;

pub type Input<'a, T> = Value<'a, Option<T>>;

#[trait_variant::make(Send + Sync)]
#[dynosaur::dynosaur(pub(crate) DynTask = dyn(box) Task)]
pub trait Task<I, D> {
    fn id(&self) -> I;

    fn dependencies(&self) -> Vec<I> {
        Vec::new()
    }

    fn is_auto(&self) -> bool {
        true
    }

    async fn run(&self, inputs: HashMap<I, Input<'_, D>>) -> Option<D>;
}

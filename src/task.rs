use futures::future::BoxFuture;
use futures::future::Shared;

#[trait_variant::make(Send + Sync)]
#[dynosaur::dynosaur(pub DynTask = dyn(box) Task)]
pub trait Task<I, D> {
    fn id(&self) -> &I;

    fn dependencies(&self) -> &[I] {
        &[]
    }

    async fn run(&self, input: Vec<Input<'_, I, D>>) -> Option<D>;
}

#[derive(Clone, Debug)]
pub struct Input<'a, I, D> {
    pub id: I,
    pub data: Shared<BoxFuture<'a, Option<D>>>,
}

impl<'a, I, D> Input<'a, I, D> {
    pub fn new(id: I, data: Shared<BoxFuture<'a, Option<D>>>) -> Self {
        Self { id, data }
    }
}

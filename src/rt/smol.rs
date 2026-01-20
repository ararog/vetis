use hyper::rt::Executor;
use std::future::Future;

#[non_exhaustive]
#[derive(Default, Debug, Clone)]
pub struct SmolExecutor {}

impl<Fut> Executor<Fut> for SmolExecutor
where
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
{
    fn execute(&self, fut: Fut) {
        smol::spawn(fut).detach();
    }
}

impl SmolExecutor {
    pub fn new() -> Self {
        Self {}
    }
}

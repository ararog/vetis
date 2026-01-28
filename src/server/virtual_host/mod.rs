use std::{future::Future, pin::Pin};

use crate::{config::VirtualHostConfig, errors::VetisError, Request, Response};

pub mod directory;

pub type BoxedHandlerClosure = Box<
    dyn Fn(Request) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send>>
        + Send
        + Sync,
>;

pub fn handler_fn<F, Fut>(f: F) -> BoxedHandlerClosure
where
    F: Fn(Request) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Response, VetisError>> + Send + Sync + 'static,
{
    Box::new(move |req| Box::pin(f(req)))
}

pub trait VirtualHost: Send + Sync + 'static {
    fn new(config: VirtualHostConfig) -> Self
    where
        Self: Sized;
    fn config(&self) -> &VirtualHostConfig;
    fn hostname(&self) -> String;
    fn port(&self) -> u16;
    fn is_secure(&self) -> bool;
    fn set_handler(&mut self, handler: BoxedHandlerClosure);
    fn execute(
        &self,
        request: Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send>>;
}

// All of them should have a handler to process requests
pub struct DefaultVirtualHost {
    config: VirtualHostConfig,
    handler: Option<BoxedHandlerClosure>,
}

impl VirtualHost for DefaultVirtualHost {
    fn new(config: VirtualHostConfig) -> Self {
        Self { config, handler: None }
    }

    fn config(&self) -> &VirtualHostConfig {
        &self.config
    }

    fn hostname(&self) -> String {
        self.config
            .hostname()
            .clone()
    }

    fn port(&self) -> u16 {
        self.config.port()
    }

    fn is_secure(&self) -> bool {
        self.config
            .security()
            .is_some()
    }

    fn set_handler(&mut self, handler: BoxedHandlerClosure) {
        self.handler = Some(handler);
    }

    fn execute(
        &self,
        request: Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send>> {
        if let Some(handler) = &self.handler {
            handler(request)
        } else {
            Box::pin(async move { Err(VetisError::Handler("No handler set".to_string())) })
        }
    }
}

impl<V: VirtualHost> VirtualHost for Box<V> {
    fn new(config: VirtualHostConfig) -> Self
    where
        Self: Sized,
    {
        Box::new(V::new(config))
    }

    fn config(&self) -> &VirtualHostConfig {
        self.as_ref()
            .config()
    }

    fn hostname(&self) -> String {
        self.as_ref()
            .hostname()
    }

    fn port(&self) -> u16 {
        self.as_ref().port()
    }

    fn is_secure(&self) -> bool {
        self.as_ref()
            .is_secure()
    }

    fn set_handler(&mut self, handler: BoxedHandlerClosure) {
        self.as_mut()
            .set_handler(handler)
    }

    fn execute(
        &self,
        request: Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send>> {
        self.as_ref()
            .execute(request)
    }
}

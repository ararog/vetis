use std::{future::Future, marker::PhantomData, pin::Pin};

use crate::{
    server::{config::VirtualHostConfig, errors::VetisError},
    RequestType, ResponseType,
};

pub mod directory;

pub type BoxedHandlerClosure = Box<
    dyn Fn(RequestType) -> Pin<Box<dyn Future<Output = Result<ResponseType, VetisError>> + Send>>
        + Send
        + Sync,
>;

pub trait VirtualHost {
    fn new(config: VirtualHostConfig, handler: BoxedHandlerClosure) -> Self
    where
        Self: Sized;
    fn config(&self) -> &VirtualHostConfig;
    fn hostname(&self) -> String;
    fn is_secure(&self) -> bool;
    fn execute(
        &self,
        request: RequestType,
    ) -> Pin<Box<dyn Future<Output = Result<ResponseType, VetisError>> + Send>>;
}

// All of them should have a handler to process requests
pub struct DefaultVirtualHost {
    config: VirtualHostConfig,
    handler: BoxedHandlerClosure,
}

impl VirtualHost for DefaultVirtualHost {
    fn new(config: VirtualHostConfig, handler: BoxedHandlerClosure) -> Self {
        Self { config, handler }
    }

    fn config(&self) -> &VirtualHostConfig {
        &self.config
    }

    fn hostname(&self) -> String {
        self.config
            .hostname()
            .clone()
    }

    fn is_secure(&self) -> bool {
        self.config
            .security()
            .is_some()
    }

    fn execute(
        &self,
        request: RequestType,
    ) -> Pin<Box<dyn Future<Output = Result<ResponseType, VetisError>> + Send>> {
        (self.handler)(request)
    }
}

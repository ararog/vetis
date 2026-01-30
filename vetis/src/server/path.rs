use std::{future::Future, pin::Pin};

use crate::{errors::VetisError, server::virtual_host::BoxedHandlerClosure, Request, Response};

pub trait Path {
    fn value(&self) -> &str;
    fn match_path(&self, path: &str) -> bool;
    fn handle(
        &self,
        request: Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send>>;
}

pub enum HostPath {
    Handler(HandlerPath),
    Proxy(ProxyPath),
    Static(StaticPath),
}

impl Path for HostPath {
    fn value(&self) -> &str {
        match self {
            HostPath::Handler(handler) => handler.value(),
            HostPath::Proxy(proxy) => proxy.value(),
            HostPath::Static(static_path) => static_path.value(),
        }
    }

    fn match_path(&self, path: &str) -> bool {
        match self {
            HostPath::Handler(handler) => handler.match_path(path),
            HostPath::Proxy(proxy) => proxy.match_path(path),
            HostPath::Static(static_path) => static_path.match_path(path),
        }
    }

    fn handle(
        &self,
        request: Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send>> {
        match self {
            HostPath::Handler(handler) => handler.handle(request),
            HostPath::Proxy(proxy) => proxy.handle(request),
            HostPath::Static(static_path) => static_path.handle(request),
        }
    }
}

pub struct HandlerPath {
    uri: String,
    handler: BoxedHandlerClosure,
}

impl HandlerPath {
    pub fn new_host_path(uri: String, handler: BoxedHandlerClosure) -> HostPath {
        HostPath::Handler(Self { uri, handler })
    }
}

impl Path for HandlerPath {
    fn value(&self) -> &str {
        &self.uri
    }

    fn match_path(&self, path: &str) -> bool {
        self.uri == path
    }

    fn handle(
        &self,
        request: Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send>> {
        (self.handler)(request)
    }
}

pub struct StaticPath {
    uri: String,
    directory: String,
}

impl Path for StaticPath {
    fn value(&self) -> &str {
        &self.uri
    }

    fn match_path(&self, path: &str) -> bool {
        self.uri == path
    }

    fn handle(
        &self,
        request: Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send>> {
        todo!()
    }
}

pub struct ProxyPath {
    uri: String,
    target: String,
}

impl Path for ProxyPath {
    fn value(&self) -> &str {
        todo!()
    }

    fn match_path(&self, path: &str) -> bool {
        todo!()
    }

    fn handle(
        &self,
        request: Request,
    ) -> Pin<Box<dyn Future<Output = Result<Response, VetisError>> + Send>> {
        todo!()
    }
}


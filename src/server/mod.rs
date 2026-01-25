use std::{collections::HashMap, future::Future, sync::Arc};

use crate::{
    server::{errors::VetisError, virtual_host::VirtualHost},
    VetisRwLock,
};

#[cfg(any(feature = "http1", feature = "http2"))]
pub mod http;
#[cfg(feature = "http3")]
pub mod quic;

#[cfg(any(feature = "http1", feature = "http2"))]
pub mod tcp;
#[cfg(feature = "http3")]
pub mod udp;

pub mod config;
pub mod errors;
pub mod tls;
pub mod virtual_host;

pub trait Server<RequestBody, ResponseBody> {
    fn port(&self) -> u16;

    fn set_virtual_hosts(
        &mut self,
        virtual_hosts: Arc<
            VetisRwLock<HashMap<String, Box<dyn VirtualHost + Send + Sync + 'static>>>,
        >,
    );

    fn start(&mut self) -> impl Future<Output = Result<(), VetisError>>;

    fn stop(&mut self) -> impl Future<Output = Result<(), VetisError>>;
}

use std::{future::Future, pin::Pin};

use rt_gate::GateTask;

use crate::{config::ListenerConfig, errors::VetisError, VetisVirtualHosts};

#[cfg(any(feature = "http1", feature = "http2"))]
pub(crate) mod tcp;

#[cfg(feature = "http3")]
pub(crate) mod udp;

pub trait ServerListener {
    fn new(config: ListenerConfig) -> Self
    where
        Self: Sized;

    fn port(&self) -> u16;

    fn set_virtual_hosts(&mut self, virtual_hosts: VetisVirtualHosts);

    fn listen(&mut self) -> Pin<Box<dyn Future<Output = Result<(), VetisError>> + Send + '_>>;

    fn stop(&mut self) -> Pin<Box<dyn Future<Output = Result<(), VetisError>> + Send + '_>>;
}

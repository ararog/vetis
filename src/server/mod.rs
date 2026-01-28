use std::future::Future;

use crate::{config::ServerConfig, errors::VetisError, VetisVirtualHosts};

pub mod conn;
pub mod http;
pub mod tls;
pub mod virtual_host;

pub trait Server {
    fn new(config: ServerConfig) -> Self;

    fn set_virtual_hosts(&mut self, virtual_hosts: VetisVirtualHosts);

    fn start(&mut self) -> impl Future<Output = Result<(), VetisError>>;

    fn stop(&mut self) -> impl Future<Output = Result<(), VetisError>>;
}

use std::future::Future;

use crate::{errors::VetisError, VetisVirtualHosts};

pub mod conn;
pub mod tls;
pub mod virtual_host;

pub trait Server<RequestBody, ResponseBody> {
    fn port(&self) -> u16;

    fn set_virtual_hosts(&mut self, virtual_hosts: VetisVirtualHosts);

    fn start(&mut self) -> impl Future<Output = Result<(), VetisError>>;

    fn stop(&mut self) -> impl Future<Output = Result<(), VetisError>>;
}

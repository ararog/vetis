use std::future::Future;

use rt_gate::GateTask;

use crate::{errors::VetisError, server::Server, VetisVirtualHosts};
use bytes::Bytes;
use http_body_util::Full;

pub(crate) mod http;

pub trait UdpServer: Server<Full<Bytes>, Full<Bytes>> {
    fn handle_connections(
        &mut self,
        endpoint: quinn::Endpoint,
        virtual_host: VetisVirtualHosts,
    ) -> impl Future<Output = Result<GateTask, VetisError>>;
}

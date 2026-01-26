use std::{collections::HashMap, future::Future, sync::Arc};

use crate::{
    errors::VetisError,
    server::{virtual_host::VirtualHost, Server},
    VetisRwLock,
};

use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use rt_gate::GateTask;

#[cfg(feature = "smol-rt")]
use smol::net::TcpListener;
#[cfg(feature = "tokio-rt")]
use tokio::net::TcpListener;

#[cfg(feature = "tokio-rt")]
type VetisTcpListener = TcpListener;
#[cfg(feature = "smol-rt")]
type VetisTcpListener = TcpListener;

pub(crate) mod http;

pub trait TcpServer: Server<Incoming, Full<Bytes>> {
    fn handle_connections(
        &mut self,
        listener: VetisTcpListener,
        virtual_host: Arc<VetisRwLock<HashMap<String, Box<dyn VirtualHost>>>>,
    ) -> impl Future<Output = Result<GateTask, VetisError>>;
}

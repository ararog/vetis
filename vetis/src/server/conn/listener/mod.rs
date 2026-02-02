use std::{future::Future, pin::Pin};

#[cfg(any(feature = "http1", feature = "http2"))]
use crate::server::conn::listener::tcp::TcpListener;
#[cfg(feature = "http3")]
use crate::server::conn::listener::udp::UdpListener;

use crate::{
    config::{ListenerConfig, Protocol},
    errors::VetisError,
    VetisVirtualHosts,
};

#[cfg(any(feature = "http1", feature = "http2"))]
pub(crate) mod tcp;

#[cfg(feature = "http3")]
pub(crate) mod udp;

pub type ListenerResult<'a, T> = Pin<Box<dyn Future<Output = Result<T, VetisError>> + Send + 'a>>;

pub trait Listener {
    fn new(config: ListenerConfig) -> Self
    where
        Self: Sized;

    fn set_virtual_hosts(&mut self, virtual_hosts: VetisVirtualHosts);

    fn listen(&mut self) -> ListenerResult<()>;

    fn stop(&mut self) -> ListenerResult<()>;
}

pub enum ServerListener {
    #[cfg(any(feature = "http1", feature = "http2"))]
    Tcp(TcpListener),
    #[cfg(feature = "http3")]
    Udp(UdpListener),
}

impl Listener for ServerListener {
    fn new(config: ListenerConfig) -> Self
    where
        Self: Sized,
    {
        match config.protocol() {
            #[cfg(feature = "http1")]
            Protocol::Http1 => ServerListener::Tcp(TcpListener::new(config)),
            #[cfg(feature = "http2")]
            Protocol::Http2 => ServerListener::Tcp(TcpListener::new(config)),
            #[cfg(feature = "http3")]
            Protocol::Http3 => ServerListener::Udp(UdpListener::new(config)),
        }
    }

    fn set_virtual_hosts(&mut self, virtual_hosts: VetisVirtualHosts) {
        match self {
            #[cfg(any(feature = "http1", feature = "http2"))]
            ServerListener::Tcp(ref mut tcp_listener) => {
                tcp_listener.set_virtual_hosts(virtual_hosts);
            }
            #[cfg(feature = "http3")]
            ServerListener::Udp(ref mut udp_listener) => {
                udp_listener.set_virtual_hosts(virtual_hosts);
            }
        }
    }

    fn listen(&mut self) -> ListenerResult<()> {
        Box::pin(async move {
            match self {
                #[cfg(any(feature = "http1", feature = "http2"))]
                ServerListener::Tcp(ref mut tcp_listener) => {
                    tcp_listener
                        .listen()
                        .await?
                }
                #[cfg(feature = "http3")]
                ServerListener::Udp(ref mut udp_listener) => {
                    udp_listener
                        .listen()
                        .await?
                }
            }

            Ok(())
        })
    }

    fn stop(&mut self) -> ListenerResult<()> {
        Box::pin(async move {
            match self {
                #[cfg(any(feature = "http1", feature = "http2"))]
                ServerListener::Tcp(ref mut tcp_listener) => {
                    tcp_listener
                        .stop()
                        .await?
                }
                #[cfg(feature = "http3")]
                ServerListener::Udp(ref mut udp_listener) => {
                    udp_listener
                        .stop()
                        .await?
                }
            }
            Ok(())
        })
    }
}

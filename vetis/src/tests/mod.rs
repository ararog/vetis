use crate::config::Protocol;

pub(crate) const CA_CERT: &[u8] = include_bytes!("certs/ca.der");

pub(crate) const SERVER_CERT: &[u8] = include_bytes!("certs/server.der");
pub(crate) const SERVER_KEY: &[u8] = include_bytes!("certs/server.key.der");

pub(crate) const IP6_SERVER_CERT: &[u8] = include_bytes!("certs/ip6-server.der");
pub(crate) const IP6_SERVER_KEY: &[u8] = include_bytes!("certs/ip6-server.key.der");

pub(crate) const fn default_protocol() -> Protocol {
    #[cfg(feature = "http1")]
    {
        Protocol::Http1
    }
    #[cfg(feature = "http2")]
    {
        Protocol::Http2
    }
    #[cfg(feature = "http3")]
    {
        Protocol::Http3
    }
}

#[cfg(test)]
mod config;
#[cfg(test)]
mod paths;
#[cfg(test)]
mod server;
#[cfg(test)]
mod tls;
#[cfg(test)]
mod virtual_host;

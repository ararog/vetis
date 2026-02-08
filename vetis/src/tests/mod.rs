pub(crate) const CA_CERT: &[u8] = include_bytes!("certs/ca.der");

pub(crate) const SERVER_CERT: &[u8] = include_bytes!("certs/server.der");
pub(crate) const SERVER_KEY: &[u8] = include_bytes!("certs/server.key.der");

pub(crate) const IP6_SERVER_CERT: &[u8] = include_bytes!("certs/ip6-server.der");
pub(crate) const IP6_SERVER_KEY: &[u8] = include_bytes!("certs/ip6-server.key.der");

#[cfg(test)]
mod config;
#[cfg(test)]
mod paths;
#[cfg(test)]
mod server;
#[cfg(test)]
mod virtual_host;

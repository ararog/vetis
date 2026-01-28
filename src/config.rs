// TODO: add support for virtual hosts and paths

use std::fs;

#[derive(Clone)]
pub enum Protocol {
    #[cfg(feature = "http1")]
    HTTP1,
    #[cfg(feature = "http2")]
    HTTP2,
    #[cfg(feature = "http3")]
    HTTP3,
}

#[derive(Clone)]
pub struct ListenerConfigBuilder {
    port: u16,
    ssl: bool,
    protocol: Protocol,
    interface: String,
}

impl ListenerConfigBuilder {
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn ssl(mut self, ssl: bool) -> Self {
        self.ssl = ssl;
        self
    }

    pub fn interface(mut self, interface: String) -> Self {
        self.interface = interface;
        self
    }

    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = protocol;
        self
    }

    pub fn build(self) -> ListenerConfig {
        ListenerConfig {
            port: self.port,
            ssl: self.ssl,
            protocol: self.protocol,
            interface: self.interface,
        }
    }
}

#[derive(Clone)]
pub struct ListenerConfig {
    port: u16,
    ssl: bool,
    protocol: Protocol,
    interface: String,
}

impl ListenerConfig {
    pub fn builder() -> ListenerConfigBuilder {
        ListenerConfigBuilder {
            port: 0,
            ssl: false,
            #[cfg(feature = "http1")]
            protocol: Protocol::HTTP1,
            #[cfg(feature = "http2")]
            protocol: Protocol::HTTP2,
            #[cfg(feature = "http3")]
            protocol: Protocol::HTTP3,
            interface: "0.0.0.0".to_string(),
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn ssl(&self) -> bool {
        self.ssl
    }

    pub fn protocol(&self) -> &Protocol {
        &self.protocol
    }

    pub fn interface(&self) -> &String {
        &self.interface
    }
}

#[derive(Clone)]
pub struct ServerConfigBuilder {
    listeners: Vec<ListenerConfig>,
}

impl ServerConfigBuilder {
    pub fn add_listener(mut self, listener: ListenerConfig) -> Self {
        self.listeners
            .push(listener);
        self
    }

    pub fn build(self) -> ServerConfig {
        ServerConfig { listeners: self.listeners }
    }
}

#[derive(Clone)]
pub struct ServerConfig {
    listeners: Vec<ListenerConfig>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig { listeners: vec![] }
    }
}

impl ServerConfig {
    pub fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder { listeners: vec![] }
    }

    pub fn listeners(&self) -> &Vec<ListenerConfig> {
        &self.listeners
    }
}

pub struct VirtualHostConfigBuilder {
    hostname: String,
    port: u16,
    security: Option<SecurityConfig>,
}

impl VirtualHostConfigBuilder {
    pub fn hostname(mut self, hostname: String) -> Self {
        self.hostname = hostname;
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn security(mut self, security: SecurityConfig) -> Self {
        self.security = Some(security);
        self
    }

    pub fn build(self) -> VirtualHostConfig {
        VirtualHostConfig { hostname: self.hostname, port: self.port, security: self.security }
    }
}

pub struct VirtualHostConfig {
    hostname: String,
    port: u16,
    security: Option<SecurityConfig>,
}

impl VirtualHostConfig {
    pub fn builder() -> VirtualHostConfigBuilder {
        VirtualHostConfigBuilder { hostname: String::new(), port: 0, security: None }
    }

    pub fn hostname(&self) -> &String {
        &self.hostname
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn security(&self) -> &Option<SecurityConfig> {
        &self.security
    }
}

#[derive(Clone)]
pub struct SecurityConfigBuilder {
    cert: Vec<u8>,
    key: Vec<u8>,
    ca_cert: Option<Vec<u8>>,
    client_auth: bool,
}

impl SecurityConfigBuilder {
    #[deprecated(note = "Use cert_from_bytes or cert_from_file instead")]
    pub fn cert(mut self, cert: Vec<u8>) -> Self {
        self.cert = cert;
        self
    }

    pub fn cert_from_bytes(mut self, cert: Vec<u8>) -> Self {
        self.cert = cert;
        self
    }

    pub fn cert_from_file(mut self, path: &str) -> Self {
        self.cert = fs::read(path).unwrap();
        self
    }

    #[deprecated(note = "Use key_from_bytes or key_from_file instead")]
    pub fn key(mut self, key: Vec<u8>) -> Self {
        self.key = key;
        self
    }

    pub fn key_from_bytes(mut self, key: Vec<u8>) -> Self {
        self.key = key;
        self
    }

    pub fn key_from_file(mut self, path: &str) -> Self {
        self.key = fs::read(path).unwrap();
        self
    }

    #[deprecated(note = "Use ca_cert_from_bytes or ca_cert_from_file instead")]
    pub fn ca_cert(mut self, ca_cert: Vec<u8>) -> Self {
        self.ca_cert = Some(ca_cert);
        self
    }

    pub fn ca_cert_from_bytes(mut self, ca_cert: Vec<u8>) -> Self {
        self.ca_cert = Some(ca_cert);
        self
    }

    pub fn ca_cert_from_file(mut self, path: &str) -> Self {
        self.ca_cert = Some(fs::read(path).unwrap());
        self
    }

    pub fn client_auth(mut self, client_auth: bool) -> Self {
        self.client_auth = client_auth;
        self
    }

    pub fn build(self) -> SecurityConfig {
        SecurityConfig {
            cert: self.cert,
            key: self.key,
            ca_cert: self.ca_cert,
            client_auth: self.client_auth,
        }
    }
}

#[derive(Clone)]
pub struct SecurityConfig {
    cert: Vec<u8>,
    key: Vec<u8>,
    ca_cert: Option<Vec<u8>>,
    client_auth: bool,
}

impl SecurityConfig {
    pub fn builder() -> SecurityConfigBuilder {
        SecurityConfigBuilder {
            cert: Vec::new(),
            key: Vec::new(),
            ca_cert: None,
            client_auth: false,
        }
    }

    pub fn cert(&self) -> &Vec<u8> {
        &self.cert
    }

    pub fn key(&self) -> &Vec<u8> {
        &self.key
    }

    pub fn ca_cert(&self) -> &Option<Vec<u8>> {
        &self.ca_cert
    }

    pub fn client_auth(&self) -> bool {
        self.client_auth
    }
}

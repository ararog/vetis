// TODO: add support for virtual hosts and paths

use std::fs;

#[derive(Clone)]
pub struct ServerConfigBuilder {
    port: u16,
    interface: String,
}

impl ServerConfigBuilder {
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn interface(mut self, interface: String) -> Self {
        self.interface = interface;
        self
    }

    pub fn build(self) -> ServerConfig {
        ServerConfig { port: self.port, interface: self.interface }
    }
}

#[derive(Clone)]
pub struct ServerConfig {
    port: u16,
    interface: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig { port: 80, interface: "0.0.0.0".to_string() }
    }
}

impl ServerConfig {
    pub fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder { port: 0, interface: "0.0.0.0".to_string() }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn interface(&self) -> &String {
        &self.interface
    }
}

pub struct VirtualHostConfigBuilder {
    hostname: String,
    security: Option<SecurityConfig>,
}

impl VirtualHostConfigBuilder {
    pub fn hostname(mut self, hostname: String) -> Self {
        self.hostname = hostname;
        self
    }

    pub fn security(mut self, security: SecurityConfig) -> Self {
        self.security = Some(security);
        self
    }

    pub fn build(self) -> VirtualHostConfig {
        VirtualHostConfig { hostname: self.hostname, security: self.security }
    }
}

pub struct VirtualHostConfig {
    hostname: String,
    security: Option<SecurityConfig>,
}

impl VirtualHostConfig {
    pub fn builder() -> VirtualHostConfigBuilder {
        VirtualHostConfigBuilder { hostname: String::new(), security: None }
    }

    pub fn hostname(&self) -> &String {
        &self.hostname
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

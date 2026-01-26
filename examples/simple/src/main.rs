use bytes::Bytes;
use http_body_util::Full;
use hyper::{Response};
use vetis::{
    config::{SecurityConfig, ServerConfig, VirtualHostConfig},
    server::virtual_host::{handler_fn, DefaultVirtualHost, VirtualHost},
    Vetis,
};

pub const CA_CERT: &[u8] = include_bytes!("../certs/ca.der");
pub const SERVER_CERT: &[u8] = include_bytes!("../certs/server.der");
pub const SERVER_KEY: &[u8] = include_bytes!("../certs/server.key.der");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std_logger::Config::logfmt().init();

    let config = ServerConfig::builder()
        .port(8443)
        .interface("0.0.0.0".to_string())
        .build();

    let security_config = SecurityConfig::builder()
        .ca_cert_from_bytes(CA_CERT.to_vec())
        .cert_from_bytes(SERVER_CERT.to_vec())
        .key_from_bytes(SERVER_KEY.to_vec())
        .build();

    let localhost_config = VirtualHostConfig::builder()
        .hostname("localhost".to_string())
        .security(security_config)
        .build();

    let server_config = VirtualHostConfig::builder()
        .hostname("server".to_string())
        .build();

    let mut localhost_virtual_host = DefaultVirtualHost::new(localhost_config);

    localhost_virtual_host.set_handler(handler_fn(|request| async move {
        Ok(Response::new(Full::new(Bytes::from("Hello from localhost"))))
    }));

    let mut server_virtual_host = DefaultVirtualHost::new(server_config);

    server_virtual_host.set_handler(handler_fn(|request| async move {
        Ok(Response::new(Full::new(Bytes::from("Hello from server"))))
    }));

    let mut server = Vetis::new(config);
    server
        .add_virtual_host(localhost_virtual_host)
        .await;
    server
        .add_virtual_host(server_virtual_host)
        .await;

    server.run().await?;

    server
        .stop()
        .await?;

    Ok(())
}

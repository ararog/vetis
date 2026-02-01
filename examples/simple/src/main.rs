use hyper::StatusCode;
use vetis::{
    Vetis,
    config::{ListenerConfig, Protocol, SecurityConfig, ServerConfig, VirtualHostConfig},
    server::{
        path::{HandlerPath, StaticPath},
        virtual_host::{VirtualHost, handler_fn},
    },
};

pub const CA_CERT: &[u8] = include_bytes!("../certs/ca.der");
pub const SERVER_CERT: &[u8] = include_bytes!("../certs/server.der");
pub const SERVER_KEY: &[u8] = include_bytes!("../certs/server.key.der");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().filter_or("RUST_LOG", "info")).init();

    let https = ListenerConfig::builder()
        .port(8443)
        .protocol(Protocol::Http1)
        .interface("0.0.0.0".to_string())
        .build();

    let config = ServerConfig::builder()
        .add_listener(https)
        .build();

    let security_config = SecurityConfig::builder()
        .ca_cert_from_bytes(CA_CERT.to_vec())
        .cert_from_bytes(SERVER_CERT.to_vec())
        .key_from_bytes(SERVER_KEY.to_vec())
        .build();

    let localhost_config = VirtualHostConfig::builder()
        .hostname("localhost".to_string())
        .port(8443)
        .security(security_config)
        .build()?;

    let mut localhost_virtual_host = VirtualHost::new(localhost_config);

    let root_path = HandlerPath::new_host_path(
        "/hello".to_string(),
        handler_fn(|request| async move {
            let response = vetis::Response::builder()
                .status(StatusCode::OK)
                .body(b"Hello from localhost");
            Ok(response)
        }),
    );

    localhost_virtual_host.add_path(root_path);

    let health_path = HandlerPath::new_host_path(
        "/health".to_string(),
        handler_fn(|request| async move {
            let response = vetis::Response::builder()
                .status(StatusCode::OK)
                .body(b"Health check");
            Ok(response)
        }),
    );

    localhost_virtual_host.add_path(health_path);

    let images_path = StaticPath::builder()
        .uri("/images".to_string())
        .directory("/home/rogerio/Downloads".to_string())
        .extensions("\\.(jpg|png|gif)$".to_string())
        .build()?;

    localhost_virtual_host.add_path(images_path);

    let mut server = Vetis::new(config);
    server
        .add_virtual_host(localhost_virtual_host)
        .await;

    server.run().await?;

    server
        .stop()
        .await?;

    Ok(())
}

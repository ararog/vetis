use hyper::StatusCode;

#[cfg(feature = "smol")]
use macro_rules_attribute::apply;
#[cfg(feature = "smol")]
use smol_macros::main;

use vetis::{
    Vetis,
    config::{
        ListenerConfig, Protocol, ProxyPathConfig, SecurityConfig, ServerConfig, StaticPathConfig,
        VirtualHostConfig,
    },
    server::{
        path::{HandlerPath, ProxyPath, StaticPath},
        virtual_host::{VirtualHost, handler_fn},
    },
};

pub(crate) const CA_CERT: &[u8] = include_bytes!("../certs/ca.der");

pub(crate) const SERVER_CERT: &[u8] = include_bytes!("../certs/server.der");
pub(crate) const SERVER_KEY: &[u8] = include_bytes!("../certs/server.key.der");

pub(crate) const IP6_SERVER_CERT: &[u8] = include_bytes!("../certs/ip6-server.der");
pub(crate) const IP6_SERVER_KEY: &[u8] = include_bytes!("../certs/ip6-server.key.der");

#[cfg(feature = "tokio")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await
}

#[cfg(feature = "smol")]
#[apply(main!)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().filter_or("RUST_LOG", "info")).init();

    let https = ListenerConfig::builder()
        .port(8443)
        .protocol(Protocol::Http1)
        .interface("::")
        .build();

    let config = ServerConfig::builder()
        .add_listener(https)
        .build();

    let security_config = SecurityConfig::builder()
        .ca_cert_from_bytes(CA_CERT.to_vec())
        .cert_from_bytes(IP6_SERVER_CERT.to_vec())
        .key_from_bytes(IP6_SERVER_KEY.to_vec())
        .build();

    let localhost_config = VirtualHostConfig::builder()
        .hostname("ip6-localhost")
        .port(8443)
        .security(security_config)
        .status_pages(maplit::hashmap! {
            404 => "/home/rogerio/Downloads/404.html".to_string(),
            500 => "/home/rogerio/Downloads/500.html".to_string(),
        })
        .build()?;

    let mut localhost_virtual_host = VirtualHost::new(localhost_config);

    let root_path = HandlerPath::builder()
        .uri("/hello")
        .handler(handler_fn(|request| async move {
            let response = vetis::Response::builder()
                .status(StatusCode::OK)
                .text("Hello from localhost");
            Ok(response)
        }))
        .build()
        .unwrap();

    localhost_virtual_host.add_path(root_path);

    let health_path = HandlerPath::builder()
        .uri("/health")
        .handler(handler_fn(|request| async move {
            let response = vetis::Response::builder()
                .status(StatusCode::OK)
                .text("Health check");
            Ok(response)
        }))
        .build()
        .unwrap();

    localhost_virtual_host.add_path(health_path);

    let proxy_path = ProxyPathConfig::builder()
        .uri("/proxy")
        .target("http://localhost:5230")
        .build()?;

    localhost_virtual_host.add_path(ProxyPath::new(proxy_path));

    let images_path = StaticPathConfig::builder()
        .uri("/images")
        .directory("/home/rogerio/Downloads")
        .extensions("\\.(jpg|png|gif|html)$")
        .index_files(vec!["index.html".to_string()])
        .build()?;

    localhost_virtual_host.add_path(StaticPath::new(images_path));

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

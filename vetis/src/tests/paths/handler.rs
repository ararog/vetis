use deboa::{cert::Certificate, request};
use http::StatusCode;

#[cfg(feature = "smol-rt")]
use macro_rules_attribute::apply;
#[cfg(feature = "smol-rt")]
use smol_macros::test;

use crate::{
    config::server::{
        virtual_host::{SecurityConfig, VirtualHostConfig},
        ListenerConfig, ServerConfig,
    },
    server::virtual_host::{handler_fn, path::HandlerPath, VirtualHost},
    tests::{default_protocol, CA_CERT, SERVER_CERT, SERVER_KEY},
};

async fn do_test_handler() -> Result<(), Box<dyn std::error::Error>> {
    let ipv4 = ListenerConfig::builder()
        .port(8082)
        .protocol(default_protocol())
        .interface("0.0.0.0")
        .build()?;

    let config = ServerConfig::builder()
        .add_listener(ipv4)
        .build()?;

    let security_config = SecurityConfig::builder()
        .ca_cert_from_bytes(CA_CERT.to_vec())
        .cert_from_bytes(SERVER_CERT.to_vec())
        .key_from_bytes(SERVER_KEY.to_vec())
        .build()?;

    let localhost_config = VirtualHostConfig::builder()
        .hostname("localhost")
        .root_directory("src/tests")
        .port(8082)
        .security(security_config)
        .build()?;

    let mut localhost_virtual_host = VirtualHost::new(localhost_config);

    let root_path = HandlerPath::builder()
        .uri("/hello")
        .handler(handler_fn(|_request| async move {
            let response = crate::server::http::Response::builder()
                .status(StatusCode::OK)
                .text("Hello from localhost");
            Ok(response)
        }))
        .build()?;

    localhost_virtual_host.add_path(root_path);

    let mut server = crate::Vetis::new(config);
    server
        .add_virtual_host(localhost_virtual_host)
        .await;

    server
        .start()
        .await?;

    let client = deboa::Client::builder()
        .certificate(Certificate::from_slice(CA_CERT, deboa::cert::ContentEncoding::DER))
        .build();

    let request = request::get("https://localhost:8082/hello")?
        .send_with(&client)
        .await?;

    assert_eq!(request.status(), StatusCode::OK);

    server
        .stop()
        .await?;

    Ok(())
}

#[cfg(feature = "tokio-rt")]
#[tokio::test]
async fn test_handler() -> Result<(), Box<dyn std::error::Error>> {
    do_test_handler().await
}

#[cfg(feature = "smol-rt")]
#[apply(test!)]
async fn test_handler_smol() -> Result<(), Box<dyn std::error::Error>> {
    do_test_handler().await
}

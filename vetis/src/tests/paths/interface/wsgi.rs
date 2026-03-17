use std::error::Error;

use deboa::{cert::Certificate, request};
use http::StatusCode;
#[cfg(feature = "smol-rt")]
use macro_rules_attribute::apply;
#[cfg(feature = "smol-rt")]
use smol_macros::test;

use crate::{
    config::server::{
        virtual_host::{path::interface::InterfacePathConfig, SecurityConfig, VirtualHostConfig},
        ListenerConfig,
    },
    server::virtual_host::{path::interface::InterfacePath, VirtualHost},
    tests::default_protocol,
    ServerConfig,
};

async fn do_wsgi_to_target() -> Result<(), Box<dyn Error>> {
    use crate::tests::{CA_CERT, SERVER_CERT, SERVER_KEY};

    let listener = ListenerConfig::builder()
        .port(8088)
        .protocol(default_protocol())
        .interface("0.0.0.0")
        .build()?;

    let config = ServerConfig::builder()
        .add_listener(listener)
        .build()?;

    let security_config = SecurityConfig::builder()
        .ca_cert_from_bytes(CA_CERT.to_vec())
        .cert_from_bytes(SERVER_CERT.to_vec())
        .key_from_bytes(SERVER_KEY.to_vec())
        .build()?;

    let host_config = VirtualHostConfig::builder()
        .hostname("localhost")
        .port(8088)
        .root_directory("src/tests")
        .security(security_config.clone())
        .build()?;

    let mut virtual_host = VirtualHost::new(host_config);
    virtual_host.add_path(InterfacePath::new(
        InterfacePathConfig::builder()
            .uri("/")
            .directory("src/tests/files/python")
            .target("main:app")
            .build()?,
    ));

    assert_eq!(
        virtual_host
            .config()
            .hostname(),
        "localhost"
    );

    let mut server = crate::Vetis::new(config);
    server
        .add_virtual_host(virtual_host)
        .await;

    server
        .start()
        .await?;

    let client = deboa::Client::builder()
        .certificate(Certificate::from_slice(CA_CERT, deboa::cert::ContentEncoding::DER))
        .build();

    let request = request::get("https://localhost:8088/")?
        .send_with(&client)
        .await?;

    assert_eq!(request.status(), StatusCode::OK);
    assert_eq!(
        request
            .text()
            .await?,
        "Hello, World!"
    );

    server
        .stop()
        .await?;

    Ok(())
}

#[cfg(feature = "tokio-rt")]
#[tokio::test]
async fn test_wsgi_to_target() -> Result<(), Box<dyn Error>> {
    do_wsgi_to_target().await
}

#[cfg(feature = "smol-rt")]
#[apply(test!)]
async fn test_wsgi_to_target() -> Result<(), Box<dyn Error>> {
    do_wsgi_to_target().await
}

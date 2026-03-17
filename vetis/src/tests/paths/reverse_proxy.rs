use std::error::Error;

#[cfg(any(feature = "http1", feature = "http2"))]
use deboa::{cert::Certificate, request};
#[cfg(any(feature = "http1", feature = "http2"))]
use http::StatusCode;
#[cfg(any(feature = "http1", feature = "http2"))]
use http_body_util::BodyExt;

#[cfg(feature = "smol-rt")]
use macro_rules_attribute::apply;
#[cfg(feature = "smol-rt")]
use smol_macros::test;

#[cfg(any(feature = "http1", feature = "http2"))]
use crate::{
    config::server::{
        virtual_host::{SecurityConfig, VirtualHostConfig},
        ListenerConfig, ServerConfig,
    },
    server::virtual_host::{
        handler_fn,
        path::{proxy::ProxyPath, HandlerPath},
        VirtualHost,
    },
    tests::{CA_CERT, SERVER_CERT, SERVER_KEY},
};

use crate::{
    config::server::virtual_host::path::proxy::ProxyPathConfig,
    errors::{ConfigError, VetisError},
};

#[test]
fn test_proxy_path() -> Result<(), Box<dyn Error>> {
    let some_path = ProxyPathConfig::builder()
        .uri("/test")
        .target("http://localhost:8080")
        .build()?;

    assert_eq!(some_path.uri(), "/test");
    assert_eq!(some_path.target(), "http://localhost:8080");

    Ok(())
}

#[test]
fn test_invalid_proxy_path() -> Result<(), Box<dyn Error>> {
    let some_path = ProxyPathConfig::builder()
        .uri("")
        .target("http://localhost:8080")
        .build();

    assert!(some_path.is_err());
    assert_eq!(
        some_path.err(),
        Some(VetisError::Config(ConfigError::Path("URI cannot be empty".into(),)))
    );

    Ok(())
}

#[test]
fn test_invalid_proxy_path_target() -> Result<(), Box<dyn Error>> {
    let some_path = ProxyPathConfig::builder()
        .uri("/test")
        .target("")
        .build();

    assert!(some_path.is_err());
    assert_eq!(
        some_path.err(),
        Some(VetisError::Config(ConfigError::Path("Target cannot be empty".into(),)))
    );

    Ok(())
}

#[cfg(any(feature = "http1", feature = "http2"))]
async fn do_get_proxy_to_target() -> Result<(), Box<dyn Error>> {
    use crate::tests::default_protocol;

    let source_listener = ListenerConfig::builder()
        .port(8084)
        .protocol(default_protocol())
        .interface("0.0.0.0")
        .build()?;

    let target_listener = ListenerConfig::builder()
        .port(8085)
        .protocol(default_protocol())
        .interface("0.0.0.0")
        .build()?;

    let config = ServerConfig::builder()
        .add_listener(source_listener)
        .add_listener(target_listener)
        .build()?;

    let security_config = SecurityConfig::builder()
        .ca_cert_from_bytes(CA_CERT.to_vec())
        .cert_from_bytes(SERVER_CERT.to_vec())
        .key_from_bytes(SERVER_KEY.to_vec())
        .build()?;

    let source_config = VirtualHostConfig::builder()
        .hostname("localhost")
        .port(8084)
        .root_directory("src/tests")
        .security(security_config.clone())
        .build()?;

    let mut source_virtual_host = VirtualHost::new(source_config);
    source_virtual_host.add_path(ProxyPath::new(
        ProxyPathConfig::builder()
            .uri("/")
            .target("http://localhost:8085")
            .build()?,
    ));

    let target_config = VirtualHostConfig::builder()
        .hostname("localhost")
        .port(8085)
        .root_directory("src/tests")
        .build()?;

    let mut target_virtual_host = VirtualHost::new(target_config);
    target_virtual_host.add_path(
        HandlerPath::builder()
            .uri("/")
            .handler(handler_fn(|_request| async move {
                Ok(crate::server::http::Response::builder()
                    .status(StatusCode::OK)
                    .text("Hello, world!"))
            }))
            .build()?,
    );

    assert_eq!(
        target_virtual_host
            .config()
            .hostname(),
        "localhost"
    );

    let mut server = crate::Vetis::new(config);
    server
        .add_virtual_host(source_virtual_host)
        .await;
    server
        .add_virtual_host(target_virtual_host)
        .await;

    server
        .start()
        .await?;

    let client = deboa::Client::builder()
        .certificate(Certificate::from_slice(CA_CERT, deboa::cert::ContentEncoding::DER))
        .build();

    let request = request::get("https://localhost:8085/")?
        .send_with(&client)
        .await?;

    assert_eq!(request.status(), StatusCode::OK);
    assert_eq!(
        request
            .text()
            .await?,
        "Hello, world!"
    );

    server
        .stop()
        .await?;

    Ok(())
}

#[cfg(all(feature = "tokio-rt", any(feature = "http1", feature = "http2")))]
#[tokio::test]
async fn test_get_proxy_to_target() -> Result<(), Box<dyn Error>> {
    do_get_proxy_to_target().await
}

#[cfg(all(feature = "smol-rt", any(feature = "http1", feature = "http2")))]
#[apply(test!)]
async fn test_get_proxy_to_target() -> Result<(), Box<dyn Error>> {
    do_get_proxy_to_target().await
}

#[cfg(any(feature = "http1", feature = "http2"))]
async fn do_post_proxy_to_target() -> Result<(), Box<dyn Error>> {
    use crate::tests::default_protocol;

    let source_listener = ListenerConfig::builder()
        .port(9093)
        .protocol(default_protocol())
        .interface("0.0.0.0")
        .build()?;

    let target_listener = ListenerConfig::builder()
        .port(9094)
        .protocol(default_protocol())
        .interface("0.0.0.0")
        .build()?;

    let config = ServerConfig::builder()
        .add_listener(source_listener)
        .add_listener(target_listener)
        .build()?;

    let security_config = SecurityConfig::builder()
        .ca_cert_from_bytes(CA_CERT.to_vec())
        .cert_from_bytes(SERVER_CERT.to_vec())
        .key_from_bytes(SERVER_KEY.to_vec())
        .build()?;

    let source_config = VirtualHostConfig::builder()
        .hostname("localhost")
        .port(9093)
        .root_directory("src/tests")
        .security(security_config.clone())
        .build()?;

    let mut source_virtual_host = VirtualHost::new(source_config);
    source_virtual_host.add_path(ProxyPath::new(
        ProxyPathConfig::builder()
            .uri("/")
            .target("http://localhost:9094")
            .build()?,
    ));

    let target_config = VirtualHostConfig::builder()
        .hostname("localhost")
        .port(9094)
        .root_directory("src/tests")
        .build()?;

    let mut target_virtual_host = VirtualHost::new(target_config);
    target_virtual_host.add_path(
        HandlerPath::builder()
            .uri("/")
            .handler(handler_fn(|request| async move {
                let (_parts, body) = request.into_parts();
                let text = body
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes();
                Ok(crate::server::http::Response::builder()
                    .status(StatusCode::OK)
                    .bytes(text.as_ref()))
            }))
            .build()?,
    );

    assert_eq!(
        target_virtual_host
            .config()
            .hostname(),
        "localhost"
    );

    let mut server = crate::Vetis::new(config);
    server
        .add_virtual_host(source_virtual_host)
        .await;
    server
        .add_virtual_host(target_virtual_host)
        .await;

    server
        .start()
        .await?;

    let client = deboa::Client::builder()
        .certificate(Certificate::from_slice(CA_CERT, deboa::cert::ContentEncoding::DER))
        .build();

    let response = request::post("https://localhost:9093/")?
        .text("Something cool!")
        .send_with(&client)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .text()
            .await?,
        "Something cool!"
    );

    server
        .stop()
        .await?;

    Ok(())
}

#[cfg(all(feature = "tokio-rt", any(feature = "http1", feature = "http2")))]
#[tokio::test]
async fn test_post_proxy_to_target() -> Result<(), Box<dyn Error>> {
    do_post_proxy_to_target().await
}

#[cfg(all(feature = "smol-rt", any(feature = "http1", feature = "http2")))]
#[apply(test!)]
async fn test_post_proxy_to_target() -> Result<(), Box<dyn Error>> {
    do_post_proxy_to_target().await
}

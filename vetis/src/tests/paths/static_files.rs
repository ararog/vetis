use std::{collections::HashMap, error::Error};

use deboa::{cert::Certificate, request};
use http::StatusCode;
use vetis_core::errors::{ConfigError, VetisError};

#[cfg(feature = "auth")]
use crate::config::server::virtual_host::path::auth::BasicAuthConfig;

#[cfg(feature = "smol-rt")]
use macro_rules_attribute::apply;
#[cfg(feature = "smol-rt")]
use smol_macros::test;

use crate::{
    config::server::{
        virtual_host::{path::static_files::StaticPathConfig, SecurityConfig, VirtualHostConfig},
        ListenerConfig, ServerConfig,
    },
    server::virtual_host::{path::static_files::StaticPath, VirtualHost},
    tests::{default_protocol, CA_CERT, SERVER_CERT, SERVER_KEY},
};

#[test]
fn test_static_path_config() -> Result<(), Box<dyn std::error::Error>> {
    let path_config = StaticPathConfig::builder()
        .uri("/test")
        .extensions(".html")
        .directory("./test")
        .build()?;

    assert_eq!(path_config.uri(), "/test");
    assert_eq!(path_config.directory(), "./test");
    assert_eq!(path_config.extensions(), ".html");

    Ok(())
}

#[test]
fn test_invalid_uri() {
    let some_path = StaticPathConfig::builder()
        .uri("")
        .build();

    assert!(some_path.is_err());
    assert_eq!(
        some_path.err(),
        Some(VetisError::Config(ConfigError::Path("URI cannot be empty".into(),)))
    );
}

#[test]
fn test_invalid_extensions() {
    let some_path = StaticPathConfig::builder()
        .uri("/test")
        .extensions("")
        .build();

    assert!(some_path.is_err());
    assert_eq!(
        some_path.err(),
        Some(VetisError::Config(ConfigError::Path("Extensions cannot be empty".into(),)))
    );
}

#[test]
fn test_invalid_directory() {
    let some_path = StaticPathConfig::builder()
        .uri("/test")
        .extensions(".html")
        .directory("")
        .build();

    assert!(some_path.is_err());
    assert_eq!(
        some_path.err(),
        Some(VetisError::Config(ConfigError::Path("Directory cannot be empty".into(),)))
    );
}

async fn do_index() -> Result<(), Box<dyn Error>> {
    let listener = ListenerConfig::builder()
        .port(9100)
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
        .port(9100)
        .root_directory("src/tests")
        .security(security_config.clone())
        .build()?;

    let mut virtual_host = VirtualHost::new(host_config);
    virtual_host.add_path(StaticPath::new(
        StaticPathConfig::builder()
            .uri("/")
            .directory("src/tests/files")
            .index_files(vec!["index.html".to_string()])
            .build()?,
    ));

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

    let request = request::get("https://localhost:9100/")?
        .send_with(&client)
        .await?;

    assert_eq!(request.status(), http::StatusCode::OK);

    assert_eq!(request
            .text()
            .await?,
            "<html>\n<head>\n  <title>\n    Tested!\n  </title>\n</head>\n<body>\n  <p>\n    Tested!\n  </p>\n</body>\n</html>");

    server
        .stop()
        .await?;

    Ok(())
}

#[cfg(feature = "tokio-rt")]
#[tokio::test]
async fn test_index() -> Result<(), Box<dyn Error>> {
    do_index().await
}

#[cfg(feature = "smol-rt")]
#[apply(test!)]
async fn test_index() -> Result<(), Box<dyn Error>> {
    do_index().await
}

async fn do_not_found() -> Result<(), Box<dyn Error>> {
    let listener = ListenerConfig::builder()
        .port(9000)
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

    let mut status_pages = HashMap::new();
    status_pages.insert(404, "src/tests/files/404.html".to_string());

    let host_config = VirtualHostConfig::builder()
        .hostname("localhost")
        .port(9000)
        .root_directory("src/tests")
        .security(security_config.clone())
        .status_pages(status_pages)
        .build()?;

    let mut virtual_host = VirtualHost::new(host_config);
    virtual_host.add_path(StaticPath::new(
        StaticPathConfig::builder()
            .uri("/")
            .directory("src/tests/files")
            .build()?,
    ));

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

    let request = request::get("https://localhost:9000/some/file/here.txt")?
        .send_with(&client)
        .await;

    assert_eq!(
        request.err(),
        Some(deboa::errors::DeboaError::Response(deboa::errors::ResponseError::Receive {
            status_code: StatusCode::NOT_FOUND,
            message: "Could not process request (404 Not Found): Not Found".to_string()
        }))
    );

    server
        .stop()
        .await?;

    Ok(())
}

#[cfg(feature = "tokio-rt")]
#[tokio::test]
async fn test_not_found() -> Result<(), Box<dyn Error>> {
    do_not_found().await
}

#[cfg(feature = "smol-rt")]
#[apply(test!)]
async fn test_not_found() -> Result<(), Box<dyn Error>> {
    do_not_found().await
}

#[cfg(feature = "auth")]
async fn do_basic_auth(
    username: Option<String>,
    password: Option<String>,
) -> Result<(), Box<dyn Error>> {
    use crate::server::virtual_host::path::auth::{basic_auth::BasicAuth, AuthType};

    let has_auth = username.is_some() && password.is_some();

    let port = if has_auth { 9200 } else { 9201 };

    let listener = ListenerConfig::builder()
        .port(port)
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
        .port(port)
        .root_directory("src/tests")
        .security(security_config.clone())
        .build()?;

    let mut virtual_host = VirtualHost::new(host_config);

    let auth_config = BasicAuthConfig::builder()
        .htpasswd(Some("src/tests/files/.htpasswd".to_string()))
        .cache_users()
        .build()?;

    virtual_host.add_path(StaticPath::new(
        StaticPathConfig::builder()
            .uri("/")
            .directory("src/tests/files")
            .auth(AuthType::Basic(BasicAuth::new(auth_config)))
            .build()?,
    ));

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

    let request = request::get(format!("https://localhost:{}/index.html", port))?;

    let request = if let Some(username) = username {
        if let Some(password) = password {
            request.basic_auth(&username, &password)
        } else {
            request
        }
    } else {
        request
    };

    let response = request
        .send_with(&client)
        .await;

    if !has_auth {
        assert_eq!(
            response.err(),
            Some(deboa::errors::DeboaError::Response(deboa::errors::ResponseError::Receive {
                status_code: StatusCode::UNAUTHORIZED,
                message: "Could not process request (401 Unauthorized): Unauthorized".to_string()
            }))
        );
    } else {
        assert_eq!(response?.status(), StatusCode::OK);
    }

    server
        .stop()
        .await?;

    Ok(())
}

#[cfg(all(feature = "auth", feature = "tokio-rt"))]
#[tokio::test]
async fn test_invalid_basic_auth() -> Result<(), Box<dyn Error>> {
    do_basic_auth(None, None).await
}

#[cfg(all(feature = "auth", feature = "smol-rt"))]
#[apply(test!)]
async fn test_invalid_basic_auth() -> Result<(), Box<dyn Error>> {
    do_basic_auth(None, None).await
}

#[cfg(all(feature = "auth", feature = "tokio-rt"))]
#[tokio::test]
async fn test_valid_basic_auth() -> Result<(), Box<dyn Error>> {
    do_basic_auth(Some("rogerio".to_string()), Some("rpa78@rio!".to_string())).await
}

#[cfg(all(feature = "auth", feature = "smol-rt"))]
#[apply(test!)]
async fn test_valid_basic_auth() -> Result<(), Box<dyn Error>> {
    do_basic_auth(Some("rogerio".to_string()), Some("rpa78@rio!".to_string())).await
}

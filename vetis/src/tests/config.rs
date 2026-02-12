use std::error::Error;

use crate::{
    config::server::{
        virtual_host::{SecurityConfig, VirtualHostConfig},
        ListenerConfig, Protocol, ServerConfig,
    },
    errors::{ConfigError, VetisError},
};

#[test]
fn test_listener_config() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "http1")]
    let protocol = Protocol::Http1;
    #[cfg(feature = "http2")]
    let protocol = Protocol::Http2;
    #[cfg(feature = "http3")]
    let protocol = Protocol::Http3;

    let listener_config = ListenerConfig::builder()
        .port(8080)
        .protocol(protocol.clone())
        .interface("127.0.0.1")
        .build()?;
    assert_eq!(listener_config.port(), 8080);
    assert_eq!(listener_config.protocol(), &protocol);
    assert_eq!(listener_config.interface(), "127.0.0.1");

    Ok(())
}

#[test]
fn test_server_config() -> Result<(), Box<dyn Error>> {
    let server_config = ServerConfig::builder()
        .add_listener(
            ListenerConfig::builder()
                .port(8080)
                .build()?,
        )
        .build()?;
    assert_eq!(
        server_config
            .listeners()
            .len(),
        1
    );

    Ok(())
}

#[test]
fn test_security_config() -> Result<(), Box<dyn Error>> {
    let security_config = SecurityConfig::builder()
        .ca_cert_from_bytes(vec![])
        .cert_from_bytes(vec![])
        .key_from_bytes(vec![])
        .build();

    assert_eq!(
        security_config.err(),
        Some(VetisError::Config(ConfigError::Security("Certificate is empty".to_string())))
    );

    Ok(())
}

#[test]
fn test_virtual_host_config() -> Result<(), Box<dyn std::error::Error>> {
    let virtual_host_config = VirtualHostConfig::builder()
        .hostname("localhost")
        .port(8080)
        .root_directory("src/tests")
        .build()?;
    assert_eq!(virtual_host_config.hostname(), "localhost");
    assert_eq!(virtual_host_config.port(), 8080);

    Ok(())
}

#[test]
fn test_default_virtual_host_config() -> Result<(), Box<dyn std::error::Error>> {
    let virtual_host_config = VirtualHostConfig::builder().build();
    assert_eq!(
        virtual_host_config.err(),
        Some(VetisError::Config(ConfigError::VirtualHost(
            "root_directory does not exist: /var/vetis/www".to_string()
        )))
    );
    Ok(())
}

#[test]
fn test_invalid_virtual_host_config() -> Result<(), Box<dyn std::error::Error>> {
    let virtual_host_config = VirtualHostConfig::builder()
        .hostname("")
        .root_directory("src/tests")
        .build();

    assert_eq!(
        virtual_host_config.err(),
        Some(VetisError::Config(ConfigError::VirtualHost("hostname is not provided".to_string())))
    );
    Ok(())
}

#[cfg(feature = "static-files")]
mod static_files_tests {
    use crate::config::server::virtual_host::path::static_files::StaticPathConfig;

    #[test]
    fn test_static_files_config() -> Result<(), Box<dyn std::error::Error>> {
        let static_files_config = StaticPathConfig::builder()
            .uri("/static")
            .extensions("html,css,js")
            .directory("/var/vetis/www")
            .index_files(vec!["index.html".to_string(), "index.htm".to_string()])
            .build()?;
        assert_eq!(static_files_config.uri(), "/static");
        assert_eq!(static_files_config.extensions(), "html,css,js");
        assert_eq!(static_files_config.directory(), "/var/vetis/www");
        assert_eq!(
            static_files_config.index_files(),
            &Some(vec!["index.html".to_string(), "index.htm".to_string()])
        );
        Ok(())
    }
}

#[cfg(feature = "reverse-proxy")]
mod reverse_proxy_tests {
    use crate::config::server::virtual_host::path::proxy::ProxyPathConfig;

    #[test]
    fn test_reverse_proxy_config() -> Result<(), Box<dyn std::error::Error>> {
        let reverse_proxy_config = ProxyPathConfig::builder()
            .uri("/")
            .target("http://localhost:8081")
            .build()?;
        assert_eq!(reverse_proxy_config.uri(), "/");
        assert_eq!(reverse_proxy_config.target(), "http://localhost:8081");
        Ok(())
    }
}

#[cfg(feature = "auth")]
mod auth_tests {
    use crate::config::server::virtual_host::path::auth::{Algorithm, BasicAuthConfig};

    #[test]
    fn test_auth_config() -> Result<(), Box<dyn std::error::Error>> {
        let auth_config = BasicAuthConfig::builder()
            .algorithm(Algorithm::BCrypt)
            .htpasswd(Some("src/tests/files/.htpasswd".to_string()))
            .build()?;
        assert_eq!(auth_config.algorithm(), &Algorithm::BCrypt);
        assert_eq!(auth_config.htpasswd(), &Some("src/tests/files/.htpasswd".to_string()));
        Ok(())
    }
}

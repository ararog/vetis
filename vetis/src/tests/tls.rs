#[cfg(test)]
mod tls_tests {
    use std::sync::Arc;

    #[cfg(feature = "smol-rt")]
    use macro_rules_attribute::apply;
    #[cfg(feature = "smol-rt")]
    use smol_macros::test;

    #[cfg(feature = "smol-rt")]
    use smol::lock::RwLock;

    #[cfg(feature = "tokio-rt")]
    use tokio::sync::RwLock;

    use crate::{
        config::server::virtual_host::{SecurityConfig, VirtualHostConfig},
        errors::VetisError,
        server::{
            tls::TlsFactory,
            virtual_host::{handler_fn, path::HandlerPath, VirtualHost},
        },
        tests::{CA_CERT, SERVER_CERT, SERVER_KEY},
        VetisVirtualHosts,
    };

    fn create_test_virtual_hosts() -> VetisVirtualHosts {
        let security_config = SecurityConfig::builder()
            .cert_from_bytes(SERVER_CERT.to_vec())
            .key_from_bytes(SERVER_KEY.to_vec())
            .ca_cert_from_bytes(CA_CERT.to_vec())
            .build()
            .expect("Failed to create security config");

        let vhost_config = VirtualHostConfig::builder()
            .hostname("localhost")
            .port(8443)
            .root_directory("src/tests")
            .security(security_config)
            .build()
            .expect("Failed to create virtual host config");

        let mut virtual_host = VirtualHost::new(vhost_config);
        virtual_host.add_path(
            HandlerPath::builder()
                .uri("/")
                .handler(handler_fn(|_req| async move {
                    Ok::<_, VetisError>(
                        crate::Response::builder()
                            .status(http::StatusCode::OK)
                            .text("Test response"),
                    )
                }))
                .build()
                .unwrap(),
        );

        let mut hosts = std::collections::HashMap::new();
        hosts.insert((Arc::from("localhost"), 8443u16), virtual_host);

        Arc::new(RwLock::new(hosts))
    }

    fn create_test_virtual_hosts_no_security() -> VetisVirtualHosts {
        let vhost_config = VirtualHostConfig::builder()
            .hostname("localhost")
            .port(8443)
            .root_directory("src/tests")
            .build()
            .expect("Failed to create virtual host config");

        let mut virtual_host = VirtualHost::new(vhost_config);
        virtual_host.add_path(
            HandlerPath::builder()
                .uri("/")
                .handler(handler_fn(|_req| async move {
                    Ok::<_, VetisError>(
                        crate::Response::builder()
                            .status(http::StatusCode::OK)
                            .text("Test response"),
                    )
                }))
                .build()
                .unwrap(),
        );

        let mut hosts = std::collections::HashMap::new();
        hosts.insert((Arc::from("localhost"), 8443u16), virtual_host);

        Arc::new(RwLock::new(hosts))
    }

    fn create_test_virtual_hosts_invalid_key() -> VetisVirtualHosts {
        let security_config = SecurityConfig::builder()
            .cert_from_bytes(SERVER_CERT.to_vec())
            .key_from_bytes(vec![0x01, 0x02, 0x03]) // Invalid key
            .build()
            .expect("Failed to create security config");

        let vhost_config = VirtualHostConfig::builder()
            .hostname("localhost")
            .port(8443)
            .root_directory("src/tests")
            .security(security_config)
            .build()
            .expect("Failed to create virtual host config");

        let mut virtual_host = VirtualHost::new(vhost_config);
        virtual_host.add_path(
            HandlerPath::builder()
                .uri("/")
                .handler(handler_fn(|_req| async move {
                    Ok::<_, VetisError>(
                        crate::Response::builder()
                            .status(http::StatusCode::OK)
                            .text("Test response"),
                    )
                }))
                .build()
                .unwrap(),
        );

        let mut hosts = std::collections::HashMap::new();
        hosts.insert((Arc::from("localhost"), 8443u16), virtual_host);

        Arc::new(RwLock::new(hosts))
    }

    async fn do_create_tls_config_success() {
        let virtual_hosts = create_test_virtual_hosts();
        let alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

        let result = TlsFactory::create_tls_config(virtual_hosts, alpn_protocols).await;

        assert!(result.is_ok(), "TLS config creation should succeed");
        let tls_config = result.unwrap();
        assert!(tls_config.is_some(), "TLS config should be Some");

        let config = tls_config.unwrap();
        assert_eq!(config.alpn_protocols, vec![b"h2".to_vec(), b"http/1.1".to_vec()]);
        assert_eq!(config.max_early_data_size, u32::MAX);
    }

    #[cfg(feature = "tokio-rt")]
    #[tokio::test]
    async fn test_create_tls_config_success() {
        do_create_tls_config_success().await;
    }

    #[cfg(feature = "smol-rt")]
    #[apply(test!)]
    async fn test_create_tls_config_success() {
        do_create_tls_config_success().await;
    }

    async fn do_create_tls_config_no_security() {
        let virtual_hosts = create_test_virtual_hosts_no_security();
        let alpn_protocols = vec![b"http/1.1".to_vec()];

        let result = TlsFactory::create_tls_config(virtual_hosts, alpn_protocols).await;

        assert!(result.is_ok(), "TLS config creation should succeed even without security");
        let tls_config = result.unwrap();
        assert!(tls_config.is_some(), "TLS config should be Some");

        let config = tls_config.unwrap();
        assert_eq!(config.alpn_protocols, vec![b"http/1.1".to_vec()]);
    }

    #[cfg(feature = "tokio-rt")]
    #[tokio::test]
    async fn test_create_tls_config_no_security() {
        do_create_tls_config_no_security().await;
    }

    #[cfg(feature = "smol-rt")]
    #[apply(test!)]
    async fn test_create_tls_config_no_security() {
        do_create_tls_config_no_security().await;
    }

    async fn do_create_tls_config_invalid_private_key() {
        let virtual_hosts = create_test_virtual_hosts_invalid_key();
        let alpn_protocols = vec![b"http/1.1".to_vec()];

        let result = TlsFactory::create_tls_config(virtual_hosts, alpn_protocols).await;

        assert!(result.is_err(), "TLS config creation should fail with invalid key");
        match result.unwrap_err() {
            VetisError::Start(crate::errors::StartError::Tls(msg)) => {
                assert!(msg.contains("Failed to parse private key"));
            }
            _ => panic!("Expected Tls error"),
        }
    }

    #[cfg(feature = "tokio-rt")]
    #[tokio::test]
    async fn test_create_tls_config_invalid_private_key() {
        do_create_tls_config_invalid_private_key().await;
    }

    #[cfg(feature = "smol-rt")]
    #[apply(test!)]
    async fn test_create_tls_config_invalid_private_key() {
        do_create_tls_config_invalid_private_key().await;
    }

    async fn do_create_tls_config_empty_alpn() {
        let virtual_hosts = create_test_virtual_hosts();
        let alpn_protocols = vec![];

        let result = TlsFactory::create_tls_config(virtual_hosts, alpn_protocols).await;

        assert!(result.is_ok(), "TLS config creation should succeed with empty ALPN");
        let tls_config = result.unwrap();
        assert!(tls_config.is_some(), "TLS config should be Some");

        let config = tls_config.unwrap();
        assert!(
            config
                .alpn_protocols
                .is_empty(),
            "ALPN protocols should be empty"
        );
    }

    #[cfg(feature = "tokio-rt")]
    #[tokio::test]
    async fn test_create_tls_config_empty_alpn() {
        do_create_tls_config_empty_alpn().await;
    }

    #[cfg(feature = "smol-rt")]
    #[apply(test!)]
    async fn test_create_tls_config_empty_alpn() {
        do_create_tls_config_empty_alpn().await;
    }

    async fn do_create_tls_config_multiple_hosts() {
        let mut hosts = std::collections::HashMap::new();

        // Create first virtual host with security
        let security_config1 = SecurityConfig::builder()
            .cert_from_bytes(SERVER_CERT.to_vec())
            .key_from_bytes(SERVER_KEY.to_vec())
            .build()
            .expect("Failed to create security config");

        let vhost_config1 = VirtualHostConfig::builder()
            .hostname("localhost")
            .port(8443)
            .root_directory("src/tests")
            .security(security_config1)
            .build()
            .expect("Failed to create virtual host config");

        let mut virtual_host1 = VirtualHost::new(vhost_config1);
        virtual_host1.add_path(
            HandlerPath::builder()
                .uri("/")
                .handler(handler_fn(|_req| async move {
                    Ok::<_, VetisError>(
                        crate::Response::builder()
                            .status(http::StatusCode::OK)
                            .text("Test response"),
                    )
                }))
                .build()
                .unwrap(),
        );

        // Create second virtual host without security
        let vhost_config2 = VirtualHostConfig::builder()
            .hostname("test.com")
            .port(8443)
            .root_directory("src/tests")
            .build()
            .expect("Failed to create virtual host config");

        let mut virtual_host2 = VirtualHost::new(vhost_config2);
        virtual_host2.add_path(
            HandlerPath::builder()
                .uri("/")
                .handler(handler_fn(|_req| async move {
                    Ok::<_, VetisError>(
                        crate::Response::builder()
                            .status(http::StatusCode::OK)
                            .text("Test response"),
                    )
                }))
                .build()
                .unwrap(),
        );

        hosts.insert((Arc::from("localhost"), 8443), virtual_host1);
        hosts.insert((Arc::from("test.com"), 8443), virtual_host2);

        let virtual_hosts = Arc::new(RwLock::new(hosts));
        let alpn_protocols = vec![b"h2".to_vec()];

        let result = TlsFactory::create_tls_config(virtual_hosts, alpn_protocols).await;

        assert!(result.is_ok(), "TLS config creation should succeed with multiple hosts");
        let tls_config = result.unwrap();
        assert!(tls_config.is_some(), "TLS config should be Some");
    }

    #[cfg(feature = "tokio-rt")]
    #[tokio::test]
    async fn test_create_tls_config_multiple_hosts() {
        do_create_tls_config_multiple_hosts().await;
    }

    #[cfg(feature = "smol-rt")]
    #[apply(test!)]
    async fn test_create_tls_config_multiple_hosts() {
        do_create_tls_config_multiple_hosts().await;
    }

    async fn do_create_tls_config_with_ca_cert() {
        let virtual_hosts = create_test_virtual_hosts();
        let alpn_protocols = vec![b"http/1.1".to_vec()];

        let result = TlsFactory::create_tls_config(virtual_hosts, alpn_protocols).await;

        assert!(result.is_ok(), "TLS config creation should succeed with CA cert");
        let tls_config = result.unwrap();
        assert!(tls_config.is_some(), "TLS config should be Some");

        let config = tls_config.unwrap();
        assert_eq!(config.alpn_protocols, vec![b"http/1.1".to_vec()]);
    }

    #[cfg(feature = "tokio-rt")]
    #[tokio::test]
    async fn test_create_tls_config_with_ca_cert() {
        do_create_tls_config_with_ca_cert().await;
    }

    #[cfg(feature = "smol-rt")]
    #[apply(test!)]
    async fn test_create_tls_config_with_ca_cert() {
        do_create_tls_config_with_ca_cert().await;
    }

    #[test]
    fn test_tls_factory_struct_exists() {
        // This test ensures the TlsFactory struct is accessible
        let _factory = TlsFactory {};
    }
}

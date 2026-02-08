mod server_tests {
    use deboa::{
        cert::{Certificate, ContentEncoding},
        request,
    };
    use http::StatusCode;
    #[cfg(feature = "smol-rt")]
    use macro_rules_attribute::apply;
    #[cfg(feature = "smol-rt")]
    use smol_macros::test;
    use std::error::Error;

    use crate::{
        config::{ListenerConfig, SecurityConfig, ServerConfig, VirtualHostConfig},
        default_protocol,
        server::{
            path::HandlerPath,
            virtual_host::{handler_fn, VirtualHost},
        },
        tests::{CA_CERT, IP6_SERVER_CERT, IP6_SERVER_KEY, SERVER_CERT, SERVER_KEY},
    };

    async fn do_multiple_interfaces() -> Result<(), Box<dyn Error>> {
        let protocol = default_protocol();

        let ipv4 = ListenerConfig::builder()
            .port(8080)
            .protocol(protocol.clone())
            .interface("0.0.0.0")
            .build();

        let ipv6 = ListenerConfig::builder()
            .port(8081)
            .protocol(protocol.clone())
            .interface("::")
            .build();

        let config = ServerConfig::builder()
            .add_listener(ipv4)
            .add_listener(ipv6)
            .build();

        let security_config = SecurityConfig::builder()
            .ca_cert_from_bytes(CA_CERT.to_vec())
            .cert_from_bytes(SERVER_CERT.to_vec())
            .key_from_bytes(SERVER_KEY.to_vec())
            .build();

        let localhost_config = VirtualHostConfig::builder()
            .hostname("localhost")
            .port(8080)
            .security(security_config)
            .build()?;

        let ip6_security_config = SecurityConfig::builder()
            .ca_cert_from_bytes(CA_CERT.to_vec())
            .cert_from_bytes(IP6_SERVER_CERT.to_vec())
            .key_from_bytes(IP6_SERVER_KEY.to_vec())
            .build();

        let ip6_localhost_config = VirtualHostConfig::builder()
            .hostname("ip6-localhost")
            .port(8081)
            .security(ip6_security_config)
            .build()?;

        let mut localhost_virtual_host = VirtualHost::new(localhost_config);
        let mut ip6_localhost_virtual_host = VirtualHost::new(ip6_localhost_config);

        let ip4_root_path = HandlerPath::builder()
            .uri("/hello")
            .handler(handler_fn(|_request| async move {
                let response = crate::Response::builder()
                    .status(StatusCode::OK)
                    .text("Hello from ipv4");
                Ok(response)
            }))
            .build()
            .unwrap();

        let ip6_root_path = HandlerPath::builder()
            .uri("/hello")
            .handler(handler_fn(|_request| async move {
                let response = crate::Response::builder()
                    .status(StatusCode::OK)
                    .text("Hello from ipv6");
                Ok(response)
            }))
            .build()
            .unwrap();

        localhost_virtual_host.add_path(ip4_root_path);
        ip6_localhost_virtual_host.add_path(ip6_root_path);

        let mut server = crate::Vetis::new(config);
        server
            .add_virtual_host(localhost_virtual_host)
            .await;
        server
            .add_virtual_host(ip6_localhost_virtual_host)
            .await;

        server
            .start()
            .await?;

        let client = deboa::Client::builder()
            .certificate(Certificate::from_slice(CA_CERT, ContentEncoding::DER))
            .build();

        let request = request::get("https://localhost:8080/hello")?
            .send_with(&client)
            .await?;

        assert_eq!(request.status(), StatusCode::OK);
        assert_eq!(
            request
                .text()
                .await?,
            "Hello from ipv4"
        );

        let client = deboa::Client::builder()
            .certificate(Certificate::from_slice(CA_CERT, ContentEncoding::DER))
            .bind_addr(
                "::1"
                    .parse()
                    .unwrap(),
            )
            .build();

        let request = request::get("https://ip6-localhost:8081/hello")?
            .send_with(&client)
            .await?;

        assert_eq!(request.status(), StatusCode::OK);
        assert_eq!(
            request
                .text()
                .await?,
            "Hello from ipv6"
        );

        server
            .stop()
            .await?;

        Ok(())
    }

    #[cfg(feature = "tokio-rt")]
    #[tokio::test]
    async fn test_multiple_interfaces() -> Result<(), Box<dyn Error>> {
        do_multiple_interfaces().await
    }

    #[cfg(feature = "smol-rt")]
    #[apply(test!)]
    async fn test_multiple_interfaces_smol() -> Result<(), Box<dyn Error>> {
        do_multiple_interfaces().await
    }
}

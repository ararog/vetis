mod server_tests {
    use std::error::Error;

    use deboa::request;
    use http::StatusCode;

    use crate::{
        config::{ListenerConfig, Protocol, ServerConfig, VirtualHostConfig},
        server::{
            path::HandlerPath,
            virtual_host::{handler_fn, VirtualHost},
        },
    };

    #[tokio::test]
    async fn test_multiple_interfaces() -> Result<(), Box<dyn Error>> {
        let ipv4 = ListenerConfig::builder()
            .port(8080)
            .protocol(Protocol::Http1)
            .interface("0.0.0.0".to_string())
            .build();

        let ipv6 = ListenerConfig::builder()
            .port(8081)
            .protocol(Protocol::Http1)
            .interface("::".to_string())
            .build();

        let config = ServerConfig::builder()
            .add_listener(ipv4)
            .add_listener(ipv6)
            .build();

        let localhost_config = VirtualHostConfig::builder()
            .hostname("localhost".to_string())
            .port(8080)
            .build()?;

        let ip6_localhost_config = VirtualHostConfig::builder()
            .hostname("ip6-localhost".to_string())
            .port(8081)
            .build()?;

        let mut localhost_virtual_host = VirtualHost::new(localhost_config);
        let mut ip6_localhost_virtual_host = VirtualHost::new(ip6_localhost_config);

        let ip4_root_path = HandlerPath::new_host_path(
            "/hello".to_string(),
            handler_fn(|_request| async move {
                let response = crate::Response::builder()
                    .status(StatusCode::OK)
                    .text("Hello from ipv4");
                Ok(response)
            }),
        );

        let ip6_root_path = HandlerPath::new_host_path(
            "/hello".to_string(),
            handler_fn(|_request| async move {
                let response = crate::Response::builder()
                    .status(StatusCode::OK)
                    .text("Hello from ipv6");
                Ok(response)
            }),
        );

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

        let client = deboa::Client::default();

        let request = request::get("http://localhost:8080/hello")?
            .send_with(&client)
            .await?;

        assert_eq!(request.status(), StatusCode::OK);
        assert_eq!(
            request
                .text()
                .await?,
            "Hello from ipv4"
        );

        let request = request::get("http://ip6-localhost:8081/hello")?
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
}

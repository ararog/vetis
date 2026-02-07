mod handler {
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
    pub async fn test_handler() -> Result<(), Box<dyn std::error::Error>> {
        let ipv4 = ListenerConfig::builder()
            .port(8082)
            .protocol(Protocol::Http1)
            .interface("0.0.0.0")
            .build();

        let config = ServerConfig::builder()
            .add_listener(ipv4)
            .build();

        let localhost_config = VirtualHostConfig::builder()
            .hostname("localhost")
            .port(8082)
            .build()?;

        let mut localhost_virtual_host = VirtualHost::new(localhost_config);

        let root_path = HandlerPath::builder()
            .uri("/hello")
            .handler(handler_fn(|_request| async move {
                let response = crate::Response::builder()
                    .status(StatusCode::OK)
                    .text("Hello from localhost");
                Ok(response)
            }))
            .build()
            .unwrap();

        localhost_virtual_host.add_path(root_path);

        let mut server = crate::Vetis::new(config);
        server
            .add_virtual_host(localhost_virtual_host)
            .await;

        server
            .start()
            .await?;

        let client = deboa::Client::default();

        let request = request::get("http://localhost:8082/hello")?
            .send_with(&client)
            .await?;

        assert_eq!(request.status(), StatusCode::OK);

        server
            .stop()
            .await?;

        Ok(())
    }
}

#[cfg(feature = "static-files")]
mod static_files {
    use crate::{
        config::StaticPathConfig,
        errors::{ConfigError, VetisError},
    };

    #[test]
    pub fn test_static_path_config() -> Result<(), Box<dyn std::error::Error>> {
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
    pub fn test_invalid_uri() {
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
    pub fn test_invalid_extensions() {
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
    pub fn test_invalid_directory() {
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
}

#[cfg(feature = "reverse-proxy")]
mod reverse_proxy {
    use std::error::Error;

    use deboa::request;
    use http::StatusCode;

    use crate::{
        config::{ListenerConfig, Protocol, ProxyPathConfig, ServerConfig, VirtualHostConfig},
        errors::{ConfigError, VetisError},
        server::{
            path::HandlerPath,
            virtual_host::{handler_fn, VirtualHost},
        },
    };

    use crate::server::path::ProxyPath;

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

    #[tokio::test]
    async fn test_proxy_to_target() -> Result<(), Box<dyn Error>> {
        let source_listener = ListenerConfig::builder()
            .port(8084)
            .protocol(Protocol::Http1)
            .interface("0.0.0.0")
            .build();

        let target_listener = ListenerConfig::builder()
            .port(8085)
            .protocol(Protocol::Http1)
            .interface("0.0.0.0")
            .build();

        let config = ServerConfig::builder()
            .add_listener(source_listener)
            .add_listener(target_listener)
            .build();

        let source_config = VirtualHostConfig::builder()
            .hostname("localhost")
            .port(8084)
            .build()
            .unwrap();

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
            .build()
            .unwrap();

        let mut target_virtual_host = VirtualHost::new(target_config);
        target_virtual_host.add_path(
            HandlerPath::builder()
                .uri("/")
                .handler(handler_fn(|_request| async move {
                    Ok(crate::Response::builder()
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

        let client = deboa::Client::default();

        let request = request::get("http://localhost:8084/")?
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
}

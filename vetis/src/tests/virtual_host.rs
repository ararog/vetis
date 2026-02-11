mod virtual_host_tests {
    use bytes::Bytes;
    use http::StatusCode;
    use http_body_util::{BodyExt, Full};
    #[cfg(feature = "smol-rt")]
    use macro_rules_attribute::apply;
    #[cfg(feature = "smol-rt")]
    use smol_macros::test;

    use crate::{
        config::VirtualHostConfig,
        server::{
            path::HandlerPath,
            virtual_host::{handler_fn, VirtualHost},
        },
        Request,
    };

    async fn do_add_virtual_host() -> Result<(), Box<dyn std::error::Error>> {
        let config = VirtualHostConfig::builder()
            .hostname("localhost")
            .root_directory("src/tests")
            .build()
            .unwrap();

        let mut virtual_host = VirtualHost::new(config);
        virtual_host.add_path(
            HandlerPath::builder()
                .uri("/")
                .handler(handler_fn(|_request| async move {
                    Ok(crate::Response::builder()
                        .status(StatusCode::OK)
                        .text("Hello, world!"))
                }))
                .build()
                .unwrap(),
        );

        assert_eq!(
            virtual_host
                .config()
                .hostname(),
            "localhost"
        );

        Ok(())
    }

    #[cfg(feature = "tokio-rt")]
    #[tokio::test]
    async fn test_add_virtual_host() -> Result<(), Box<dyn std::error::Error>> {
        do_add_virtual_host().await
    }

    #[cfg(feature = "smol-rt")]
    #[apply(test!)]
    async fn test_add_virtual_host() -> Result<(), Box<dyn std::error::Error>> {
        do_add_virtual_host().await
    }

    async fn do_handle_request() -> Result<(), Box<dyn std::error::Error>> {
        let config = VirtualHostConfig::builder()
            .hostname("localhost")
            .root_directory("src/tests")
            .build()
            .unwrap();

        let mut virtual_host = VirtualHost::new(config);
        virtual_host.add_path(
            HandlerPath::builder()
                .uri("/")
                .handler(handler_fn(|_request| async move {
                    Ok(crate::Response::builder()
                        .status(StatusCode::OK)
                        .text("Hello, world!"))
                }))
                .build()
                .unwrap(),
        );

        assert_eq!(
            virtual_host
                .config()
                .hostname(),
            "localhost"
        );

        let request = http::Request::builder()
            .uri("/")
            .body(Full::new(Bytes::from(b"Test".to_vec())))
            .unwrap();

        let request = Request::from_quic(request);

        let response = virtual_host
            .route(request)
            .await?;

        let (parts, body) = response
            .into_inner()
            .into_parts();
        assert_eq!(parts.status, StatusCode::OK);
        assert_eq!(
            body.collect()
                .await
                .unwrap()
                .to_bytes()
                .as_ref(),
            b"Hello, world!"
        );

        Ok(())
    }

    #[cfg(feature = "tokio-rt")]
    #[tokio::test]
    async fn test_handle_request() -> Result<(), Box<dyn std::error::Error>> {
        do_handle_request().await
    }

    #[cfg(feature = "smol-rt")]
    #[apply(test!)]
    async fn test_handle_request() -> Result<(), Box<dyn std::error::Error>> {
        do_handle_request().await
    }
}

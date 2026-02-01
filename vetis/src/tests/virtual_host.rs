mod virtual_host_tests {
    use bytes::Bytes;
    use http::StatusCode;
    use http_body_util::{BodyExt, Full};

    use crate::config::VirtualHostConfig;
    use crate::server::path::HandlerPath;
    use crate::server::virtual_host::{handler_fn, VirtualHost};
    use crate::Request;

    #[tokio::test]
    async fn test_add_virtual_host() -> Result<(), Box<dyn std::error::Error>> {
        let config = VirtualHostConfig::builder()
            .hostname("localhost".to_string())
            .build()
            .unwrap();

        let mut virtual_host = VirtualHost::new(config);
        virtual_host.add_path(HandlerPath::new_host_path(
            "/".to_string(),
            handler_fn(|_request| async move {
                Ok(crate::Response::builder()
                    .status(StatusCode::OK)
                    .text("Hello, world!"))
            }),
        ));

        assert_eq!(
            virtual_host
                .config()
                .hostname(),
            "localhost"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_handle_request() -> Result<(), Box<dyn std::error::Error>> {
        let config = VirtualHostConfig::builder()
            .hostname("localhost".to_string())
            .build()
            .unwrap();

        let mut virtual_host = VirtualHost::new(config);
        virtual_host.add_path(HandlerPath::new_host_path(
            "/".to_string(),
            handler_fn(|_request| async move {
                Ok(crate::Response::builder()
                    .status(StatusCode::OK)
                    .text("Hello, world!"))
            }),
        ));

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
}

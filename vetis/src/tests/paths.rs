use crate::{
    errors::{VetisError, VirtualHostError},
    server::path::{HostPath, StaticPath},
};

#[test]
pub fn test_static_path() {
    let some_path = StaticPath::builder()
        .uri("/test".to_string())
        .extensions(".html".to_string())
        .directory("./test".to_string())
        .build();

    let Ok(HostPath::Static(static_path)) = some_path else {
        panic!("Failed to build static path");
    };

    assert_eq!(static_path.uri(), "/test");
    assert_eq!(static_path.directory(), "./test");
    assert_eq!(static_path.extensions(), ".html");
}

#[test]
pub fn test_invalid_uri() {
    let some_path = StaticPath::builder().build();

    assert!(some_path.is_err());
    assert_eq!(
        some_path.err(),
        Some(VetisError::VirtualHost(VirtualHostError::InvalidPath(
            "URI cannot be empty".to_string(),
        )))
    );
}

#[test]
pub fn test_invalid_extensions() {
    let some_path = StaticPath::builder()
        .uri("/test".to_string())
        .build();

    assert!(some_path.is_err());
    assert_eq!(
        some_path.err(),
        Some(VetisError::VirtualHost(VirtualHostError::InvalidPath(
            "Extensions cannot be empty".to_string(),
        )))
    );
}

#[test]
pub fn test_invalid_directory() {
    let some_path = StaticPath::builder()
        .uri("/test".to_string())
        .extensions(".html".to_string())
        .build();

    assert!(some_path.is_err());
    assert_eq!(
        some_path.err(),
        Some(VetisError::VirtualHost(VirtualHostError::InvalidPath(
            "Directory cannot be empty".to_string(),
        )))
    );
}

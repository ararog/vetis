use crate::server::config::{SecurityConfig, ServerConfig};

#[cfg(test)]
mod server_config_tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.port(), 80);
        assert_eq!(config.interface(), "0.0.0.0");
    }

    #[test]
    fn test_server_config_builder_default_values() {
        let config = ServerConfig::builder().build();
        assert_eq!(config.port(), 0);
        assert_eq!(config.interface(), "0.0.0.0");
    }

    #[test]
    fn test_server_config_builder_with_port() {
        let config = ServerConfig::builder()
            .port(8080)
            .build();
        assert_eq!(config.port(), 8080);
        assert_eq!(config.interface(), "0.0.0.0");
    }

    #[test]
    fn test_server_config_builder_with_interface() {
        let config = ServerConfig::builder()
            .interface("127.0.0.1".to_string())
            .build();
        assert_eq!(config.port(), 0);
        assert_eq!(config.interface(), "127.0.0.1");
    }

    #[test]
    fn test_server_config_set_port() {
        let mut config = ServerConfig::default();
        assert_eq!(config.port(), 80);

        config.set_port(9090);
        assert_eq!(config.port(), 9090);
    }

    #[test]
    fn test_server_config_clone() {
        let original = ServerConfig::builder()
            .port(1234)
            .interface("192.168.1.1".to_string())
            .build();

        let cloned = original.clone();
        assert_eq!(original.port(), cloned.port());
        assert_eq!(original.interface(), cloned.interface());
    }
}

#[cfg(test)]
mod security_config_tests {
    use super::*;

    #[test]
    fn test_security_config_builder_with_cert_bytes() {
        let cert_data = vec![10, 20, 30, 40];
        let config = SecurityConfig::builder()
            .cert_from_bytes(cert_data.clone())
            .build();

        assert_eq!(config.cert(), &cert_data);
        assert!(config
            .ca_cert()
            .is_none());
        assert!(!config.client_auth());
    }

    #[test]
    fn test_security_config_builder_with_key_bytes() {
        let key_data = vec![50, 60, 70, 80];
        let config = SecurityConfig::builder()
            .key_from_bytes(key_data.clone())
            .build();

        assert_eq!(config.key(), &key_data);
        assert!(config
            .ca_cert()
            .is_none());
        assert!(!config.client_auth());
    }

    #[test]
    fn test_security_config_builder_with_ca_cert_bytes() {
        let ca_cert_data = vec![90, 100, 110, 120];
        let config = SecurityConfig::builder()
            .ca_cert_from_bytes(ca_cert_data.clone())
            .build();
        assert_eq!(config.ca_cert(), &Some(ca_cert_data));
        assert!(!config.client_auth());
    }

    #[test]
    fn test_security_config_builder_with_client_auth() {
        let config = SecurityConfig::builder()
            .client_auth(true)
            .build();

        assert!(config.client_auth());
    }

    #[test]
    fn test_security_config_builder_complete() {
        let cert_data = vec![1, 2, 3, 4];
        let key_data = vec![5, 6, 7, 8];
        let ca_cert_data = vec![9, 10, 11, 12];

        let config = SecurityConfig::builder()
            .cert_from_bytes(cert_data.clone())
            .key_from_bytes(key_data.clone())
            .ca_cert_from_bytes(ca_cert_data.clone())
            .client_auth(true)
            .build();

        assert_eq!(config.cert(), &cert_data);
        assert_eq!(config.key(), &key_data);
        assert_eq!(config.ca_cert(), &Some(ca_cert_data));
        assert!(config.client_auth());
    }

    #[test]
    fn test_security_config_builder_chaining() {
        let config = SecurityConfig::builder()
            .cert_from_bytes(vec![1, 2, 3])
            .key_from_bytes(vec![4, 5, 6])
            .ca_cert_from_bytes(vec![7, 8, 9])
            .client_auth(false)
            .build();

        assert_eq!(config.cert(), &vec![1, 2, 3]);
        assert_eq!(config.key(), &vec![4, 5, 6]);
        assert_eq!(config.ca_cert(), &Some(vec![7, 8, 9]));
        assert!(!config.client_auth());
    }

    #[test]
    fn test_security_config_clone() {
        let original = SecurityConfig::builder()
            .cert_from_bytes(vec![10, 20])
            .key_from_bytes(vec![30, 40])
            .client_auth(true)
            .build();

        let cloned = original.clone();
        assert_eq!(original.cert(), cloned.cert());
        assert_eq!(original.key(), cloned.key());
        assert_eq!(original.ca_cert(), cloned.ca_cert());
        assert_eq!(original.client_auth(), cloned.client_auth());
    }

    #[test]
    fn test_security_config_empty_data() {
        let config = SecurityConfig::builder()
            .cert_from_bytes(vec![])
            .key_from_bytes(vec![])
            .ca_cert_from_bytes(vec![])
            .build();

        assert_eq!(config.cert(), &vec![]);
        assert_eq!(config.key(), &vec![]);
        assert_eq!(config.ca_cert(), &Some(vec![]));
        assert!(!config.client_auth());
    }

    #[test]
    fn test_security_config_large_data() {
        let large_cert = vec![0u8; 100];
        let large_key = vec![1u8; 235];
        let large_ca_cert = vec![2u8; 255];

        let config = SecurityConfig::builder()
            .cert_from_bytes(large_cert.clone())
            .key_from_bytes(large_key.clone())
            .ca_cert_from_bytes(large_ca_cert.clone())
            .client_auth(true)
            .build();

        assert_eq!(config.cert(), &large_cert);
        assert_eq!(config.key(), &large_key);
        assert_eq!(config.ca_cert(), &Some(large_ca_cert));
        assert!(config.client_auth());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_server_config_with_security_integration() {
        let security_config = SecurityConfig::builder()
            .cert_from_bytes(
                b"-----BEGIN CERTIFICATE-----\nMOCK CERT\n-----END CERTIFICATE-----".to_vec(),
            )
            .key_from_bytes(
                b"-----BEGIN PRIVATE KEY-----\nMOCK KEY\n-----END PRIVATE KEY-----".to_vec(),
            )
            .ca_cert_from_bytes(
                b"-----BEGIN CERTIFICATE-----\nMOCK CA\n-----END CERTIFICATE-----".to_vec(),
            )
            .client_auth(true)
            .build();

        assert!(security_config.client_auth());
    }

    #[test]
    fn test_multiple_config_instances() {
        let config1 = ServerConfig::builder()
            .port(8080)
            .interface("127.0.0.1".to_string())
            .build();

        let config2 = ServerConfig::builder()
            .port(9090)
            .interface("192.168.1.1".to_string())
            .build();

        assert_ne!(config1.port(), config2.port());
        assert_ne!(config1.interface(), config2.interface());
    }

    #[test]
    fn test_config_immutability() {
        let config = ServerConfig::builder()
            .port(3000)
            .interface("localhost".to_string())
            .build();

        // Original config should remain unchanged
        assert_eq!(config.port(), 3000);
        assert_eq!(config.interface(), "localhost");

        // Creating a new config shouldn't affect the original
        let _new_config = ServerConfig::builder()
            .port(4000)
            .build();

        assert_eq!(config.port(), 3000);
        assert_eq!(config.interface(), "localhost");
    }
}

use std::sync::Arc;

use crate::{
    errors::{StartError::Tls, VetisError},
    VetisVirtualHosts,
};

use rustls::{
    pki_types::{CertificateDer, PrivateKeyDer},
    server::ResolvesServerCertUsingSni,
    sign::CertifiedKey,
    ServerConfig,
};

pub struct TlsFactory {}

impl TlsFactory {
    pub async fn create_tls_config(
        virtual_hosts: VetisVirtualHosts,
        alpn_protocols: Vec<Vec<u8>>,
    ) -> Result<Option<ServerConfig>, VetisError> {
        let virtual_hosts = virtual_hosts.clone();
        #[cfg(feature = "__rustls_awc_lc_rs")]
        let provider = rustls::crypto::aws_lc_rs::default_provider();
        #[cfg(feature = "__rustls_ring")]
        let provider = rustls::crypto::ring::default_provider();
        #[cfg(feature = "__rustls_rustcrypto")]
        let provider = rustls_rustcrypto::provider();
        let mut resolver = ResolvesServerCertUsingSni::new();
        let virtual_hosts = virtual_hosts
            .read()
            .await;
        for (hostname, virtual_host) in virtual_hosts.iter() {
            if let Some(security) = virtual_host
                .config()
                .security()
            {
                let cert = security.cert();
                let key = security.key();

                let cert = CertificateDer::try_from(cert.to_vec())
                    .map_err(|_| Tls("Failed to parse certificate".to_string()))?;
                let mut chain = vec![cert];
                if let Some(ca_cert) = security.ca_cert() {
                    let ca_cert = CertificateDer::try_from(ca_cert.to_vec())
                        .map_err(|_| Tls("Failed to parse CA certificate".to_string()))?;
                    chain.push(ca_cert);
                }

                let key = PrivateKeyDer::try_from(key.to_vec())
                    .map_err(|_| Tls("Failed to parse private key".to_string()))?;
                let certified_key = CertifiedKey::from_der(chain, key, &provider)
                    .map_err(|_| Tls("Failed to create certified key".to_string()))?;

                let hostname = hostname.0.clone();

                resolver
                    .add(&hostname, certified_key)
                    .map_err(|e| Tls(e.to_string()))?;
            }
        }

        let builder = rustls::ServerConfig::builder_with_provider(Arc::new(provider))
            .with_protocol_versions(&[&rustls::version::TLS13])
            .map_err(|e| VetisError::Start(Tls(e.to_string())))?;

        let mut tls_config = builder
            .with_no_client_auth()
            .with_cert_resolver(Arc::new(resolver));

        tls_config.max_early_data_size = u32::MAX;
        tls_config.alpn_protocols = alpn_protocols;

        Ok(Some(tls_config))
    }
}

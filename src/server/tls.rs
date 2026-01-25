use std::{collections::HashMap, future::Future, sync::Arc};

use crate::{
    server::{
        errors::{StartError::Tls, VetisError},
        virtual_host::VirtualHost,
    },
    RequestType, ResponseType, VetisRwLock,
};

use rustls::{
    pki_types::{CertificateDer, PrivateKeyDer},
    server::ResolvesServerCertUsingSni,
    sign::CertifiedKey,
};
#[cfg(feature = "tokio-rt")]
use tokio_rustls::TlsAcceptor;

#[cfg(feature = "smol-rt")]
use futures_rustls::TlsAcceptor;

#[cfg(feature = "tokio-rt")]
type VetisTlsAcceptor = TlsAcceptor;

#[cfg(feature = "smol-rt")]
type VetisTlsAcceptor = TlsAcceptor;

pub struct TlsFactory {}

impl TlsFactory {
    pub async fn create_tls_acceptor(
        virtual_hosts: Arc<
            VetisRwLock<HashMap<String, Box<dyn VirtualHost + Send + Sync + 'static>>>,
        >,
        alpn_protocols: Vec<u8>,
    ) -> Result<Option<VetisTlsAcceptor>, VetisError> {
        let virtual_hosts = virtual_hosts.clone();
        let provider = rustls::crypto::aws_lc_rs::default_provider();
        let mut resolver = ResolvesServerCertUsingSni::new();
        while let Some((hostname, virtual_host)) = virtual_hosts
            .read()
            .await
            .iter()
            .next()
        {
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

                resolver
                    .add(hostname, certified_key)
                    .map_err(|_| Tls("Failed to add certified key".to_string()))?;
            }
        }

        let builder = rustls::ServerConfig::builder_with_provider(Arc::new(provider))
            .with_protocol_versions(&[&rustls::version::TLS13])
            .map_err(|e| VetisError::Start(Tls(e.to_string())))?;

        let mut tls_config = builder
            .with_no_client_auth()
            .with_cert_resolver(Arc::new(resolver));

        tls_config.max_early_data_size = u32::MAX;
        tls_config.alpn_protocols = vec![alpn_protocols];

        Ok(Some(VetisTlsAcceptor::from(Arc::new(tls_config))))
    }
}

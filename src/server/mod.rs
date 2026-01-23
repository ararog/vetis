use std::{future::Future, sync::Arc};

use ::http::{Request, Response};
use rustls::{
    pki_types::{CertificateDer, PrivateKeyDer},
    server::WebPkiClientVerifier,
    RootCertStore,
};

use crate::{
    server::config::SecurityConfig,
    server::errors::{StartError::Tls, VetisError},
};

#[cfg(any(feature = "http1", feature = "http2"))]
pub mod http;
#[cfg(feature = "http3")]
pub mod quic;

#[cfg(any(feature = "http1", feature = "http2"))]
pub mod tcp;
#[cfg(feature = "http3")]
pub mod udp;

pub mod config;
pub mod errors;
pub mod virtual_host;

pub trait Server<RequestBody, ResponseBody> {
    fn port(&self) -> u16;

    fn setup_tls(
        &self,
        config: &SecurityConfig,
        alpn_protocols: Vec<u8>,
    ) -> Result<rustls::server::ServerConfig, VetisError> {
        let cert = if let Some(cert) = config.cert() {
            cert.clone()
        } else {
            return Err(VetisError::Start(Tls("No certificate provided".to_string())));
        };

        let key = if let Some(key) = config.key() {
            key.clone()
        } else {
            return Err(VetisError::Start(Tls("No key provided".to_string())));
        };

        let cert = CertificateDer::from(cert);

        let key =
            PrivateKeyDer::try_from(key).map_err(|e| VetisError::Start(Tls(e.to_string())))?;

        let provider = rustls::crypto::aws_lc_rs::default_provider();
        let builder = rustls::ServerConfig::builder_with_provider(Arc::new(provider))
            .with_protocol_versions(&[&rustls::version::TLS13])
            .map_err(|e| VetisError::Start(Tls(e.to_string())))?;

        let builder = if config.client_auth() {
            let ca_cert = if let Some(ca_cert) = config.ca_cert() {
                ca_cert.clone()
            } else {
                return Err(VetisError::Start(Tls("No CA certificate provided".to_string())));
            };

            let mut store = RootCertStore::empty();
            let cert = CertificateDer::from(ca_cert);
            store
                .add(cert)
                .unwrap();

            let client_verifier = WebPkiClientVerifier::builder(Arc::new(store))
                .build()
                .unwrap();
            builder.with_client_cert_verifier(client_verifier)
        } else {
            builder.with_no_client_auth()
        };

        let mut tls_config = builder
            .with_single_cert(vec![cert], key)
            .map_err(|e| VetisError::Start(Tls(e.to_string())))?;

        tls_config.max_early_data_size = u32::MAX;
        tls_config.alpn_protocols = vec![alpn_protocols];

        Ok(tls_config)
    }

    fn start<H, Fut>(&mut self, handler: H) -> impl Future<Output = Result<(), VetisError>>
    where
        H: Fn(Request<RequestBody>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response<ResponseBody>, VetisError>> + Send + 'static;

    fn stop(&mut self) -> impl Future<Output = Result<(), VetisError>>;
}

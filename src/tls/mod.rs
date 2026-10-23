use std::sync::Arc;

use rustls::ClientConfig;
use rustls::crypto::aws_lc_rs;

mod no_certificate_verification;

use no_certificate_verification::NoCertificateVerification;

pub fn unverified_client_config() -> Result<Arc<ClientConfig>, rustls::Error> {
    let provider = Arc::new(aws_lc_rs::default_provider());

    let config = ClientConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoCertificateVerification(provider)))
        .with_no_client_auth();

    Ok(Arc::new(config))
}

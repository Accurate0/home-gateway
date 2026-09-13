use std::net::IpAddr;
use std::time::Duration;

use reqwest::redirect::Policy;
use reqwest::{Method, Url};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, RequestBuilder};
use reqwest_tracing::TracingMiddleware;

use super::public_resolver::{PublicResolver, is_public};
use super::{HttpCreationError, TimeTrace};

const MAX_REDIRECTS: usize = 10;

#[derive(Clone)]
pub struct PublicHttpClient(ClientWithMiddleware);

impl PublicHttpClient {
    pub fn new(timeout: Duration) -> Result<Self, HttpCreationError> {
        let redirects = Policy::custom(|attempt| {
            if attempt.previous().len() >= MAX_REDIRECTS {
                return attempt.error("too many redirects");
            }

            match ensure_public_url(attempt.url()) {
                Ok(()) => attempt.follow(),
                Err(error) => attempt.error(error),
            }
        });

        let client = reqwest::ClientBuilder::new()
            .timeout(timeout)
            .no_proxy()
            .dns_resolver(PublicResolver)
            .redirect(redirects)
            .build()?;

        Ok(Self(
            ClientBuilder::new(client)
                .with(TracingMiddleware::<TimeTrace>::new())
                .build(),
        ))
    }

    pub fn request(&self, method: Method, raw: &str) -> Result<RequestBuilder, String> {
        let url = Url::parse(raw).map_err(|error| format!("invalid url `{raw}`: {error}"))?;

        ensure_public_url(&url)?;

        Ok(self.0.request(method, url))
    }
}

fn ensure_public_url(url: &Url) -> Result<(), String> {
    if !matches!(url.scheme(), "http" | "https") {
        return Err(format!("unsupported url scheme `{}`", url.scheme()));
    }

    let Some(host) = url.host_str() else {
        return Err(format!("url `{url}` has no host"));
    };

    let Ok(address) = host
        .trim_start_matches('[')
        .trim_end_matches(']')
        .parse::<IpAddr>()
    else {
        return Ok(());
    };

    if is_public(address) {
        Ok(())
    } else {
        tracing::warn!("refusing a request to non-public address {address}");

        Err(format!("`{address}` is not a public address"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(raw: &str) -> Result<(), String> {
        ensure_public_url(&Url::parse(raw).expect("expected a valid url"))
    }

    #[test]
    fn public_urls_pass() {
        assert!(check("https://api.example.com/v1").is_ok());
        assert!(check("http://1.1.1.1/").is_ok());
    }

    #[test]
    fn private_ip_literals_are_refused() {
        for raw in [
            "http://10.43.0.10:8123/api",
            "http://127.0.0.1/",
            "http://0x7f.1/",
            "http://2130706433/",
            "http://[::1]/",
            "http://[::ffff:192.168.1.1]/",
            "http://169.254.169.254/latest/meta-data",
        ] {
            assert!(check(raw).is_err(), "{raw} should be refused");
        }
    }

    #[test]
    fn non_http_schemes_are_refused() {
        assert!(check("file:///etc/passwd").is_err());
    }
}

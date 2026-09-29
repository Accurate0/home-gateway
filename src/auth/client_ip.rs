use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};

use axum::extract::{ConnectInfo, FromRequestParts};
use http::request::Parts;
use http::{Extensions, HeaderMap};

const REAL_IP_HEADER: &str = "x-real-ip";

pub struct ClientIp(pub Option<IpAddr>);

impl<S> FromRequestParts<S> for ClientIp
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(ClientIp(client_ip(&parts.headers, &parts.extensions)))
    }
}

pub fn client_ip(headers: &HeaderMap, extensions: &Extensions) -> Option<IpAddr> {
    let forwarded = headers
        .get(REAL_IP_HEADER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse().ok());

    forwarded.or_else(|| {
        extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| addr.ip())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_real_ip_header_wins_over_the_peer_address() {
        let mut headers = HeaderMap::new();
        headers.insert(REAL_IP_HEADER, "203.0.113.7".parse().unwrap());

        let mut extensions = Extensions::new();
        extensions.insert(ConnectInfo::<SocketAddr>("10.0.0.5:4000".parse().unwrap()));

        assert_eq!(
            client_ip(&headers, &extensions),
            Some("203.0.113.7".parse().unwrap())
        );
    }

    #[test]
    fn the_peer_address_is_used_without_the_header() {
        let mut extensions = Extensions::new();
        extensions.insert(ConnectInfo::<SocketAddr>("10.0.0.5:4000".parse().unwrap()));

        assert_eq!(
            client_ip(&HeaderMap::new(), &extensions),
            Some("10.0.0.5".parse().unwrap())
        );
    }

    #[test]
    fn an_unparseable_header_falls_back_to_the_peer_address() {
        let mut headers = HeaderMap::new();
        headers.insert(REAL_IP_HEADER, "not an ip".parse().unwrap());

        let mut extensions = Extensions::new();
        extensions.insert(ConnectInfo::<SocketAddr>("10.0.0.5:4000".parse().unwrap()));

        assert_eq!(
            client_ip(&headers, &extensions),
            Some("10.0.0.5".parse().unwrap())
        );
    }
}

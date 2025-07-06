use axum::extract::FromRequestParts;
use http::{StatusCode, request::Parts};
use itertools::Itertools;
use std::net::{IpAddr, SocketAddr};
use tokio::net::lookup_host;

pub struct RequireIpAuth;

impl<S> FromRequestParts<S> for RequireIpAuth
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if cfg!(debug_assertions) {
            return Ok(Self);
        }

        let remote_addr = parts
            .extensions
            .get::<axum::extract::ConnectInfo<SocketAddr>>()
            .map(|ci| ci.0);

        match remote_addr {
            Some(ip_header) if is_ip_valid(ip_header.ip()).await => Ok(Self),
            _ => Err(StatusCode::UNAUTHORIZED),
        }
    }
}

async fn is_ip_valid(ip: IpAddr) -> bool {
    match lookup_host("home-ip.anurag.sh:80").await {
        Ok(it) => it.map(|s| s.ip()).contains(&ip),
        Err(e) => {
            tracing::error!("error looking up host: {e}");
            false
        }
    }
}

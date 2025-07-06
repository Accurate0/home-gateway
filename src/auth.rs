use axum::extract::FromRequestParts;
use http::{StatusCode, request::Parts};
use itertools::Itertools;
use std::{net::IpAddr, str::FromStr};
use tokio::net::lookup_host;

// An extractor that performs authorization.
pub struct RequireAuth;

impl<S> FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let ip_header = parts
            .headers
            .get("CF-Connecting-IP")
            .and_then(|value| value.to_str().ok())
            .and_then(|v| IpAddr::from_str(v).ok());

        match ip_header {
            Some(ip_header) if is_ip_valid(ip_header).await => Ok(Self),
            _ => Err(StatusCode::UNAUTHORIZED),
        }
    }
}

async fn is_ip_valid(ip: IpAddr) -> bool {
    if cfg!(debug_assertions) {
        return true;
    }

    match lookup_host("home-ip.anurag.sh:80").await {
        Ok(it) => it.map(|s| s.ip()).contains(&ip),
        Err(e) => {
            tracing::error!("error looking up host: {e}");
            false
        }
    }
}

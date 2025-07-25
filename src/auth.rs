use crate::types::ApiState;
use axum::extract::FromRequestParts;
use http::{StatusCode, request::Parts};
use itertools::Itertools;
use std::{net::IpAddr, str::FromStr};
use tokio::net::lookup_host;

pub struct RequireIpAuth;

impl FromRequestParts<ApiState> for RequireIpAuth {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ApiState,
    ) -> Result<Self, Self::Rejection> {
        if cfg!(debug_assertions) {
            return Ok(Self);
        }

        let feature_flag_client = &state.feature_flag_client;
        let is_access_restricted = feature_flag_client
            .is_feature_enabled("home-gateway-restrict-access-to-home-ip", true)
            .await;

        if !is_access_restricted {
            return Ok(Self);
        }

        let ip_header = parts
            .headers
            .get("X-Forwarded-For")
            .and_then(|value| value.to_str().ok())
            .and_then(|s| s.split(",").next())
            .map(|s| s.trim())
            .and_then(|v| IpAddr::from_str(v).ok());

        match ip_header {
            Some(ip_header) if is_ip_valid(ip_header).await => Ok(Self),
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

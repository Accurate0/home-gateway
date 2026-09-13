use std::io::{Error, ErrorKind};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use reqwest::dns::{Addrs, Name, Resolve, Resolving};

use super::nightly;

#[derive(Clone, Copy, Default)]
pub struct PublicResolver;

impl Resolve for PublicResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().to_owned();

        Box::pin(async move {
            let addrs: Vec<SocketAddr> = tokio::net::lookup_host((host.as_str(), 0))
                .await?
                .filter(|addr| is_public(addr.ip()))
                .collect();

            if addrs.is_empty() {
                tracing::warn!("refusing to connect to `{host}`: it has no public address");

                return Err(Error::new(
                    ErrorKind::PermissionDenied,
                    format!("`{host}` does not resolve to a public address"),
                )
                .into());
            }

            Ok(Box::new(addrs.into_iter()) as Addrs)
        })
    }
}

pub fn is_public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => !ip.is_multicast() && nightly::is_global_v4(ip),
        IpAddr::V6(ip) => match embedded_ipv4(ip) {
            Some(inner) => !inner.is_multicast() && nightly::is_global_v4(inner),
            None => !ip.is_multicast() && nightly::is_global_v6(ip),
        },
    }
}

fn embedded_ipv4(ip: Ipv6Addr) -> Option<Ipv4Addr> {
    let [s0, s1, s2, s3, s4, s5, s6, s7] = ip.segments();

    let compatible = [s0, s1, s2, s3, s4, s5] == [0; 6] && (s6 != 0 || s7 > 1);
    let nat64 = [s0, s1, s2, s3, s4, s5] == [0x64, 0xff9b, 0, 0, 0, 0];

    (compatible || nat64)
        .then(|| Ipv4Addr::new((s6 >> 8) as u8, s6 as u8, (s7 >> 8) as u8, s7 as u8))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn public(raw: &str) -> bool {
        is_public(raw.parse().expect("expected a valid ip"))
    }

    #[test]
    fn public_addresses_are_allowed() {
        for raw in ["1.1.1.1", "142.250.70.14", "2606:4700:4700::1111"] {
            assert!(public(raw), "{raw} should be public");
        }
    }

    #[test]
    fn cluster_and_private_addresses_are_refused() {
        for raw in [
            "10.43.0.1",
            "172.16.5.4",
            "192.168.1.10",
            "127.0.0.1",
            "169.254.169.254",
            "100.64.0.1",
            "0.0.0.0",
            "255.255.255.255",
            "::1",
            "::",
            "fd00::1",
            "fe80::1",
            "ff0e::1",
        ] {
            assert!(!public(raw), "{raw} should not be public");
        }
    }

    #[test]
    fn private_addresses_wrapped_in_ipv6_are_refused() {
        for raw in [
            "::ffff:10.0.0.1",
            "::ffff:127.0.0.1",
            "::127.0.0.1",
            "64:ff9b::a00:1",
            "2002:a00:1::",
            "2001:0:0:0:0:0:a00:1",
        ] {
            assert!(!public(raw), "{raw} should not be public");
        }
    }
}

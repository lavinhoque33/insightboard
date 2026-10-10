//! Status-widget outbound policy (D1, M4 decision 5): which addresses a user-supplied URL may
//! reach. The check runs on IP-literal hosts before sending and, through [`PolicyResolver`], on
//! every address DNS returns, so the address actually dialled is always one that passed.

use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};

/// Whether `ip` is outside the public unicast space the status widget may reach.
pub fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_v4(v4),
        IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
            Some(v4) => is_blocked_v4(v4),
            None => is_blocked_v6(v6),
        },
    }
}

fn in_v4(ip: Ipv4Addr, net: [u8; 4], prefix: u32) -> bool {
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    (u32::from(ip) & mask) == (u32::from(Ipv4Addr::from(net)) & mask)
}

/// IANA special-purpose IPv4 ranges that are not globally reachable, plus multicast/reserved.
const BLOCKED_V4: [([u8; 4], u32); 15] = [
    ([0, 0, 0, 0], 8),       // "this network", unspecified
    ([10, 0, 0, 0], 8),      // private
    ([100, 64, 0, 0], 10),   // CGNAT shared space (incl. 100.100.100.200 metadata)
    ([127, 0, 0, 0], 8),     // loopback
    ([169, 254, 0, 0], 16),  // link-local (incl. 169.254.169.254 metadata)
    ([172, 16, 0, 0], 12),   // private
    ([192, 0, 0, 0], 24),    // IETF protocol assignments
    ([192, 0, 2, 0], 24),    // documentation TEST-NET-1
    ([192, 88, 99, 0], 24),  // deprecated 6to4 relay anycast
    ([192, 168, 0, 0], 16),  // private
    ([198, 18, 0, 0], 15),   // benchmarking
    ([198, 51, 100, 0], 24), // documentation TEST-NET-2
    ([203, 0, 113, 0], 24),  // documentation TEST-NET-3
    ([224, 0, 0, 0], 4),     // multicast
    ([240, 0, 0, 0], 4),     // reserved, incl. 255.255.255.255 broadcast
];

fn is_blocked_v4(ip: Ipv4Addr) -> bool {
    BLOCKED_V4
        .iter()
        .any(|(net, prefix)| in_v4(ip, *net, *prefix))
}

fn in_v6(ip: Ipv6Addr, net: [u16; 8], prefix: u32) -> bool {
    let mask = if prefix == 0 {
        0
    } else {
        u128::MAX << (128 - prefix)
    };
    (u128::from(ip) & mask) == (u128::from(Ipv6Addr::from(net)) & mask)
}

/// Only global unicast (`2000::/3`) is allowed, minus its special-purpose blocks. Everything
/// else — unspecified, loopback, IPv4-compatible, NAT64, ULA (incl. `fd00:ec2::254` metadata),
/// link-local, site-local, multicast — is blocked.
fn is_blocked_v6(ip: Ipv6Addr) -> bool {
    const BLOCKED_IN_GLOBAL: [([u16; 8], u32); 4] = [
        ([0x2001, 0, 0, 0, 0, 0, 0, 0], 23), // IETF protocol assignments (Teredo, benchmarking, ORCHID)
        ([0x2001, 0x0db8, 0, 0, 0, 0, 0, 0], 32), // documentation
        ([0x2002, 0, 0, 0, 0, 0, 0, 0], 16), // 6to4 (embeds arbitrary IPv4)
        ([0x3fff, 0, 0, 0, 0, 0, 0, 0], 20), // documentation
    ];
    let global = in_v6(ip, [0x2000, 0, 0, 0, 0, 0, 0, 0], 3);
    !global
        || BLOCKED_IN_GLOBAL
            .iter()
            .any(|(net, prefix)| in_v6(ip, *net, *prefix))
}

/// The host resolved only to blocked addresses.
#[derive(Debug)]
pub struct BlockedAddress;

impl fmt::Display for BlockedAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("host resolves only to blocked addresses")
    }
}

impl std::error::Error for BlockedAddress {}

/// The host did not resolve.
#[derive(Debug)]
pub struct DnsFailure;

impl fmt::Display for DnsFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("host did not resolve")
    }
}

impl std::error::Error for DnsFailure {}

/// DNS resolver for the status client: system resolution, then every blocked address is
/// dropped. A host with no allowed address fails with [`BlockedAddress`].
pub struct PolicyResolver;

impl Resolve for PolicyResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().to_string();
        Box::pin(async move {
            let resolved: Vec<SocketAddr> = tokio::net::lookup_host((host.as_str(), 0))
                .await
                .map_err(|_| DnsFailure)?
                .collect();
            if resolved.is_empty() {
                return Err(DnsFailure.into());
            }
            let allowed: Vec<SocketAddr> = resolved
                .into_iter()
                .filter(|addr| !is_blocked_ip(addr.ip()))
                .collect();
            if allowed.is_empty() {
                return Err(BlockedAddress.into());
            }
            let addrs: Addrs = Box::new(allowed.into_iter());
            Ok(addrs)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{is_blocked_ip, BlockedAddress, PolicyResolver};
    use reqwest::dns::{Name, Resolve};
    use std::{net::IpAddr, str::FromStr};

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn every_rejected_class_is_blocked() {
        for addr in [
            // loopback
            "127.0.0.1",
            "127.255.255.254",
            "::1",
            // unspecified / this network
            "0.0.0.0",
            "0.1.2.3",
            "::",
            // private
            "10.0.0.1",
            "172.16.0.1",
            "172.31.255.255",
            "192.168.1.1",
            // CGNAT
            "100.64.0.1",
            "100.127.255.255",
            // link-local and metadata
            "169.254.0.1",
            "169.254.169.254",
            "fe80::1",
            "fd00:ec2::254",
            // ULA
            "fc00::1",
            "fdff::1",
            // multicast / broadcast / reserved
            "224.0.0.1",
            "239.255.255.250",
            "255.255.255.255",
            "240.0.0.1",
            "ff02::1",
            // documentation / benchmarking / protocol assignments
            "192.0.2.1",
            "198.51.100.7",
            "203.0.113.9",
            "198.18.0.1",
            "198.19.255.255",
            "192.0.0.8",
            "2001:db8::1",
            "3fff::1",
            "2001:2::1",
            // IPv4-mapped and embedding forms of blocked addresses
            "::ffff:127.0.0.1",
            "::ffff:169.254.169.254",
            "::ffff:10.0.0.1",
            "::127.0.0.1",
            "64:ff9b::7f00:1",
            "2002:7f00:1::1",
            "2001::1",
            // site-local (deprecated)
            "fec0::1",
        ] {
            assert!(is_blocked_ip(ip(addr)), "{addr} must be blocked");
        }
    }

    #[test]
    fn public_unicast_is_allowed() {
        // Classification only; nothing is dialled.
        for addr in [
            "93.184.215.14",
            "1.1.1.1",
            "8.8.8.8",
            "172.32.0.1",
            "100.128.0.1",
            "::ffff:93.184.215.14",
            "2606:2800:21f:cb07:6820:80da:af6b:8b2c",
        ] {
            assert!(!is_blocked_ip(ip(addr)), "{addr} must be allowed");
        }
    }

    #[tokio::test]
    async fn resolver_rejects_names_that_resolve_to_loopback() {
        // `localhost` resolves from /etc/hosts; no external DNS query.
        let err = PolicyResolver
            .resolve(Name::from_str("localhost").unwrap())
            .await
            .err()
            .expect("localhost must be rejected");
        assert!(err.downcast_ref::<BlockedAddress>().is_some(), "{err}");
    }
}

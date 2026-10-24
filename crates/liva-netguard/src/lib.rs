//! LIVA Banking — `liva-netguard` Crate
//!
//! On-premise zero-egress network isolation and firewall enforcement.
//! Guarantees zero byte leakage beyond local machine loopback interface.

use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NetguardError {
    #[error("Egress blocked: Attempted network connection to unauthorized destination {target_ip}:{target_port}")]
    EgressBlocked {
        target_ip: String,
        target_port: u16,
        reason: String,
    },

    #[error("Invalid IP address string: {0}")]
    InvalidIp(String),
}

/// Zero-Egress Network Enforcement Policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroEgressPolicy {
    pub allow_loopback_only: bool,
    pub enforce_strict_drop: bool,
}

impl Default for ZeroEgressPolicy {
    fn default() -> Self {
        Self {
            allow_loopback_only: true,
            enforce_strict_drop: true,
        }
    }
}

/// Local Network Egress Verifier.
pub struct LocalEgressVerifier;

impl LocalEgressVerifier {
    /// Evaluates whether a destination address is strictly inside loopback.
    pub fn is_allowed(addr: &SocketAddr) -> bool {
        match addr.ip() {
            IpAddr::V4(ipv4) => ipv4.is_loopback(),
            IpAddr::V6(ipv6) => ipv6.is_loopback(),
        }
    }

    /// Verifies destination and returns Err if non-loopback egress is attempted.
    pub fn verify_connection(ip_str: &str, port: u16) -> Result<(), NetguardError> {
        let ip: IpAddr = ip_str.parse().map_err(|_| NetguardError::InvalidIp(ip_str.to_string()))?;
        let addr = SocketAddr::new(ip, port);

        if Self::is_allowed(&addr) {
            Ok(())
        } else {
            Err(NetguardError::EgressBlocked {
                target_ip: ip_str.to_string(),
                target_port: port,
                reason: "Destination violates Decree 13 and Circular 09 Zero-Egress invariant".to_string(),
            })
        }
    }

    /// Returns authoritative nftables firewall configuration for host-level isolation.
    pub fn generate_nftables_ruleset() -> &'static str {
        r#"#!/usr/sbin/nft -f
# LIVA Banking Harness — Zero Data Egress nftables Configuration
# Strict Air-Gap Isolation for On-Premise Daemon

flush ruleset

table inet liva_filter {
    chain output {
        type filter hook output priority 0; policy drop;

        # Allow traffic on loopback interface only
        oifname "lo" accept

        # Reject and log all outbound connection attempts
        log prefix "[LIVA-ZERO-EGRESS-BLOCKED] " drop
    }

    chain input {
        type filter hook input priority 0; policy drop;

        # Allow traffic on loopback interface only
        iifname "lo" accept
    }

    chain forward {
        type filter hook forward priority 0; policy drop;
    }
}
"#
    }

    /// Returns seccomp profile JSON specification for Linux container sandbox.
    pub fn generate_seccomp_profile() -> &'static str {
        r#"{
  "defaultAction": "SCMP_ACT_ERRNO",
  "architectures": [
    "SCMP_ARCH_X86_64",
    "SCMP_ARCH_AARCH64"
  ],
  "syscalls": [
    {
      "names": [
        "read", "write", "open", "openat", "close", "stat", "fstat", "lstat",
        "poll", "lseek", "mmap", "mprotect", "munmap", "brk", "rt_sigaction",
        "rt_sigprocmask", "rt_sigreturn", "ioctl", "futex", "epoll_create1",
        "epoll_ctl", "epoll_wait", "clock_gettime", "getrandom"
      ],
      "action": "SCMP_ACT_ALLOW"
    }
  ]
}"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loopback_allowed() {
        assert!(LocalEgressVerifier::verify_connection("127.0.0.1", 8080).is_ok());
        assert!(LocalEgressVerifier::verify_connection("127.0.0.1", 5432).is_ok());
        assert!(LocalEgressVerifier::verify_connection("::1", 8080).is_ok());
    }

    #[test]
    fn test_external_egress_strictly_blocked() {
        // Public Cloud / Internet
        let google_dns = LocalEgressVerifier::verify_connection("8.8.8.8", 53);
        assert!(matches!(google_dns, Err(NetguardError::EgressBlocked { .. })));

        let cloudflare = LocalEgressVerifier::verify_connection("1.1.1.1", 443);
        assert!(matches!(cloudflare, Err(NetguardError::EgressBlocked { .. })));

        // LAN / Private Cloud Egress
        let lan_gw = LocalEgressVerifier::verify_connection("192.168.1.1", 80);
        assert!(matches!(lan_gw, Err(NetguardError::EgressBlocked { .. })));

        let corporate_internal = LocalEgressVerifier::verify_connection("10.0.0.1", 443);
        assert!(matches!(corporate_internal, Err(NetguardError::EgressBlocked { .. })));
    }

    #[test]
    fn test_nftables_and_seccomp_generation() {
        let nft = LocalEgressVerifier::generate_nftables_ruleset();
        assert!(nft.contains("policy drop"));
        assert!(nft.contains("oifname \"lo\" accept"));

        let seccomp = LocalEgressVerifier::generate_seccomp_profile();
        assert!(seccomp.contains("SCMP_ACT_ERRNO"));
    }
}

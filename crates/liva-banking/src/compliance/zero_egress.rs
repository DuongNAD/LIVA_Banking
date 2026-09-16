//! Zero-Egress Network Policy & Traffic Guard.
//!
//! Strictly enforces on-premise air-gap network policy:
//! - Only loopback (127.0.0.1, ::1) or specified internal LAN endpoints are allowed
//! - Zero external internet egress bytes permitted
//! - Immediate security violation error on non-permitted outbound destination

use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EgressDecision {
    AllowedLoopback,
    AllowedInternalLan,
    BlockedExternal { destination: String },
}

pub struct ZeroEgressNetfilter {
    pub allowed_subnets: Vec<String>,
}

impl ZeroEgressNetfilter {
    pub fn new() -> Self {
        Self {
            allowed_subnets: vec!["127.0.0.1".to_string(), "::1".to_string(), "localhost".to_string()],
        }
    }

    pub fn inspect_destination(&self, dest: &str) -> EgressDecision {
        let clean = dest.trim().to_lowercase();
        let host = clean.split(':').next().unwrap_or(&clean);

        if host == "127.0.0.1" || host == "::1" || host == "localhost" {
            return EgressDecision::AllowedLoopback;
        }

        if let Ok(ip) = host.parse::<IpAddr>() {
            match ip {
                IpAddr::V4(ipv4) => {
                    if ipv4.is_loopback() {
                        return EgressDecision::AllowedLoopback;
                    }
                    if ipv4.is_private() {
                        return EgressDecision::AllowedInternalLan;
                    }
                }
                IpAddr::V6(ipv6) => {
                    if ipv6.is_loopback() {
                        return EgressDecision::AllowedLoopback;
                    }
                }
            }
        }

        EgressDecision::BlockedExternal {
            destination: dest.to_string(),
        }
    }

    pub fn assert_zero_egress(&self, dest: &str) -> Result<(), String> {
        match self.inspect_destination(dest) {
            EgressDecision::AllowedLoopback | EgressDecision::AllowedInternalLan => Ok(()),
            EgressDecision::BlockedExternal { destination } => {
                Err(format!("SecurityEgressViolation: Outbound call to '{destination}' blocked by Zero-Egress policy"))
            }
        }
    }
}

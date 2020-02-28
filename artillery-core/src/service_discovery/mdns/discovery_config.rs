use crate::constants::*;
use std::net::SocketAddr;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct MDNSServiceDiscoveryConfig {
    pub reply_ttl: Duration,
    pub local_service_addr: SocketAddr,
}

impl Default for MDNSServiceDiscoveryConfig {
    fn default() -> Self {
        let local_service_addr = SocketAddr::from(([127, 0, 0, 1], CONST_INFECTION_PORT));

        Self {
            reply_ttl: Duration::from_secs(120),
            local_service_addr,
        }
    }
}

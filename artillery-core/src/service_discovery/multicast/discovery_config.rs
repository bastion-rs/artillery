use crate::constants::*;
use chrono::Duration;
use std::net::{SocketAddr, ToSocketAddrs};

pub struct MulticastServiceDiscoveryConfig {
    pub timeout_delta: Duration,
    pub seeking_addr: SocketAddr,
    pub discovery_addr: SocketAddr,
}

impl Default for MulticastServiceDiscoveryConfig {
    fn default() -> Self {
        let discovery_addr = SocketAddr::from(([0, 0, 0, 0], CONST_SERVICE_DISCOVERY_PORT));
        let seeking_addr = SocketAddr::from(([255, 255, 255, 255], CONST_SERVICE_DISCOVERY_PORT));

        Self {
            timeout_delta: Duration::seconds(1),
            //            packet_size: CONST_PACKET_SIZE,
            //            ping_request_host_count: 3,
            //            discovery_timeout: Duration::seconds(3),
            seeking_addr: seeking_addr.to_socket_addrs().unwrap().next().unwrap(),
            discovery_addr: discovery_addr.to_socket_addrs().unwrap().next().unwrap(),
        }
    }
}

use crate::constants::*;
use chrono::Duration;
use std::net::{SocketAddr, ToSocketAddrs};

#[derive(Debug, Clone)]
pub struct ClusterConfig {
    pub cluster_key: Vec<u8>,
    pub ping_interval: Duration,
    pub network_mtu: usize,
    pub ping_request_host_count: usize,
    pub ping_timeout: Duration,
    pub listen_addr: SocketAddr,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        let directed = SocketAddr::from(([127, 0, 0, 1], CONST_INFECTION_PORT));

        ClusterConfig {
            cluster_key: b"default".to_vec(),
            ping_interval: Duration::seconds(1),
            network_mtu: CONST_PACKET_SIZE,
            ping_request_host_count: 3,
            ping_timeout: Duration::seconds(3),
            listen_addr: directed.to_socket_addrs().unwrap().next().unwrap(),
        }
    }
}

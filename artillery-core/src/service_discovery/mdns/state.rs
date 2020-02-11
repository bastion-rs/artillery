use std::net::SocketAddr;

pub struct MDNSServiceDiscoveryEvent(pub SocketAddr);

impl MDNSServiceDiscoveryEvent {
    pub fn get(&self) -> SocketAddr {
        self.0
    }
}
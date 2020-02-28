use std::net::SocketAddr;

pub struct MDNSServiceDiscoveryEvent(pub SocketAddr);

unsafe impl Send for MDNSServiceDiscoveryEvent {}
unsafe impl Sync for MDNSServiceDiscoveryEvent {}

impl MDNSServiceDiscoveryEvent {
    pub fn get(&self) -> SocketAddr {
        self.0
    }
}

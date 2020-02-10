use crate::errors::*;
use crate::service_discovery::multicast::discovery_config::MulticastServiceDiscoveryConfig;
use crate::service_discovery::multicast::state::MulticastServiceDiscoveryState;
use crate::service_discovery::multicast::state::{ServiceDiscoveryReply, ServiceDiscoveryRequest};
use cuneiform_fields::arch::ArchPadding;
use std::sync::mpsc;
use std::sync::mpsc::{channel, Sender, Receiver};

pub struct MulticastServiceDiscovery {
    comm: ArchPadding<Sender<ServiceDiscoveryRequest>>,
}

impl MulticastServiceDiscovery {
    pub fn new_service_discovery(config: MulticastServiceDiscoveryConfig, discovery_reply: ServiceDiscoveryReply) -> Result<Self> {
        let (internal_tx, mut internal_rx) = channel::<ServiceDiscoveryRequest>();
        let (poll, state) =
            MulticastServiceDiscoveryState::new(config, discovery_reply)?;

        debug!("Starting Artillery Multicast SD");
        std::thread::Builder::new()
            .name("artillery-mcast-service-discovery-state".to_string())
            .spawn(move || {
                MulticastServiceDiscoveryState::event_loop(&mut internal_rx, poll, state)
                    .expect("Failed to create event loop");
            })
            .expect("cannot start multicast service discovery state thread");

        Ok(Self {
            comm: ArchPadding::new(internal_tx),
        })
    }

    /// Register a new observer to be notified whenever we
    /// successfully find peers by interrogating the network.
    pub fn register_seeker(
        &self,
        observer: mpsc::Sender<ServiceDiscoveryReply>,
    ) -> Result<()> {
        let observer = ArchPadding::new(observer);
        Ok(self
            .comm
            .send(ServiceDiscoveryRequest::RegisterObserver(observer))?)
    }

    /// Enable or disable listening and responding to peers searching for us. This will
    /// correspondingly allow or disallow others from finding us by interrogating the network.
    pub fn set_listen_for_peers(&self, listen: bool) -> Result<()> {
        Ok(self.comm.send(ServiceDiscoveryRequest::SetBroadcastListen(listen))?)
    }

    /// Explore the network to find nodes using multicast SD.
    pub fn seek_peers(&self) -> Result<()> {
        Ok(self.comm.send(ServiceDiscoveryRequest::SeekPeers)?)
    }

    /// Shutdown Service Discovery
    pub fn shutdown(&self) -> Result<()> {
        Ok(std::mem::drop(self))
    }
}

unsafe impl Send for MulticastServiceDiscovery {}
unsafe impl Sync for MulticastServiceDiscovery {}

impl Drop for MulticastServiceDiscovery {
    fn drop(&mut self) {
        let (tx, rx) = channel();

        self.comm.send(ServiceDiscoveryRequest::Exit(tx)).unwrap();

        rx.recv().unwrap();
    }
}

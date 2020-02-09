use crate::errors::*;
use crate::service_discovery::multicast::discovery_config::MulticastServiceDiscoveryConfig;
use crate::service_discovery::multicast::state::MulticastServiceDiscoveryState;
use crate::service_discovery::multicast::state::{ServiceDiscoveryReply, ServiceDiscoveryRequest};
use cuneiform_fields::arch::ArchPadding;
use std::sync::mpsc;
use std::sync::mpsc::{channel, Sender};

pub struct MulticastServiceDiscovery {
    comm: ArchPadding<Sender<ServiceDiscoveryRequest>>,
}

impl MulticastServiceDiscovery {
    pub fn new_service_discovery(config: MulticastServiceDiscoveryConfig) -> Result<Self> {
        let (internal_tx, mut internal_rx) = channel::<ServiceDiscoveryRequest>();
        let (poll, state) = MulticastServiceDiscoveryState::new(config, internal_tx.clone())?;

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
    pub fn register_seek_peer_observer(
        &self,
        observer: mpsc::Sender<ServiceDiscoveryReply>,
    ) -> Result<()> {
        let observer = ArchPadding::new(observer);
        Ok(self
            .comm
            .send(ServiceDiscoveryRequest::RegisterObserver(observer))?)
    }

    /// Explore the network to find nodes using multicast SD.
    /// Return value indicates acknowledgement of the request.
    pub fn seek_peers(&self) -> Result<()> {
        let (tx, rx) = channel();
        self.comm.send(ServiceDiscoveryRequest::SeekPeers(tx))?;
        Ok(rx.recv()?)
    }
}

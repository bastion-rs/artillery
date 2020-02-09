use crate::service_discovery::multicast::discovery_config::MulticastServiceDiscoveryConfig;
use cuneiform_fields::arch::ArchPadding;
use std::sync::mpsc::{channel, Receiver, Sender};
use crate::errors::*;
use crate::service_discovery::multicast::state::{MulticastServiceDiscoveryState, ServiceDiscoveryRequest, ServiceDiscoveryReply};

pub struct MulticastServiceDiscovery<T>
where
    T: ServiceDiscoveryReply
{
    pub events: ArchPadding<Receiver<ServiceDiscoveryEvent>>,
    comm: ArchPadding<Sender<ServiceDiscoveryRequest<T>>>,
}

impl<T> MulticastServiceDiscovery<T>
where
    T: ServiceDiscoveryReply
{
    pub fn new_service_discovery(config: MulticastServiceDiscoveryConfig) -> Result<Self> {
        let (event_tx, event_rx) = channel::<ServiceDiscoveryEvent>();
        let (internal_tx, mut internal_rx) = channel::<ServiceDiscoveryRequest<T>>();

        let (poll, state) = MulticastServiceDiscoveryState::new(
            config,
            event_tx,
            internal_tx.clone()
        )?;

        debug!("Starting Artillery Multicast SD");
        std::thread::Builder::new()
            .name("artillery-mcast-service-discovery-state".to_string())
            .spawn(move || {
                MulticastServiceDiscoveryState::event_loop(&mut internal_rx, poll, state)
                    .expect("Failed to create event loop");
            })
            .expect("cannot start multicast service discovery state thread");


        Ok(Self {
            events: ArchPadding::new(event_rx),
            comm: ArchPadding::new(internal_tx)
        })
    }

    /// Explore the network to find nodes using multicast SD.
    /// Return value indicates acknowledgement of the request.
    pub fn seek_peers(&self) -> Result<()> {
        let (tx, rx) = channel();
        self.comm.send(ServiceDiscoveryRequest::SeekPeers(tx))?;
        Ok(rx.recv()?)
    }
}


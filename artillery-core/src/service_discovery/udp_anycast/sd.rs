use crate::errors::*;
use crate::service_discovery::udp_anycast::discovery_config::MulticastServiceDiscoveryConfig;
use crate::service_discovery::udp_anycast::state::MulticastServiceDiscoveryState;
use crate::service_discovery::udp_anycast::state::{
    ServiceDiscoveryReply, ServiceDiscoveryRequest,
};
use bastion_executor::blocking::spawn_blocking;
use cuneiform_fields::arch::ArchPadding;
use lightproc::proc_stack::ProcStack;
use std::sync::mpsc;
use std::sync::mpsc::{channel, Sender};

pub struct MulticastServiceDiscovery {
    comm: ArchPadding<Sender<ServiceDiscoveryRequest>>,
}

impl MulticastServiceDiscovery {
    pub fn new_service_discovery(
        config: MulticastServiceDiscoveryConfig,
        discovery_reply: ServiceDiscoveryReply,
    ) -> Result<Self> {
        let (internal_tx, mut internal_rx) = channel::<ServiceDiscoveryRequest>();
        let (poll, state) = MulticastServiceDiscoveryState::new(config, discovery_reply)?;

        debug!("Starting Artillery Multicast SD");
        let _multicast_sd_handle = spawn_blocking(
            async move {
                MulticastServiceDiscoveryState::event_loop(&mut internal_rx, poll, state)
                    .expect("Failed to create event loop");
            },
            ProcStack::default(),
        );

        Ok(Self {
            comm: ArchPadding::new(internal_tx),
        })
    }

    /// Register a new observer to be notified whenever we
    /// successfully find peers by interrogating the network.
    pub fn register_seeker(&self, observer: mpsc::Sender<ServiceDiscoveryReply>) -> Result<()> {
        let observer = ArchPadding::new(observer);
        Ok(self
            .comm
            .send(ServiceDiscoveryRequest::RegisterObserver(observer))?)
    }

    /// Enable or disable listening and responding to peers searching for us. This will
    /// correspondingly allow or disallow others from finding us by interrogating the network.
    pub fn set_listen_for_peers(&self, listen: bool) -> Result<()> {
        Ok(self
            .comm
            .send(ServiceDiscoveryRequest::SetBroadcastListen(listen))?)
    }

    /// Explore the network to find nodes using `udp_anycast` SD.
    pub fn seek_peers(&self) -> Result<()> {
        Ok(self.comm.send(ServiceDiscoveryRequest::SeekPeers)?)
    }

    /// Shutdown Service Discovery
    pub fn shutdown(&mut self) -> Result<()> {
        self.discovery_exit();
        Ok(())
    }

    fn discovery_exit(&mut self) {
        let (tx, rx) = channel();
        self.comm.send(ServiceDiscoveryRequest::Exit(tx)).unwrap();
        rx.recv().unwrap();
    }
}

unsafe impl Send for MulticastServiceDiscovery {}
unsafe impl Sync for MulticastServiceDiscovery {}

impl Drop for MulticastServiceDiscovery {
    fn drop(&mut self) {
        self.discovery_exit();
    }
}

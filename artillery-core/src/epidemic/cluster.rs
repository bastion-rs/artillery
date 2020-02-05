use uuid::Uuid;
use crate::epidemic::cluster_config::ClusterConfig;
use std::net::SocketAddr;
use std::sync::mpsc::{channel, Receiver, Sender};
use super::state::ArtilleryState;
use crate::epidemic::state::{ArtilleryClusterRequest, ArtilleryClusterEvent};
use crate::errors::*;

pub struct Cluster {
    pub events: Receiver<ArtilleryClusterEvent>,
    comm: Sender<ArtilleryClusterRequest>,
}

impl Cluster {
    pub fn new_cluster(host_key: Uuid, config: ClusterConfig) -> Result<Self> {
        let (event_tx, event_rx) = channel::<ArtilleryClusterEvent>();
        let (internal_tx, mut internal_rx) = channel::<ArtilleryClusterRequest>();

        let mut state = ArtilleryState::new(host_key, config, event_tx, internal_tx.clone())?;

        std::thread::Builder::new()
            .name("artillery-epidemic-cluster-state".to_string())
            .spawn(move || {
                ArtilleryState::event_loop(&mut internal_rx, &mut state)
                    .expect("Failed to create event loop");
            })
            .expect("cannot start epidemic cluster state management thread");

        Ok(Self {
            events: event_rx,
            comm: internal_tx
        })
    }

    pub fn add_seed_node(&self, addr: SocketAddr) {
        self.comm.send(ArtilleryClusterRequest::AddSeed(addr)).unwrap();
    }

    pub fn leave_cluster(&self) {
        self.comm.send(ArtilleryClusterRequest::LeaveCluster).unwrap();
    }
}

impl Drop for Cluster {
    fn drop(&mut self) {
        let (tx, rx) = channel();

        self.comm.send(ArtilleryClusterRequest::Exit(tx)).unwrap();

        rx.recv().unwrap();
    }
}

//#[inline]
//pub(crate) fn cluster_state() -> &'static ArtilleryState {
//    lazy_static! {
//        static ref CLUSTER_STATE: ArtilleryState = {
//            std::thread::Builder::new()
//                .name("artillery-epidemic-cluster-state".to_string())
//                .spawn(move || {
//                    ArtilleryState::event_loop()
//                        .expect("Failed to create event loop");
//                })
//                .expect("cannot start epidemic cluster state management thread");
//        }
//
//        ArtilleryState::new()
//    }
//
//    &*CLUSTER_STATE
//}
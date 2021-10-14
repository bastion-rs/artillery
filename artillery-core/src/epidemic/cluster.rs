use super::state::ArtilleryEpidemic;
use crate::epidemic::cluster_config::ClusterConfig;
use crate::epidemic::state::{ArtilleryClusterEvent, ArtilleryClusterRequest};
use crate::errors::*;
use bastion_executor::prelude::*;
use lightproc::{proc_stack::ProcStack, recoverable_handle::RecoverableHandle};
use serde::Serialize;
use std::net::SocketAddr;
use std::{
    future::Future,
    pin::Pin,
    sync::mpsc::{channel, Receiver, Sender},
    task::{Context, Poll},
};
use uuid::Uuid;

#[derive(Debug)]
pub struct Cluster {
    pub events: Receiver<ArtilleryClusterEvent>,
    comm: Sender<ArtilleryClusterRequest>,
}

impl Cluster {
    pub fn new_cluster(
        host_key: Uuid,
        config: ClusterConfig,
    ) -> Result<(Self, RecoverableHandle<()>)> {
        let (event_tx, event_rx) = channel::<ArtilleryClusterEvent>();
        let (internal_tx, mut internal_rx) = channel::<ArtilleryClusterRequest>();

        let (poll, state) =
            ArtilleryEpidemic::new(host_key, config, event_tx, internal_tx.clone())?;

        debug!("Starting Artillery Cluster");
        let cluster_handle = spawn_blocking(
            async move {
                ArtilleryEpidemic::event_loop(&mut internal_rx, poll, state)
                    .expect("Failed to create event loop");
            },
            ProcStack::default(),
        );

        Ok((
            Self {
                events: event_rx,
                comm: internal_tx,
            },
            cluster_handle,
        ))
    }

    pub fn add_seed_node(&self, addr: SocketAddr) {
        let _ = self.comm.send(ArtilleryClusterRequest::AddSeed(addr));
    }

    pub fn send_payload<T: Serialize>(&self, id: Uuid, msg: &T) {
        self.comm
            .send(ArtilleryClusterRequest::Payload(
                id,
                bincode::serialize(msg).expect("Failed to serialize payload"),
            ))
            .unwrap();
    }

    pub fn leave_cluster(&self) {
        let _ = self.comm.send(ArtilleryClusterRequest::LeaveCluster);
    }
}

impl Future for Cluster {
    type Output = ArtilleryClusterEvent;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        match self.events.recv() {
            Ok(kv) => Poll::Ready(kv),
            Err(_) => Poll::Pending,
        }
    }
}

unsafe impl Send for Cluster {}
unsafe impl Sync for Cluster {}

impl Drop for Cluster {
    fn drop(&mut self) {
        let (tx, rx) = channel();

        let _ = self.comm.send(ArtilleryClusterRequest::Exit(tx));

        rx.recv().unwrap();
    }
}

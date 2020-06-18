use crate::epidemic::prelude::*;
use crate::errors::*;
use crate::service_discovery::mdns::prelude::*;

use lightproc::prelude::*;

use futures::{select, FutureExt};
use pin_utils::pin_mut;
use std::{cell::Cell, sync::Arc};
use uuid::Uuid;

#[derive(Default, Debug, Clone)]
pub struct ArtilleryAPClusterConfig {
    pub app_name: String,
    pub node_id: Uuid,
    pub cluster_config: ClusterConfig,
    pub sd_config: MDNSServiceDiscoveryConfig,
}

pub struct ArtilleryAPCluster {
    config: ArtilleryAPClusterConfig,
    cluster: Arc<Cluster>,
    sd: Arc<MDNSServiceDiscovery>,
    cluster_ev_loop_handle: Cell<RecoverableHandle<()>>,
}

unsafe impl Send for ArtilleryAPCluster {}
unsafe impl Sync for ArtilleryAPCluster {}

pub type DiscoveryLaunch = RecoverableHandle<()>;

impl ArtilleryAPCluster {
    pub fn new(config: ArtilleryAPClusterConfig) -> Result<Self> {
        let sd = MDNSServiceDiscovery::new_service_discovery(config.sd_config.clone())?;

        let (cluster, cluster_listener) =
            Cluster::new_cluster(config.node_id, config.cluster_config.clone())?;

        Ok(Self {
            config,
            cluster: Arc::new(cluster),
            sd: Arc::new(sd),
            cluster_ev_loop_handle: Cell::new(cluster_listener),
        })
    }

    pub fn cluster(&self) -> Arc<Cluster> {
        self.cluster.clone()
    }

    pub fn service_discovery(&self) -> Arc<MDNSServiceDiscovery> {
        self.sd.clone()
    }

    pub fn shutdown(&self) {
        self.cluster().leave_cluster();
    }

    pub async fn launch(&self) {
        let (_, eh) = LightProc::recoverable(async {}, |_| (), ProcStack::default());
        let ev_loop_handle = self.cluster_ev_loop_handle.replace(eh);

        // do fusing
        let ev_loop_handle = ev_loop_handle.fuse();
        let discover_nodes_handle = self.discover_nodes().fuse();

        pin_mut!(ev_loop_handle);
        pin_mut!(discover_nodes_handle);

        select! {
            ev_loop_res = ev_loop_handle => { dbg!(ev_loop_res); ev_loop_res.unwrap() },
            _ = discover_nodes_handle => panic!("Node discovery unexpectedly shutdown.")
        };
    }

    async fn discover_nodes(&self) {
        self.service_discovery()
            .events()
            .iter()
            .filter(|discovery| {
                discovery.get().port() != self.config.sd_config.local_service_addr.port()
            })
            .for_each(|discovery| self.cluster.add_seed_node(discovery.get()))
    }
}

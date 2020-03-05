use crate::epidemic::prelude::*;
use crate::errors::*;
use crate::service_discovery::mdns::prelude::*;

use lightproc::prelude::*;

use std::future::Future;

use std::sync::Arc;
use uuid::Uuid;
use futures::{join, future};

#[derive(Default, Clone)]
pub struct ArtilleryAPClusterConfig {
    pub app_name: String,
    pub node_id: Uuid,
    pub cluster_config: ClusterConfig,
    pub sd_config: MDNSServiceDiscoveryConfig,
}

pub struct ArtilleryAPCluster {
    config: ArtilleryAPClusterConfig,
    cluster: Arc<Cluster>,
    sd: Arc<MDNSServiceDiscovery>
}

unsafe impl Send for ArtilleryAPCluster {}
unsafe impl Sync for ArtilleryAPCluster {}

pub type DiscoveryLaunch = RecoverableHandle<()>;

impl ArtilleryAPCluster {
    pub fn new(config: ArtilleryAPClusterConfig) -> Result<(Self, RecoverableHandle<()>)> {
        let sd = MDNSServiceDiscovery::new_service_discovery(config.sd_config.clone())?;

        let (cluster, cluster_listener) = Cluster::new_cluster(config.node_id, config.cluster_config.clone())?;

        Ok((Self {
            config,
            cluster: Arc::new(cluster),
            sd: Arc::new(sd)
        }, cluster_listener))
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
        self
            .service_discovery()
            .events()
            .iter()
            .filter(|discovery| {
                discovery.get().port() != self.config.sd_config.local_service_addr.port()
            })
            .for_each(|discovery| self.cluster.add_seed_node(discovery.get()))
    }
}

use crate::epidemic::prelude::*;
use crate::errors::*;
use crate::service_discovery::mdns::prelude::*;

use lightproc::prelude::*;

use std::future::Future;

use std::sync::Arc;
use uuid::Uuid;

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
    sd: Arc<MDNSServiceDiscovery>,
}

unsafe impl Send for ArtilleryAPCluster {}
unsafe impl Sync for ArtilleryAPCluster {}

pub type DiscoveryLaunch = RecoverableHandle<()>;

impl ArtilleryAPCluster {
    pub fn new(config: ArtilleryAPClusterConfig) -> Result<Self> {
        let sd = MDNSServiceDiscovery::new_service_discovery(config.sd_config.clone())?;

        let cluster = Cluster::new_cluster(config.node_id, config.cluster_config.clone())?;

        Ok(Self {
            config,
            cluster: Arc::new(cluster),
            sd: Arc::new(sd),
        })
    }

    pub fn cluster(&self) -> Arc<Cluster> {
        self.cluster.clone()
    }

    pub fn service_discovery(&self) -> Arc<MDNSServiceDiscovery> {
        self.sd.clone()
    }

    pub fn launch(&self) -> impl Future<Output = ()> + '_ {
        let config = self.config.clone();
        let events = self.service_discovery().events();
        let cluster = self.cluster.clone();

        async {
            let config_inner = config;
            let events_inner = events;
            let cluster_inner = cluster;

            events_inner
                .iter()
                .filter(|discovery| {
                    discovery.get().port() != config_inner.sd_config.local_service_addr.port()
                })
                .for_each(|discovery| cluster_inner.add_seed_node(discovery.get()))
        }
    }
}

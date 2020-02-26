use crate::epidemic::prelude::*;
use crate::service_discovery::mdns::prelude::*;
use bastion_executor::blocking::spawn_blocking;
use lightproc::proc_handle::ProcHandle;
use lightproc::proc_stack::ProcStack;
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use lightproc::prelude::*;
use crate::errors::*;
use std::rc::Rc;
use std::future::Future;
use std::error::Error;


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
    pub fn new(config: ArtilleryAPClusterConfig) -> Result<Self> {
        let sd =
            MDNSServiceDiscovery::new_service_discovery(
                config.sd_config.clone())?;

        let cluster =
            Cluster::new_cluster(
                config.node_id,
                config.cluster_config.clone()
            )?;

        Ok(Self {
            config,
            cluster: Arc::new(cluster),
            sd: Arc::new(sd)
        })
    }

    pub fn cluster(&self) -> Arc<Cluster> {
        self.cluster.clone()
    }

    pub fn service_discovery(&self) -> Arc<MDNSServiceDiscovery> {
        self.sd.clone()
    }

    pub fn launch(&self) -> impl Future<Output=()> + '_ {
        let config = self.config.clone();

        async {
            for discovery in self.service_discovery().events.iter() {
                if discovery.get().port() != config.sd_config.local_service_addr.port() {
                    self.cluster.add_seed_node(discovery.get());
                }
            }
        }
    }
}

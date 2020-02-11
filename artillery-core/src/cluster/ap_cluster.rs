use crate::service_discovery::udp_anycast::prelude::*;
use crate::epidemic::prelude::*;

#[derive(Default)]
pub struct ArtilleryAPClusterConfig {
    app_name: String,
    cluster_config: ClusterConfig
}

pub struct ArtilleryAPCluster {
    config: ArtilleryAPClusterConfig
}

impl ArtilleryAPCluster {
    pub fn new(config: ArtilleryAPClusterConfig) -> Self {
        Self {
            config
        }
    }

    pub fn new_with_defaults() -> Self {
        Self {
            config: ArtilleryAPClusterConfig::default()
        }
    }

    pub fn launch(&self) -> Self {
        unimplemented!()
    }
}
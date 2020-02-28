extern crate pretty_env_logger;

#[macro_use]
extern crate log;

use clap::*;
use std::convert::TryInto;
use std::fs::File;
use std::io::{Read, Write};
use std::net::ToSocketAddrs;
use std::path::Path;
use uuid::Uuid;

use artillery_core::epidemic::prelude::*;
use artillery_core::service_discovery::mdns::prelude::*;

use once_cell::sync::OnceCell;
use serde::*;

use std::thread;
use std::time::Duration;
use artillery_core::cluster::ap_cluster::*;

use bastion_executor::blocking::spawn_blocking;
use lightproc::proc_handle::ProcHandle;
use lightproc::proc_stack::ProcStack;


#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct ExampleSDReply {
    ip: String,
    port: u16,
}

fn main() {
    pretty_env_logger::init();

    let port = get_port();
    let ap_cluster_config = ArtilleryAPClusterConfig {
        app_name: String::from("artillery-ap"),
        node_id: Uuid::new_v4(),
        sd_config: {
            let mut config = MDNSServiceDiscoveryConfig::default();
            config.local_service_addr.set_port(port);
            config
        },
        cluster_config: {
            let listen_addr = format!("127.0.0.1:{}", port);

            ClusterConfig {
                listen_addr: (&listen_addr as &str)
                    .to_socket_addrs()
                    .unwrap()
                    .next()
                    .unwrap(),
                ..Default::default()
            }
        },
    };

    let ap_cluster = ArtilleryAPCluster::new(ap_cluster_config).unwrap();
    spawn_blocking(async { ap_cluster.launch().await }, ProcStack::default());
}

fn get_port() -> u16 {
    use rand::{thread_rng, Rng};

    let mut rng = thread_rng();
    let port: u16 = rng.gen();
    if port > 1025 && port < 65535 {
        port
    } else {
        get_port()
    }
}

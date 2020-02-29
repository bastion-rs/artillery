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
use futures::future;
use std::time::Duration;
use artillery_core::cluster::ap_cluster::*;

use bastion_executor::prelude::*;
use lightproc::proc_handle::ProcHandle;
use lightproc::proc_stack::ProcStack;
use std::sync::Arc;



fn main() {
    pretty_env_logger::init();

    // Let's find a broadcast port
    let port = get_port();

    // Initialize our cluster configuration
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

    // Configure our cluster node
    let ap_cluster = Arc::new(ArtilleryAPCluster::new(ap_cluster_config).unwrap());

    // Launch the cluster node
    run( async {
        let cluster_stack = ProcStack::default()
            .with_pid(2);
        let events_stack = ProcStack::default()
            .with_pid(3);

        let ap_events = ap_cluster.clone();

        // Detach cluster launch
        let cluster_handle =
            spawn_blocking(async move {
                ap_cluster.launch().await }, cluster_stack);

        // Detach event consumption
        let events_handle =
            spawn_blocking(async move {
                warn!("STARTED: Event Poller");
                for (members, event) in ap_events.cluster().clone().events.iter() {
                    warn!("");
                    warn!(" CLUSTER EVENT ");
                    warn!("===============");
                    warn!("{:?}", event);
                    warn!("");

                    for member in members {
                        info!("MEMBER  {:?}", member);
                    }
                }
                warn!("STOPPED: Event Poller");
            }, events_stack);

        future::join(events_handle, cluster_handle).await
    }, ProcStack::default().with_pid(1));
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

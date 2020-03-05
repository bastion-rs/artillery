extern crate pretty_env_logger;

#[macro_use]
extern crate log;


use std::sync::Once;
use bastion::prelude::*;
use fail::FailScenario;

//

use std::net::ToSocketAddrs;

use uuid::Uuid;

use artillery_core::epidemic::prelude::*;
use artillery_core::service_discovery::mdns::prelude::*;

use artillery_core::cluster::ap::*;
use futures::future;

use bastion_executor::prelude::*;

use lightproc::prelude::*;
use lightproc::proc_stack::ProcStack;
use std::sync::Arc;
use std::time::{Duration, Instant};


fn test_epidemic_periodic_index_fp(port: u16) ->
    (Arc<ArtilleryAPCluster>, RecoverableHandle<()>, RecoverableHandle<()>, RecoverableHandle<()>) {
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
    let (cluster, cluster_listener) = ArtilleryAPCluster::new(ap_cluster_config).unwrap();
    let ap_cluster = Arc::new(cluster);

    // Launch the cluster node
    let cluster_stack = ProcStack::default().with_pid(2);
    let events_stack = ProcStack::default().with_pid(3);

    let ap_events = ap_cluster.clone();
    let ap_ref = ap_cluster.clone();

    // Detach cluster launch
    let cluster_handle = spawn_blocking(
        async move { ap_cluster.launch().await }, cluster_stack);

        // Detach event consumption
    let events_handle = spawn_blocking(
        async move {
            warn!("STARTED: Event Poller");
            for (members, event) in ap_events.cluster().events.iter() {
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

    (ap_ref, events_handle, cluster_handle, cluster_listener)
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

static LOGGER_INIT: Once = Once::new();

macro_rules! cluster_fault_recovery_test {
	  ($fp_name:expr) => {
        LOGGER_INIT.call_once(|| pretty_env_logger::init());
        let scenario = FailScenario::setup();
        fail::cfg($fp_name, "panic").unwrap();


        // Let's see how reliable you are.
        let node1 = spawn_blocking(async {
            let (c, events, cluster_handle, cluster_listener) = test_epidemic_periodic_index_fp(get_port());
            match cluster_listener.await {
                Some(_) => assert!(false),
                _ => {
                    // Test passed.
                    warn!("This node is leaving.");
                    c.shutdown();
                    warn!("Stopping the setup");
                },
            }
        }, ProcStack::default());

        let node2 = spawn_blocking(async {
            let (c, events, cluster_handle, cluster_listener) = test_epidemic_periodic_index_fp(get_port());
            match cluster_listener.await {
                Some(_) => assert!(false),
                _ => {
                    // Test passed.
                    warn!("This node is leaving.");
                    c.shutdown();
                    warn!("Stopping the setup");
                },
            }
        }, ProcStack::default());


        run(async { future::join(node1, node2).await }, ProcStack::default());

        scenario.teardown();
    }
}


#[test]
fn epidemic_periodic_index_fp() {
    cluster_fault_recovery_test!("epidemic-periodic-index-fp");
}


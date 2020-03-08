#[macro_export]
macro_rules! cluster_init {
	  () => {
        use std::sync::Once;

        //
        use kaos::*;

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

        fn node_setup(
            port: u16,
        ) -> (
            Arc<ArtilleryAPCluster>,
            RecoverableHandle<()>,
            RecoverableHandle<()>,
        ) {
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
            let cluster = ArtilleryAPCluster::new(ap_cluster_config).unwrap();
            let ap_cluster = Arc::new(cluster);

            // Launch the cluster node
            let cluster_stack = ProcStack::default().with_pid(2);
            let events_stack = ProcStack::default().with_pid(3);

            let ap_events = ap_cluster.clone();
            let ap_ref = ap_cluster.clone();

            // Detach cluster launch
            let cluster_handle = spawn_blocking(async move { ap_cluster.launch().await }, cluster_stack);

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
                },
                events_stack,
            );

            (ap_ref, events_handle, cluster_handle)
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
        LOGGER_INIT.call_once(|| pretty_env_logger::init());
	  };
}

#[macro_export]
macro_rules! ap_events_check_node_spawn {
    ($node_handle:ident) => {
        let $node_handle = spawn_blocking(
            async {
                let (c, events, cluster_handle) = node_setup(get_port());
                match events.await {
                    Some(a) => {
                        // Test passed.
                        warn!("This node is leaving.");
                        c.shutdown();
                        warn!("Stopping the setup");
                    },
                    _ => {
                        assert!(false);
                    }
                }
            },
            ProcStack::default(),
        );
    }
}

#[macro_export]
macro_rules! ap_sd_check_node_spawn {
    ($node_handle:ident) => {
        let $node_handle = spawn_blocking(
            async {
                let (c, events, cluster_handle) = node_setup(get_port());
                match cluster_handle.await {
                    Some(a) => {
                        // Test passed.
                        warn!("This node is leaving.");
                        c.shutdown();
                        warn!("Stopping the setup");
                    },
                    _ => {
                        assert!(false);
                    }
                }
            },
            ProcStack::default(),
        );
    }
}

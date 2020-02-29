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

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct ExampleSDReply {
    ip: String,
    port: u16,
}

fn main() {
    pretty_env_logger::init();
    let matches = App::new("Cannonball :: MDNS + Epidemic")
        .author("Mahmut Bulut, vertexclique [ta] gmail [tod] com")
        .version(crate_version!())
        .about("Artillery Epidemic Protocol Tester")
        .arg(
            Arg::with_name("node-data")
                .index(1)
                .long("node-data")
                .aliases(&["data-folder"])
                .required(true)
                .help("Node State Data Folder"),
        )
        .after_help(
            "Enables Artillery MDNS Service Discovery + Epidemic Protocol to be tested \
                               in the cluster configuration",
        )
        .get_matches();

    let data_folder = matches
        .value_of("node-data")
        .expect("Can't be None, required");

    let data_folder_path = Path::new(&data_folder);
    let host_key = read_host_key(&data_folder_path);
    warn!("Host key: {}", host_key.to_hyphenated());

    let this_node_cluster_port = get_port();
    let sd_config = {
        let mut config = MDNSServiceDiscoveryConfig::default();
        config.local_service_addr.set_port(this_node_cluster_port);
        config
    };
    let sd = MDNSServiceDiscovery::new_service_discovery(sd_config).unwrap();

    let this_node_cluster_listen_addr = format!("127.0.0.1:{}", this_node_cluster_port);
    let cluster = get_cluster(this_node_cluster_listen_addr.as_str(), host_key);

    std::thread::Builder::new()
        .name("cluster-event-poller".to_string())
        .spawn(move || poll_cluster_events(this_node_cluster_listen_addr.as_str(), host_key))
        .expect("cannot start cluster-event-poller");

    thread::sleep(Duration::from_secs(1));
    for discovery in sd.events().iter() {
        if discovery.get().port() != this_node_cluster_port {
            cluster.add_seed_node(discovery.get());
        }
    }
}

fn poll_cluster_events(listen_addr: &str, host_key: Uuid) {
    warn!("STARTED: Event Poller");
    for (members, event) in get_cluster(listen_addr, host_key).events.iter() {
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
}

fn read_host_key(root_folder: &Path) -> Uuid {
    let host_key_path = root_folder.join("host_key");

    if let Ok(mut config_file) = File::open(&host_key_path) {
        let mut host_key_contents = Vec::<u8>::new();
        config_file.read_to_end(&mut host_key_contents).unwrap();

        let u: [u8; 16] = host_key_contents.as_slice().try_into().unwrap();
        return Uuid::from_bytes(u);
    }

    let host_key = Uuid::new_v4();
    dbg!(host_key_path.clone());
    let mut host_key_file = File::create(&host_key_path).unwrap();
    host_key_file.write_all(host_key.as_bytes()).unwrap();
    host_key
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

#[inline]
fn get_cluster(listen_addr: &str, host_key: Uuid) -> &'static Cluster {
    static CLUSTER: OnceCell<Cluster> = OnceCell::new();
    CLUSTER.get_or_init(|| {
        let config = ClusterConfig {
            cluster_key: "artillery_local".as_bytes().to_vec(),
            listen_addr: (&listen_addr as &str)
                .to_socket_addrs()
                .unwrap()
                .next()
                .unwrap(),
            ..Default::default()
        };

        Cluster::new_cluster(host_key, config).unwrap()
    })
}

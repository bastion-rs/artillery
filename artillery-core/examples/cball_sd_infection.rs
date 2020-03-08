extern crate pretty_env_logger;

#[macro_use]
extern crate log;

use clap::*;
use std::convert::TryInto;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::Path;
use uuid::Uuid;

use artillery_core::constants::*;
use artillery_core::epidemic::prelude::*;
use artillery_core::service_discovery::udp_anycast::prelude::*;

use chrono::Duration;
use once_cell::sync::OnceCell;
use serde::*;
use std::str::FromStr;
use std::sync::mpsc::channel;

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct ExampleSDReply {
    ip: String,
    port: u16,
}

fn main() {
    pretty_env_logger::init();
    let matches = App::new("Cannonball :: UDP-SD + Epidemic")
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
        .arg(
            Arg::with_name("seeker")
                .index(2)
                .long("seeker")
                .aliases(&["seeker"])
                .help("Seeker or Listener"),
        )
        .after_help(
            "Enables Artillery Service Discovery + Epidemic Protocol to be tested \
                               in the cluster configuration",
        )
        .get_matches();

    let data_folder = matches
        .value_of("node-data")
        .expect("Can't be None, required");
    let seeker = matches.value_of("seeker");

    let data_folder_path = Path::new(&data_folder);
    let host_key = read_host_key(&data_folder_path);
    warn!("Host key: {}", host_key.to_hyphenated());

    let service_discovery = {
        let sd_port = get_port();
        if seeker.is_some() {
            MulticastServiceDiscoveryConfig {
                timeout_delta: Duration::seconds(1),
                discovery_addr: SocketAddr::from(([0, 0, 0, 0], sd_port)),
                seeking_addr: SocketAddr::from(([0, 0, 0, 0], CONST_SERVICE_DISCOVERY_PORT)),
            }
        } else {
            MulticastServiceDiscoveryConfig {
                timeout_delta: Duration::seconds(1),
                discovery_addr: SocketAddr::from(([0, 0, 0, 0], CONST_SERVICE_DISCOVERY_PORT)),
                seeking_addr: SocketAddr::from(([0, 0, 0, 0], sd_port)),
            }
        }
    };

    let epidemic_sd_config = ExampleSDReply {
        ip: "127.0.0.1".into(),
        port: get_port(),
    };

    let reply = ServiceDiscoveryReply {
        serialized_data: serde_json::to_string(&epidemic_sd_config).unwrap(),
    };

    let sd = MulticastServiceDiscovery::new_service_discovery(service_discovery, reply).unwrap();

    let listen_addr = format!("{}:{}", "127.0.0.1", epidemic_sd_config.port);
    let _listen_addr_sd = listen_addr.clone();
    let cluster = get_cluster(listen_addr.as_str(), host_key);

    let (tx, discoveries) = channel();
    sd.register_seeker(tx).unwrap();
    if seeker.is_some() {
        sd.seek_peers().unwrap();
    } else {
        sd.set_listen_for_peers(true).unwrap();
    }

    std::thread::Builder::new()
        .name("cluster-event-poller".to_string())
        .spawn(move || poll_cluster_events(listen_addr.as_str(), host_key))
        .expect("cannot start cluster-event-poller");

    for discovery in discoveries.iter() {
        let discovery: ExampleSDReply = serde_json::from_str(&discovery.serialized_data).unwrap();
        if discovery.port != epidemic_sd_config.port {
            debug!("Seed node address came");
            let seed_node = format!("{}:{}", epidemic_sd_config.ip, discovery.port);
            cluster.add_seed_node(FromStr::from_str(&seed_node).unwrap());
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
            cluster_key: b"artillery_local".to_vec(),
            listen_addr: (&listen_addr as &str)
                .to_socket_addrs()
                .unwrap()
                .next()
                .unwrap(),
            ..Default::default()
        };

        let (cluster, _) = Cluster::new_cluster(host_key, config).unwrap();
        cluster
    })
}

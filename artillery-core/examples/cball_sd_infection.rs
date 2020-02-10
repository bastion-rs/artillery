extern crate pretty_env_logger;

#[macro_use]
extern crate log;

use clap::*;
use std::convert::TryInto;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{ToSocketAddrs, SocketAddr};
use std::path::Path;
use uuid::Uuid;

use artillery_core::prelude::*;
use std::str::FromStr;
use serde::*;
use bastion_utils::math;
use once_cell::sync::{Lazy, OnceCell};
use artillery_core::prelude::discovery_config::MulticastServiceDiscoveryConfig;
use std::sync::mpsc::channel;
use std::thread;
use chrono::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct ExampleSDReply {
    ip: String,
    port: u16
}

fn main() {
    pretty_env_logger::init();
    let matches = App::new("Cannonball :: Epidemic")
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
            "Enables Artillery Service Discovery + Epidemic Protocol to be tested \
                               in the cluster configuration",
        )
        .get_matches();

    let data_folder = matches
        .value_of("node-data")
        .expect("Can't be None, required");

    let data_folder_path = Path::new(&data_folder);
    let host_key = read_host_key(&data_folder_path);
    warn!("Host key: {}", host_key.to_hyphenated());

    let service_discovery =
        {
            let sd_port = get_port();
            dbg!(sd_port.clone());
            discovery_config::MulticastServiceDiscoveryConfig {
                timeout_delta: Duration::seconds(1),
                discovery_addr: SocketAddr::from(([0, 0, 0, 0], sd_port)),
                seeking_addr: SocketAddr::from(([192, 168, 1, 255], sd_port)),
            }

//            discovery_config::MulticastServiceDiscoveryConfig::default()
        };
    let epidemic_sd_config = ExampleSDReply {
        ip: "127.0.0.1".into(),
        port: get_port(),
    };

    let reply =
        state::ServiceDiscoveryReply {
            serialized_data: serde_json::to_string(&epidemic_sd_config).unwrap()
        };

    let sd =
        sd::MulticastServiceDiscovery::new_service_discovery(
            service_discovery, reply).unwrap();

    let listen_addr = format!("{}:{}", "127.0.0.1", epidemic_sd_config.port);
    let listen_addr_sd = listen_addr.clone();
    let cluster = get_cluster(listen_addr.as_str(), host_key);

    let (tx, discoveries) = channel();
    sd.register_seeker(tx).unwrap();
//    sd.set_listen_for_peers(true).unwrap();
    sd.seek_peers().unwrap();

    std::thread::Builder::new()
        .name("cluster-event-poller".to_string())
        .spawn(move || {
            poll_cluster_events(listen_addr.as_str(), host_key)
        })
        .expect("cannot start cluster-event-poller");

    while let Ok(disco) = discoveries.try_recv() {
        dbg!(disco);
    }

    for discovery in discoveries.iter() {
        dbg!(discovery.clone());
        let discovery: ExampleSDReply = serde_json::from_str(&discovery.serialized_data).unwrap();
        dbg!(discovery.clone());
        dbg!(epidemic_sd_config.clone());
        if discovery.port != epidemic_sd_config.port {
            dbg!("SEED NODE CAME");
            let seed_node = format!("{}:{}", epidemic_sd_config.ip, discovery.port);
            cluster
                .add_seed_node(FromStr::from_str(&seed_node).unwrap());
        }
    }
}

fn poll_cluster_events(listen_addr: &str, host_key: Uuid) {
    warn!("STARTED: Event Poller");
    for (members, event) in
        get_cluster(listen_addr, host_key).events.iter() {
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
    // https://open.spotify.com/track/6Btbw0SV8FqxJ0AAA26Xrd?si=4BR-pO7nQki-DlCrvCTGgg
    host_key
}

fn get_port() -> u16 {
    use rand::{thread_rng, Rng};

    let mut rng = thread_rng();
    let port: u16 = rng.gen();
    if port > 1025 && port < 65535 {
        port
    } else { get_port() }
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


extern crate pretty_env_logger;

#[macro_use]
extern crate log;


use clap::*;
use std::path::Path;
use uuid::Uuid;
use std::fs::File;
use std::io::{Read, Write};
use std::convert::TryInto;
use std::net::ToSocketAddrs;

use artillery_core::prelude::*;
use std::str::FromStr;

fn main() {
    pretty_env_logger::init();
    let matches = App::new("Cannonball")
        .author("Mahmut Bulut, vertexclique [ta] gmail [tod] com")
        .version(crate_version!())
        .about("Artillery Epidemic Protocol Tester")
        .arg(
            Arg::with_name("data-folder").index(1)
                .long("data-folder")
                .aliases(&["data-folder"])
                .required(true)
                .help("State Data Folder")
        )
        .arg(
            Arg::with_name("cluster-key").index(2)
                .long("cluster-key")
                .aliases(&["cluster-key"])
                .required(true)
                .help("Cluster Key")
        )
        .arg(
            Arg::with_name("listen-addr").index(3)
                .long("listen-addr")
                .aliases(&["listen-addr"])
                .required(true)
                .help("Listen Address")
        )
        .arg(
            Arg::with_name("seed-node").index(4)
                .long("seed-node")
                .aliases(&["seed-node"])
                .help("Seed Node")
        )
        .after_help("Enables Artillery epidemic protocol to be tested \
                               in the cluster configuration")
        .get_matches();

    let data_folder = matches.value_of("data-folder")
        .expect("Can't be None, required");

    let data_folder_path = Path::new(&data_folder);
    let host_key = read_host_key(&data_folder_path);
    warn!("Host key: {}", host_key.to_hyphenated());

    let cluster_key = matches.value_of("cluster-key")
        .expect("Can't be None, required");
    let listen_addr = matches.value_of("listen-addr")
        .expect("Can't be None, required");
    let seed_node = matches.value_of("seed-node");

    let config = ClusterConfig {
        cluster_key: cluster_key.as_bytes().to_vec(),
        listen_addr: (&listen_addr as &str).to_socket_addrs().unwrap().next().unwrap(),
        .. Default::default()
    };

    let cluster = Cluster::new_cluster(host_key, config).unwrap();

    if let Some(seed_node) = seed_node {
        cluster.add_seed_node(FromStr::from_str(&seed_node).unwrap());
    }

    warn!("STARTED: Event Poller");
    for (members, event) in cluster.events.iter() {
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
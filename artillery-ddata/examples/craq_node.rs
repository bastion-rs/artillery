extern crate pretty_env_logger;

#[macro_use]
extern crate log;

use artillery_ddata::craq::prelude::*;
use clap::*;

fn main() {
    pretty_env_logger::init();

    let matches = App::new("Artillery CRAQ")
        .author("Mahmut Bulut, vertexclique [ta] gmail [tod] com")
        .version(crate_version!())
        .about("Artillery Distributed Data Protocol Tester")
        .subcommand(
            SubCommand::with_name("server")
                .about("Runs a CRAQ server")
                .arg(
                    Arg::with_name("cr_mode")
                        .required(true)
                        .help("CR mode that server would use: 0 for CRAQ, 1 for CR")
                        .index(1),
                )
                .arg(
                    Arg::with_name("node_index")
                        .required(true)
                        .help("Node index this server would use")
                        .index(2),
                )
                .arg(
                    Arg::with_name("chain_servers")
                        .required(true)
                        .multiple(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("client")
                .about("Runs a CRAQ client")
                .arg(
                    Arg::with_name("server_ip_port")
                        .required(true)
                        .help("Server ip and port to connect")
                        .index(1),
                )
                .arg(
                    Arg::with_name("test_method")
                        .required(true)
                        .help("Test method of client to test against the server")
                        .index(2),
                )
                .arg(
                    Arg::with_name("extra_args")
                        .required(true)
                        .multiple(true)
                        .min_values(3),
                ),
        )
        .after_help("Enables Artillery CRAQ protocol to be tested in the server/client fashion")
        .get_matches();

    match matches.subcommand() {
        ("server", Some(server_matches)) => {
            let cr_mode = match server_matches.value_of("cr_mode") {
                Some("0") => CRMode::Craq,
                Some("1") => CRMode::Cr,
                _ => panic!("CR mode not as expected"),
            };

            if let Some(node_index) = server_matches.value_of("node_index") {
                let node_index = node_index.parse::<usize>().unwrap();
                let varargs: Vec<&str> =
                    server_matches.values_of("chain_servers").unwrap().collect();

                let nodes: Vec<ChainNode> = varargs.iter().flat_map(ChainNode::new).collect();

                assert_eq!(nodes.len(), varargs.len(), "Node address parsing failed");

                let chain = CraqChain::new(&nodes, node_index).unwrap();
                CraqNode::start(cr_mode, chain, CraqConfig::default())
                    .expect("couldn't start CRAQ node");
            }
        }
        ("client", Some(client_matches)) => {
            let _sip = client_matches
                .value_of("server_ip_port")
                .unwrap()
                .to_string();

            match client_matches.value_of("test_method") {
                Some("bench_read") => todo!(),
                _ => unreachable!(),
            }
        }
        _ => {
            error!("Couldn't find any known subcommands");
        }
    }
}

extern crate pretty_env_logger;

#[macro_use]
extern crate log;

mod base;
use base::*;

fn main() {
    cluster_init!();

    kaostest!("mdns-protocol-fp",
              {
                  ap_sd_check_node_spawn!(node1);
                  ap_sd_check_node_spawn!(node2);
                  ap_sd_check_node_spawn!(node3);

                  run(
                      async {
                          future::join_all(
                              vec![node1, node2, node3]
                          ).await
                      },
                      ProcStack::default(),
                  );
              }
    );
}

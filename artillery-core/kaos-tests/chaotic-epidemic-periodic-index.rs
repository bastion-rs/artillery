extern crate pretty_env_logger;

#[macro_use]
extern crate log;

mod base;
use base::*;


fn main() {
    cluster_init!();

    kaostest!("epidemic-periodic-index-fp",
              {
                  // What Bastion does is doing this asynchronously. Thanks.
                  let mut restarts = 0;

                  loop {
                      restarts += 1;
                      ap_events_check_node_spawn!(node1);
                      ap_events_check_node_spawn!(node2);
                      ap_events_check_node_spawn!(node3);

                      run(
                          async {
                              future::join_all(
                                  vec![node1, node2, node3]
                              ).await
                          },
                          ProcStack::default(),
                      );

                      if restarts == 10 {
                          break;
                      }
                  }
              }
    );
}

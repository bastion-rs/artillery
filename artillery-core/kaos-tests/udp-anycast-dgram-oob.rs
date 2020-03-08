extern crate pretty_env_logger;

#[macro_use]
extern crate log;

mod base;
use base::*;

fn main() {
    // cluster_init!();
    // "udp-anycast-dgram-oop-fp"

    // TODO: This will obviously pass because AP cluster doesn't use UDP anycast by default.
    // Fix it after having different prepared cluster.
    std::thread::sleep(std::time::Duration::from_secs(3));
}

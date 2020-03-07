extern crate pretty_env_logger;

#[macro_use]
extern crate log;

#[macro_use]
mod chaos;

use chaos::*;

fn main() {
    cluster_init!();
    chaos_unleash!("epidemic-periodic-index-fp");
}

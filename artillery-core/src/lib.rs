#[macro_use]
extern crate log;

#[macro_use]
mod errors;

mod epidemic;

pub mod prelude {
    pub use super::epidemic::cluster::*;
    pub use super::epidemic::cluster_config::*;
}
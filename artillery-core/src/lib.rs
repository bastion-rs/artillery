#[macro_use]
extern crate log;

#[macro_use]
mod errors;

mod constants;

/// Infection-style clustering
mod epidemic;

/// Service discovery types
mod service_discovery;

pub mod prelude {
    pub use super::constants::*;
    pub use super::service_discovery::multicast::*;
    pub use super::epidemic::cluster::*;
    pub use super::epidemic::cluster_config::*;
}

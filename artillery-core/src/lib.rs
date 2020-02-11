#[macro_use]
extern crate log;

#[macro_use]
pub mod errors;

/// Constants of the Artillery
pub mod constants;

/// Infection-style clustering
pub mod epidemic;

/// Service discovery strategies
pub mod service_discovery;

/// Cluster types
pub mod cluster;

use super::node::CRMode;

#[derive(Debug, Clone)]
pub struct CraqConfig {
    pub fallback_replication_port: u16,
    pub operation_mode: CRMode,
    pub connection_sleep_time: u64,
    pub connection_pool_size: usize,
    pub protocol_worker_size: usize,
}

impl Default for CraqConfig {
    fn default() -> Self {
        CraqConfig {
            fallback_replication_port: 22991_u16,
            operation_mode: CRMode::Craq,
            connection_sleep_time: 1000_u64,
            connection_pool_size: 50_usize,
            protocol_worker_size: 100_usize,
        }
    }
}

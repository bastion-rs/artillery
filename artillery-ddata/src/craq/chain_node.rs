use super::errors::*;
use std::net::{SocketAddr, ToSocketAddrs};

///
/// Chain node representation
#[derive(Debug, Clone)]
pub struct ChainNode {
    host: SocketAddr,
}

impl ChainNode {
    pub fn new<A>(addr: A) -> Result<Self>
    where
        A: ToSocketAddrs,
    {
        let host: SocketAddr = addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| CraqError::SocketAddrError("No node address given or parsed.".into()))?;
        Ok(Self { host })
    }

    pub fn get_addr(&self) -> &SocketAddr {
        &self.host
    }
}

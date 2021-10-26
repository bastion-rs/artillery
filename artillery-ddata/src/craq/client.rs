use std::fmt;

use super::errors::*;
use super::{
    node::CraqClient,
    proto::{CraqConsistencyModel, CraqObject, TCraqServiceSyncClient},
};
use std::net::{SocketAddr, ToSocketAddrs};
use bytes::Bytes;
use thrift::protocol::{TBinaryInputProtocol, TBinaryOutputProtocol};
use thrift::transport::{
    TFramedReadTransport, TFramedWriteTransport, TIoChannel, TTcpChannel as BiTcp,
};

pub struct ReadObject {
    ///
    /// Object's value.
    value: Bytes,
    ///
    /// Whether the read was dirty (true) or clean (false).
    dirty: bool,
}

impl ReadObject {
    ///
    /// Creates a new wrapper Read Object
    pub fn new(value: Bytes, dirty: bool) -> Self {
        Self { value, dirty }
    }
}

impl fmt::Debug for ReadObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReadObject")
            .field("value", &self.value)
            .field("dirty", &self.dirty)
            .finish()
    }
}

impl fmt::Display for ReadObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReadObject")
            .field("value", &self.value)
            .field("dirty", &self.dirty)
            .finish()
    }
}

// Will be fixed as we implement stuff
#[allow(dead_code)]
pub struct DDataCraqClient {
    host: SocketAddr,
    cc: CraqClient,
}

impl DDataCraqClient {
    pub fn connect_host_port<T>(host: T, port: u16) -> Result<Self>
    where
        T: AsRef<str>,
    {
        Self::connect(format!("{}:{}", host.as_ref(), port))
    }

    pub fn connect<A>(addr: A) -> Result<Self>
    where
        A: ToSocketAddrs,
    {
        let host: SocketAddr = addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| CraqError::SocketAddrError("No node address given or parsed.".into()))?;

        debug!("Client is initiating connection to: {}", host);

        let mut c = BiTcp::new();
        c.open(&host.to_string())?;
        let (i_chan, o_chan) = c.split()?;
        let (i_tran, o_tran) = (
            TFramedReadTransport::new(i_chan),
            TFramedWriteTransport::new(o_chan),
        );
        let (i_prot, o_prot) = (
            TBinaryInputProtocol::new(i_tran, true),
            TBinaryOutputProtocol::new(o_tran, true),
        );

        debug!("Created client: {}", host);
        let cc = CraqClient::new(i_prot, o_prot);
        Ok(Self { host, cc })
    }

    ///
    /// Writes an object to the cluster, returning the new object version or -1 upon failure.
    pub fn write(&mut self, value: String) -> Result<i64> {
        let mut obj = CraqObject::default();
        obj.value = Some(value.into());
        Ok(self.cc.write(obj)?)
    }

    ///
    /// Reads an object with given bound version.
    pub fn read(&mut self, model: CraqConsistencyModel, version_bound: i64) -> Result<ReadObject> {
        let obj = self.cc.read(model, version_bound)?;

        match (obj.value, obj.dirty) {
            (Some(v), Some(d)) => Ok(ReadObject::new(v, d)),
            _ => bail!(CraqError::ReadError, "Read request failed"),
        }
    }

    ///
    /// Performs a test-and-set operation, returning the new object version or -1 upon failure.
    pub fn test_and_set(&mut self, value: String, expected_version: i64) -> Result<i64> {
        let mut obj = CraqObject::default();
        obj.value = Some(value.into());
        Ok(self.cc.test_and_set(obj, expected_version)?)
    }
}

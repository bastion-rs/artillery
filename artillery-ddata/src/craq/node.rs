use super::errors::*;

use super::chain::CraqChain;
use super::{craq_config::CraqConfig, erwlock::ERwLock, proto::*, server::CraqProtoServer};

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs},
    sync::Arc,
};
use thrift::protocol::{
    TBinaryInputProtocol, TBinaryInputProtocolFactory, TBinaryOutputProtocol,
    TBinaryOutputProtocolFactory,
};
use thrift::transport::{
    ReadHalf, TFramedReadTransport, TFramedReadTransportFactory, TFramedWriteTransport,
    TFramedWriteTransportFactory, TIoChannel, TTcpChannel as BiTcp, WriteHalf,
};

use thrift::server::TServer;

use crossbeam_channel::{unbounded, Receiver, Sender};

///
/// CR mode that will be used.
#[derive(Debug, Clone, PartialEq)]
pub enum CRMode {
    /// Standard CR mode.
    Cr,
    /// CRAQ mode.
    Craq,
}

impl Default for CRMode {
    fn default() -> Self {
        CRMode::Craq
    }
}

type CraqClientInputProtocol = TBinaryInputProtocol<TFramedReadTransport<ReadHalf<BiTcp>>>;
type CraqClientOutputProtocol = TBinaryOutputProtocol<TFramedWriteTransport<WriteHalf<BiTcp>>>;
pub(crate) type CraqClient =
    CraqServiceSyncClient<CraqClientInputProtocol, CraqClientOutputProtocol>;

///
/// Representation of a physical CRAQ node.
#[derive(Default)]
pub struct CraqNode {
    /// Run mode which is either CR or CRAQ mode.
    pub cr_mode: CRMode,
    /// Whole chain.
    pub chain: Arc<CraqChain>,
    /// Tail connection pool receiver.
    pub tail_pool_rx: Option<Arc<Receiver<CraqClient>>>,
    /// Successor connection pool receiver.
    pub successor_pool_rx: Option<Arc<Receiver<CraqClient>>>,
    /// Tail connection pool sender.
    pub tail_pool_tx: Option<Arc<Sender<CraqClient>>>,
    /// Successor connection pool sender.
    pub successor_pool_tx: Option<Arc<Sender<CraqClient>>>,
    /// Stored node configuration to be reused across iterations
    pub config: CraqConfig,
}

impl CraqNode {
    ///
    /// Initialize a CRAQ node with given chain
    fn new_node(cr_mode: CRMode, chain: CraqChain, config: CraqConfig) -> Result<Self> {
        Ok(Self {
            cr_mode,
            chain: Arc::new(chain),
            config,
            ..Default::default()
        })
    }

    ///
    /// Initial connection to the underlying protocol server.
    fn connect_to_first<A>(&self, server_addr: A) -> CraqClient
    where
        A: ToSocketAddrs,
    {
        self.connect_to_server(&server_addr).unwrap_or_else(|_e| {
            std::thread::sleep(std::time::Duration::from_millis(
                self.config.connection_sleep_time,
            ));
            self.connect_to_first(server_addr)
        })
    }

    ///
    /// Creates a connection pool to the given underlying protocol server.
    fn create_conn_pool<A>(
        &self,
        server_addr: A,
    ) -> Result<(Sender<CraqClient>, Receiver<CraqClient>)>
    where
        A: ToSocketAddrs,
    {
        let (tx, rx) = unbounded();
        let client = self.connect_to_first(&server_addr);
        // TODO: tryize
        let _ = tx.try_send(client);

        let _ = (0..self.config.connection_pool_size)
            .flat_map(|_| -> Result<_> { Ok(tx.try_send(self.connect_to_server(&server_addr)?)) });
        // while let Ok(_) = tx.try_send(self.connect_to_server(&server_addr)?) {}

        Ok((tx, rx))
    }

    ///
    /// Connects to the other nodes in the given chain.
    fn connect(noderef: Arc<ERwLock<CraqNode>>) -> Result<()> {
        let mut nodew = noderef.write();

        debug!("Trying to connect");
        if nodew.chain.is_tail() {
            return Ok(());
        }

        debug!("Checking tail connection...");
        if let Some(tail) = nodew.chain.clone().get_tail() {
            let tail = tail.clone();
            let (t_tx, t_rx) = nodew.create_conn_pool(tail.get_addr())?;
            nodew.tail_pool_rx = Some(Arc::new(t_rx));
            nodew.tail_pool_tx = Some(Arc::new(t_tx));
            info!(
                "[CR Node {}] Connected to tail at {}",
                nodew.chain.get_index(),
                tail.get_addr()
            );
        } else {
            // NOTE: shouldn't happen
            error!("Shouldn't have happened - tail follows");
            unreachable!()
        }

        debug!("Checking node before the tail...");
        // Is this the node before the tail?
        if nodew.chain.get_index() == nodew.chain.chain_size().saturating_sub(2) {
            nodew.successor_pool_tx = nodew.tail_pool_tx.clone();
            nodew.successor_pool_rx = nodew.tail_pool_rx.clone();
            info!("[CR Node {}] Node before the tail", nodew.chain.get_index());
            return Ok(());
        }

        debug!("Checking successor...");
        if let Some(successor) = nodew.chain.get_successor() {
            let successor = successor.clone();
            info!(
                "[CR Node {}] Connecting to successor at {}",
                nodew.chain.get_index(),
                successor.get_addr()
            );
            let (s_tx, s_rx) = nodew.create_conn_pool(successor.get_addr())?;
            nodew.successor_pool_rx = Some(Arc::new(s_rx));
            nodew.successor_pool_tx = Some(Arc::new(s_tx));
            info!(
                "[CR Node {}] Connected to successor at {}",
                nodew.chain.get_index(),
                successor.get_addr()
            );
        } else {
            // NOTE: shouldn't happen
            error!("Shouldn't have happened - successor interval");
            unreachable!()
        }

        debug!("All aligned...");

        Ok(())
    }

    ///
    /// Entrypoint / Start procedure of this node
    pub fn start(cr_mode: CRMode, chain: CraqChain, config: CraqConfig) -> Result<()> {
        let port = chain
            .get_node()
            .map_or(config.fallback_replication_port, |n| n.get_addr().port());

        let node = Arc::new(ERwLock::new(CraqNode::new_node(cr_mode, chain, config)?));

        let connector_node = node.clone();
        let handle = std::thread::spawn(move || {
            Self::connect(connector_node).expect("Successor connections has failed");
        });

        let _ = handle.join();

        info!("Starting protocol server at port: {}", port);
        Self::run_protocol_server(node, port)
    }

    ///
    /// Connect to given server address using CRAQ client.
    fn connect_to_server<A>(&self, addr: A) -> Result<CraqClient>
    where
        A: ToSocketAddrs,
    {
        let host: SocketAddr = addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| CraqError::SocketAddrError("No node address given or parsed.".into()))?;

        debug!("Issuing connection to: {}", host);

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

        debug!("Creating client: {}", host);
        Ok(CraqClient::new(i_prot, o_prot))
    }

    ///
    /// Start local protocol server
    fn run_protocol_server(node: Arc<ERwLock<CraqNode>>, port: u16) -> Result<()> {
        let node = node.read();
        debug!("Protocol medium getting set up");

        let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);

        let (i_tran_fact, i_prot_fact) = (
            TFramedReadTransportFactory::new(),
            TBinaryInputProtocolFactory::new(),
        );
        let (o_tran_fact, o_prot_fact) = (
            TFramedWriteTransportFactory::new(),
            TBinaryOutputProtocolFactory::new(),
        );

        let processor = CraqServiceSyncProcessor::new(CraqProtoServer::new(
            node.tail_pool_rx.clone(),
            node.tail_pool_tx.clone(),
            node.successor_pool_rx.clone(),
            node.successor_pool_tx.clone(),
            node.chain.clone(),
            node.cr_mode.clone(),
        ));

        debug!("Server started");
        let mut server = TServer::new(
            i_tran_fact,
            i_prot_fact,
            o_tran_fact,
            o_prot_fact,
            processor,
            node.config.protocol_worker_size,
        );

        debug!("Started listening");
        Ok(server.listen(&server_addr.to_string())?)
    }
}

use crate::constants::*;
use crate::errors::*;
use crate::service_discovery::udp_anycast::discovery_config::MulticastServiceDiscoveryConfig;

use cuneiform_fields::arch::ArchPadding;
use mio::net::UdpSocket;
use mio::{Events, Interest, Poll, Token};

use serde::*;
use std::collections::VecDeque;

use std::net::SocketAddr;

use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Ord, Eq)]
/// Default acknowledgement reply for the Discovery.
pub struct ServiceDiscoveryReply {
    /// Serialized data which can be contained in replies.
    pub serialized_data: String,
}

impl Default for ServiceDiscoveryReply {
    fn default() -> Self {
        Self {
            serialized_data: "DONE".into(),
        }
    }
}

pub(crate) enum ServiceDiscoveryRequest {
    RegisterObserver(ArchPadding<Sender<ServiceDiscoveryReply>>),
    SetBroadcastListen(bool),
    SeekPeers,
    Exit(Sender<()>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum ServiceDiscoveryMessage {
    Request,
    Response {
        uid: u32,
        content: ServiceDiscoveryReply,
    },
}

const ON_DISCOVERY: Token = Token(0);
const SEEK_NODES: Token = Token(1);

pub struct MulticastServiceDiscoveryState {
    config: MulticastServiceDiscoveryConfig,
    server_socket: UdpSocket,
    seek_request: Vec<u8>,
    observers: Vec<ArchPadding<Sender<ServiceDiscoveryReply>>>,
    seeker_replies: VecDeque<SocketAddr>,
    default_reply: ServiceDiscoveryReply,
    uid: u32,
    running: bool,
    listen: bool,
}

pub type ServiceDiscoveryReactor = (Poll, MulticastServiceDiscoveryState);

impl MulticastServiceDiscoveryState {
    pub(crate) fn new(
        config: MulticastServiceDiscoveryConfig,
        discovery_reply: ServiceDiscoveryReply,
    ) -> Result<ServiceDiscoveryReactor> {
        let poll: Poll = Poll::new()?;

        //        let interests = get_interests();
        //        let interests = get_interests();
        let mut server_socket = UdpSocket::bind(config.discovery_addr)?;
        server_socket.set_broadcast(true)?;

        poll.registry()
            .register(&mut server_socket, ON_DISCOVERY, get_interests())?;

        let uid = rand::random();
        let seek_request = serde_json::to_string(&ServiceDiscoveryMessage::Request)?;

        let state = MulticastServiceDiscoveryState {
            config,
            server_socket,
            seek_request: seek_request.as_bytes().into(),
            observers: Vec::new(),
            seeker_replies: VecDeque::new(),
            default_reply: discovery_reply,
            uid,
            listen: false,
            running: true,
        };

        Ok((poll, state))
    }

    fn readable(&mut self, buf: &mut [u8], poll: &mut Poll) -> Result<()> {
        if let Ok((_bytes_read, peer_addr)) = self.server_socket.recv_from(buf) {
            let serialized = std::str::from_utf8(buf)?.to_string().trim().to_string();
            let serialized = serialized.trim_matches(char::from(0x00));
            let msg: ServiceDiscoveryMessage = if let Ok(msg) = serde_json::from_str(serialized) {
                msg
            } else {
                return Ok(());
            };

            match msg {
                ServiceDiscoveryMessage::Request => {
                    if self.listen {
                        self.seeker_replies.push_back(peer_addr);
                        poll.registry().reregister(
                            &mut self.server_socket,
                            ON_DISCOVERY,
                            Interest::WRITABLE,
                        )?;
                    } else {
                        poll.registry().reregister(
                            &mut self.server_socket,
                            ON_DISCOVERY,
                            Interest::READABLE,
                        )?;
                    }
                }
                ServiceDiscoveryMessage::Response { uid, content } => {
                    if uid != self.uid {
                        self.observers
                            .retain(|observer| observer.send(content.clone()).is_ok());
                    }
                    poll.registry().reregister(
                        &mut self.server_socket,
                        ON_DISCOVERY,
                        Interest::READABLE,
                    )?;
                }
            }
        }

        Ok(())
    }

    fn writable(&mut self, poll: &mut Poll, token: Token) -> Result<()> {
        if token == ON_DISCOVERY {
            let reply = ServiceDiscoveryMessage::Response {
                uid: self.uid,
                content: self.default_reply.clone(),
            };
            let discovery_reply = serde_json::to_vec(&reply)?;

            while let Some(peer_addr) = self.seeker_replies.pop_front() {
                let mut sent_bytes = 0;
                while sent_bytes != discovery_reply.len() {
                    if let Ok(bytes_tx) = self
                        .server_socket
                        .send_to(&discovery_reply[sent_bytes..], peer_addr)
                    {
                        sent_bytes += bytes_tx;
                    } else {
                        poll.registry().reregister(
                            &mut self.server_socket,
                            ON_DISCOVERY,
                            Interest::WRITABLE,
                        )?;
                        return Ok(());
                    }
                }
            }
        } else if token == SEEK_NODES {
            let mut sent_bytes = 0;
            while sent_bytes != self.seek_request.len() {
                if let Ok(bytes_tx) = self
                    .server_socket
                    .send_to(&self.seek_request[sent_bytes..], self.config.seeking_addr)
                {
                    sent_bytes += bytes_tx;
                } else {
                    poll.registry().reregister(
                        &mut self.server_socket,
                        SEEK_NODES,
                        Interest::WRITABLE,
                    )?;
                    return Ok(());
                }
            }
        }

        Ok(poll
            .registry()
            .reregister(&mut self.server_socket, ON_DISCOVERY, Interest::WRITABLE)?)
    }

    pub(crate) fn event_loop(
        receiver: &mut Receiver<ServiceDiscoveryRequest>,
        mut poll: Poll,
        mut state: MulticastServiceDiscoveryState,
    ) -> Result<()> {
        let mut events = Events::with_capacity(1);
        let mut buf = [0_u8; CONST_PACKET_SIZE];

        let mut start = Instant::now();
        let timeout = Duration::from_millis(state.config.timeout_delta.num_milliseconds() as u64);

        // Our event loop.
        loop {
            let elapsed = start.elapsed();

            if elapsed >= timeout {
                start = Instant::now();
            }

            if !state.running {
                debug!("Stopping artillery udp_anycast service discovery evloop");
                break;
            }

            // Poll to check if we have events waiting for us.
            if let Some(remaining) = timeout.checked_sub(elapsed) {
                trace!("Polling events in SD evloop");
                poll.poll(&mut events, Some(remaining))?;
            }

            // Process our own events that are submitted to event loop
            // This is internal state machinery.
            while let Ok(msg) = receiver.try_recv() {
                let exit_tx = state.process_internal_request(&mut poll, msg);

                if let Some(exit_tx) = exit_tx {
                    debug!("Exit received!");
                    state.running = false;
                    exit_tx.send(()).unwrap();
                }
            }

            // Process inbound events
            for event in events.iter() {
                if event.is_readable() && event.token() == ON_DISCOVERY {
                    if let Err(err) = state.readable(&mut buf, &mut poll) {
                        error!("Service discovery error in READABLE: {:?}", err);
                        break;
                    }
                }

                if event.is_writable() {
                    if let Err(err) = state.writable(&mut poll, event.token()) {
                        error!("Service discovery error in WRITABLE: {:?}", err);
                        break;
                    }
                }
            }
        }

        info!("Exiting...");
        Ok(())
    }

    fn process_internal_request(
        &mut self,
        poll: &mut Poll,
        msg: ServiceDiscoveryRequest,
    ) -> Option<Sender<()>> {
        use ServiceDiscoveryRequest::*;

        match msg {
            RegisterObserver(sender) => self.observers.push(sender),
            SetBroadcastListen(bcast_listen) => {
                self.listen = bcast_listen;
            }
            SeekPeers => {
                match self
                    .server_socket
                    .send_to(&self.seek_request, self.config.seeking_addr)
                {
                    Ok(_) => {
                        if let Err(err) = poll.registry().reregister(
                            &mut self.server_socket,
                            ON_DISCOVERY,
                            Interest::READABLE,
                        ) {
                            error!("Reregistry error for Discovery: {:?}", err);
                            self.running = false;
                        }
                    }
                    Err(_err) => {
                        if let Err(err) = poll.registry().reregister(
                            &mut self.server_socket,
                            SEEK_NODES,
                            Interest::WRITABLE,
                        ) {
                            error!("Reregistry error for Seeking: {:?}", err);
                            self.running = false;
                        }
                    }
                }
            }
            Exit(tx) => return Some(tx),
        };

        None
    }
}

#[inline]
fn get_interests() -> Interest {
    Interest::READABLE.add(Interest::WRITABLE)
}

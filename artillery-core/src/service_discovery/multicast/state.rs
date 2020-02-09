use std::sync::mpsc::{Sender, Receiver};
use cuneiform_fields::arch::ArchPadding;
use crate::service_discovery::multicast::discovery_config::MulticastServiceDiscoveryConfig;
use mio::net::UdpSocket;
use mio::{Token, Events, Poll, Interest};
use crate::constants::*;
use crate::errors::*;
use failure::_core::sync::atomic::AtomicBool;
use std::time::{Instant, Duration};
use std::sync::atomic::Ordering;
use std::io;
use serde::*;
use bastion_utils::math::random;


pub trait ServiceDiscoveryReply: 'static + Serialize + Deserialize<'static> + Send + Clone {}

pub(crate) enum ServiceDiscoveryRequest<T> where T: ServiceDiscoveryReply {
    RegisterObserver(ArchPadding<Sender<T>>),
    SetBroadcastListen(bool),
    SeekPeers(Sender<()>),
    Exit(Sender<()>),
}

#[derive(Serialize, Deserialize)]
enum ServiceDiscoveryMessage<T>
where
    T: ServiceDiscoveryReply
{
    Request,
    Response { uid: u32, content: T },
}

const ON_DISCOVERY: Token = Token(0);
const SEEK_NODES: Token = Token(1);

pub struct MulticastServiceDiscoveryState<T>
where T: ServiceDiscoveryReply
{
    config: MulticastServiceDiscoveryConfig,
    server_socket: UdpSocket,
    request_tx: ArchPadding<Sender<ServiceDiscoveryRequest<T>>>,
    event_tx: ArchPadding<Sender<ServiceDiscoveryEvent>>,
    seek_request: Vec<u8>,
    observers: Vec<Sender<T>>,
    uid: u32,
    running: bool,
    listen: bool
}

pub type ServiceDiscoveryReactor<T> = (Poll, MulticastServiceDiscoveryState<T>);

impl<T> MulticastServiceDiscoveryState<T>
where T: ServiceDiscoveryReply
{
    pub fn new(config: MulticastServiceDiscoveryConfig,
               event_tx: Sender<ServiceDiscoveryEvent>,
               internal_tx: Sender<ServiceDiscoveryRequest<T>>) -> Result<ServiceDiscoveryReactor<T>> {
        let poll: Poll = Poll::new()?;

        let seek_request = serde_json::to_string(ServiceDiscoveryMessage::Request)?;

        let interests = get_interests();
        let mut server_socket = UdpSocket::bind(config.seeking_addr)?;
        server_socket.set_broadcast(true);

        poll.registry()
            .register(&mut server_socket, SEEK_NODES, interests)?;

        let uid = random(std::u32::MAX - 1);

        let state = MulticastServiceDiscoveryState {
            config,
            server_socket,
            request_tx: ArchPadding::new(internal_tx),
            event_tx: ArchPadding::new(event_tx),
            seek_request: vec![],
            observers: Vec::new(),
            uid,
            listen: false,
            running: true
        };

        Ok((poll, state))
    }

    fn readable(&mut self, buf: &mut [u8], poll: &mut Poll) -> Result<()> {
        if let Some((bytes_read, peer_addr)) = self.server_socket.recv_from(buf) {
            let msg: ServiceDiscoveryMessage<T> = if let Ok(msg) = serde_json::from_slice(buf) {
                msg
            } else {
                return Ok(());
            };

            match msg {
                ServiceDiscoveryMessage::Request => {
                    if self.listen {
                        self.reply_to.push_back(peer_addr);
                        poll.registry().reregister(&self.server_socket,
                                                   ON_DISCOVERY,
                                                   Interest::WRITABLE)?;
                    } else {
                        poll.registry().reregister(&self.server_socket,
                                                   ON_DISCOVERY,
                                                   Interest::READABLE)?;
                    }
                }
                ServiceDiscoveryMessage::Response { uid, content } => {
                    if uid != self.uid {
                        self.observers.retain(|observer| observer.send(content.clone()).is_ok());
                    }
                    poll.registry().reregister(&self.server_socket,
                                               ON_DISCOVERY,
                                               Interest::READABLE)?;
                }
            }
        }

        Ok(())
    }

    pub fn event_loop(receiver: &mut Receiver<ServiceDiscoveryRequest<T>>, mut poll: Poll, mut state: MulticastServiceDiscoveryState<T>) -> Result<()> {
        let mut events = Events::with_capacity(1);
        let mut buf = [0_u8; CONST_PACKET_SIZE];

        let mut start = Instant::now();
        let timeout = Duration::from_millis(state.config.ping_interval.num_milliseconds() as u64);

        // Our event loop.
        loop {
            let elapsed = start.elapsed();

            dbg!(elapsed);
            dbg!(timeout);
            if elapsed >= timeout {

                start = Instant::now();
            }

            if !state.running {
                debug!("Stopping artillery multicast service discovery evloop");
                break;
            }

            // Poll to check if we have events waiting for us.
            if let Some(remaining) = timeout.checked_sub(elapsed) {
                poll.poll(&mut events, Some(remaining))?;
            }

            // Process our own events that are submitted to event loop
            // This is internal state machinery.
            while let Ok(msg) = receiver.try_recv() {
                let exit_tx = state.process_internal_request(&mut poll, msg);

                if let Some(exit_tx) = exit_tx {
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

                if events.is_writable() {
                    if let Err(err) = state.writable(event_loop, token) {
                        error!("Service discovery error in WRITABLE: {:?}", err);
                        break;
                    }
                }
            }

//            for event in events.iter() {
//                match event.token() {
//                    ON_DISCOVERY => loop {
//                        match state.server_socket.recv_from(&mut buf) {
//                            Ok((packet_size, source_address)) => {
//                                Ok(())
//                            }
//                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
//                                // If we get a `WouldBlock` error we know our socket
//                                // has no more packets queued, so we can return to
//                                // polling and wait for some more.
//                                break;
//                            }
//                            Err(e) => {
//                                // If it was any other kind of error, something went
//                                // wrong and we terminate with an error.
//                                bail!(
//                                    ArtilleryError::UnexpectedError,
//                                    format!(
//                                        "Unexpected error occured in SD DISCOVERY event loop: {}",
//                                        e.to_string()
//                                    )
//                                )
//                            }
//                        }
//                    },
//                    SEEK_NODES => loop {
//                        match state.server_socket.recv_from(&mut buf) {
//                            Ok((packet_size, source_address)) => {
//                                Ok(())
//                            }
//                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
//                                // If we get a `WouldBlock` error we know our socket
//                                // has no more packets queued, so we can return to
//                                // polling and wait for some more.
//                                break;
//                            }
//                            Err(e) => {
//                                // If it was any other kind of error, something went
//                                // wrong and we terminate with an error.
//                                bail!(
//                                    ArtilleryError::UnexpectedError,
//                                    format!(
//                                        "Unexpected error occured in SD SEEK event loop: {}",
//                                        e.to_string()
//                                    )
//                                )
//                            }
//                        }
//                    }
//                    _ => {
//                        warn!("Got event for unexpected token: {:?}", event);
//                    }
//                }
//            }
        }

        info!("Exiting...");
        Ok(())
    }

    fn process_internal_request(&mut self, poll: &mut Poll, msg: ServiceDiscoveryRequest<T>) -> Option<Sender<()>> {
        use ServiceDiscoveryRequest::*;

        match message {
            RegisterObserver(sender) => self.observers.push(observer),
            SetBroadcastListen(bcast_listen) => {
                self.listen = bcast_listen;
            }
            SeekPeers(tx) => {
                match self.server_socket
                    .send_to(&self.seek_request, self.config.seeking_addr) {
                    Ok(_) => {
                        if let Err(err) = poll.registry()
                            .reregister(&mut self.server_socket,
                                        ON_DISCOVERY,
                                        Interest::READABLE) {
                            error!("Reregistry error for Discovery: {:?}", err);
                            return Some(tx);
                        }
                    }
                    Err(err) => {
                        if let Err(err) = poll.registry()
                            .reregister(&mut self.server_socket,
                                        SEEK_NODES,
                                        Interest::WRITABLE) {
                            error!("Reregistry error for Seeking: {:?}", err);
                            return Some(tx);
                        }
//                        error!("General Error for Service Discovery Internal Request: {:?}", err);
//                        return Some(tx);
                    }
                    _ => {
                        error!("Unexpected state response for Service Discovery Internal Request");
                        return None;
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
use crate::errors::*;
use super::cluster_config::ClusterConfig;
use uuid::Uuid;
use std::net::{SocketAddr};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use cuneiform_fields::prelude::*;
use super::membership::ArtilleryMemberList;
use crate::epidemic::member::{ArtilleryStateChange, ArtilleryMember};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use serde::*;
use mio::{Events, Interest, Poll, Token};
use std::io;
use mio::net::UdpSocket;

pub type ArtilleryClusterEvent = (Vec<ArtilleryMember>, ArtilleryMemberEvent);
pub type WaitList = HashMap<SocketAddr, Vec<SocketAddr>>;

#[derive(Debug)]
pub enum ArtilleryMemberEvent {
    MemberJoined(ArtilleryMember),
    MemberWentUp(ArtilleryMember),
    MemberSuspectedDown(ArtilleryMember),
    MemberWentDown(ArtilleryMember),
    MemberLeft(ArtilleryMember),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArtilleryMessage {
    sender: Uuid,
    cluster_key: Vec<u8>,
    request: Request,
    state_changes: Vec<ArtilleryStateChange>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct EncSocketAddr(SocketAddr);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
enum Request {
    Ping,
    Ack,
    PingRequest(EncSocketAddr),
    AckHost(ArtilleryMember),
}

#[derive(Debug, Clone)]
pub struct TargetedRequest {
    request: Request,
    target: SocketAddr,
}

#[derive(Clone)]
pub enum ArtilleryClusterRequest {
    AddSeed(SocketAddr),
    Respond(SocketAddr, ArtilleryMessage),
    React(TargetedRequest),
    LeaveCluster,
    Exit(Sender<()>),
}

const UDP_SERVER: Token = Token(0);

pub struct ArtilleryState {
    host_key: Uuid,
    config: ClusterConfig,
    members: ArtilleryMemberList,
    seed_queue: Vec<SocketAddr>,
    pending_responses: Vec<(DateTime<Utc>, SocketAddr, Vec<ArtilleryStateChange>)>,
    state_changes: Vec<ArtilleryStateChange>,
    wait_list: WaitList,
    server_socket: UdpSocket,
    poller: Poll,
    request_tx: ArchPadding<Sender<ArtilleryClusterRequest>>,
    event_tx: ArchPadding<Sender<ArtilleryClusterEvent>>,
}

pub type ClusterReactor = (Poll, ArtilleryState);

impl ArtilleryState {
    pub fn new(host_key: Uuid,
           config: ClusterConfig,
           event_tx: Sender<ArtilleryClusterEvent>,
           internal_tx: Sender<ArtilleryClusterRequest>) -> Result<ArtilleryState> {
        let mut poll: Poll = Poll::new()?;

        let interests = Interest::READABLE.add(Interest::WRITABLE);
        let mut server_socket = UdpSocket::bind(config.listen_addr)?;
        poll.registry()
            .register(&mut server_socket, UDP_SERVER, interests)?;

        let me = ArtilleryMember::current(host_key.clone());

        let state = ArtilleryState {
            host_key,
            config,
            members: ArtilleryMemberList::new(me.clone()),
            seed_queue: Vec::new(),
            pending_responses: Vec::new(),
            state_changes: vec![ArtilleryStateChange::new(me)],
            wait_list: HashMap::new(),
            server_socket,
            poller: poll,
            request_tx: ArchPadding::new(internal_tx),
            event_tx: ArchPadding::new(event_tx),
        };

//        event_loop.timeout_ms((), state.config.ping_interval.num_milliseconds() as u64).unwrap();

        Ok(state)
    }

    pub(crate) fn event_loop(receiver: &mut Receiver<ArtilleryClusterRequest>, state: &mut ArtilleryState) -> Result<()> {
        let mut poll: &mut Poll = &mut state.poller;
        let mut events = Events::with_capacity(1_000);
        let mut buf = [0_u8; 1 << 16];

        // Our event loop.
        loop {
            // Poll to check if we have events waiting for us.
            poll.poll(&mut events, None)?;

            // Process our own events that are submitted to event loop
            // Aka outbound events
            while let Ok(msg) = receiver.try_recv() {
                unimplemented!()
            }

            // Process inbound events
            for event in events.iter() {
                // Validate the token we registered our socket with,
                // in this example it will only ever be one but we
                // make sure it's valid none the less.
                match event.token() {
                    UDP_SERVER => loop {
                        match state.server_socket.recv_from(&mut buf) {
                            Ok((packet_size, source_address)) => {
                                // Echo the data.
                                let message = serde_json::from_str(&*String::from_utf8_lossy(&buf[..packet_size]))?;
                                state.request_tx.send(ArtilleryClusterRequest::Respond(source_address, message))?;
                            }
                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                                // If we get a `WouldBlock` error we know our socket
                                // has no more packets queued, so we can return to
                                // polling and wait for some more.
                                break;
                            }
                            Err(e) => {
                                // If it was any other kind of error, something went
                                // wrong and we terminate with an error.
                                bail!(
                                    ArtilleryError::UnexpectedError,
                                    format!(
                                        "Unexpected error occured in event loop: {}",
                                        e.to_string()
                                    )
                                )
                            }
                        }
                    },
                    _ => {
                        // This should never happen as we only registered our
                        // `UdpSocket` using the `UDP_SOCKET` token, but if it ever
                        // does we'll log it.
                        warn!("Got event for unexpected token: {:?}", event);
                    }
                }
            }
        }
    }
}

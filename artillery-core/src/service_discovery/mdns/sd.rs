use crate::errors::*;
use crate::service_discovery::mdns::discovery_config::MDNSServiceDiscoveryConfig;
use crate::service_discovery::mdns::state::MDNSServiceDiscoveryEvent;
use bastion_executor::blocking::spawn_blocking;

use libp2p::mdns::service::*;
use libp2p::multiaddr::Protocol;
use libp2p::{identity, Multiaddr, PeerId};
use lightproc::proc_stack::ProcStack;

use std::sync::mpsc::{channel, Receiver};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::sync::Arc;


pub struct MDNSServiceDiscovery {
    events: Arc<Receiver<MDNSServiceDiscoveryEvent>>,
}

unsafe impl Send for MDNSServiceDiscovery {}
unsafe impl Sync for MDNSServiceDiscovery {}

impl MDNSServiceDiscovery {
    pub fn new_service_discovery(config: MDNSServiceDiscoveryConfig) -> Result<Self> {
        let (event_tx, event_rx) = channel::<MDNSServiceDiscoveryEvent>();

        let peer_id = PeerId::from(identity::Keypair::generate_ed25519().public());

        let _discovery_handle = spawn_blocking(
            async move {
                let mut service = MdnsService::new().expect("Can't launch the MDNS service");

                loop {
                    let (mut srv, packet) = service.next().await;
                    match packet {
                        MdnsPacket::Query(query) => {
                            debug!("Query from {:?}", query.remote_addr());
                            let address: Multiaddr = format!(
                                "/ip4/{}/udp/{}",
                                config.local_service_addr.ip().to_string(),
                                config.local_service_addr.port()
                            )
                            .parse()
                            .unwrap();
                            let resp = build_query_response(
                                query.query_id(),
                                peer_id.clone(),
                                vec![address].into_iter(),
                                config.reply_ttl,
                            )
                            .unwrap();
                            srv.enqueue_response(resp);
                        }
                        MdnsPacket::Response(response) => {
                            // We detected a libp2p mDNS response on the network. Responses are for
                            // everyone and not just for the requester, which makes it possible to
                            // passively listen.
                            for peer in response.discovered_peers() {
                                debug!("Discovered peer {:?}", peer.id());
                                // These are the self-reported addresses of the peer we just discovered.
                                for addr in peer.addresses() {
                                    debug!(" Address = {:?}", addr);
                                    let components = addr.iter().collect::<Vec<_>>();
                                    if let Protocol::Ip4(discovered_ip) = components[0] {
                                        if let Protocol::Udp(discovered_port) = components[1] {
                                            let discovered =
                                                format!("{}:{}", discovered_ip, discovered_port)
                                                    .parse()
                                                    .unwrap();
                                            event_tx
                                                .send(MDNSServiceDiscoveryEvent(discovered))
                                                .unwrap();
                                        } else {
                                            error!(
                                                "Unexpected protocol received: {}",
                                                components[1]
                                            );
                                        }
                                    } else {
                                        error!("Unexpected IP received: {}", components[0]);
                                    }
                                }
                            }
                        }
                        MdnsPacket::ServiceDiscovery(query) => {
                            // The last possibility is a service detection query from DNS-SD.
                            // Just like `Query`, in a real application you probably want to call
                            // `query.respond`.
                            debug!("Detected service query from {:?}", query.remote_addr());
                        }
                    }
                    service = srv
                }
            },
            ProcStack::default(),
        );

        Ok(Self { events: Arc::new(event_rx) })
    }

    pub fn events(&self) -> Arc<Receiver<MDNSServiceDiscoveryEvent>> {
        self.events.clone()
    }
}

impl Future for MDNSServiceDiscovery {
    type Output = MDNSServiceDiscoveryEvent;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        loop {
            return match self.events.recv() {
                Ok(kv) => Poll::Ready(kv),
                Err(_) => Poll::Pending
            }
        }
    }
}

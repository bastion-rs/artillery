use cuneiform_fields::prelude::*;
use std::sync::mpsc::{Sender, Receiver, channel};
use crate::errors::*;
use crate::service_discovery::mdns::discovery_config::MDNSServiceDiscoveryConfig;
use libp2p::mdns::service::*;
use crate::service_discovery::mdns::state::MDNSServiceDiscoveryEvent;
use bastion_executor::blocking::spawn_blocking;
use lightproc::proc_stack::ProcStack;
use lightproc::recoverable_handle::RecoverableHandle;
use libp2p::{identity, Multiaddr, PeerId};
use libp2p::multiaddr::Protocol;
use std::net::SocketAddr;


pub struct MDNSServiceDiscovery {
    pub events: Receiver<MDNSServiceDiscoveryEvent>
}

impl MDNSServiceDiscovery {
    pub fn new_service_discovery(
        config: MDNSServiceDiscoveryConfig,
    ) -> Result<Self> {
        let (event_tx, event_rx) =
            channel::<MDNSServiceDiscoveryEvent>();

        let peer_id = PeerId::from(identity::Keypair::generate_ed25519().public());

        let discovery_handle = spawn_blocking(async move {
            let mut service = MdnsService::new()
                .expect("Can't launch the MDNS service");

            loop {
                let (mut srv, packet) = service.next().await;
                match packet {
                    MdnsPacket::Query(query) => {
                        debug!("Query from {:?}", query.remote_addr());
                        let mut address: Multiaddr = format!(
                            "/ip4/{}/udp/{}",
                            config.local_service_addr.ip().to_string(),
                            config.local_service_addr.port()
                        ).parse().unwrap();
                        let resp = build_query_response(
                            query.query_id(),
                            peer_id.clone(),
                            vec![address].into_iter(),
                            config.reply_ttl,
                        ).unwrap();
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
                                        event_tx.send(MDNSServiceDiscoveryEvent(discovered))
                                            .unwrap();
                                    } else {
                                        error!("Unexpected protocol received: {}", components[1]);
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
        }, ProcStack::default());

        Ok(Self {
            events: event_rx
        })
    }
}
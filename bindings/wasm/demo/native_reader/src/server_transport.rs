use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Mutex, mpsc},
};
use tokio_tungstenite::tungstenite::Message;

use dust_dds::transport::{
    interface::{RtpsParticipant, Transport, Write},
    types::{ENTITYID_PARTICIPANT, Guid, GuidPrefix, Locator},
};
use webrtc::{
    api::{APIBuilder, setting_engine::SettingEngine},
    data_channel::{RTCDataChannel, data_channel_init::RTCDataChannelInit},
    ice_transport::ice_candidate::RTCIceCandidateInit,
    peer_connection::{
        configuration::RTCConfiguration, sdp::session_description::RTCSessionDescription,
    },
};

pub const LOCATOR_KIND_WEBRTC: i32 = 100;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum SignalingMessage {
    Register {
        domain_id: i32,
        guid_prefix: [u8; 12],
    },
    SdpOffer {
        from: [u8; 12],
        to: [u8; 12],
        sdp: String,
    },
    SdpAnswer {
        from: [u8; 12],
        to: [u8; 12],
        sdp: String,
    },
    IceCandidate {
        from: [u8; 12],
        to: [u8; 12],
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
    },
    PeerJoined {
        guid_prefix: [u8; 12],
    },
    PeerLeft {
        guid_prefix: [u8; 12],
    },
}

pub struct OutboundDatagram {
    pub locators: Vec<Locator>,
    pub data: Vec<u8>,
}

pub struct RtpsWebRtcServerWriter<'a> {
    locators: &'a [Locator],
    outgoing_tx: &'a mpsc::Sender<OutboundDatagram>,
    buffer: Vec<u8>,
}

impl<'a> Write for RtpsWebRtcServerWriter<'a> {
    fn write(&mut self, buf: &[u8]) {
        self.buffer.extend_from_slice(buf);
    }

    fn flush(&mut self) {
        if !self.buffer.is_empty() {
            let data = std::mem::take(&mut self.buffer);
            let _ = self.outgoing_tx.try_send(OutboundDatagram {
                locators: self.locators.to_vec(),
                data,
            });
        }
    }
}

pub struct RtpsWebRtcServerParticipant {
    guid: Guid,
    default_unicast_locator_list: Vec<Locator>,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    outgoing_tx: mpsc::Sender<OutboundDatagram>,
    incoming_rx: Arc<Mutex<mpsc::Receiver<Vec<u8>>>>,
    receive_buffer: Vec<u8>,
    fragment_size: usize,
}

impl RtpsParticipant for RtpsWebRtcServerParticipant {
    type Writer<'a> = RtpsWebRtcServerWriter<'a>;

    fn guid(&self) -> Guid {
        self.guid
    }

    fn default_unicast_locator_list(&self) -> &[Locator] {
        &self.default_unicast_locator_list
    }

    fn metatraffic_unicast_locator_list(&self) -> &[Locator] {
        &self.metatraffic_unicast_locator_list
    }

    fn metatraffic_multicast_locator_list(&self) -> &[Locator] {
        &self.metatraffic_multicast_locator_list
    }

    fn default_multicast_locator_list(&self) -> &[Locator] {
        &self.default_multicast_locator_list
    }

    fn fragment_size(&self) -> usize {
        self.fragment_size
    }

    fn writer<'a>(&'a mut self, locators: &'a [Locator]) -> Self::Writer<'a> {
        RtpsWebRtcServerWriter {
            locators,
            outgoing_tx: &self.outgoing_tx,
            buffer: Vec::new(),
        }
    }

    async fn read(&mut self) -> &[u8] {
        if let Some(data) = self.incoming_rx.lock().await.recv().await {
            self.receive_buffer = data;
        } else {
            self.receive_buffer.clear();
        }
        &self.receive_buffer
    }
}

struct ClientSession {
    ws_tx: mpsc::Sender<Message>,
    data_channel: Arc<Mutex<Option<Arc<RTCDataChannel>>>>,
}

pub struct RtpsWebRtcServerTransport {
    bind_addr: String,
    tokio_handle: tokio::runtime::Handle,
    fragment_size: usize,
    participant_counter: u32,
}

impl RtpsWebRtcServerTransport {
    pub fn new(bind_addr: impl Into<String>, tokio_handle: tokio::runtime::Handle) -> Self {
        Self {
            bind_addr: bind_addr.into(),
            tokio_handle,
            fragment_size: 1300,
            participant_counter: 0,
        }
    }
}

impl Transport for RtpsWebRtcServerTransport {
    type Participant = RtpsWebRtcServerParticipant;

    fn create_participant(&mut self, domain_id: i32) -> Self::Participant {
        self.participant_counter = self.participant_counter.wrapping_add(1);
        let c = self.participant_counter.to_le_bytes();

        let pid = std::process::id().to_le_bytes();
        let guid_prefix: GuidPrefix = [
            0x02, 0x01, pid[0], pid[1], pid[2], pid[3], 0xaa, 0xbb, c[0], c[1], c[2], c[3],
        ];
        let guid = Guid::new(guid_prefix, ENTITYID_PARTICIPANT);

        let mut locator_address = [0u8; 16];
        locator_address[0..12].copy_from_slice(&guid_prefix);

        let unicast_locator = Locator::new(LOCATOR_KIND_WEBRTC, 0, locator_address);
        let multicast_locator = Locator::new(LOCATOR_KIND_WEBRTC, 0, [0u8; 16]);

        let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<OutboundDatagram>(128);
        let (incoming_tx, incoming_rx) = mpsc::channel::<Vec<u8>>(128);

        let clients: Arc<Mutex<HashMap<[u8; 12], ClientSession>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let clients_for_server = clients.clone();
        let clients_for_writer = clients.clone();

        let bind_addr = self.bind_addr.clone();

        self.tokio_handle.spawn(async move {
            let socket_addr: SocketAddr = match bind_addr.parse() {
                Ok(a) => a,
                Err(_) => match tokio::net::lookup_host(&bind_addr).await {
                    Ok(mut addrs) => addrs.next().unwrap_or(([0, 0, 0, 0], 9001).into()),
                    Err(_) => ([0, 0, 0, 0], 9001).into(),
                },
            };

            let listener = match TcpListener::bind(socket_addr).await {
                Ok(l) => l,
                Err(e) => {
                    tracing::error!("Failed to bind WebSocket server on {socket_addr}: {e}");
                    return;
                }
            };

            tracing::info!("Dust-DDS WebRTC/WebSocket transport listening on {socket_addr}");

            while let Ok((stream, _)) = listener.accept().await {
                let clients_map = clients_for_server.clone();
                let incoming_sender = incoming_tx.clone();

                tokio::spawn(async move {
                    handle_incoming_connection(
                        stream,
                        domain_id,
                        guid_prefix,
                        clients_map,
                        incoming_sender,
                    )
                    .await;
                });
            }
        });

        // Outgoing write loop
        self.tokio_handle.spawn(async move {
            while let Some(msg) = outgoing_rx.recv().await {
                let clients_guard = clients_for_writer.lock().await;

                for loc in &msg.locators {
                    let addr = loc.address();
                    let is_multicast = addr == [0u8; 16];

                    if is_multicast {
                        for session in clients_guard.values() {
                            if let Some(ref dc) = *session.data_channel.lock().await {
                                if dc.ready_state() == webrtc::data_channel::data_channel_state::RTCDataChannelState::Open {
                                    let _ = dc.send(&msg.data.clone().into()).await;
                                }
                            }
                        }
                    } else {
                        let mut target = [0u8; 12];
                        target.copy_from_slice(&addr[0..12]);

                        if let Some(session) = clients_guard.get(&target) {
                            if let Some(ref dc) = *session.data_channel.lock().await {
                                if dc.ready_state() == webrtc::data_channel::data_channel_state::RTCDataChannelState::Open {
                                    let _ = dc.send(&msg.data.clone().into()).await;
                                }
                            }
                        }
                    }
                }
            }
        });

        RtpsWebRtcServerParticipant {
            guid,
            default_unicast_locator_list: vec![unicast_locator],
            metatraffic_unicast_locator_list: vec![unicast_locator],
            metatraffic_multicast_locator_list: vec![multicast_locator],
            default_multicast_locator_list: Vec::new(),
            outgoing_tx,
            incoming_rx: Arc::new(Mutex::new(incoming_rx)),
            receive_buffer: Vec::new(),
            fragment_size: self.fragment_size,
        }
    }

    fn delete_participant(&mut self, _handle: Guid) {}
}

async fn handle_incoming_connection(
    stream: TcpStream,
    native_domain_id: i32,
    native_guid_prefix: GuidPrefix,
    clients: Arc<Mutex<HashMap<[u8; 12], ClientSession>>>,
    incoming_tx: mpsc::Sender<Vec<u8>>,
) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            tracing::warn!("Failed WebSocket handshake: {e}");
            return;
        }
    };

    let (mut ws_sink, mut ws_stream) = ws_stream.split();
    let (ws_out_tx, mut ws_out_rx) = mpsc::channel::<Message>(32);

    // Forward outbound WebSocket messages
    tokio::spawn(async move {
        while let Some(msg) = ws_out_rx.recv().await {
            if ws_sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    let dc_holder: Arc<Mutex<Option<Arc<RTCDataChannel>>>> = Arc::new(Mutex::new(None));
    let mut registered_prefix: Option<[u8; 12]> = None;

    // Initialize WebRTC PeerConnection with loopback candidates enabled
    let mut setting_engine = SettingEngine::default();
    setting_engine.set_include_loopback_candidate(true);
    let webrtc_api = APIBuilder::new()
        .with_setting_engine(setting_engine)
        .build();
    let pc = match webrtc_api
        .new_peer_connection(RTCConfiguration::default())
        .await
    {
        Ok(pc) => Arc::new(pc),
        Err(e) => {
            tracing::error!("Failed to create WebRTC PeerConnection: {e}");
            return;
        }
    };

    pc.on_peer_connection_state_change(Box::new(move |s| {
        tracing::info!("[Native WebRTC] Peer connection state changed: {s}");
        Box::pin(async move {})
    }));

    pc.on_ice_connection_state_change(Box::new(move |s| {
        tracing::info!("[Native WebRTC] ICE connection state changed: {s}");
        Box::pin(async move {})
    }));

    // DataChannel message handling
    let incoming_for_dc = incoming_tx.clone();
    let dc_holder_clone = dc_holder.clone();

    // Setup on_data_channel from client
    let dc_holder_for_remote = dc_holder.clone();
    let incoming_for_remote_dc = incoming_tx.clone();
    pc.on_data_channel(Box::new(move |d: Arc<RTCDataChannel>| {
        let dc_holder = dc_holder_for_remote.clone();
        let incoming_tx = incoming_for_remote_dc.clone();
        tracing::info!("[Native WebRTC] Remote DataChannel received: {}", d.label());
        d.on_open(Box::new(move || {
            tracing::info!("[Native WebRTC] Remote DataChannel OPENED!");
            Box::pin(async move {})
        }));
        Box::pin(async move {
            *dc_holder.lock().await = Some(d.clone());
            d.on_message(Box::new(move |msg| {
                let incoming = incoming_tx.clone();
                let bytes = msg.data.to_vec();
                Box::pin(async move {
                    let _ = incoming.send(bytes).await;
                })
            }));
        })
    }));

    // Setup local data channel
    let dc_init = RTCDataChannelInit {
        ordered: Some(false),
        max_retransmits: Some(0),
        ..Default::default()
    };
    if let Ok(dc) = pc.create_data_channel("dust_rtps", Some(dc_init)).await {
        *dc_holder_clone.lock().await = Some(dc.clone());
        dc.on_open(Box::new(move || {
            tracing::info!("[Native WebRTC] Local DataChannel 'dust_rtps' OPENED!");
            Box::pin(async move {})
        }));
        let incoming_tx = incoming_for_dc.clone();
        dc.on_message(Box::new(move |msg| {
            let incoming = incoming_tx.clone();
            let bytes = msg.data.to_vec();
            Box::pin(async move {
                let _ = incoming.send(bytes).await;
            })
        }));
    }

    // ICE candidate signaling
    let ws_out_for_ice = ws_out_tx.clone();
    pc.on_ice_candidate(Box::new(move |c| {
        let ws_tx = ws_out_for_ice.clone();
        Box::pin(async move {
            if let Some(candidate) = c {
                if let Ok(json_cand) = candidate.to_json() {
                    tracing::info!(
                        "[Native WebRTC] Generated local ICE candidate: {}",
                        json_cand.candidate
                    );
                    let msg = SignalingMessage::IceCandidate {
                        from: native_guid_prefix,
                        to: [0; 12],
                        candidate: json_cand.candidate,
                        sdp_mid: json_cand.sdp_mid,
                        sdp_m_line_index: json_cand.sdp_mline_index,
                    };
                    if let Ok(text) = serde_json::to_string(&msg) {
                        let _ = ws_tx.send(Message::Text(text.into())).await;
                    }
                }
            }
        })
    }));

    // Receive messages from the client
    while let Some(Ok(msg)) = ws_stream.next().await {
        match msg {
            Message::Binary(_) => {
                tracing::warn!(
                    "Ignored binary frame on discovery WebSocket: data must flow via WebRTC"
                );
            }
            Message::Text(txt) => {
                if let Ok(signaling) = serde_json::from_str::<SignalingMessage>(&txt) {
                    match signaling {
                        SignalingMessage::Register {
                            domain_id,
                            guid_prefix,
                        } => {
                            if domain_id == native_domain_id {
                                registered_prefix = Some(guid_prefix);
                                clients.lock().await.insert(
                                    guid_prefix,
                                    ClientSession {
                                        ws_tx: ws_out_tx.clone(),
                                        data_channel: dc_holder.clone(),
                                    },
                                );

                                // Send offer to client
                                if let Ok(offer) = pc.create_offer(None).await {
                                    if pc.set_local_description(offer.clone()).await.is_ok() {
                                        let offer_msg = SignalingMessage::SdpOffer {
                                            from: native_guid_prefix,
                                            to: guid_prefix,
                                            sdp: offer.sdp,
                                        };
                                        if let Ok(text) = serde_json::to_string(&offer_msg) {
                                            let _ =
                                                ws_out_tx.send(Message::Text(text.into())).await;
                                        }
                                    }
                                }

                                // Announce peer joined to other clients
                                let joined_msg = SignalingMessage::PeerJoined { guid_prefix };
                                if let Ok(joined_text) = serde_json::to_string(&joined_msg) {
                                    let clients_map = clients.lock().await;
                                    for (other_prefix, other_session) in clients_map.iter() {
                                        if *other_prefix != guid_prefix {
                                            let _ = other_session
                                                .ws_tx
                                                .send(Message::Text(joined_text.clone().into()))
                                                .await;
                                        }
                                    }
                                }
                            }
                        }
                        SignalingMessage::SdpOffer { to, .. } => {
                            let clients_map = clients.lock().await;
                            if let Some(target_session) = clients_map.get(&to) {
                                let _ = target_session.ws_tx.send(Message::Text(txt.into())).await;
                            }
                        }
                        SignalingMessage::SdpAnswer { from, sdp, .. } => {
                            if registered_prefix == Some(from) {
                                let answer = RTCSessionDescription::answer(sdp).unwrap_or_default();
                                let _ = pc.set_remote_description(answer).await;
                            } else {
                                let clients_map = clients.lock().await;
                                if let Some(target_session) = clients_map.get(&from) {
                                    let _ =
                                        target_session.ws_tx.send(Message::Text(txt.into())).await;
                                }
                            }
                        }
                        SignalingMessage::IceCandidate {
                            candidate,
                            sdp_mid,
                            sdp_m_line_index,
                            to,
                            ..
                        } => {
                            if to == native_guid_prefix || to == [0; 12] {
                                tracing::info!(
                                    "[Native WebRTC] Received remote ICE candidate: {candidate}"
                                );
                                let init = RTCIceCandidateInit {
                                    candidate,
                                    sdp_mid,
                                    sdp_mline_index: sdp_m_line_index,
                                    username_fragment: None,
                                };
                                if let Err(e) = pc.add_ice_candidate(init).await {
                                    tracing::warn!(
                                        "[Native WebRTC] Failed to add ICE candidate: {e}"
                                    );
                                }
                            } else {
                                let clients_map = clients.lock().await;
                                if let Some(target_session) = clients_map.get(&to) {
                                    let _ =
                                        target_session.ws_tx.send(Message::Text(txt.into())).await;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(prefix) = registered_prefix {
        clients.lock().await.remove(&prefix);
        let left_msg = SignalingMessage::PeerLeft {
            guid_prefix: prefix,
        };
        if let Ok(left_text) = serde_json::to_string(&left_msg) {
            let clients_map = clients.lock().await;
            for session in clients_map.values() {
                let _ = session
                    .ws_tx
                    .send(Message::Text(left_text.clone().into()))
                    .await;
            }
        }
    }
}

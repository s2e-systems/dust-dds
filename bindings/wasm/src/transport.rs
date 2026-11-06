use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::spawn_local;

use dust_dds::transport::{
    interface::{RtpsParticipant, Transport, Write},
    types::{ENTITYID_PARTICIPANT, Guid, GuidPrefix, Locator},
};
use serde::{Deserialize, Serialize};

pub const LOCATOR_KIND_WEBRTC: i32 = 100;
pub const DEFAULT_DISCOVERY_URL: &str = "ws://dust-dds.local:9001";

/// Signaling message envelope exchanged over WebSocket.
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

pub enum TransportOutboundMail {
    SendDatagram {
        locators: Vec<Locator>,
        data: Vec<u8>,
    },
    Close,
}

pub struct WasmWriter<'a> {
    locators: &'a [Locator],
    sender: &'a async_channel::Sender<TransportOutboundMail>,
    buffer: Vec<u8>,
}

impl<'a> Write for WasmWriter<'a> {
    fn write(&mut self, buf: &[u8]) {
        self.buffer.extend_from_slice(buf);
    }

    fn flush(&mut self) {
        if !self.buffer.is_empty() {
            let data = std::mem::take(&mut self.buffer);
            let _ = self.sender.try_send(TransportOutboundMail::SendDatagram {
                locators: self.locators.to_vec(),
                data,
            });
        }
    }
}

pub struct WasmRtpsParticipant {
    guid: Guid,
    default_unicast_locator_list: Vec<Locator>,
    metatraffic_unicast_locator_list: Vec<Locator>,
    metatraffic_multicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    outgoing_sender: async_channel::Sender<TransportOutboundMail>,
    incoming_receiver: async_channel::Receiver<Vec<u8>>,
    receive_buffer: Vec<u8>,
}

impl RtpsParticipant for WasmRtpsParticipant {
    type Writer<'a> = WasmWriter<'a>;

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
        1300
    }

    fn writer<'a>(&'a mut self, locators: &'a [Locator]) -> Self::Writer<'a> {
        WasmWriter {
            locators,
            sender: &self.outgoing_sender,
            buffer: Vec::new(),
        }
    }

    async fn read(&mut self) -> &[u8] {
        if let Ok(data) = self.incoming_receiver.recv().await {
            self.receive_buffer = data;
        } else {
            self.receive_buffer.clear();
        }
        &self.receive_buffer
    }
}

pub struct WasmTransport {
    server_url: String,
    participant_counter: u32,
}

impl WasmTransport {
    pub fn new() -> Self {
        Self::with_url(DEFAULT_DISCOVERY_URL)
    }

    pub fn with_url(server_url: impl Into<String>) -> Self {
        Self {
            server_url: server_url.into(),
            participant_counter: 0,
        }
    }
}

impl Default for WasmTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for WasmTransport {
    type Participant = WasmRtpsParticipant;

    fn create_participant(&mut self, domain_id: i32) -> Self::Participant {
        self.participant_counter = self.participant_counter.wrapping_add(1);
        let c = self.participant_counter.to_le_bytes();

        let random_bytes = js_sys::Math::random().to_bits().to_le_bytes();
        let guid_prefix: GuidPrefix = [
            0x01,
            0x0f,
            random_bytes[0],
            random_bytes[1],
            random_bytes[2],
            random_bytes[3],
            random_bytes[4],
            random_bytes[5],
            c[0],
            c[1],
            c[2],
            c[3],
        ];

        let guid = Guid::new(guid_prefix, ENTITYID_PARTICIPANT);

        let mut locator_address = [0u8; 16];
        locator_address[0..12].copy_from_slice(&guid_prefix);

        let unicast_locator = Locator::new(LOCATOR_KIND_WEBRTC, 0, locator_address);
        let multicast_locator = Locator::new(LOCATOR_KIND_WEBRTC, 0, [0u8; 16]);

        let (outgoing_sender, outgoing_receiver) =
            async_channel::bounded::<TransportOutboundMail>(64);
        let (incoming_sender, incoming_receiver) = async_channel::bounded::<Vec<u8>>(64);

        let server_url = self.server_url.clone();
        spawn_transport_actor(
            server_url,
            domain_id,
            guid_prefix,
            outgoing_receiver,
            incoming_sender,
        );

        WasmRtpsParticipant {
            guid,
            default_unicast_locator_list: vec![unicast_locator],
            metatraffic_unicast_locator_list: vec![unicast_locator],
            metatraffic_multicast_locator_list: vec![multicast_locator],
            default_multicast_locator_list: Vec::new(),
            outgoing_sender,
            incoming_receiver,
            receive_buffer: Vec::new(),
        }
    }

    fn delete_participant(&mut self, _handle: Guid) {}
}

struct PeerState {
    peer_connection: web_sys::RtcPeerConnection,
    data_channel: Option<web_sys::RtcDataChannel>,
}

fn spawn_transport_actor(
    server_url: String,
    domain_id: i32,
    guid_prefix: GuidPrefix,
    outgoing_receiver: async_channel::Receiver<TransportOutboundMail>,
    incoming_sender: async_channel::Sender<Vec<u8>>,
) {
    spawn_local(async move {
        let ws = match web_sys::WebSocket::new(&server_url) {
            Ok(ws) => ws,
            Err(e) => {
                web_sys::console::warn_2(
                    &"[DustDds WasmTransport] Failed to open WebSocket:".into(),
                    &e,
                );
                return;
            }
        };

        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let peers: Rc<RefCell<HashMap<[u8; 12], PeerState>>> =
            Rc::new(RefCell::new(HashMap::new()));
        let ws_rc = Rc::new(RefCell::new(Some(ws.clone())));

        // Setup WebSocket onopen
        let ws_clone = ws.clone();
        let onopen = Closure::<dyn FnMut()>::new(move || {
            let reg = SignalingMessage::Register {
                domain_id,
                guid_prefix,
            };
            if let Ok(json) = serde_json::to_string(&reg) {
                let _ = ws_clone.send_with_str(&json);
            }
        });
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        // Setup WebSocket onmessage
        let incoming_tx = incoming_sender.clone();
        let ws_for_signaling = ws.clone();
        let peers_for_signaling = peers.clone();
        let incoming_tx_for_dc = incoming_sender.clone();

        let onmessage =
            Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |ev: web_sys::MessageEvent| {
                let data = ev.data();
                if let Ok(ab) = data.clone().dyn_into::<js_sys::ArrayBuffer>() {
                    let u8_array = js_sys::Uint8Array::new(&ab);
                    let _ = incoming_tx.try_send(u8_array.to_vec());
                } else if let Some(txt) = data.as_string() {
                    if let Ok(msg) = serde_json::from_str::<SignalingMessage>(&txt) {
                        handle_signaling_message(
                            msg,
                            guid_prefix,
                            &ws_for_signaling,
                            &peers_for_signaling,
                            &incoming_tx_for_dc,
                        );
                    }
                }
            });
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        // Process outgoing messages from Dust-DDS
        while let Ok(mail) = outgoing_receiver.recv().await {
            match mail {
                TransportOutboundMail::SendDatagram { locators, data } => {
                    let mut sent_to_any_dc = false;

                    for loc in &locators {
                        let addr = loc.address();
                        let is_multicast = addr == [0u8; 16];

                        if is_multicast {
                            // Send binary datagram through WebSocket relay/broadcast
                            if let Some(ref ws) = *ws_rc.borrow() {
                                let _ = ws.send_with_u8_array(&data);
                            }
                        } else {
                            let mut target_prefix = [0u8; 12];
                            target_prefix.copy_from_slice(&addr[0..12]);

                            let peers_map = peers.borrow();
                            if let Some(peer) = peers_map.get(&target_prefix) {
                                if let Some(ref dc) = peer.data_channel {
                                    if dc.ready_state() == web_sys::RtcDataChannelState::Open {
                                        let _ = dc.send_with_u8_array(&data);
                                        sent_to_any_dc = true;
                                        continue;
                                    }
                                }
                            }

                            // Fallback to WebSocket relay if DC not open yet
                            if !sent_to_any_dc {
                                if let Some(ref ws) = *ws_rc.borrow() {
                                    let _ = ws.send_with_u8_array(&data);
                                }
                            }
                        }
                    }
                }
                TransportOutboundMail::Close => {
                    if let Some(ws) = ws_rc.borrow_mut().take() {
                        let _ = ws.close();
                    }
                    break;
                }
            }
        }
    });
}

fn handle_signaling_message(
    msg: SignalingMessage,
    my_guid_prefix: GuidPrefix,
    ws: &web_sys::WebSocket,
    peers: &Rc<RefCell<HashMap<[u8; 12], PeerState>>>,
    incoming_tx: &async_channel::Sender<Vec<u8>>,
) {
    match msg {
        SignalingMessage::PeerJoined { guid_prefix } => {
            if guid_prefix != my_guid_prefix {
                initiate_peer_connection(guid_prefix, my_guid_prefix, ws, peers, incoming_tx);
            }
        }
        SignalingMessage::SdpOffer { from, sdp, .. } => {
            handle_remote_offer(from, my_guid_prefix, sdp, ws, peers, incoming_tx);
        }
        SignalingMessage::SdpAnswer { from, sdp, .. } => {
            handle_remote_answer(from, sdp, peers);
        }
        SignalingMessage::IceCandidate {
            from,
            candidate,
            sdp_mid,
            sdp_m_line_index,
            ..
        } => {
            handle_remote_ice_candidate(from, candidate, sdp_mid, sdp_m_line_index, peers);
        }
        SignalingMessage::PeerLeft { guid_prefix } => {
            peers.borrow_mut().remove(&guid_prefix);
        }
        _ => {}
    }
}

fn initiate_peer_connection(
    target_prefix: [u8; 12],
    my_guid_prefix: GuidPrefix,
    ws: &web_sys::WebSocket,
    peers: &Rc<RefCell<HashMap<[u8; 12], PeerState>>>,
    incoming_tx: &async_channel::Sender<Vec<u8>>,
) {
    let config = web_sys::RtcConfiguration::new();
    let pc = match web_sys::RtcPeerConnection::new_with_configuration(&config) {
        Ok(pc) => pc,
        Err(_) => return,
    };

    let dc_init = web_sys::RtcDataChannelInit::new();
    dc_init.set_ordered(false);
    dc_init.set_max_retransmits(0);
    let dc = pc.create_data_channel_with_data_channel_dict("dust_rtps", &dc_init);
    dc.set_binary_type(web_sys::RtcDataChannelType::Arraybuffer);

    setup_data_channel_onmessage(&dc, incoming_tx);
    setup_ice_candidate_handler(&pc, my_guid_prefix, target_prefix, ws);

    peers.borrow_mut().insert(
        target_prefix,
        PeerState {
            peer_connection: pc.clone(),
            data_channel: Some(dc),
        },
    );

    let ws_clone = ws.clone();
    spawn_local(async move {
        if let Ok(offer_val) = wasm_bindgen_futures::JsFuture::from(pc.create_offer()).await {
            let offer: web_sys::RtcSessionDescriptionInit = offer_val.unchecked_into();
            let _ = wasm_bindgen_futures::JsFuture::from(pc.set_local_description(&offer)).await;
            if let Some(desc) = pc.local_description() {
                let msg = SignalingMessage::SdpOffer {
                    from: my_guid_prefix,
                    to: target_prefix,
                    sdp: desc.sdp(),
                };
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = ws_clone.send_with_str(&json);
                }
            }
        }
    });
}

fn handle_remote_offer(
    from: [u8; 12],
    my_guid_prefix: GuidPrefix,
    sdp: String,
    ws: &web_sys::WebSocket,
    peers: &Rc<RefCell<HashMap<[u8; 12], PeerState>>>,
    incoming_tx: &async_channel::Sender<Vec<u8>>,
) {
    let config = web_sys::RtcConfiguration::new();
    let pc = match web_sys::RtcPeerConnection::new_with_configuration(&config) {
        Ok(pc) => pc,
        Err(_) => return,
    };

    setup_ice_candidate_handler(&pc, my_guid_prefix, from, ws);

    let peers_clone = peers.clone();
    let incoming_clone = incoming_tx.clone();
    let ondatachannel = Closure::<dyn FnMut(web_sys::RtcDataChannelEvent)>::new(
        move |ev: web_sys::RtcDataChannelEvent| {
            let dc = ev.channel();
            dc.set_binary_type(web_sys::RtcDataChannelType::Arraybuffer);
            setup_data_channel_onmessage(&dc, &incoming_clone);

            if let Some(peer_state) = peers_clone.borrow_mut().get_mut(&from) {
                peer_state.data_channel = Some(dc);
            }
        },
    );
    pc.set_ondatachannel(Some(ondatachannel.as_ref().unchecked_ref()));
    ondatachannel.forget();

    peers.borrow_mut().insert(
        from,
        PeerState {
            peer_connection: pc.clone(),
            data_channel: None,
        },
    );

    let ws_clone = ws.clone();
    spawn_local(async move {
        let desc = web_sys::RtcSessionDescriptionInit::new(web_sys::RtcSdpType::Offer);
        desc.set_sdp(&sdp);
        if wasm_bindgen_futures::JsFuture::from(pc.set_remote_description(&desc))
            .await
            .is_ok()
        {
            if let Ok(answer_val) = wasm_bindgen_futures::JsFuture::from(pc.create_answer()).await {
                let answer: web_sys::RtcSessionDescriptionInit = answer_val.unchecked_into();
                let _ =
                    wasm_bindgen_futures::JsFuture::from(pc.set_local_description(&answer)).await;
                if let Some(desc) = pc.local_description() {
                    let reply = SignalingMessage::SdpAnswer {
                        from: my_guid_prefix,
                        to: from,
                        sdp: desc.sdp(),
                    };
                    if let Ok(json) = serde_json::to_string(&reply) {
                        let _ = ws_clone.send_with_str(&json);
                    }
                }
            }
        }
    });
}

fn handle_remote_answer(
    from: [u8; 12],
    sdp: String,
    peers: &Rc<RefCell<HashMap<[u8; 12], PeerState>>>,
) {
    if let Some(peer) = peers.borrow().get(&from) {
        let pc = peer.peer_connection.clone();
        spawn_local(async move {
            let desc = web_sys::RtcSessionDescriptionInit::new(web_sys::RtcSdpType::Answer);
            desc.set_sdp(&sdp);
            let _ = wasm_bindgen_futures::JsFuture::from(pc.set_remote_description(&desc)).await;
        });
    }
}

fn handle_remote_ice_candidate(
    from: [u8; 12],
    candidate_str: String,
    sdp_mid: Option<String>,
    sdp_m_line_index: Option<u16>,
    peers: &Rc<RefCell<HashMap<[u8; 12], PeerState>>>,
) {
    if let Some(peer) = peers.borrow().get(&from) {
        let pc = peer.peer_connection.clone();
        spawn_local(async move {
            let init = web_sys::RtcIceCandidateInit::new(&candidate_str);
            if let Some(ref mid) = sdp_mid {
                init.set_sdp_mid(Some(mid.as_str()));
            }
            if let Some(index) = sdp_m_line_index {
                init.set_sdp_m_line_index(Some(index));
            }
            if let Ok(cand) = web_sys::RtcIceCandidate::new(&init) {
                let _ = wasm_bindgen_futures::JsFuture::from(
                    pc.add_ice_candidate_with_opt_rtc_ice_candidate(Some(&cand)),
                )
                .await;
            }
        });
    }
}

fn setup_ice_candidate_handler(
    pc: &web_sys::RtcPeerConnection,
    my_guid_prefix: GuidPrefix,
    target_prefix: [u8; 12],
    ws: &web_sys::WebSocket,
) {
    let ws_clone = ws.clone();
    let onice = Closure::<dyn FnMut(web_sys::RtcPeerConnectionIceEvent)>::new(
        move |ev: web_sys::RtcPeerConnectionIceEvent| {
            if let Some(candidate) = ev.candidate() {
                let msg = SignalingMessage::IceCandidate {
                    from: my_guid_prefix,
                    to: target_prefix,
                    candidate: candidate.candidate(),
                    sdp_mid: candidate.sdp_mid(),
                    sdp_m_line_index: candidate.sdp_m_line_index(),
                };
                if let Ok(json) = serde_json::to_string(&msg) {
                    let _ = ws_clone.send_with_str(&json);
                }
            }
        },
    );
    pc.set_onicecandidate(Some(onice.as_ref().unchecked_ref()));
    onice.forget();
}

fn setup_data_channel_onmessage(
    dc: &web_sys::RtcDataChannel,
    incoming_tx: &async_channel::Sender<Vec<u8>>,
) {
    let tx = incoming_tx.clone();
    let onmessage =
        Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |ev: web_sys::MessageEvent| {
            if let Ok(ab) = ev.data().dyn_into::<js_sys::ArrayBuffer>() {
                let u8_array = js_sys::Uint8Array::new(&ab);
                let _ = tx.try_send(u8_array.to_vec());
            }
        });
    dc.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();
}

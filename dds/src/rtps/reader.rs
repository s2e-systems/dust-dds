use super::{
    behavior_types::Duration,
    endpoint::RtpsEndpoint,
    types::{Guid, Locator},
    writer_proxy::RtpsWriterProxy,
};
use crate::implementation::{
    actor::ActorAddress, actors::message_sender_actor::MessageSenderActor,
};

pub struct RtpsReader {
    endpoint: RtpsEndpoint,
    _heartbeat_response_delay: Duration,
    _heartbeat_suppression_duration: Duration,
    _expects_inline_qos: bool,
}

impl RtpsReader {
    pub fn new(
        endpoint: RtpsEndpoint,
        heartbeat_response_delay: Duration,
        heartbeat_suppression_duration: Duration,
        expects_inline_qos: bool,
    ) -> Self {
        Self {
            endpoint,
            _heartbeat_response_delay: heartbeat_response_delay,
            _heartbeat_suppression_duration: heartbeat_suppression_duration,
            _expects_inline_qos: expects_inline_qos,
        }
    }

    pub fn guid(&self) -> Guid {
        self.endpoint.guid()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.endpoint.unicast_locator_list()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.endpoint.multicast_locator_list()
    }
}

pub struct RtpsStatelessReader {
    rtps_reader: RtpsReader,
}

impl RtpsStatelessReader {
    pub fn new(rtps_reader: RtpsReader) -> Self {
        Self { rtps_reader }
    }

    pub fn guid(&self) -> Guid {
        self.rtps_reader.guid()
    }
}

pub struct RtpsStatefulReader {
    rtps_reader: RtpsReader,
    matched_writers: Vec<RtpsWriterProxy>,
}

impl RtpsStatefulReader {
    pub fn new(rtps_reader: RtpsReader) -> Self {
        Self {
            rtps_reader,
            matched_writers: Vec::new(),
        }
    }

    pub fn matched_writer_add(&mut self, a_writer_proxy: RtpsWriterProxy) {
        if !self
            .matched_writers
            .iter()
            .any(|x| x.remote_writer_guid() == a_writer_proxy.remote_writer_guid())
        {
            self.matched_writers.push(a_writer_proxy);
        }
    }

    pub fn matched_writer_remove(&mut self, writer_proxy_guid: Guid) {
        self.matched_writers
            .retain(|x| x.remote_writer_guid() != writer_proxy_guid)
    }

    pub fn matched_writer_lookup(&mut self, a_writer_guid: Guid) -> Option<&mut RtpsWriterProxy> {
        self.matched_writers
            .iter_mut()
            .find(|x| x.remote_writer_guid() == a_writer_guid)
    }
}

// The methods in this impl block are not defined by the standard
impl RtpsStatefulReader {
    pub fn is_historical_data_received(&self) -> bool {
        !self
            .matched_writers
            .iter()
            .any(|p| !p.is_historical_data_received())
    }

    pub fn send_message(&mut self, message_sender_actor: &ActorAddress<MessageSenderActor>) {
        for writer_proxy in self.matched_writers.iter_mut() {
            writer_proxy.send_message(&self.rtps_reader.guid(), message_sender_actor)
        }
    }
}

pub enum RtpsReaderKind {
    Stateful(RtpsStatefulReader),
    Stateless(RtpsStatelessReader),
}

impl RtpsReaderKind {
    pub fn guid(&self) -> Guid {
        match self {
            RtpsReaderKind::Stateful(r) => r.rtps_reader.guid(),
            RtpsReaderKind::Stateless(r) => r.rtps_reader.guid(),
        }
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        match self {
            RtpsReaderKind::Stateful(r) => r.rtps_reader.unicast_locator_list(),
            RtpsReaderKind::Stateless(r) => r.rtps_reader.unicast_locator_list(),
        }
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        match self {
            RtpsReaderKind::Stateful(r) => r.rtps_reader.multicast_locator_list(),
            RtpsReaderKind::Stateless(r) => r.rtps_reader.multicast_locator_list(),
        }
    }
}

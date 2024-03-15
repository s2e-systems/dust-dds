use super::{
    endpoint::RtpsEndpoint,
    types::{Guid, Locator},
    writer_proxy::RtpsWriterProxy,
};
use crate::infrastructure::time::Duration;

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
    pub rtps_reader: RtpsReader,
}

impl RtpsStatelessReader {
    pub fn new(rtps_reader: RtpsReader) -> Self {
        Self { rtps_reader }
    }
}

pub struct RtpsStatefulReader {
    pub rtps_reader: RtpsReader,
    pub matched_writers: Vec<RtpsWriterProxy>,
}

impl RtpsStatefulReader {
    pub fn new(rtps_reader: RtpsReader) -> Self {
        Self {
            rtps_reader,
            matched_writers: Vec::new(),
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

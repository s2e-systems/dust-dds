use mockall::mock;
use rtps_pim::{
    behavior::{reader::stateless_reader::RtpsStatelessReaderConstructor, types::Duration},
    structure::types::{Guid, Locator, ReliabilityKind, TopicKind},
};

mock! {
    pub RtpsStatelessReader{}
}

impl RtpsStatelessReaderConstructor for MockRtpsStatelessReader {
    fn new(
        _guid: Guid,
        _topic_kind: TopicKind,
        _reliability_level: ReliabilityKind,
        _unicast_locator_list: &[Locator],
        _multicast_locator_list: &[Locator],
        _heartbeat_response_delay: Duration,
        _heartbeat_suppression_duration: Duration,
        _expects_inline_qos: bool,
    ) -> Self {
        MockRtpsStatelessReader::new()
    }
}

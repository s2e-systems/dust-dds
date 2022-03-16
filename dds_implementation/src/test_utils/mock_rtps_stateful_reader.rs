use mockall::mock;
use rtps_pim::{
    behavior::{reader::stateful_reader::RtpsStatefulReaderConstructor, types::Duration},
    structure::types::{Guid, Locator, ReliabilityKind, TopicKind},
};

mock! {
    pub RtpsStatefulReader{}
}

impl RtpsStatefulReaderConstructor for MockRtpsStatefulReader {
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
        MockRtpsStatefulReader::new()
    }
}

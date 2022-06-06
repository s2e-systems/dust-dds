use mockall::mock;
use rtps_pim::{
    behavior::{
        reader::{
            reader::RtpsReaderAttributes,
            stateful_reader::{RtpsStatefulReaderAttributes, RtpsStatefulReaderConstructor},
        },
        types::Duration,
    },
    structure::types::{Guid, Locator, ReliabilityKind, TopicKind},
};

use super::mock_rtps_history_cache::MockRtpsHistoryCache;

mock! {
    pub RtpsStatefulReader{}

    impl RtpsReaderAttributes for RtpsStatefulReader{
        type HistoryCacheType = MockRtpsHistoryCache;

        fn heartbeat_response_delay(&self) -> Duration;
        fn heartbeat_suppression_duration(&self) -> Duration;
        fn reader_cache(&mut self) -> &mut MockRtpsHistoryCache;
        fn expects_inline_qos(&self) -> bool;
    }
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

impl RtpsStatefulReaderAttributes<'_> for MockRtpsStatefulReader {
    type WriterProxyListType = Vec<()>;

    fn matched_writers(&'_ mut self) -> Self::WriterProxyListType {
        vec![]
    }
}

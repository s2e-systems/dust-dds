use std::vec;

use mockall::mock;
use rtps_pim::{
    behavior::{
        stateful_writer_behavior::RtpsStatefulWriterSendSubmessages,
        types::Duration,
        writer::{
            stateful_writer::{RtpsStatefulWriterAttributes, RtpsStatefulWriterConstructor},
            writer::{RtpsWriterAttributes, RtpsWriterOperations},
        },
    },
    messages::{
        submessage_elements::Parameter,
        submessages::{DataSubmessage, GapSubmessage, HeartbeatSubmessage},
    },
    structure::{
        history_cache::RtpsHistoryCacheOperations,
        types::{
            ChangeKind, Guid, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind,
        },
    },
};

use super::{
    mock_rtps_cache_change::MockRtpsCacheChange, mock_rtps_history_cache::MockRtpsHistoryCache,
    mock_rtps_reader_proxy::MockRtpsReaderProxy,
};

mock! {
    pub RtpsStatefulWriter{
        pub fn add_change_(&mut self, change: MockRtpsCacheChange);
        pub fn get_seq_num_min_(&self) -> Option<SequenceNumber>;
        pub fn get_seq_num_max_(&self) -> Option<SequenceNumber>;
        pub fn send_submessages_<'a>(&'a self,
            send_data: &mut dyn FnMut(&MockRtpsReaderProxy, DataSubmessage<Vec<Parameter<'a>>, &'a [u8]>),
            send_gap: &mut dyn FnMut(&MockRtpsReaderProxy, GapSubmessage<Vec<SequenceNumber>>),
            send_heartbeat: &mut dyn FnMut(&MockRtpsReaderProxy, HeartbeatSubmessage));
    }

    impl RtpsWriterAttributes for RtpsStatefulWriter {
        type HistoryCacheType = MockRtpsHistoryCache;

        fn push_mode(&self) -> bool;
        fn heartbeat_period(&self) -> Duration;
        fn nack_response_delay(&self) -> Duration;
        fn nack_suppression_duration(&self) -> Duration;
        fn last_change_sequence_number(&self) -> SequenceNumber;
        fn data_max_size_serialized(&self) -> Option<i32>;
        fn writer_cache(&mut self) -> &mut MockRtpsHistoryCache;
    }

    impl RtpsWriterOperations for RtpsStatefulWriter {
        type DataType = Vec<u8>;
        type ParameterListType = Vec<u8>;
        type CacheChangeType = MockRtpsCacheChange;

        fn new_change(
            &mut self,
            kind: ChangeKind,
            data: Vec<u8>,
            inline_qos: Vec<u8>,
            handle: InstanceHandle,
        ) -> MockRtpsCacheChange;
    }
}

impl RtpsHistoryCacheOperations for MockRtpsStatefulWriter {
    type CacheChangeType = MockRtpsCacheChange;
    fn add_change(&mut self, change: MockRtpsCacheChange) {
        self.add_change_(change)
    }

    fn remove_change<F>(&mut self, _f: F)
    where
        F: FnMut(&MockRtpsCacheChange) -> bool,
    {
        todo!()
    }

    fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.get_seq_num_min_()
    }

    fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.get_seq_num_max_()
    }
}

impl RtpsStatefulWriterConstructor for MockRtpsStatefulWriter {
    fn new(
        _guid: Guid,
        _topic_kind: TopicKind,
        _reliability_level: ReliabilityKind,
        _unicast_locator_list: &[Locator],
        _multicast_locator_list: &[Locator],
        _push_mode: bool,
        _heartbeat_period: Duration,
        _nack_response_delay: Duration,
        _nack_suppression_duration: Duration,
        _data_max_size_serialized: Option<i32>,
    ) -> Self {
        MockRtpsStatefulWriter::new()
    }
}

impl RtpsStatefulWriterAttributes<'_> for MockRtpsStatefulWriter {
    type ReaderProxyListType = Vec<()>;

    fn matched_readers(&'_ mut self) -> Self::ReaderProxyListType {
        vec![]
    }
}

impl<'a> RtpsStatefulWriterSendSubmessages<'a, Vec<Parameter<'a>>, &'a [u8], Vec<SequenceNumber>>
    for MockRtpsStatefulWriter
{
    type ReaderProxyType = MockRtpsReaderProxy;

    fn send_submessages(
        &'a mut self,
        mut send_data: impl FnMut(&Self::ReaderProxyType, DataSubmessage<Vec<Parameter<'a>>, &'a [u8]>),
        mut send_gap: impl FnMut(&Self::ReaderProxyType, GapSubmessage<Vec<SequenceNumber>>),
        mut send_heartbeat: impl FnMut(&Self::ReaderProxyType, HeartbeatSubmessage),
    ) {
        self.send_submessages_(&mut send_data, &mut send_gap, &mut send_heartbeat)
    }
}

use rust_rtps_pim::{
    behavior::{
        stateful_writer::{RTPSReaderProxy, RTPSStatefulWriter},
        stateless_writer::{RTPSReaderLocator, RTPSStatelessWriter},
        RTPSWriter,
    },
    structure::{
        types::{ChangeKind, Locator, ReliabilityKind, SequenceNumber, TopicKind, GUID},
        RTPSCacheChange, RTPSEndpoint, RTPSEntity, RTPSHistoryCache,
    },
};

use super::{
    rtps_cache_change_impl::RTPSCacheChangeImpl, rtps_history_cache_impl::RTPSHistoryCacheImpl,
    rtps_reader_locator_impl::RTPSReaderLocatorImpl, rtps_reader_proxy_impl::RTPSReaderProxyImpl,
};

pub struct RTPSWriterImpl {
    guid: GUID,
    topic_kind: TopicKind,
    reliability_level: ReliabilityKind,
    push_mode: bool,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    heartbeat_period: <Self as RTPSWriter>::DurationType,
    nack_response_delay: <Self as RTPSWriter>::DurationType,
    nack_suppression_duration: <Self as RTPSWriter>::DurationType,
    last_change_sequence_number: SequenceNumber,
    data_max_size_serialized: i32,
    reader_locators: Vec<RTPSReaderLocatorImpl>,
    matched_readers: Vec<RTPSReaderProxyImpl>,
    writer_cache: RTPSHistoryCacheImpl,
}

impl RTPSWriterImpl {
    pub fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        push_mode: bool,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        heartbeat_period: <Self as RTPSWriter>::DurationType,
        nack_response_delay: <Self as RTPSWriter>::DurationType,
        nack_suppression_duration: <Self as RTPSWriter>::DurationType,
        data_max_size_serialized: i32,
    ) -> Self
    where
        SequenceNumber: PartialEq + Ord,
    {
        Self {
            guid,
            topic_kind,
            reliability_level,
            push_mode,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
            last_change_sequence_number: 0.into(),
            reader_locators: Vec::new(),
            matched_readers: Vec::new(),
            writer_cache: RTPSHistoryCacheImpl::new(),
        }
    }

    pub fn writer_cache_and_reader_locators(
        &mut self,
    ) -> (&RTPSHistoryCacheImpl, &mut Vec<RTPSReaderLocatorImpl>) {
        (&self.writer_cache, &mut self.reader_locators)
    }
}

impl RTPSEntity for RTPSWriterImpl {
    fn guid(&self) -> &GUID {
        &self.guid
    }
}

impl RTPSWriter for RTPSWriterImpl {
    type HistoryCacheType = RTPSHistoryCacheImpl;
    type DurationType = i64;

    fn push_mode(&self) -> bool {
        self.push_mode
    }

    fn heartbeat_period(&self) -> &Self::DurationType {
        &self.heartbeat_period
    }

    fn nack_response_delay(&self) -> &Self::DurationType {
        &self.nack_response_delay
    }

    fn nack_suppression_duration(&self) -> &Self::DurationType {
        &self.nack_suppression_duration
    }

    fn last_change_sequence_number(&self) -> &SequenceNumber {
        &self.last_change_sequence_number
    }

    fn data_max_size_serialized(&self) -> i32 {
        self.data_max_size_serialized
    }

    fn writer_cache(&self) -> &RTPSHistoryCacheImpl {
        &self.writer_cache
    }

    fn writer_cache_mut(&mut self) -> &mut RTPSHistoryCacheImpl {
        &mut self.writer_cache
    }

    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChange as RTPSCacheChange>::DataType,
        inline_qos: <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChange as RTPSCacheChange>::InlineQosType,
        handle:  <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChange as RTPSCacheChange>::InstanceHandleType,
    ) -> <Self::HistoryCacheType as RTPSHistoryCache>::CacheChange
    where
        <Self::HistoryCacheType as RTPSHistoryCache>::CacheChange: RTPSCacheChange,
    {
        self.last_change_sequence_number = self.last_change_sequence_number + 1;
        RTPSCacheChangeImpl::new(
            kind,
            self.guid,
            handle,
            self.last_change_sequence_number,
            data,
            inline_qos,
        )
    }
}

impl RTPSEndpoint for RTPSWriterImpl {
    fn topic_kind(&self) -> &TopicKind {
        &self.topic_kind
    }

    fn reliability_level(&self) -> &ReliabilityKind {
        &self.reliability_level
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        &self.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        &self.multicast_locator_list
    }
}

impl RTPSStatelessWriter for RTPSWriterImpl {
    type ReaderLocatorPIM = RTPSReaderLocatorImpl;

    fn reader_locators(&mut self) -> &mut [Self::ReaderLocatorPIM] {
        &mut self.reader_locators
    }

    fn reader_locator_add(&mut self, a_locator: Self::ReaderLocatorPIM) {
        self.reader_locators.push(a_locator)
    }

    fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.retain(|x| x.locator() != a_locator)
    }

    fn unsent_changes_reset(&mut self) {
        todo!()
    }
}

impl RTPSStatefulWriter for RTPSWriterImpl {
    type ReaderProxyType = RTPSReaderProxyImpl;

    fn matched_readers(&self) -> &[Self::ReaderProxyType] {
        &self.matched_readers
    }

    fn matched_reader_add(&mut self, a_reader_proxy: Self::ReaderProxyType) {
        self.matched_readers.push(a_reader_proxy)
    }

    fn matched_reader_remove(&mut self, reader_proxy_guid: &GUID) {
        self.matched_readers
            .retain(|x| x.remote_reader_guid() != reader_proxy_guid)
    }

    fn matched_reader_lookup(&self, a_reader_guid: &GUID) -> Option<&Self::ReaderProxyType> {
        self.matched_readers
            .iter()
            .find(|&x| x.remote_reader_guid() == a_reader_guid)
    }

    fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::{types::GUID_UNKNOWN, RTPSCacheChange};

    use super::*;

    #[test]
    fn new_change() {
        let push_mode = true;
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let heartbeat_period = 0;
        let nack_response_delay = 0;
        let nack_suppression_duration = 0;
        let data_max_size_serialized = i32::MAX;
        let mut writer: RTPSWriterImpl = RTPSWriterImpl::new(
            GUID_UNKNOWN,
            topic_kind,
            reliability_level,
            push_mode,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        );
        let change1 = writer.new_change(ChangeKind::Alive, vec![], (), 0);
        let change2 = writer.new_change(ChangeKind::Alive, vec![], (), 0);

        assert_eq!(change1.sequence_number(), &1);
        assert_eq!(change2.sequence_number(), &2);
    }

    #[test]
    fn reader_locator_add() {
        let push_mode = true;
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let heartbeat_period = 0;
        let nack_response_delay = 0;
        let nack_suppression_duration = 0;
        let data_max_size_serialized = i32::MAX;
        let mut writer: RTPSWriterImpl = RTPSWriterImpl::new(
            GUID_UNKNOWN,
            topic_kind,
            reliability_level,
            push_mode,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        );
        let reader_locator1 =
            RTPSReaderLocatorImpl::new(Locator::new([1; 4], [1; 4], [1; 16]), false);
        let reader_locator2 =
            RTPSReaderLocatorImpl::new(Locator::new([2; 4], [2; 4], [2; 16]), false);
        writer.reader_locator_add(reader_locator1);
        writer.reader_locator_add(reader_locator2);

        assert_eq!(writer.reader_locators().len(), 2)
    }

    #[test]
    fn reader_locator_remove() {
        let push_mode = true;
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let heartbeat_period = 0;
        let nack_response_delay = 0;
        let nack_suppression_duration = 0;
        let data_max_size_serialized = i32::MAX;
        let mut writer: RTPSWriterImpl = RTPSWriterImpl::new(
            GUID_UNKNOWN,
            topic_kind,
            reliability_level,
            push_mode,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        );

        let reader_locator1 =
            RTPSReaderLocatorImpl::new(Locator::new([1; 4], [1; 4], [1; 16]), false);
        let reader_locator2 =
            RTPSReaderLocatorImpl::new(Locator::new([2; 4], [2; 4], [2; 16]), false);
        writer.reader_locator_add(reader_locator1);
        writer.reader_locator_add(reader_locator2);
        writer.reader_locator_remove(&Locator::new([1; 4], [1; 4], [1; 16]));

        assert_eq!(writer.reader_locators().len(), 1)
    }

    #[test]
    fn matched_reader_add() {
        let push_mode = true;
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let heartbeat_period = 0;
        let nack_response_delay = 0;
        let nack_suppression_duration = 0;
        let data_max_size_serialized = i32::MAX;
        let mut writer: RTPSWriterImpl = RTPSWriterImpl::new(
            GUID_UNKNOWN,
            topic_kind,
            reliability_level,
            push_mode,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        );
        let reader_proxy1 =
            RTPSReaderProxyImpl::new(GUID_UNKNOWN, [0; 4], vec![], vec![], false, true);
        let reader_proxy2 =
            RTPSReaderProxyImpl::new(GUID_UNKNOWN, [0; 4], vec![], vec![], false, true);
        writer.matched_reader_add(reader_proxy1);
        writer.matched_reader_add(reader_proxy2);
        assert_eq!(writer.matched_readers().len(), 2)
    }

    #[test]
    fn matched_reader_remove() {
        let push_mode = true;
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let heartbeat_period = 0;
        let nack_response_delay = 0;
        let nack_suppression_duration = 0;
        let data_max_size_serialized = i32::MAX;
        let mut writer: RTPSWriterImpl = RTPSWriterImpl::new(
            GUID_UNKNOWN,
            topic_kind,
            reliability_level,
            push_mode,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        );

        let reader_proxy1 =
            RTPSReaderProxyImpl::new(GUID_UNKNOWN, [0; 4], vec![], vec![], false, true);
        let reader_proxy2 =
            RTPSReaderProxyImpl::new(GUID_UNKNOWN, [0; 4], vec![], vec![], false, true);
        writer.matched_reader_add(reader_proxy1);
        writer.matched_reader_add(reader_proxy2);
        writer.matched_reader_remove(&GUID_UNKNOWN);

        assert_eq!(writer.matched_readers().len(), 2)
    }

    #[test]
    fn matched_reader_lookup() {
        let push_mode = true;
        let topic_kind = TopicKind::WithKey;
        let reliability_level = ReliabilityKind::BestEffort;
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let heartbeat_period = 0;
        let nack_response_delay = 0;
        let nack_suppression_duration = 0;
        let data_max_size_serialized = i32::MAX;
        let mut writer: RTPSWriterImpl = RTPSWriterImpl::new(
            GUID_UNKNOWN,
            topic_kind,
            reliability_level,
            push_mode,
            unicast_locator_list,
            multicast_locator_list,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        );

        let reader_proxy1 =
            RTPSReaderProxyImpl::new(GUID_UNKNOWN, [0; 4], vec![], vec![], false, true);
        let reader_proxy2 =
            RTPSReaderProxyImpl::new(GUID_UNKNOWN, [0; 4], vec![], vec![], false, true);
        writer.matched_reader_add(reader_proxy1);
        writer.matched_reader_add(reader_proxy2);

        assert!(writer.matched_reader_lookup(&GUID_UNKNOWN).is_some());
        assert!(writer.matched_reader_lookup(&GUID_UNKNOWN).is_none());
    }
}

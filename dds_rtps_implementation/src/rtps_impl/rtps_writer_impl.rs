use rust_rtps_pim::{
    behavior::{
        types::Duration,
        writer::{
            reader_locator::RTPSReaderLocator,
            reader_proxy::RTPSReaderProxy,
            stateful_writer::{RTPSStatefulWriter, RTPSStatefulWriterOperations},
            stateless_writer::{RTPSStatelessWriter, RTPSStatelessWriterOperations},
            writer::{RTPSWriter, RTPSWriterOperations},
        },
    },
    structure::{
        types::{
            ChangeKind, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind, GUID,
        },
        RTPSCacheChange, RTPSCacheChangeOperations, RTPSEndpoint, RTPSEntity, RTPSHistoryCache,
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
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    push_mode: bool,
    heartbeat_period: Duration,
    nack_response_delay: Duration,
    nack_suppression_duration: Duration,
    last_change_sequence_number: SequenceNumber,
    data_max_size_serialized: Option<i32>,
    reader_locators: Vec<RTPSReaderLocatorImpl>,
    matched_readers: Vec<RTPSReaderProxyImpl>,
    writer_cache: RTPSHistoryCacheImpl,
}

impl RTPSEntity for RTPSWriterImpl {
    fn guid(&self) -> &GUID {
        &self.guid
    }
}

impl RTPSWriter for RTPSWriterImpl {
    type HistoryCacheType = RTPSHistoryCacheImpl;

    fn push_mode(&self) -> bool {
        self.push_mode
    }

    fn heartbeat_period(&self) -> &Duration {
        &self.heartbeat_period
    }

    fn nack_response_delay(&self) -> &Duration {
        &self.nack_response_delay
    }

    fn nack_suppression_duration(&self) -> &Duration {
        &self.nack_suppression_duration
    }

    fn last_change_sequence_number(&self) -> &SequenceNumber {
        &self.last_change_sequence_number
    }

    fn data_max_size_serialized(&self) -> &Option<i32> {
        &self.data_max_size_serialized
    }

    fn writer_cache(&self) -> &RTPSHistoryCacheImpl {
        &self.writer_cache
    }

    fn writer_cache_mut(&mut self) -> &mut RTPSHistoryCacheImpl {
        &mut self.writer_cache
    }
}

impl RTPSWriterOperations for RTPSWriterImpl {
    fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_size_serialized: Option<i32>,
    ) -> Self {
        Self {
            guid,
            topic_kind,
            reliability_level,
            push_mode,
            unicast_locator_list: unicast_locator_list.into_iter().cloned().collect(),
            multicast_locator_list: multicast_locator_list.into_iter().cloned().collect(),
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

    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: &[u8],
        inline_qos: <<<Self as RTPSWriter>::HistoryCacheType as RTPSHistoryCache>::CacheChange as RTPSCacheChange>::InlineQosType,
        handle: InstanceHandle,
    ) -> <<Self as RTPSWriter>::HistoryCacheType as RTPSHistoryCache>::CacheChange
    where
        Self: RTPSWriter,
        <Self as RTPSWriter>::HistoryCacheType: RTPSHistoryCache,
        <<Self as RTPSWriter>::HistoryCacheType as RTPSHistoryCache>::CacheChange: RTPSCacheChange,
    {
        self.last_change_sequence_number = self.last_change_sequence_number + 1;
        RTPSCacheChangeImpl::new(
            kind,
            self.guid,
            handle,
            self.last_change_sequence_number,
            data.as_ref(),
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
    type ReaderLocatorType = RTPSReaderLocatorImpl;

    fn reader_locators(&mut self) -> &mut [Self::ReaderLocatorType] {
        &mut self.reader_locators
    }

    fn writer_cache_and_reader_locators(
        &mut self,
    ) -> (
        &<Self as RTPSWriter>::HistoryCacheType,
        &mut [Self::ReaderLocatorType],
    )
    where
        Self: RTPSWriter,
    {
        (&self.writer_cache, &mut self.reader_locators)
    }
}

impl RTPSStatelessWriterOperations for RTPSWriterImpl {
    fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_size_serialized: Option<i32>,
    ) -> Self {
        <Self as RTPSWriterOperations>::new(
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        )
    }

    fn reader_locator_add(&mut self, a_locator: <Self as RTPSStatelessWriter>::ReaderLocatorType) {
        self.reader_locators.push(a_locator);
    }

    fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.retain(|x| x.locator() != a_locator)
    }

    fn unsent_changes_reset(&mut self) {
        for reader_locator in &mut self.reader_locators {
            reader_locator.unsent_changes_reset();
        }
    }
}

impl RTPSStatefulWriter for RTPSWriterImpl {
    type ReaderProxyType = RTPSReaderProxyImpl;

    fn matched_readers(&self) -> &[Self::ReaderProxyType] {
        &self.matched_readers
    }
}

impl RTPSStatefulWriterOperations for RTPSWriterImpl {
    fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_size_serialized: Option<i32>,
    ) -> Self {
        <Self as RTPSWriterOperations>::new(
            guid,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_size_serialized,
        )
    }

    fn matched_reader_add(&mut self, a_reader_proxy: <Self as RTPSStatefulWriter>::ReaderProxyType)
    where
        Self: RTPSStatefulWriter,
    {
        self.matched_readers.push(a_reader_proxy)
    }

    fn matched_reader_remove(&mut self, reader_proxy_guid: &GUID) {
        self.matched_readers
            .retain(|x| x.remote_reader_guid() != reader_proxy_guid)
    }

    fn matched_reader_lookup(
        &self,
        a_reader_guid: &GUID,
    ) -> Option<&<Self as RTPSStatefulWriter>::ReaderProxyType> {
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
    use rust_rtps_pim::{
        behavior::{
            types::DURATION_ZERO,
            writer::{
                reader_locator::RTPSReaderLocatorOperations,
                reader_proxy::RTPSReaderProxyOperations,
                stateful_writer::{RTPSStatefulWriter, RTPSStatefulWriterOperations},
                stateless_writer::{RTPSStatelessWriter, RTPSStatelessWriterOperations},
                writer::RTPSWriterOperations,
            },
        },
        structure::{
            types::{
                ChangeKind, EntityId, Locator, ReliabilityKind, TopicKind, GUID, GUID_UNKNOWN,
            },
            RTPSCacheChange, RTPSHistoryCache,
        },
    };

    use crate::rtps_impl::{
        rtps_history_cache_impl::RTPSHistoryCacheImpl,
        rtps_reader_locator_impl::RTPSReaderLocatorImpl,
        rtps_reader_proxy_impl::RTPSReaderProxyImpl,
    };

    use super::RTPSWriterImpl;

    #[test]
    fn new_change() {
        let mut writer = RTPSWriterImpl {
            guid: GUID_UNKNOWN,
            topic_kind: TopicKind::WithKey,
            reliability_level: ReliabilityKind::BestEffort,
            push_mode: true,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            heartbeat_period: DURATION_ZERO,
            nack_response_delay: DURATION_ZERO,
            nack_suppression_duration: DURATION_ZERO,
            last_change_sequence_number: 0,
            data_max_size_serialized: None,
            reader_locators: Vec::new(),
            matched_readers: Vec::new(),
            writer_cache: RTPSHistoryCacheImpl::new(),
        };
        let change1 = writer.new_change(ChangeKind::Alive, &[], (), 0);
        let change2 = writer.new_change(ChangeKind::Alive, &[], (), 0);

        assert_eq!(change1.sequence_number(), &1);
        assert_eq!(change2.sequence_number(), &2);
    }

    #[test]
    fn reader_locator_add() {
        let mut writer = RTPSWriterImpl {
            guid: GUID_UNKNOWN,
            topic_kind: TopicKind::WithKey,
            reliability_level: ReliabilityKind::BestEffort,
            push_mode: true,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            heartbeat_period: DURATION_ZERO,
            nack_response_delay: DURATION_ZERO,
            nack_suppression_duration: DURATION_ZERO,
            last_change_sequence_number: 0,
            data_max_size_serialized: None,
            reader_locators: Vec::new(),
            matched_readers: Vec::new(),
            writer_cache: RTPSHistoryCacheImpl::new(),
        };
        let reader_locator1 = RTPSReaderLocatorImpl::new(Locator::new(1, 1, [1; 16]), false);
        let reader_locator2 = RTPSReaderLocatorImpl::new(Locator::new(2, 2, [2; 16]), false);
        writer.reader_locator_add(reader_locator1);
        writer.reader_locator_add(reader_locator2);

        assert_eq!(writer.reader_locators().len(), 2)
    }

    #[test]
    fn reader_locator_remove() {
        let mut writer = RTPSWriterImpl {
            guid: GUID_UNKNOWN,
            topic_kind: TopicKind::WithKey,
            reliability_level: ReliabilityKind::BestEffort,
            push_mode: true,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            heartbeat_period: DURATION_ZERO,
            nack_response_delay: DURATION_ZERO,
            nack_suppression_duration: DURATION_ZERO,
            last_change_sequence_number: 0,
            data_max_size_serialized: None,
            reader_locators: Vec::new(),
            matched_readers: Vec::new(),
            writer_cache: RTPSHistoryCacheImpl::new(),
        };

        let reader_locator1 = RTPSReaderLocatorImpl::new(Locator::new(1, 1, [1; 16]), false);
        let reader_locator2 = RTPSReaderLocatorImpl::new(Locator::new(2, 2, [2; 16]), false);
        writer.reader_locator_add(reader_locator1);
        writer.reader_locator_add(reader_locator2);
        writer.reader_locator_remove(&Locator::new(1, 1, [1; 16]));

        assert_eq!(writer.reader_locators().len(), 1)
    }

    #[test]
    fn matched_reader_add() {
        let mut writer = RTPSWriterImpl {
            guid: GUID_UNKNOWN,
            topic_kind: TopicKind::WithKey,
            reliability_level: ReliabilityKind::BestEffort,
            push_mode: true,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            heartbeat_period: DURATION_ZERO,
            nack_response_delay: DURATION_ZERO,
            nack_suppression_duration: DURATION_ZERO,
            last_change_sequence_number: 0,
            data_max_size_serialized: None,
            reader_locators: Vec::new(),
            matched_readers: Vec::new(),
            writer_cache: RTPSHistoryCacheImpl::new(),
        };
        let unknown_remote_group_entity_id = EntityId {
            entity_key: [0; 3],
            entity_kind: rust_rtps_pim::structure::types::EntityKind::UserDefinedUnknown,
        };
        let reader_proxy_guid1 = GUID::new(
            [1; 12],
            EntityId {
                entity_key: [1; 3],
                entity_kind: rust_rtps_pim::structure::types::EntityKind::UserDefinedReaderNoKey,
            },
        );
        let reader_proxy1 = RTPSReaderProxyImpl::new(
            reader_proxy_guid1,
            unknown_remote_group_entity_id,
            None,
            None,
            false,
            true,
        );
        let reader_proxy_guid2 = GUID::new(
            [2; 12],
            EntityId {
                entity_key: [2; 3],
                entity_kind: rust_rtps_pim::structure::types::EntityKind::UserDefinedReaderNoKey,
            },
        );
        let reader_proxy2 = RTPSReaderProxyImpl::new(
            reader_proxy_guid2,
            unknown_remote_group_entity_id,
            None,
            None,
            false,
            true,
        );
        writer.matched_reader_add(reader_proxy1);
        writer.matched_reader_add(reader_proxy2);
        assert_eq!(writer.matched_readers().len(), 2)
    }

    #[test]
    fn matched_reader_remove() {
        let mut writer = RTPSWriterImpl {
            guid: GUID_UNKNOWN,
            topic_kind: TopicKind::WithKey,
            reliability_level: ReliabilityKind::BestEffort,
            push_mode: true,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            heartbeat_period: DURATION_ZERO,
            nack_response_delay: DURATION_ZERO,
            nack_suppression_duration: DURATION_ZERO,
            last_change_sequence_number: 0,
            data_max_size_serialized: None,
            reader_locators: Vec::new(),
            matched_readers: Vec::new(),
            writer_cache: RTPSHistoryCacheImpl::new(),
        };

        let unknown_remote_group_entity_id = EntityId {
            entity_key: [0; 3],
            entity_kind: rust_rtps_pim::structure::types::EntityKind::UserDefinedUnknown,
        };
        let reader_proxy_guid1 = GUID::new(
            [1; 12],
            EntityId {
                entity_key: [1; 3],
                entity_kind: rust_rtps_pim::structure::types::EntityKind::UserDefinedReaderNoKey,
            },
        );
        let reader_proxy1 = RTPSReaderProxyImpl::new(
            reader_proxy_guid1,
            unknown_remote_group_entity_id,
            None,
            None,
            false,
            true,
        );
        let reader_proxy_guid2 = GUID::new(
            [2; 12],
            EntityId {
                entity_key: [2; 3],
                entity_kind: rust_rtps_pim::structure::types::EntityKind::UserDefinedReaderNoKey,
            },
        );
        let reader_proxy2 = RTPSReaderProxyImpl::new(
            reader_proxy_guid2,
            unknown_remote_group_entity_id,
            None,
            None,
            false,
            true,
        );
        writer.matched_reader_add(reader_proxy1);
        writer.matched_reader_add(reader_proxy2);
        writer.matched_reader_remove(&reader_proxy_guid2);

        assert_eq!(writer.matched_readers().len(), 1)
    }

    #[test]
    fn matched_reader_lookup() {
        let mut writer = RTPSWriterImpl {
            guid: GUID_UNKNOWN,
            topic_kind: TopicKind::WithKey,
            reliability_level: ReliabilityKind::BestEffort,
            push_mode: true,
            unicast_locator_list: vec![],
            multicast_locator_list: vec![],
            heartbeat_period: DURATION_ZERO,
            nack_response_delay: DURATION_ZERO,
            nack_suppression_duration: DURATION_ZERO,
            last_change_sequence_number: 0,
            data_max_size_serialized: None,
            reader_locators: Vec::new(),
            matched_readers: Vec::new(),
            writer_cache: RTPSHistoryCacheImpl::new(),
        };

        let unknown_remote_group_entity_id = EntityId {
            entity_key: [0; 3],
            entity_kind: rust_rtps_pim::structure::types::EntityKind::UserDefinedUnknown,
        };
        let reader_proxy_guid1 = GUID::new(
            [1; 12],
            EntityId {
                entity_key: [1; 3],
                entity_kind: rust_rtps_pim::structure::types::EntityKind::UserDefinedReaderNoKey,
            },
        );
        let reader_proxy1 = RTPSReaderProxyImpl::new(
            reader_proxy_guid1,
            unknown_remote_group_entity_id,
            None,
            None,
            false,
            true,
        );
        let reader_proxy_guid2 = GUID::new(
            [2; 12],
            EntityId {
                entity_key: [2; 3],
                entity_kind: rust_rtps_pim::structure::types::EntityKind::UserDefinedReaderNoKey,
            },
        );
        let reader_proxy2 = RTPSReaderProxyImpl::new(
            reader_proxy_guid2,
            unknown_remote_group_entity_id,
            None,
            None,
            false,
            true,
        );
        writer.matched_reader_add(reader_proxy1);
        writer.matched_reader_add(reader_proxy2);

        assert!(writer.matched_reader_lookup(&reader_proxy_guid1).is_some());
        assert!(writer.matched_reader_lookup(&GUID_UNKNOWN).is_none());
    }
}

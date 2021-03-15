use std::sync::atomic;

use rust_rtps::{
    behavior::{types::Duration, RTPSWriter},
    messages::submessages::submessage_elements::ParameterList,
    structure::{RTPSCacheChange, RTPSEndpoint, RTPSEntity, RTPSHistoryCache},
    types::{
        ChangeKind, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind, GUID,
    },
};

pub struct Writer<H: RTPSHistoryCache> {
    guid: GUID,
    topic_kind: TopicKind,
    reliablility_level: ReliabilityKind,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
    push_mode: bool,
    heartbeat_period: Duration,
    nack_response_delay: Duration,
    nack_suppression_duration: Duration,
    last_change_sequence_number: atomic::AtomicI64,
    data_max_sized_serialized: i32,
    writer_cache: H,
}

impl<H: RTPSHistoryCache> RTPSEntity for Writer<H> {
    fn guid(&self) -> GUID {
        self.guid
    }
}

impl<H: RTPSHistoryCache> RTPSEndpoint for Writer<H> {
    fn unicast_locator_list(&self) -> &[Locator] {
        &self.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        &self.multicast_locator_list
    }

    fn topic_kind(&self) -> TopicKind {
        self.topic_kind
    }

    fn reliability_level(&self) -> ReliabilityKind {
        self.reliablility_level
    }
}

impl<H: RTPSHistoryCache> RTPSWriter for Writer<H> {
    type HistoryCacheType = H;

    fn new(
        guid: GUID,
        topic_kind: TopicKind,
        reliablility_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_sized_serialized: i32,
    ) -> Self {
        Self {
            guid,
            topic_kind,
            reliablility_level,
            unicast_locator_list: unicast_locator_list.to_vec(),
            multicast_locator_list: multicast_locator_list.to_vec(),
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            last_change_sequence_number: atomic::AtomicI64::new(0),
            data_max_sized_serialized,
            writer_cache: H::new(),
        }
    }

    fn push_mode(&self) -> bool {
        self.push_mode
    }

    fn heartbeat_period(&self) -> Duration {
        self.heartbeat_period
    }

    fn nack_response_delay(&self) -> Duration {
        self.nack_response_delay
    }

    fn nack_suppression_duration(&self) -> Duration {
        self.nack_suppression_duration
    }

    fn last_change_sequence_number(&self) -> SequenceNumber {
        self.last_change_sequence_number
            .load(atomic::Ordering::Acquire)
    }

    fn data_max_sized_serialized(&self) -> i32 {
        self.data_max_sized_serialized
    }

    fn writer_cache(&self) -> &Self::HistoryCacheType {
        &self.writer_cache
    }

    fn new_change(
        &self,
        kind: ChangeKind,
        data: <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType as RTPSCacheChange>::Data,
        inline_qos: ParameterList,
        handle: InstanceHandle,
    ) -> <Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType {
        let sn = self
            .last_change_sequence_number
            .fetch_add(1, atomic::Ordering::Acquire);
        <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType>::new(
            kind,
            self.guid,
            handle,
            sn + 1,
            data,
            inline_qos,
        )
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps::{
        behavior::types::constants::{DURATION_INFINITE, DURATION_ZERO},
        structure::history_cache::RTPSHistoryCacheRead,
        types::constants::ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
    };

    use super::*;

    struct MockCacheChange {
        kind: ChangeKind,
        sequence_number: SequenceNumber,
        instance_handle: InstanceHandle,
    }

    impl RTPSCacheChange for MockCacheChange {
        type Data = ();

        fn new(
            kind: ChangeKind,
            _writer_guid: GUID,
            instance_handle: InstanceHandle,
            sequence_number: SequenceNumber,
            _data_value: Self::Data,
            _inline_qos: ParameterList,
        ) -> Self {
            Self {
                kind,
                sequence_number,
                instance_handle,
            }
        }

        fn kind(&self) -> ChangeKind {
            self.kind
        }

        fn writer_guid(&self) -> GUID {
            todo!()
        }

        fn instance_handle(&self) -> &InstanceHandle {
            &self.instance_handle
        }

        fn sequence_number(&self) -> SequenceNumber {
            self.sequence_number
        }

        fn data_value(&self) -> &Self::Data {
            todo!()
        }

        fn inline_qos(&self) -> &ParameterList {
            todo!()
        }
    }
    struct MockHistoryCache;
    impl<'a> RTPSHistoryCacheRead<'a> for MockHistoryCache {
        type CacheChangeType = MockCacheChange;
        type Item = &'a MockCacheChange;
    }

    impl RTPSHistoryCache for MockHistoryCache {
        type CacheChangeType = MockCacheChange;
        type HistoryCacheStorageType = Self;

        fn new() -> Self {
            todo!()
        }

        fn add_change(&self, _change: Self::CacheChangeType) {
            todo!()
        }

        fn remove_change(&self, _seq_num: SequenceNumber) {
            todo!()
        }

        fn get_change<'a>(
            &'a self,
            _seq_num: SequenceNumber,
        ) -> Option<<Self::HistoryCacheStorageType as RTPSHistoryCacheRead<'a>>::Item> {
            todo!()
        }

        fn get_seq_num_min(&self) -> Option<SequenceNumber> {
            todo!()
        }

        fn get_seq_num_max(&self) -> Option<SequenceNumber> {
            todo!()
        }
    }

    #[test]
    fn new_change() {
        let guid_prefix = [1; 12];
        let entity_id = ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER;
        let guid = GUID::new(guid_prefix, entity_id);
        let topic_kind = TopicKind::WithKey;
        let reliablility_level = ReliabilityKind::BestEffort;
        let unicast_locator_list = vec![];
        let multicast_locator_list = vec![];
        let push_mode = true;
        let heartbeat_period = DURATION_INFINITE;
        let nack_response_delay = DURATION_ZERO;
        let nack_suppression_duration = DURATION_ZERO;
        let data_max_sized_serialized = i32::MAX;
        let writer: Writer<MockHistoryCache> = Writer::new(
            guid,
            topic_kind,
            reliablility_level,
            &unicast_locator_list,
            &multicast_locator_list,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            data_max_sized_serialized,
        );

        let kind1 = ChangeKind::Alive;
        let kind2 = ChangeKind::NotAliveUnregistered;
        let handle1 = [1; 16];
        let handle2 = [2; 16];

        assert_eq!(writer.last_change_sequence_number(), 0);

        let cc1 = writer.new_change(kind1, (), ParameterList::new(), handle1);
        let cc2 = writer.new_change(kind2, (), ParameterList::new(), handle2);

        assert_eq!(cc1.sequence_number(), 1);
        assert_eq!(cc1.kind(), kind1);
        assert_eq!(cc1.instance_handle(), &handle1);

        assert_eq!(cc2.sequence_number(), 2);
        assert_eq!(cc2.kind(), kind2);
        assert_eq!(cc2.instance_handle(), &handle2);
    }
}

use crate::{
    messages::{
        submessage_elements::{
            CountSubmessageElement, EntityIdSubmessageElement, SequenceNumberSetSubmessageElement,
        },
        submessages::{AckNackSubmessage, DataSubmessage, GapSubmessage, HeartbeatSubmessage},
        types::Count,
    },
    structure::{
        cache_change::RtpsCacheChangeAttributes,
        history_cache::RtpsHistoryCacheOperations,
        types::{Guid, GuidPrefix, SequenceNumber, ENTITYID_UNKNOWN},
    },
};

use super::reader::{
    reader::RtpsReaderAttributes,
    stateful_reader::RtpsStatefulReaderOperations,
    writer_proxy::{RtpsWriterProxyAttributes, RtpsWriterProxyOperations},
};

pub trait FromDataSubmessageAndGuidPrefix<P, D> {
    fn from(source_guid_prefix: GuidPrefix, data: &DataSubmessage<P, D>) -> Self;
}

pub trait BestEffortStatefulReaderReceiveDataBehavior<P, D> {
    fn receive_data(&mut self, source_guid_prefix: GuidPrefix, data: &DataSubmessage<P, D>);
}

impl<T, P, D> BestEffortStatefulReaderReceiveDataBehavior<P, D> for T
where
    T: RtpsStatefulReaderOperations + RtpsReaderAttributes,
    T::HistoryCacheType: RtpsHistoryCacheOperations,
    T::WriterProxyType: RtpsWriterProxyOperations,
    <T::HistoryCacheType as RtpsHistoryCacheOperations>::CacheChangeType:
        FromDataSubmessageAndGuidPrefix<P, D> + RtpsCacheChangeAttributes,
{
    fn receive_data(&mut self, source_guid_prefix: GuidPrefix, data: &DataSubmessage<P, D>) {
        let writer_guid = Guid::new(source_guid_prefix, data.writer_id.value);
        let a_change = FromDataSubmessageAndGuidPrefix::from(source_guid_prefix, data);
        if let Some(writer_proxy) = self.matched_writer_lookup(writer_guid) {
            let expected_seq_num = writer_proxy.available_changes_max() + 1;
            if RtpsCacheChangeAttributes::sequence_number(&a_change) >= expected_seq_num {
                writer_proxy
                    .received_change_set(RtpsCacheChangeAttributes::sequence_number(&a_change));
                if RtpsCacheChangeAttributes::sequence_number(&a_change) > expected_seq_num {
                    writer_proxy
                        .lost_changes_update(RtpsCacheChangeAttributes::sequence_number(&a_change));
                }
                self.reader_cache().add_change(a_change);
            }
        }
    }
}

pub trait BestEffortWriterProxyReceiveGapBehavior<S> {
    fn receive_gap(&mut self, gap: &GapSubmessage<S>);
}

impl<S, T> BestEffortWriterProxyReceiveGapBehavior<S> for T
where
    T: RtpsWriterProxyOperations,
    for<'a> &'a S: IntoIterator<Item = &'a SequenceNumber>,
{
    fn receive_gap(&mut self, gap: &GapSubmessage<S>) {
        for seq_num in gap.gap_start.value..=gap.gap_list.base - 1 {
            self.irrelevant_change_set(seq_num);
        }
        for seq_num in gap.gap_list.set.into_iter() {
            self.irrelevant_change_set(*seq_num);
        }
    }
}

pub trait ReliableStatefulReaderReceiveDataBehavior<P, D> {
    fn receive_data(&mut self, source_guid_prefix: GuidPrefix, data: &DataSubmessage<P, D>);
}

impl<T, P, D, C> ReliableStatefulReaderReceiveDataBehavior<P, D> for T
where
    T: RtpsStatefulReaderOperations + RtpsReaderAttributes,
    T::HistoryCacheType: RtpsHistoryCacheOperations<CacheChangeType = C>,
    T::WriterProxyType: RtpsWriterProxyOperations,
    <T::HistoryCacheType as RtpsHistoryCacheOperations>::CacheChangeType:
        FromDataSubmessageAndGuidPrefix<P, D> + RtpsCacheChangeAttributes,
{
    fn receive_data(&mut self, source_guid_prefix: GuidPrefix, data: &DataSubmessage<P, D>) {
        let writer_guid = Guid::new(source_guid_prefix, data.writer_id.value);
        let a_change = FromDataSubmessageAndGuidPrefix::from(source_guid_prefix, data);
        if let Some(writer_proxy) = self.matched_writer_lookup(writer_guid) {
            writer_proxy.received_change_set(RtpsCacheChangeAttributes::sequence_number(&a_change));
            self.reader_cache().add_change(a_change);
        }
    }
}

pub trait ReliableWriterProxyReceiveGapBehavior<S> {
    fn receive_gap(&mut self, gap: &GapSubmessage<S>);
}

impl<T, S> ReliableWriterProxyReceiveGapBehavior<S> for T
where
    T: RtpsWriterProxyOperations,
    for<'a> &'a S: IntoIterator<Item = &'a SequenceNumber>,
{
    fn receive_gap(&mut self, gap: &GapSubmessage<S>) {
        for seq_num in gap.gap_start.value..=gap.gap_list.base - 1 {
            self.irrelevant_change_set(seq_num);
        }
        for seq_num in gap.gap_list.set.into_iter() {
            self.irrelevant_change_set(*seq_num);
        }
    }
}

pub trait ReliableWriterProxyReceiveHeartbeat {
    fn receive_heartbeat(&mut self, heartbeat: &HeartbeatSubmessage);
}

impl<T> ReliableWriterProxyReceiveHeartbeat for T
where
    T: RtpsWriterProxyOperations,
{
    fn receive_heartbeat(&mut self, heartbeat: &HeartbeatSubmessage) {
        self.missing_changes_update(heartbeat.last_sn.value);
        self.lost_changes_update(heartbeat.first_sn.value);
    }
}

pub trait ReliableWriterProxySendAckNack<S> {
    fn send_ack_nack(
        &mut self,
        acknack_count: Count,
        send_acknack: impl FnMut(&Self, AckNackSubmessage<S>),
    );
}

impl<T, S> ReliableWriterProxySendAckNack<S> for T
where
    T: RtpsWriterProxyOperations<SequenceNumberListType = S> + RtpsWriterProxyAttributes,
{
    fn send_ack_nack(
        &mut self,
        acknack_count: Count,
        mut send_acknack: impl FnMut(&Self, AckNackSubmessage<S>),
    ) {
        let endianness_flag = true;
        let final_flag = true;
        let reader_id = EntityIdSubmessageElement {
            value: ENTITYID_UNKNOWN,
        };
        let writer_id = EntityIdSubmessageElement {
            value: self.remote_writer_guid().entity_id,
        };

        let reader_sn_state = SequenceNumberSetSubmessageElement {
            base: self.available_changes_max() + 1,
            set: self.missing_changes(),
        };
        let count = CountSubmessageElement {
            value: acknack_count,
        };

        let acknack_submessage = AckNackSubmessage {
            endianness_flag,
            final_flag,
            reader_id,
            writer_id,
            reader_sn_state,
            count,
        };

        send_acknack(self, acknack_submessage);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        behavior::types::Duration,
        messages::submessage_elements::{
            ParameterListSubmessageElement, SequenceNumberSubmessageElement,
            SerializedDataSubmessageElement,
        },
        structure::types::{ChangeKind, EntityId, InstanceHandle, Locator, ENTITYID_UNKNOWN},
    };

    use super::*;

    use mockall::mock;

    // Cache change is not mocked with the mocking framework since
    // both the constructor and the attributes don't need to be defined as part of the test run
    #[derive(Debug, PartialEq)]
    struct MockCacheChange {
        kind: ChangeKind,
        sequence_number: SequenceNumber,
    }

    impl<P, D> FromDataSubmessageAndGuidPrefix<P, D> for MockCacheChange {
        fn from(_source_guid_prefix: GuidPrefix, data: &DataSubmessage<P, D>) -> Self {
            MockCacheChange {
                kind: ChangeKind::Alive,
                sequence_number: data.writer_sn.value,
            }
        }
    }

    struct MockData;

    struct MockParameterList;

    impl RtpsCacheChangeAttributes for MockCacheChange {
        type DataType = MockData;
        type ParameterListType = MockParameterList;

        fn kind(&self) -> ChangeKind {
            self.kind
        }

        fn writer_guid(&self) -> Guid {
            unimplemented!()
        }

        fn instance_handle(&self) -> InstanceHandle {
            unimplemented!()
        }

        fn sequence_number(&self) -> SequenceNumber {
            self.sequence_number
        }

        fn data_value(&self) -> &Self::DataType {
            unimplemented!()
        }

        fn inline_qos(&self) -> &Self::ParameterListType {
            unimplemented!()
        }
    }

    mock! {
        HistoryCache{
            fn add_change_(&mut self, change: MockCacheChange);
        }
    }

    impl RtpsHistoryCacheOperations for MockHistoryCache {
        type CacheChangeType = MockCacheChange;

        fn add_change(&mut self, change: Self::CacheChangeType) {
            self.add_change_(change)
        }

        fn remove_change<F>(&mut self, _f: F)
        where
            F: FnMut(&Self::CacheChangeType) -> bool,
        {
            unimplemented!()
        }

        fn get_seq_num_min(&self) -> Option<SequenceNumber> {
            unimplemented!()
        }

        fn get_seq_num_max(&self) -> Option<SequenceNumber> {
            unimplemented!()
        }
    }

    mock! {
        WriterProxy{}

        impl RtpsWriterProxyOperations for WriterProxy {
            type SequenceNumberListType = Vec<SequenceNumber>;

            fn available_changes_max(&self) -> SequenceNumber;
            fn irrelevant_change_set(&mut self, a_seq_num: SequenceNumber);
            fn lost_changes_update(&mut self, first_available_seq_num: SequenceNumber);
            fn missing_changes(&self) -> Vec<SequenceNumber>;
            fn missing_changes_update(&mut self, last_available_seq_num: SequenceNumber);
            fn received_change_set(&mut self, a_seq_num: SequenceNumber);
        }

        impl RtpsWriterProxyAttributes for WriterProxy {
            fn remote_writer_guid(&self) -> Guid;
            fn unicast_locator_list(&self) -> &[Locator];
            fn multicast_locator_list(&self) -> &[Locator];
            fn data_max_size_serialized(&self) -> Option<i32>;
            fn remote_group_entity_id(&self) -> EntityId;
        }
    }

    struct MockStatefulReader {
        reader_cache: MockHistoryCache,
        matched_writer: Option<MockWriterProxy>,
    }

    impl RtpsReaderAttributes for MockStatefulReader {
        type HistoryCacheType = MockHistoryCache;

        fn heartbeat_response_delay(&self) -> Duration {
            todo!()
        }
        fn heartbeat_suppression_duration(&self) -> Duration {
            todo!()
        }
        fn reader_cache(&mut self) -> &mut MockHistoryCache {
            &mut self.reader_cache
        }
        fn expects_inline_qos(&self) -> bool {
            todo!()
        }
    }

    impl RtpsStatefulReaderOperations for MockStatefulReader {
        type WriterProxyType = MockWriterProxy;

        fn matched_writer_add(&mut self, _a_writer_proxy: Self::WriterProxyType) {
            todo!()
        }
        fn matched_writer_remove<F>(&mut self, _f: F)
        where
            F: FnMut(&Self::WriterProxyType) -> bool,
        {
            todo!()
        }
        fn matched_writer_lookup(
            &mut self,
            _a_writer_guid: Guid,
        ) -> Option<&mut Self::WriterProxyType> {
            self.matched_writer.as_mut()
        }
    }

    #[test]
    fn best_effort_stateful_reader_receive_data_no_missed_samples() {
        let writer_sn = 1;

        let mut writer_proxy = MockWriterProxy::new();
        writer_proxy
            .expect_available_changes_max()
            .once()
            .return_const(writer_sn - 1);
        writer_proxy
            .expect_received_change_set()
            .with(mockall::predicate::eq(writer_sn))
            .once()
            .return_const(());

        let source_guid_prefix = GuidPrefix([1; 12]);
        let data = DataSubmessage {
            endianness_flag: true,
            inline_qos_flag: false,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_sn: SequenceNumberSubmessageElement { value: writer_sn },
            inline_qos: ParameterListSubmessageElement { parameter: () },
            serialized_payload: SerializedDataSubmessageElement { value: () },
        };

        let mut reader_cache = MockHistoryCache::new();
        reader_cache
            .expect_add_change_()
            .with(mockall::predicate::eq(MockCacheChange {
                kind: ChangeKind::Alive,
                sequence_number: 1,
            }))
            .once()
            .return_const(());

        let mut stateful_reader = MockStatefulReader {
            reader_cache,
            matched_writer: Some(writer_proxy),
        };

        BestEffortStatefulReaderReceiveDataBehavior::receive_data(
            &mut stateful_reader,
            source_guid_prefix,
            &data,
        );
    }

    #[test]
    fn best_effort_stateful_reader_receive_data_with_missed_samples() {
        let writer_sn = 4;

        let mut writer_proxy = MockWriterProxy::new();
        writer_proxy
            .expect_available_changes_max()
            .once()
            .return_const(writer_sn - 2);
        writer_proxy
            .expect_received_change_set()
            .with(mockall::predicate::eq(writer_sn))
            .once()
            .return_const(());
        writer_proxy
            .expect_lost_changes_update()
            .with(mockall::predicate::eq(writer_sn))
            .once()
            .return_const(());

        let source_guid_prefix = GuidPrefix([1; 12]);
        let data = DataSubmessage {
            endianness_flag: true,
            inline_qos_flag: false,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_sn: SequenceNumberSubmessageElement { value: writer_sn },
            inline_qos: ParameterListSubmessageElement { parameter: () },
            serialized_payload: SerializedDataSubmessageElement { value: () },
        };
        let mut reader_cache = MockHistoryCache::new();
        reader_cache
            .expect_add_change_()
            .with(mockall::predicate::eq(MockCacheChange {
                kind: ChangeKind::Alive,
                sequence_number: writer_sn,
            }))
            .once()
            .return_const(());

        let mut stateful_reader = MockStatefulReader {
            reader_cache,
            matched_writer: Some(writer_proxy),
        };

        BestEffortStatefulReaderReceiveDataBehavior::receive_data(
            &mut stateful_reader,
            source_guid_prefix,
            &data,
        );
    }

    #[test]
    fn best_effort_stateful_reader_receive_gap() {
        let endianness_flag = true;
        let reader_id = EntityIdSubmessageElement {
            value: ENTITYID_UNKNOWN,
        };
        let writer_id = EntityIdSubmessageElement {
            value: ENTITYID_UNKNOWN,
        };
        let gap_start = SequenceNumberSubmessageElement { value: 3 };
        let gap_list = SequenceNumberSetSubmessageElement {
            base: 5,
            set: vec![8, 9],
        };
        let gap = GapSubmessage {
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        };

        let mut writer_proxy = MockWriterProxy::new();
        writer_proxy
            .expect_irrelevant_change_set()
            .with(mockall::predicate::eq(3))
            .once()
            .return_const(());
        writer_proxy
            .expect_irrelevant_change_set()
            .with(mockall::predicate::eq(4))
            .once()
            .return_const(());
        writer_proxy
            .expect_irrelevant_change_set()
            .with(mockall::predicate::eq(8))
            .once()
            .return_const(());
        writer_proxy
            .expect_irrelevant_change_set()
            .with(mockall::predicate::eq(9))
            .once()
            .return_const(());

        BestEffortWriterProxyReceiveGapBehavior::receive_gap(&mut writer_proxy, &gap)
    }

    #[test]
    fn reliable_stateful_reader_receive_data() {
        let writer_sn = 1;

        let mut writer_proxy = MockWriterProxy::new();
        writer_proxy
            .expect_received_change_set()
            .with(mockall::predicate::eq(writer_sn))
            .once()
            .return_const(());

        let source_guid_prefix = GuidPrefix([1; 12]);
        let data = DataSubmessage {
            endianness_flag: true,
            inline_qos_flag: false,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
            reader_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_sn: SequenceNumberSubmessageElement { value: writer_sn },
            inline_qos: ParameterListSubmessageElement { parameter: () },
            serialized_payload: SerializedDataSubmessageElement { value: () },
        };
        let mut reader_cache = MockHistoryCache::new();
        reader_cache
            .expect_add_change_()
            .with(mockall::predicate::eq(MockCacheChange {
                kind: ChangeKind::Alive,
                sequence_number: writer_sn,
            }))
            .once()
            .return_const(());

        let mut stateful_reader = MockStatefulReader {
            reader_cache,
            matched_writer: Some(writer_proxy),
        };

        ReliableStatefulReaderReceiveDataBehavior::receive_data(
            &mut stateful_reader,
            source_guid_prefix,
            &data,
        );
    }

    #[test]
    fn reliable_stateful_reader_receive_heartbeat() {
        let first_sequence_number = 2;
        let last_sequence_number = 6;

        let endianness_flag = true;
        let final_flag = true;
        let liveliness_flag = true;
        let reader_id = EntityIdSubmessageElement {
            value: ENTITYID_UNKNOWN,
        };
        let writer_id = EntityIdSubmessageElement {
            value: ENTITYID_UNKNOWN,
        };
        let first_sn = SequenceNumberSubmessageElement {
            value: first_sequence_number,
        };
        let last_sn = SequenceNumberSubmessageElement {
            value: last_sequence_number,
        };
        let count = CountSubmessageElement { value: Count(1) };
        let heartbeat = HeartbeatSubmessage {
            endianness_flag,
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        };

        let mut writer_proxy = MockWriterProxy::new();
        writer_proxy
            .expect_missing_changes_update()
            .with(mockall::predicate::eq(last_sequence_number))
            .once()
            .return_const(());
        writer_proxy
            .expect_lost_changes_update()
            .with(mockall::predicate::eq(first_sequence_number))
            .once()
            .return_const(());

        ReliableWriterProxyReceiveHeartbeat::receive_heartbeat(&mut writer_proxy, &heartbeat)
    }

    #[test]
    fn reliable_stateful_reader_send_acknack_no_missing_changes() {
        let writer_id = EntityId::new([1; 3], 2);
        let acknack_count = Count(0);
        let writer_available_changes_max = 3;
        let missing_changes = vec![];

        let expected_acknack = AckNackSubmessage {
            endianness_flag: true,
            final_flag: true,
            reader_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_id: EntityIdSubmessageElement { value: writer_id },
            reader_sn_state: SequenceNumberSetSubmessageElement {
                base: writer_available_changes_max + 1,
                set: missing_changes.clone(),
            },
            count: CountSubmessageElement {
                value: acknack_count,
            },
        };

        let mut writer_proxy = MockWriterProxy::new();
        writer_proxy
            .expect_remote_writer_guid()
            .once()
            .return_const(Guid::new(GuidPrefix([1; 12]), writer_id));
        writer_proxy
            .expect_available_changes_max()
            .once()
            .return_const(writer_available_changes_max);
        writer_proxy
            .expect_missing_changes()
            .once()
            .return_const(missing_changes);

        ReliableWriterProxySendAckNack::send_ack_nack(
            &mut writer_proxy,
            acknack_count,
            |_, acknack| assert_eq!(acknack, expected_acknack),
        )
    }

    #[test]
    fn reliable_stateful_reader_send_acknack_with_missing_changes() {
        let writer_id = EntityId::new([1; 3], 2);
        let acknack_count = Count(0);
        let writer_available_changes_max = 5;
        let missing_changes = vec![1, 2];

        let expected_acknack = AckNackSubmessage {
            endianness_flag: true,
            final_flag: true,
            reader_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_id: EntityIdSubmessageElement { value: writer_id },
            reader_sn_state: SequenceNumberSetSubmessageElement {
                base: writer_available_changes_max + 1,
                set: missing_changes.clone(),
            },
            count: CountSubmessageElement {
                value: acknack_count,
            },
        };

        let mut writer_proxy = MockWriterProxy::new();
        writer_proxy
            .expect_remote_writer_guid()
            .once()
            .return_const(Guid::new(GuidPrefix([1; 12]), writer_id));
        writer_proxy
            .expect_available_changes_max()
            .once()
            .return_const(writer_available_changes_max);
        writer_proxy
            .expect_missing_changes()
            .once()
            .return_const(missing_changes);

        ReliableWriterProxySendAckNack::send_ack_nack(
            &mut writer_proxy,
            acknack_count,
            |_, acknack| assert_eq!(acknack, expected_acknack),
        )
    }

    #[test]
    fn reliable_stateful_reader_receive_gap() {
        let endianness_flag = true;
        let reader_id = EntityIdSubmessageElement {
            value: ENTITYID_UNKNOWN,
        };
        let writer_id = EntityIdSubmessageElement {
            value: ENTITYID_UNKNOWN,
        };
        let gap_start = SequenceNumberSubmessageElement { value: 3 };
        let gap_list = SequenceNumberSetSubmessageElement {
            base: 5,
            set: vec![8, 9],
        };
        let gap = GapSubmessage {
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        };

        let mut writer_proxy = MockWriterProxy::new();
        writer_proxy
            .expect_irrelevant_change_set()
            .with(mockall::predicate::eq(3))
            .once()
            .return_const(());
        writer_proxy
            .expect_irrelevant_change_set()
            .with(mockall::predicate::eq(4))
            .once()
            .return_const(());
        writer_proxy
            .expect_irrelevant_change_set()
            .with(mockall::predicate::eq(8))
            .once()
            .return_const(());
        writer_proxy
            .expect_irrelevant_change_set()
            .with(mockall::predicate::eq(9))
            .once()
            .return_const(());

        ReliableWriterProxyReceiveGapBehavior::receive_gap(&mut writer_proxy, &gap)
    }
}

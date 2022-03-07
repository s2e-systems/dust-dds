use crate::{
    messages::{
        submessage_elements::{
            CountSubmessageElement, EntityIdSubmessageElement, SequenceNumberSetSubmessageElement,
        },
        submessages::{AckNackSubmessage, DataSubmessage, GapSubmessage, HeartbeatSubmessage},
        types::Count,
    },
    structure::{
        cache_change::{RtpsCacheChangeAttributes, RtpsCacheChangeConstructor},
        history_cache::RtpsHistoryCacheOperations,
        types::{ChangeKind, EntityId, Guid, GuidPrefix, SequenceNumber},
    },
};

use super::reader::writer_proxy::{RtpsWriterProxyAttributes, RtpsWriterProxyOperations};

pub struct BestEffortStatefulReaderBehavior;

impl BestEffortStatefulReaderBehavior {
    // 8.4.12.1.2 Transition T2
    pub fn receive_data<'a, C, P, D>(
        writer_proxy: &mut impl RtpsWriterProxyOperations,
        reader_cache: &mut impl RtpsHistoryCacheOperations<CacheChangeType = C>,
        source_guid_prefix: GuidPrefix,
        data: &'a DataSubmessage<P, D>,
    ) where
        C: RtpsCacheChangeConstructor + RtpsCacheChangeAttributes<'a>,
        for<'b> &'b D: Into<<C as RtpsCacheChangeConstructor>::DataType>,
        for<'b> &'b P: Into<<C as RtpsCacheChangeConstructor>::ParameterListType>,
    {
        let writer_guid = Guid::new(source_guid_prefix, data.writer_id.value);
        let kind = match (data.data_flag, data.key_flag) {
            (true, false) => ChangeKind::Alive,
            (false, true) => ChangeKind::NotAliveDisposed,
            _ => todo!(),
        };
        let instance_handle = 0;
        let sequence_number = data.writer_sn.value;
        let data_value = (&data.serialized_payload.value).into();
        let inline_qos = (&data.inline_qos.parameter).into();
        let a_change: C = RtpsCacheChangeConstructor::new(
            kind,
            writer_guid,
            instance_handle,
            sequence_number,
            data_value,
            inline_qos,
        );

        let expected_seq_num = writer_proxy.available_changes_max() + 1;
        if a_change.sequence_number() >= expected_seq_num {
            writer_proxy.received_change_set(a_change.sequence_number());
            if a_change.sequence_number() > expected_seq_num {
                writer_proxy.lost_changes_update(a_change.sequence_number());
            }
            reader_cache.add_change(a_change);
        }
    }

    // 8.4.12.1.4 Transition T4
    pub fn receive_gap<S>(writer_proxy: &mut impl RtpsWriterProxyOperations, gap: GapSubmessage<S>)
    where
        S: IntoIterator<Item = SequenceNumber>,
    {
        for seq_num in gap.gap_start.value..=gap.gap_list.base - 1 {
            writer_proxy.irrelevant_change_set(seq_num);
        }
        for seq_num in gap.gap_list.set {
            writer_proxy.irrelevant_change_set(seq_num);
        }
    }
}

pub struct ReliableStatefulReaderBehavior;

impl ReliableStatefulReaderBehavior {
    // 8.4.12.2.8 Transition T8
    pub fn receive_data<'a, C, P, D>(
        writer_proxy: &mut impl RtpsWriterProxyOperations,
        reader_cache: &mut impl RtpsHistoryCacheOperations<CacheChangeType = C>,
        source_guid_prefix: GuidPrefix,
        data: &'a DataSubmessage<P, D>,
    ) where
        C: RtpsCacheChangeConstructor + RtpsCacheChangeAttributes<'a>,
        for<'b> <C as RtpsCacheChangeConstructor>::DataType: From<&'b D>,
        for<'b> <C as RtpsCacheChangeConstructor>::ParameterListType: From<&'b P>,
    {
        let writer_guid = Guid::new(source_guid_prefix, data.writer_id.value);

        let kind = match (data.data_flag, data.key_flag) {
            (true, false) => ChangeKind::Alive,
            (false, true) => ChangeKind::NotAliveDisposed,
            _ => todo!(),
        };
        let instance_handle = 0;
        let sequence_number = data.writer_sn.value;
        let data_value = (&data.serialized_payload.value).into();
        let inline_qos = (&data.inline_qos.parameter).into();
        let a_change = C::new(
            kind,
            writer_guid,
            instance_handle,
            sequence_number,
            data_value,
            inline_qos,
        );
        writer_proxy.received_change_set(a_change.sequence_number());
        reader_cache.add_change(a_change);
    }

    // 8.4.12.2.5 Transition T5
    pub fn send_ack_nack<S>(
        // Might be a bit too restrictive to oblige the missing changes and the AckNack to be the same type S.
        // However this will generally be S=Vec<SequenceNumber> so it stays like this for now for easy implementation
        writer_proxy: &mut (impl RtpsWriterProxyOperations<SequenceNumberListType = S>
                  + RtpsWriterProxyAttributes),
        reader_id: EntityId,
        acknack_count: Count,
        mut send_acknack: impl FnMut(AckNackSubmessage<S>),
    ) {
        let endianness_flag = true;
        let final_flag = true;
        let reader_id = EntityIdSubmessageElement { value: reader_id };
        let writer_id = EntityIdSubmessageElement {
            value: writer_proxy.remote_writer_guid().entity_id,
        };

        let reader_sn_state = SequenceNumberSetSubmessageElement {
            base: writer_proxy.available_changes_max() + 1,
            set: writer_proxy.missing_changes(),
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

        send_acknack(acknack_submessage);
    }

    // 8.4.12.2.7 Transition T7
    pub fn receive_heartbeat(
        writer_proxy: &mut impl RtpsWriterProxyOperations,
        heartbeat: HeartbeatSubmessage,
    ) {
        writer_proxy.missing_changes_update(heartbeat.last_sn.value);
        writer_proxy.lost_changes_update(heartbeat.first_sn.value);
    }

    // 8.4.12.2.9 Transition T9
    pub fn receive_gap<S>(writer_proxy: &mut impl RtpsWriterProxyOperations, gap: GapSubmessage<S>)
    where
        S: IntoIterator<Item = SequenceNumber>,
    {
        for seq_num in gap.gap_start.value..=gap.gap_list.base - 1 {
            writer_proxy.irrelevant_change_set(seq_num);
        }
        for seq_num in gap.gap_list.set {
            writer_proxy.irrelevant_change_set(seq_num);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        messages::submessage_elements::{
            ParameterListSubmessageElement, SequenceNumberSubmessageElement,
            SerializedDataSubmessageElement,
        },
        structure::types::{InstanceHandle, Locator, ENTITYID_UNKNOWN},
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

    struct MockData;
    impl From<&&[u8]> for MockData {
        fn from(_: &&[u8]) -> Self {
            MockData
        }
    }

    struct MockParameterList;
    impl From<&()> for MockParameterList {
        fn from(_: &()) -> Self {
            MockParameterList
        }
    }

    impl RtpsCacheChangeConstructor for MockCacheChange {
        type DataType = MockData;
        type ParameterListType = MockParameterList;

        fn new(
            kind: ChangeKind,
            _writer_guid: Guid,
            _instance_handle: InstanceHandle,
            sequence_number: SequenceNumber,
            _data_value: Self::DataType,
            _inline_qos: Self::ParameterListType,
        ) -> Self {
            Self {
                kind,
                sequence_number,
            }
        }
    }

    impl<'a> RtpsCacheChangeAttributes<'a> for MockCacheChange {
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

        let mut reader_cache = MockHistoryCache::new();
        reader_cache
            .expect_add_change_()
            .with(mockall::predicate::eq(MockCacheChange {
                kind: ChangeKind::Alive,
                sequence_number: 1,
            }))
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
            serialized_payload: SerializedDataSubmessageElement { value: &[][..] },
        };
        BestEffortStatefulReaderBehavior::receive_data(
            &mut writer_proxy,
            &mut reader_cache,
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

        let mut reader_cache = MockHistoryCache::new();
        reader_cache
            .expect_add_change_()
            .with(mockall::predicate::eq(MockCacheChange {
                kind: ChangeKind::Alive,
                sequence_number: 4,
            }))
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
            serialized_payload: SerializedDataSubmessageElement { value: &[][..] },
        };
        BestEffortStatefulReaderBehavior::receive_data(
            &mut writer_proxy,
            &mut reader_cache,
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

        BestEffortStatefulReaderBehavior::receive_gap(&mut writer_proxy, gap)
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

        let mut reader_cache = MockHistoryCache::new();
        reader_cache
            .expect_add_change_()
            .with(mockall::predicate::eq(MockCacheChange {
                kind: ChangeKind::Alive,
                sequence_number: 1,
            }))
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
            serialized_payload: SerializedDataSubmessageElement { value: &[][..] },
        };
        ReliableStatefulReaderBehavior::receive_data(
            &mut writer_proxy,
            &mut reader_cache,
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

        ReliableStatefulReaderBehavior::receive_heartbeat(&mut writer_proxy, heartbeat)
    }

    #[test]
    fn reliable_stateful_reader_send_acknack_no_missing_changes() {
        let reader_id = ENTITYID_UNKNOWN;
        let writer_id = EntityId::new([1; 3], 2);
        let acknack_count = Count(2);
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

        ReliableStatefulReaderBehavior::send_ack_nack(
            &mut writer_proxy,
            reader_id,
            acknack_count,
            |acknack| assert_eq!(acknack, expected_acknack),
        )
    }

    #[test]
    fn reliable_stateful_reader_send_acknack_with_missing_changes() {
        let reader_id = ENTITYID_UNKNOWN;
        let writer_id = EntityId::new([1; 3], 2);
        let acknack_count = Count(2);
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

        ReliableStatefulReaderBehavior::send_ack_nack(
            &mut writer_proxy,
            reader_id,
            acknack_count,
            |acknack| assert_eq!(acknack, expected_acknack),
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

        ReliableStatefulReaderBehavior::receive_gap(&mut writer_proxy, gap)
    }
}

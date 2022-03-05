use core::iter::FromIterator;

use crate::{
    messages::{
        submessage_elements::{
            CountSubmessageElement, EntityIdSubmessageElement, Parameter,
            SequenceNumberSetSubmessageElement,
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
    pub fn receive_data<'a, C, P>(
        writer_proxy: &mut impl RtpsWriterProxyOperations,
        reader_cache: &mut impl RtpsHistoryCacheOperations<CacheChangeType = C>,
        source_guid_prefix: GuidPrefix,
        data: &'a DataSubmessage<'_, P>,
    ) where
        C: RtpsCacheChangeConstructor<
                'a,
                DataType = &'a [u8],
                ParameterListType = &'a [Parameter<'a>],
            > + RtpsCacheChangeAttributes<'a>,
        P: AsRef<[Parameter<'a>]>,
    {
        let writer_guid = Guid::new(source_guid_prefix, data.writer_id.value);
        let kind = match (data.data_flag, data.key_flag) {
            (true, false) => ChangeKind::Alive,
            (false, true) => ChangeKind::NotAliveDisposed,
            _ => todo!(),
        };
        let instance_handle = 0;
        let sequence_number = data.writer_sn.value;
        let data_value = data.serialized_payload.value;
        let inline_qos = data.inline_qos.parameter.as_ref();
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
        for seq_num in gap.gap_start.value..gap.gap_list.base - 1 {
            writer_proxy.irrelevant_change_set(seq_num);
        }
        for seq_num in gap.gap_list.set {
            writer_proxy.irrelevant_change_set(seq_num);
        }
    }
}

pub struct ReliableStatefulReaderBehavior;

impl ReliableStatefulReaderBehavior {
    pub fn receive_data<'a, C, P>(
        writer_proxy: &mut impl RtpsWriterProxyOperations,
        reader_cache: &mut impl RtpsHistoryCacheOperations<CacheChangeType = C>,
        source_guid_prefix: GuidPrefix,
        data: &'a DataSubmessage<'_, P>,
    ) where
        C: RtpsCacheChangeConstructor<
                'a,
                DataType = &'a [u8],
                ParameterListType = &'a [Parameter<'a>],
            > + RtpsCacheChangeAttributes<'a>,
        P: AsRef<[Parameter<'a>]>,
    {
        let writer_guid = Guid::new(source_guid_prefix, data.writer_id.value);

        let kind = match (data.data_flag, data.key_flag) {
            (true, false) => ChangeKind::Alive,
            (false, true) => ChangeKind::NotAliveDisposed,
            _ => todo!(),
        };
        let instance_handle = 0;
        let sequence_number = data.writer_sn.value;
        let data_value = data.serialized_payload.value;
        let inline_qos = data.inline_qos.parameter.as_ref();
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

    pub fn send_ack_nack<S>(
        writer_proxy: &mut (impl RtpsWriterProxyOperations + RtpsWriterProxyAttributes),
        reader_id: EntityId,
        acknack_count: Count,
        mut send_acknack: impl FnMut(AckNackSubmessage<S>),
    ) where
        S: FromIterator<SequenceNumber>,
    {
        let endianness_flag = true;
        let final_flag = false;
        let reader_id = EntityIdSubmessageElement { value: reader_id };
        let writer_id = EntityIdSubmessageElement {
            value: writer_proxy.remote_writer_guid().entity_id,
        };
        let reader_sn_state = SequenceNumberSetSubmessageElement {
            base: writer_proxy.available_changes_max() + 1,
            set: core::iter::empty().collect(), // FOREACH change IN the_writer_proxy.missing_changes() DO ADD change.sequenceNumber TO missing_seq_num_set.set;
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

    pub fn receive_heartbeat(
        writer_proxy: &mut impl RtpsWriterProxyOperations,
        heartbeat: HeartbeatSubmessage,
    ) {
        writer_proxy.missing_changes_update(heartbeat.last_sn.value);
        writer_proxy.lost_changes_update(heartbeat.first_sn.value);
    }

    pub fn receive_gap<S>(writer_proxy: &mut impl RtpsWriterProxyOperations, gap: GapSubmessage<S>)
    where
        S: IntoIterator<Item = SequenceNumber>,
    {
        for seq_num in gap.gap_start.value..gap.gap_list.base - 1 {
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
        structure::types::{InstanceHandle, ENTITYID_UNKNOWN},
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

    impl<'a> RtpsCacheChangeConstructor<'a> for MockCacheChange {
        type DataType = &'a [u8];
        type ParameterListType = &'a [Parameter<'a>];

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
        type DataType = [u8];
        type ParameterListType = [Parameter<'a>];

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
            todo!()
        }

        fn get_seq_num_min(&self) -> Option<SequenceNumber> {
            todo!()
        }

        fn get_seq_num_max(&self) -> Option<SequenceNumber> {
            todo!()
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
            inline_qos: ParameterListSubmessageElement { parameter: vec![] },
            serialized_payload: SerializedDataSubmessageElement { value: &[] },
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
            inline_qos: ParameterListSubmessageElement { parameter: vec![] },
            serialized_payload: SerializedDataSubmessageElement { value: &[] },
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
            .with(mockall::predicate::eq(5))
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
}

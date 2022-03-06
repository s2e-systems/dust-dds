use core::iter::FromIterator;

/// This file implements the behaviors described in 8.4.9 RTPS StatefulWriter Behavior
use crate::{
    messages::{
        submessage_elements::{
            CountSubmessageElement, EntityIdSubmessageElement, Parameter,
            ParameterListSubmessageElement, SequenceNumberSetSubmessageElement,
            SequenceNumberSubmessageElement, SerializedDataSubmessageElement,
        },
        submessages::{AckNackSubmessage, DataSubmessage, GapSubmessage, HeartbeatSubmessage},
        types::Count,
    },
    structure::{
        cache_change::RtpsCacheChangeAttributes,
        history_cache::{RtpsHistoryCacheAttributes, RtpsHistoryCacheOperations},
        types::{ChangeKind, EntityId, SequenceNumber, ENTITYID_UNKNOWN},
    },
};

use super::writer::reader_proxy::RtpsReaderProxyOperations;

pub struct BestEffortStatefulWriterBehavior;

impl BestEffortStatefulWriterBehavior {
    /// 8.4.9.1.4 Transition T4
    pub fn send_unsent_changes<'a, CacheChange, S, P>(
        reader_proxy: &mut impl RtpsReaderProxyOperations<ChangeForReaderType = SequenceNumber>,
        writer_cache: &'a impl RtpsHistoryCacheAttributes<CacheChangeType = CacheChange>,
        reader_id: EntityId,
        mut send_data: impl FnMut(DataSubmessage<'a, P>),
        mut send_gap: impl FnMut(GapSubmessage<S>),
    ) where
        CacheChange: RtpsCacheChangeAttributes<'a, DataType = [u8]> + 'a,
        &'a <CacheChange as RtpsCacheChangeAttributes<'a>>::ParameterListType:
            IntoIterator<Item = Parameter<'a>> + 'a,
        S: FromIterator<SequenceNumber>,
        P: FromIterator<Parameter<'a>>,
    {
        while let Some(seq_num) = reader_proxy.next_unsent_change() {
            let change = writer_cache
                .changes()
                .iter()
                .filter(|cc| cc.sequence_number() == seq_num)
                .next();
            if let Some(change) = change {
                let endianness_flag = true;
                let inline_qos_flag = true;
                let (data_flag, key_flag) = match change.kind() {
                    ChangeKind::Alive => (true, false),
                    ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => {
                        (false, true)
                    }
                    _ => todo!(),
                };
                let non_standard_payload_flag = false;
                let reader_id = EntityIdSubmessageElement { value: reader_id };
                let writer_id = EntityIdSubmessageElement {
                    value: change.writer_guid().entity_id(),
                };
                let writer_sn = SequenceNumberSubmessageElement {
                    value: change.sequence_number(),
                };
                let inline_qos = ParameterListSubmessageElement {
                    parameter: change.inline_qos().into_iter().collect(),
                };
                let serialized_payload = SerializedDataSubmessageElement {
                    value: change.data_value(),
                };
                let data_submessage = DataSubmessage {
                    endianness_flag,
                    inline_qos_flag,
                    data_flag,
                    key_flag,
                    non_standard_payload_flag,
                    reader_id,
                    writer_id,
                    writer_sn,
                    inline_qos,
                    serialized_payload,
                };
                send_data(data_submessage)
            } else {
                let endianness_flag = true;
                let reader_id = EntityIdSubmessageElement { value: reader_id };
                let writer_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let gap_start = SequenceNumberSubmessageElement { value: seq_num };
                let gap_list = SequenceNumberSetSubmessageElement {
                    base: seq_num,
                    set: core::iter::empty().collect(),
                };
                let gap_submessage = GapSubmessage {
                    endianness_flag,
                    reader_id,
                    writer_id,
                    gap_start,
                    gap_list,
                };
                send_gap(gap_submessage)
            }
        }
    }
}

/// This struct is a wrapper for the implementation of the behaviors described in 8.4.9.2 Reliable StatefulWriter Behavior
pub struct ReliableStatefulWriterBehavior;

impl ReliableStatefulWriterBehavior {
    /// Implement 8.4.9.2.4 Transition T4
    pub fn send_unsent_changes<'a, CacheChange, S, P>(
        reader_proxy: &mut impl RtpsReaderProxyOperations<ChangeForReaderType = SequenceNumber>,
        writer_cache: &'a impl RtpsHistoryCacheAttributes<CacheChangeType = CacheChange>,
        reader_id: EntityId,
        mut send_data: impl FnMut(DataSubmessage<'a, P>),
        mut send_gap: impl FnMut(GapSubmessage<S>),
    ) where
        CacheChange: RtpsCacheChangeAttributes<'a, DataType = [u8]> + 'a,
        &'a <CacheChange as RtpsCacheChangeAttributes<'a>>::ParameterListType:
            IntoIterator<Item = Parameter<'a>> + 'a,
        S: FromIterator<SequenceNumber>,
        P: FromIterator<Parameter<'a>>,
    {
        while let Some(seq_num) = reader_proxy.next_unsent_change() {
            let change = writer_cache
                .changes()
                .iter()
                .filter(|cc| cc.sequence_number() == seq_num)
                .next();
            if let Some(change) = change {
                let endianness_flag = true;
                let inline_qos_flag = true;
                let (data_flag, key_flag) = match change.kind() {
                    ChangeKind::Alive => (true, false),
                    ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => {
                        (false, true)
                    }
                    _ => todo!(),
                };
                let non_standard_payload_flag = false;
                let reader_id = EntityIdSubmessageElement { value: reader_id };
                let writer_id = EntityIdSubmessageElement {
                    value: change.writer_guid().entity_id(),
                };
                let writer_sn = SequenceNumberSubmessageElement {
                    value: change.sequence_number(),
                };
                let inline_qos = ParameterListSubmessageElement {
                    parameter: change.inline_qos().into_iter().collect(),
                };
                let serialized_payload = SerializedDataSubmessageElement {
                    value: change.data_value(),
                };
                let data_submessage = DataSubmessage {
                    endianness_flag,
                    inline_qos_flag,
                    data_flag,
                    key_flag,
                    non_standard_payload_flag,
                    reader_id,
                    writer_id,
                    writer_sn,
                    inline_qos,
                    serialized_payload,
                };
                send_data(data_submessage)
            } else {
                let endianness_flag = true;
                let reader_id = EntityIdSubmessageElement { value: reader_id };
                let writer_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let gap_start = SequenceNumberSubmessageElement { value: seq_num };
                let gap_list = SequenceNumberSetSubmessageElement {
                    base: seq_num,
                    set: core::iter::empty().collect(),
                };
                let gap_submessage = GapSubmessage {
                    endianness_flag,
                    reader_id,
                    writer_id,
                    gap_start,
                    gap_list,
                };
                send_gap(gap_submessage)
            }
        }
    }

    /// 8.4.9.2.7 Transition T7
    pub fn send_heartbeat(
        writer_cache: &impl RtpsHistoryCacheOperations,
        writer_id: EntityId,
        heartbeat_count: Count,
        mut send_heartbeat: impl FnMut(HeartbeatSubmessage),
    ) {
        let endianness_flag = true;
        let final_flag = false;
        let liveliness_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: ENTITYID_UNKNOWN,
        };
        let writer_id = EntityIdSubmessageElement { value: writer_id };
        let first_sn = SequenceNumberSubmessageElement {
            value: writer_cache.get_seq_num_min().unwrap_or(0),
        };
        let last_sn = SequenceNumberSubmessageElement {
            value: writer_cache.get_seq_num_max().unwrap_or(0),
        };
        let count = CountSubmessageElement {
            value: heartbeat_count,
        };
        let heartbeat_submessage = HeartbeatSubmessage {
            endianness_flag,
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        };
        send_heartbeat(heartbeat_submessage)
    }

    /// 8.4.9.2.8 Transition T8
    pub fn receive_acknack<S>(
        reader_proxy: &mut impl RtpsReaderProxyOperations,
        acknack: &AckNackSubmessage<S>,
    ) where
        S: AsRef<[SequenceNumber]>,
    {
        reader_proxy.acked_changes_set(acknack.reader_sn_state.base - 1);
        reader_proxy.requested_changes_set(acknack.reader_sn_state.set.as_ref());
    }

    /// 8.4.8.2.10 Transition T10
    pub fn send_requested_changes<'a, CacheChange, S, P>(
        reader_proxy: &mut impl RtpsReaderProxyOperations<ChangeForReaderType = SequenceNumber>,
        writer_cache: &'a impl RtpsHistoryCacheAttributes<CacheChangeType = CacheChange>,
        reader_id: EntityId,
        mut send_data: impl FnMut(DataSubmessage<'a, P>),
        mut send_gap: impl FnMut(GapSubmessage<S>),
    ) where
        CacheChange: RtpsCacheChangeAttributes<'a, DataType = [u8]> + 'a,
        &'a <CacheChange as RtpsCacheChangeAttributes<'a>>::ParameterListType:
            IntoIterator<Item = Parameter<'a>> + 'a,
        S: FromIterator<SequenceNumber>,
        P: FromIterator<Parameter<'a>>,
    {
        while let Some(seq_num) = reader_proxy.next_requested_change() {
            let change = writer_cache
                .changes()
                .iter()
                .filter(|cc| cc.sequence_number() == seq_num)
                .next();
            if let Some(change) = change {
                let endianness_flag = true;
                let inline_qos_flag = true;
                let (data_flag, key_flag) = match change.kind() {
                    ChangeKind::Alive => (true, false),
                    ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => {
                        (false, true)
                    }
                    _ => todo!(),
                };
                let non_standard_payload_flag = false;
                let reader_id = EntityIdSubmessageElement { value: reader_id };
                let writer_id = EntityIdSubmessageElement {
                    value: change.writer_guid().entity_id(),
                };
                let writer_sn = SequenceNumberSubmessageElement {
                    value: change.sequence_number(),
                };
                let inline_qos = ParameterListSubmessageElement {
                    parameter: change.inline_qos().into_iter().collect(),
                };
                let serialized_payload = SerializedDataSubmessageElement {
                    value: change.data_value(),
                };
                let data_submessage = DataSubmessage {
                    endianness_flag,
                    inline_qos_flag,
                    data_flag,
                    key_flag,
                    non_standard_payload_flag,
                    reader_id,
                    writer_id,
                    writer_sn,
                    inline_qos,
                    serialized_payload,
                };
                send_data(data_submessage)
            } else {
                let endianness_flag = true;
                let reader_id = EntityIdSubmessageElement { value: reader_id };
                let writer_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let gap_start = SequenceNumberSubmessageElement { value: seq_num };
                let gap_list = SequenceNumberSetSubmessageElement {
                    base: seq_num,
                    set: core::iter::empty().collect(),
                };
                let gap_submessage = GapSubmessage {
                    endianness_flag,
                    reader_id,
                    writer_id,
                    gap_start,
                    gap_list,
                };
                send_gap(gap_submessage)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use mockall::mock;

    use crate::structure::types::{Guid, GuidPrefix, InstanceHandle};

    use super::*;

    pub struct MockParameterList;

    impl<'a> IntoIterator for &'a MockParameterList {
        type Item = Parameter<'a>;
        type IntoIter = std::vec::IntoIter<Parameter<'a>>;

        fn into_iter(self) -> Self::IntoIter {
            vec![].into_iter()
        }
    }
    pub struct MockCacheChange {
        kind: ChangeKind,
        writer_guid: Guid,
        instance_handle: InstanceHandle,
        sequence_number: SequenceNumber,
        data_value: Vec<u8>,
        inline_qos: MockParameterList,
    }

    impl<'a> RtpsCacheChangeAttributes<'a> for MockCacheChange {
        type DataType = [u8];
        type ParameterListType = MockParameterList;

        fn kind(&self) -> ChangeKind {
            self.kind
        }

        fn writer_guid(&self) -> Guid {
            self.writer_guid
        }

        fn instance_handle(&self) -> InstanceHandle {
            self.instance_handle
        }

        fn sequence_number(&self) -> SequenceNumber {
            self.sequence_number
        }

        fn data_value(&self) -> &Self::DataType {
            self.data_value.as_ref()
        }

        fn inline_qos(&self) -> &Self::ParameterListType {
            &self.inline_qos
        }
    }

    mock! {
        HistoryCache{
            fn get_seq_num_min_(&self) -> Option<SequenceNumber>;
            fn get_seq_num_max_(&self) -> Option<SequenceNumber>;
        }

        impl RtpsHistoryCacheAttributes for HistoryCache{
            type CacheChangeType = MockCacheChange;

            fn changes(&self) -> &[MockCacheChange];
        }
    }

    impl RtpsHistoryCacheOperations for MockHistoryCache {
        type CacheChangeType = MockCacheChange;

        fn add_change(&mut self, _change: Self::CacheChangeType) {
            todo!()
        }

        fn remove_change<F>(&mut self, _f: F)
        where
            F: FnMut(&Self::CacheChangeType) -> bool,
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

    mock! {
        ReaderProxy{}

        impl RtpsReaderProxyOperations for ReaderProxy {
            type ChangeForReaderType = SequenceNumber;
            type ChangeForReaderListType = Vec<SequenceNumber>;

            fn acked_changes_set(&mut self, committed_seq_num: SequenceNumber);
            fn next_requested_change(&mut self) -> Option<SequenceNumber>;
            fn next_unsent_change(&mut self) -> Option<SequenceNumber>;
            fn unsent_changes(&self) ->  Vec<SequenceNumber>;
            fn requested_changes(&self) ->  Vec<SequenceNumber>;
            fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]);
            fn unacked_changes(&self) ->  Vec<SequenceNumber>;
            fn changes_for_reader(&self) ->  Vec<SequenceNumber>;
        }
    }

    mock! {
        DataMessageSender<'a>{
            fn send_data(&mut self, data: DataSubmessage<'a, Vec<Parameter<'a>>> );
        }
    }
    mock! {
        GapMessageSender {
            fn send_gap(&mut self, gap: GapSubmessage<Vec<SequenceNumber>>);
        }
    }

    mock! {
        HeartbeatMessageSender {
            fn send_heartbeat(&mut self, heartbeat: HeartbeatSubmessage);
        }
    }

    #[test]
    fn best_effort_stateful_writer_send_unsent_changes_single_data_submessage() {
        let mut seq = mockall::Sequence::new();

        let mut reader_proxy = MockReaderProxy::new();
        let mut writer_cache = MockHistoryCache::new();
        let mut data_message_sender = MockDataMessageSender::new();
        let mut gap_message_sender = MockGapMessageSender::new();
        let reader_id = EntityId::new([1; 3], 1);

        reader_proxy
            .expect_next_unsent_change()
            .once()
            .return_const(Some(1))
            .in_sequence(&mut seq);
        writer_cache
            .expect_changes()
            .once()
            .return_const(vec![MockCacheChange {
                kind: ChangeKind::Alive,
                writer_guid: Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)),
                instance_handle: 1,
                sequence_number: 1,
                data_value: vec![1, 2, 3, 4],
                inline_qos: MockParameterList,
            }])
            .in_sequence(&mut seq);
        data_message_sender
            .expect_send_data()
            // Can't use a complete expected DataSubmessage due to issues with the lifetime.
            .withf(|data| {
                data.data_flag == true
                    && data.key_flag == false
                    && data.non_standard_payload_flag == false
                    && data.writer_sn.value == 1
                    && data.reader_id.value == EntityId::new([1; 3], 1)
                    && data.inline_qos.parameter.is_empty()
                    && data.serialized_payload.value == &[1, 2, 3, 4]
            })
            .once()
            .return_const(())
            .in_sequence(&mut seq);
        reader_proxy
            .expect_next_unsent_change()
            .once()
            .return_const(None)
            .in_sequence(&mut seq);

        BestEffortStatefulWriterBehavior::send_unsent_changes(
            &mut reader_proxy,
            &writer_cache,
            reader_id,
            |data| data_message_sender.send_data(data),
            |gap| gap_message_sender.send_gap(gap),
        )
    }

    #[test]
    fn best_effort_stateful_writer_send_unsent_changes_single_gap_submessage() {
        let mut seq = mockall::Sequence::new();

        let mut reader_proxy = MockReaderProxy::new();
        let mut writer_cache = MockHistoryCache::new();
        let mut data_message_sender = MockDataMessageSender::new();
        let mut gap_message_sender = MockGapMessageSender::new();
        let reader_id = EntityId::new([1; 3], 1);

        reader_proxy
            .expect_next_unsent_change()
            .once()
            .return_const(Some(1))
            .in_sequence(&mut seq);
        writer_cache
            .expect_changes()
            .once()
            .return_const(vec![])
            .in_sequence(&mut seq);
        gap_message_sender
            .expect_send_gap()
            .with(mockall::predicate::eq(GapSubmessage {
                endianness_flag: true,
                reader_id: EntityIdSubmessageElement { value: reader_id },
                writer_id: EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                },
                gap_start: SequenceNumberSubmessageElement { value: 1 },
                gap_list: SequenceNumberSetSubmessageElement {
                    base: 1,
                    set: vec![],
                },
            }))
            .once()
            .return_const(())
            .in_sequence(&mut seq);
        reader_proxy
            .expect_next_unsent_change()
            .once()
            .return_const(None)
            .in_sequence(&mut seq);

        BestEffortStatefulWriterBehavior::send_unsent_changes(
            &mut reader_proxy,
            &writer_cache,
            reader_id,
            |data| data_message_sender.send_data(data),
            |gap| gap_message_sender.send_gap(gap),
        )
    }

    #[test]
    fn reliable_stateful_writer_send_unsent_changes_single_data_submessage() {
        let mut seq = mockall::Sequence::new();

        let mut reader_proxy = MockReaderProxy::new();
        let mut writer_cache = MockHistoryCache::new();
        let mut data_message_sender = MockDataMessageSender::new();
        let mut gap_message_sender = MockGapMessageSender::new();

        let reader_id = EntityId::new([1; 3], 1);

        reader_proxy
            .expect_next_unsent_change()
            .once()
            .return_const(Some(1))
            .in_sequence(&mut seq);
        writer_cache
            .expect_changes()
            .once()
            .return_const(vec![MockCacheChange {
                kind: ChangeKind::Alive,
                writer_guid: Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)),
                instance_handle: 1,
                sequence_number: 1,
                data_value: vec![1, 2, 3, 4],
                inline_qos: MockParameterList,
            }])
            .in_sequence(&mut seq);
        data_message_sender
            .expect_send_data()
            // Can't use a complete expected DataSubmessage due to issues with the lifetime.
            .withf(|data| {
                data.data_flag == true
                    && data.key_flag == false
                    && data.non_standard_payload_flag == false
                    && data.writer_sn.value == 1
                    && data.reader_id.value == EntityId::new([1; 3], 1)
                    && data.inline_qos.parameter.is_empty()
                    && data.serialized_payload.value == &[1, 2, 3, 4]
            })
            .once()
            .return_const(())
            .in_sequence(&mut seq);
        reader_proxy
            .expect_next_unsent_change()
            .once()
            .return_const(None)
            .in_sequence(&mut seq);

        ReliableStatefulWriterBehavior::send_unsent_changes(
            &mut reader_proxy,
            &writer_cache,
            reader_id,
            |data| data_message_sender.send_data(data),
            |gap| gap_message_sender.send_gap(gap),
        )
    }

    #[test]
    fn reliable_stateful_writer_send_unsent_changes_single_gap_submessage() {
        let mut seq = mockall::Sequence::new();

        let mut reader_proxy = MockReaderProxy::new();
        let mut writer_cache = MockHistoryCache::new();
        let mut data_message_sender = MockDataMessageSender::new();
        let mut gap_message_sender = MockGapMessageSender::new();

        let reader_id = EntityId::new([1; 3], 1);

        reader_proxy
            .expect_next_unsent_change()
            .once()
            .return_const(Some(1))
            .in_sequence(&mut seq);
        writer_cache
            .expect_changes()
            .once()
            .return_const(vec![])
            .in_sequence(&mut seq);
        gap_message_sender
            .expect_send_gap()
            .with(mockall::predicate::eq(GapSubmessage {
                endianness_flag: true,
                reader_id: EntityIdSubmessageElement { value: reader_id },
                writer_id: EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                },
                gap_start: SequenceNumberSubmessageElement { value: 1 },
                gap_list: SequenceNumberSetSubmessageElement {
                    base: 1,
                    set: vec![],
                },
            }))
            .once()
            .return_const(())
            .in_sequence(&mut seq);
        reader_proxy
            .expect_next_unsent_change()
            .once()
            .return_const(None)
            .in_sequence(&mut seq);

        ReliableStatefulWriterBehavior::send_unsent_changes(
            &mut reader_proxy,
            &writer_cache,
            reader_id,
            |data| data_message_sender.send_data(data),
            |gap| gap_message_sender.send_gap(gap),
        )
    }

    #[test]
    fn reliable_stateful_writer_send_heartbeat() {
        let mut writer_cache = MockHistoryCache::new();
        let mut heartbeat_submessage_sender = MockHeartbeatMessageSender::new();
        let writer_id = EntityId::new([1; 3], 1);
        let heartbeat_count = Count(1);

        writer_cache
            .expect_get_seq_num_min_()
            .once()
            .return_const(Some(1));
        writer_cache
            .expect_get_seq_num_max_()
            .once()
            .return_const(Some(4));

        heartbeat_submessage_sender
            .expect_send_heartbeat()
            .once()
            .with(mockall::predicate::eq(HeartbeatSubmessage {
                endianness_flag: true,
                final_flag: false,
                liveliness_flag: false,
                reader_id: EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                },
                writer_id: EntityIdSubmessageElement { value: writer_id },
                first_sn: SequenceNumberSubmessageElement { value: 1 },
                last_sn: SequenceNumberSubmessageElement { value: 4 },
                count: CountSubmessageElement {
                    value: heartbeat_count,
                },
            }))
            .return_const(());

        ReliableStatefulWriterBehavior::send_heartbeat(
            &mut writer_cache,
            writer_id,
            heartbeat_count,
            |heartbeat| heartbeat_submessage_sender.send_heartbeat(heartbeat),
        );
    }

    #[test]
    fn reliable_stateful_writer_receive_acknack() {
        let mut reader_proxy = MockReaderProxy::new();
        let acknack = AckNackSubmessage {
            endianness_flag: true,
            final_flag: true,
            reader_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            writer_id: EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            },
            reader_sn_state: SequenceNumberSetSubmessageElement {
                base: 1,
                set: vec![2, 3],
            },
            count: CountSubmessageElement { value: Count(1) },
        };

        reader_proxy
            .expect_acked_changes_set()
            .with(mockall::predicate::eq(0))
            .once()
            .return_const(());
        reader_proxy
            .expect_requested_changes_set()
            .with(mockall::predicate::eq(&[2, 3][..]))
            .once()
            .return_const(());

        ReliableStatefulWriterBehavior::receive_acknack(&mut reader_proxy, &acknack);
    }

    #[test]
    fn reliable_stateful_writer_send_requested_changes_single_data_submessage() {
        let mut seq = mockall::Sequence::new();

        let mut reader_proxy = MockReaderProxy::new();
        let mut writer_cache = MockHistoryCache::new();
        let mut data_message_sender = MockDataMessageSender::new();
        let mut gap_message_sender = MockGapMessageSender::new();

        let reader_id = EntityId::new([1; 3], 1);
        reader_proxy
            .expect_next_requested_change()
            .once()
            .return_const(Some(1))
            .in_sequence(&mut seq);
        writer_cache
            .expect_changes()
            .once()
            .return_const(vec![MockCacheChange {
                kind: ChangeKind::Alive,
                writer_guid: Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)),
                instance_handle: 1,
                sequence_number: 1,
                data_value: vec![1, 2, 3, 4],
                inline_qos: MockParameterList,
            }])
            .in_sequence(&mut seq);
        data_message_sender
            .expect_send_data()
            // Can't use a complete expected DataSubmessage due to issues with the lifetime.
            .withf(|data| {
                data.data_flag == true
                    && data.key_flag == false
                    && data.non_standard_payload_flag == false
                    && data.writer_sn.value == 1
                    && data.inline_qos.parameter.is_empty()
                    && data.serialized_payload.value == &[1, 2, 3, 4]
            })
            .once()
            .return_const(())
            .in_sequence(&mut seq);
        reader_proxy
            .expect_next_requested_change()
            .once()
            .return_const(None)
            .in_sequence(&mut seq);

        ReliableStatefulWriterBehavior::send_requested_changes(
            &mut reader_proxy,
            &writer_cache,
            reader_id,
            |data| data_message_sender.send_data(data),
            |gap| gap_message_sender.send_gap(gap),
        )
    }

    #[test]
    fn reliable_stateless_writer_send_requested_changes_single_gap_submessage() {
        let mut seq = mockall::Sequence::new();

        let mut reader_proxy = MockReaderProxy::new();
        let mut writer_cache = MockHistoryCache::new();
        let mut data_message_sender = MockDataMessageSender::new();
        let mut gap_message_sender = MockGapMessageSender::new();

        let reader_id = EntityId::new([1; 3], 1);

        reader_proxy
            .expect_next_requested_change()
            .once()
            .return_const(Some(1))
            .in_sequence(&mut seq);
        writer_cache
            .expect_changes()
            .once()
            .return_const(vec![])
            .in_sequence(&mut seq);
        gap_message_sender
            .expect_send_gap()
            .with(mockall::predicate::eq(GapSubmessage {
                endianness_flag: true,
                reader_id: EntityIdSubmessageElement { value: reader_id },
                writer_id: EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                },
                gap_start: SequenceNumberSubmessageElement { value: 1 },
                gap_list: SequenceNumberSetSubmessageElement {
                    base: 1,
                    set: vec![],
                },
            }))
            .once()
            .return_const(())
            .in_sequence(&mut seq);
        reader_proxy
            .expect_next_requested_change()
            .once()
            .return_const(None)
            .in_sequence(&mut seq);

        ReliableStatefulWriterBehavior::send_requested_changes(
            &mut reader_proxy,
            &writer_cache,
            reader_id,
            |data| data_message_sender.send_data(data),
            |gap| gap_message_sender.send_gap(gap),
        )
    }
}

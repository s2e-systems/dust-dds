use core::iter::FromIterator;

/// This file implements the behaviors described in 8.4.8 RTPS StatelessWriter Behavior
use crate::{
    messages::{
        submessage_elements::{
            CountSubmessageElement, EntityIdSubmessageElement, ParameterListSubmessageElement,
            SequenceNumberSetSubmessageElement, SequenceNumberSubmessageElement,
            SerializedDataSubmessageElement,
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

use super::writer::reader_locator::RtpsReaderLocatorOperations;

pub struct BestEffortStatelessWriterBehavior;

impl BestEffortStatelessWriterBehavior {
    /// 8.4.8.1.4 Transition T4
    pub fn send_unsent_changes<'a, CacheChange, S, P, D>(
        reader_locator: &mut impl RtpsReaderLocatorOperations<CacheChangeType = CacheChange>,
        writer_cache: &'a impl RtpsHistoryCacheAttributes,
        mut send_data: impl FnMut(DataSubmessage<P, D>),
        mut send_gap: impl FnMut(GapSubmessage<S>),
    ) where
        CacheChange: RtpsCacheChangeAttributes,
        CacheChange: Into<DataSubmessage<P, D>>,
        S: FromIterator<SequenceNumber>,
    {
        while let Some(change) = reader_locator.next_unsent_change() {
            let seq_num = change.sequence_number();
            // if writer_cache
            //     .changes()
            //     .iter()
            //     .any(|cc| cc.sequence_number() == seq_num)
            if true
            {
                let data_submessage = change.into();
                send_data(data_submessage);
            } else {
                let endianness_flag = true;
                let reader_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
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
                send_gap(gap_submessage);
            }
        }
    }
}

/// This struct is a wrapper for the implementation of the behaviors described in 8.4.8.2 Reliable StatelessWriter Behavior
pub struct ReliableStatelessWriterBehavior;

impl ReliableStatelessWriterBehavior {
    /// 8.4.8.2.4 Transition T4
    pub fn send_unsent_changes<'a, CacheChange, S, P, D>(
        reader_locator: &mut impl RtpsReaderLocatorOperations<CacheChangeType = CacheChange>,
        writer_cache: &'a impl RtpsHistoryCacheAttributes,
        mut send_data: impl FnMut(DataSubmessage<P, D>),
        mut send_gap: impl FnMut(GapSubmessage<S>),
    ) where
        // CacheChange: RtpsCacheChangeAttributes + 'a,
        // &'a <CacheChange as RtpsCacheChangeAttributes>::DataType: Into<D>,
        // &'a <CacheChange as RtpsCacheChangeAttributes>::ParameterListType: Into<P>,
        // S: FromIterator<SequenceNumber>,
    {
        // while let Some(seq_num) = reader_locator.next_unsent_change() {
        //     let change = writer_cache
        //         .changes()
        //         .iter()
        //         .filter(|cc| cc.sequence_number() == seq_num)
        //         .next();
        //     if let Some(change) = change {
        //         let endianness_flag = true;
        //         let inline_qos_flag = true;
        //         let (data_flag, key_flag) = match change.kind() {
        //             ChangeKind::Alive => (true, false),
        //             ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => {
        //                 (false, true)
        //             }
        //             _ => todo!(),
        //         };
        //         let non_standard_payload_flag = false;
        //         let reader_id = EntityIdSubmessageElement {
        //             value: ENTITYID_UNKNOWN,
        //         };
        //         let writer_id = EntityIdSubmessageElement {
        //             value: change.writer_guid().entity_id(),
        //         };
        //         let writer_sn = SequenceNumberSubmessageElement {
        //             value: change.sequence_number(),
        //         };
        //         let inline_qos = ParameterListSubmessageElement {
        //             parameter: change.inline_qos().into(),
        //         };
        //         let serialized_payload = SerializedDataSubmessageElement {
        //             value: change.data_value().into(),
        //         };
        //         let data_submessage = DataSubmessage {
        //             endianness_flag,
        //             inline_qos_flag,
        //             data_flag,
        //             key_flag,
        //             non_standard_payload_flag,
        //             reader_id,
        //             writer_id,
        //             writer_sn,
        //             inline_qos,
        //             serialized_payload,
        //         };
        //         send_data(data_submessage);
        //     } else {
        //         let endianness_flag = true;
        //         let reader_id = EntityIdSubmessageElement {
        //             value: ENTITYID_UNKNOWN,
        //         };
        //         let writer_id = EntityIdSubmessageElement {
        //             value: ENTITYID_UNKNOWN,
        //         };
        //         let gap_start = SequenceNumberSubmessageElement { value: seq_num };
        //         let gap_list = SequenceNumberSetSubmessageElement {
        //             base: seq_num,
        //             set: core::iter::empty().collect(),
        //         };
        //         let gap_submessage = GapSubmessage {
        //             endianness_flag,
        //             reader_id,
        //             writer_id,
        //             gap_start,
        //             gap_list,
        //         };
        //         send_gap(gap_submessage);
        //     }
        // }
    }

    /// 8.4.8.2.5 Transition T5
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

    /// 8.4.8.2.5 Transition T6
    /// Implementation does not include the part correponding to searching the reader locator
    /// on the stateless writer
    pub fn receive_acknack<S>(
        reader_locator: &mut impl RtpsReaderLocatorOperations,
        acknack: &AckNackSubmessage<S>,
    ) where
        S: AsRef<[SequenceNumber]>,
    {
        reader_locator.requested_changes_set(acknack.reader_sn_state.set.as_ref());
    }

    /// 8.4.9.2.12 Transition T10
    pub fn send_requested_changes<'a, P, D, CacheChange, S>(
        reader_locator: &mut impl RtpsReaderLocatorOperations,
        writer_cache: &'a impl RtpsHistoryCacheAttributes<CacheChangeType = CacheChange>,
        mut send_data: impl FnMut(DataSubmessage<P, D>),
        mut send_gap: impl FnMut(GapSubmessage<S>),
    ) where
    //     CacheChange: RtpsCacheChangeAttributes + 'a,
    //     &'a <CacheChange as RtpsCacheChangeAttributes>::DataType: Into<D>,
    //     &'a <CacheChange as RtpsCacheChangeAttributes>::ParameterListType: Into<P>,
    //     S: FromIterator<SequenceNumber>,
    {
        // while let Some(seq_num) = reader_locator.next_requested_change() {
        //     let change = writer_cache
        //         .changes()
        //         .iter()
        //         .filter(|cc| cc.sequence_number() == seq_num)
        //         .next();
        //     if let Some(change) = change {
        //         let endianness_flag = true;
        //         let inline_qos_flag = true;
        //         let (data_flag, key_flag) = match change.kind() {
        //             ChangeKind::Alive => (true, false),
        //             ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => {
        //                 (false, true)
        //             }
        //             _ => todo!(),
        //         };
        //         let non_standard_payload_flag = false;
        //         let reader_id = EntityIdSubmessageElement {
        //             value: ENTITYID_UNKNOWN,
        //         };
        //         let writer_id = EntityIdSubmessageElement {
        //             value: change.writer_guid().entity_id(),
        //         };
        //         let writer_sn = SequenceNumberSubmessageElement {
        //             value: change.sequence_number(),
        //         };
        //         let inline_qos = ParameterListSubmessageElement {
        //             parameter: change.inline_qos().into(),
        //         };
        //         let serialized_payload = SerializedDataSubmessageElement {
        //             value: change.data_value().into(),
        //         };
        //         let data_submessage = DataSubmessage {
        //             endianness_flag,
        //             inline_qos_flag,
        //             data_flag,
        //             key_flag,
        //             non_standard_payload_flag,
        //             reader_id,
        //             writer_id,
        //             writer_sn,
        //             inline_qos,
        //             serialized_payload,
        //         };
        //         send_data(data_submessage)
        //     } else {
        //         let endianness_flag = true;
        //         let reader_id = EntityIdSubmessageElement {
        //             value: ENTITYID_UNKNOWN,
        //         };
        //         let writer_id = EntityIdSubmessageElement {
        //             value: ENTITYID_UNKNOWN,
        //         };
        //         let gap_start = SequenceNumberSubmessageElement { value: seq_num };
        //         let gap_list = SequenceNumberSetSubmessageElement {
        //             base: seq_num,
        //             set: core::iter::empty().collect(),
        //         };
        //         let gap_submessage = GapSubmessage {
        //             endianness_flag,
        //             reader_id,
        //             writer_id,
        //             gap_start,
        //             gap_list,
        //         };
        //         send_gap(gap_submessage)
        //     }
        // }
    }
}

#[cfg(test)]
mod tests {

    use mockall::mock;

    use crate::structure::types::{Guid, GuidPrefix, InstanceHandle};

    use super::*;

    struct MockData;
    impl From<&MockData> for () {
        fn from(_: &MockData) -> Self {
            ()
        }
    }

    struct MockParameterList;
    impl From<&MockParameterList> for () {
        fn from(_: &MockParameterList) -> Self {
            ()
        }
    }

    struct MockCacheChange {
        kind: ChangeKind,
        writer_guid: Guid,
        instance_handle: InstanceHandle,
        sequence_number: SequenceNumber,
        data_value: MockData,
        inline_qos: MockParameterList,
    }

    impl RtpsCacheChangeAttributes for MockCacheChange {
        type DataType = MockData;
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
            &self.data_value
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
        ReaderLocator{}

        impl RtpsReaderLocatorOperations for ReaderLocator {
            type CacheChangeType = i64;
            type CacheChangeListType = Vec<i64>;

            fn next_requested_change(&mut self) -> Option<i64>;
            fn next_unsent_change(&mut self) -> Option<i64>;
            fn requested_changes(&self) -> Vec<i64>;
            fn requested_changes_set(&mut self, req_seq_num_set: &[SequenceNumber]);
            fn unsent_changes(&self) -> Vec<i64>;
        }
    }

    mock! {
        DataMessageSender<'a>{
            fn send_data(&mut self, data: DataSubmessage<(), ()> );
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

    // #[test]
    // fn best_effort_stateless_writer_send_unsent_changes_single_data_submessage() {
    //     let mut seq = mockall::Sequence::new();

    //     let mut reader_locator = MockReaderLocator::new();
    //     let mut writer_cache = MockHistoryCache::new();
    //     let mut data_message_sender = MockDataMessageSender::new();
    //     let mut gap_message_sender = MockGapMessageSender::new();

    //     reader_locator
    //         .expect_next_unsent_change()
    //         .once()
    //         .return_const(Some(1))
    //         .in_sequence(&mut seq);
    //     writer_cache
    //         .expect_changes()
    //         .once()
    //         .return_const(vec![MockCacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)),
    //             instance_handle: 1,
    //             sequence_number: 1,
    //             data_value: MockData,
    //             inline_qos: MockParameterList,
    //         }])
    //         .in_sequence(&mut seq);
    //     data_message_sender
    //         .expect_send_data()
    //         // Can't use a complete expected DataSubmessage due to issues with the lifetime.
    //         .withf(|data| {
    //             data.data_flag == true
    //                 && data.key_flag == false
    //                 && data.non_standard_payload_flag == false
    //                 && data.writer_sn.value == 1
    //         })
    //         .once()
    //         .return_const(())
    //         .in_sequence(&mut seq);
    //     reader_locator
    //         .expect_next_unsent_change()
    //         .once()
    //         .return_const(None)
    //         .in_sequence(&mut seq);

    //     BestEffortStatelessWriterBehavior::send_unsent_changes(
    //         &mut reader_locator,
    //         &writer_cache,
    //         |data| data_message_sender.send_data(data),
    //         |gap| gap_message_sender.send_gap(gap),
    //     )
    // }

    // #[test]
    // fn best_effort_stateless_writer_send_unsent_changes_single_gap_submessage() {
    //     let mut seq = mockall::Sequence::new();

    //     let mut reader_locator = MockReaderLocator::new();
    //     let mut writer_cache = MockHistoryCache::new();
    //     let mut data_message_sender = MockDataMessageSender::new();
    //     let mut gap_message_sender = MockGapMessageSender::new();

    //     reader_locator
    //         .expect_next_unsent_change()
    //         .once()
    //         .return_const(Some(1))
    //         .in_sequence(&mut seq);
    //     writer_cache
    //         .expect_changes()
    //         .once()
    //         .return_const(vec![])
    //         .in_sequence(&mut seq);
    //     gap_message_sender
    //         .expect_send_gap()
    //         .with(mockall::predicate::eq(GapSubmessage {
    //             endianness_flag: true,
    //             reader_id: EntityIdSubmessageElement {
    //                 value: ENTITYID_UNKNOWN,
    //             },
    //             writer_id: EntityIdSubmessageElement {
    //                 value: ENTITYID_UNKNOWN,
    //             },
    //             gap_start: SequenceNumberSubmessageElement { value: 1 },
    //             gap_list: SequenceNumberSetSubmessageElement {
    //                 base: 1,
    //                 set: vec![],
    //             },
    //         }))
    //         .once()
    //         .return_const(())
    //         .in_sequence(&mut seq);
    //     reader_locator
    //         .expect_next_unsent_change()
    //         .once()
    //         .return_const(None)
    //         .in_sequence(&mut seq);

    //     BestEffortStatelessWriterBehavior::send_unsent_changes(
    //         &mut reader_locator,
    //         &writer_cache,
    //         |data| data_message_sender.send_data(data),
    //         |gap| gap_message_sender.send_gap(gap),
    //     )
    // }

    // #[test]
    // fn reliable_stateless_writer_send_unsent_changes_single_data_submessage() {
    //     let mut seq = mockall::Sequence::new();

    //     let mut reader_locator = MockReaderLocator::new();
    //     let mut writer_cache = MockHistoryCache::new();
    //     let mut data_message_sender = MockDataMessageSender::new();
    //     let mut gap_message_sender = MockGapMessageSender::new();

    //     reader_locator
    //         .expect_next_unsent_change()
    //         .once()
    //         .return_const(Some(1))
    //         .in_sequence(&mut seq);
    //     writer_cache
    //         .expect_changes()
    //         .once()
    //         .return_const(vec![MockCacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)),
    //             instance_handle: 1,
    //             sequence_number: 1,
    //             data_value: MockData,
    //             inline_qos: MockParameterList,
    //         }])
    //         .in_sequence(&mut seq);
    //     data_message_sender
    //         .expect_send_data()
    //         // Can't use a complete expected DataSubmessage due to issues with the lifetime.
    //         .withf(|data| {
    //             data.data_flag == true
    //                 && data.key_flag == false
    //                 && data.non_standard_payload_flag == false
    //                 && data.writer_sn.value == 1
    //         })
    //         .once()
    //         .return_const(())
    //         .in_sequence(&mut seq);
    //     reader_locator
    //         .expect_next_unsent_change()
    //         .once()
    //         .return_const(None)
    //         .in_sequence(&mut seq);

    //     ReliableStatelessWriterBehavior::send_unsent_changes(
    //         &mut reader_locator,
    //         &writer_cache,
    //         |data| data_message_sender.send_data(data),
    //         |gap| gap_message_sender.send_gap(gap),
    //     )
    // }

    // #[test]
    // fn reliable_stateless_writer_send_unsent_changes_single_gap_submessage() {
    //     let mut seq = mockall::Sequence::new();

    //     let mut reader_locator = MockReaderLocator::new();
    //     let mut writer_cache = MockHistoryCache::new();
    //     let mut data_message_sender = MockDataMessageSender::new();
    //     let mut gap_message_sender = MockGapMessageSender::new();

    //     reader_locator
    //         .expect_next_unsent_change()
    //         .once()
    //         .return_const(Some(1))
    //         .in_sequence(&mut seq);
    //     writer_cache
    //         .expect_changes()
    //         .once()
    //         .return_const(vec![])
    //         .in_sequence(&mut seq);
    //     gap_message_sender
    //         .expect_send_gap()
    //         .with(mockall::predicate::eq(GapSubmessage {
    //             endianness_flag: true,
    //             reader_id: EntityIdSubmessageElement {
    //                 value: ENTITYID_UNKNOWN,
    //             },
    //             writer_id: EntityIdSubmessageElement {
    //                 value: ENTITYID_UNKNOWN,
    //             },
    //             gap_start: SequenceNumberSubmessageElement { value: 1 },
    //             gap_list: SequenceNumberSetSubmessageElement {
    //                 base: 1,
    //                 set: vec![],
    //             },
    //         }))
    //         .once()
    //         .return_const(())
    //         .in_sequence(&mut seq);
    //     reader_locator
    //         .expect_next_unsent_change()
    //         .once()
    //         .return_const(None)
    //         .in_sequence(&mut seq);

    //     ReliableStatelessWriterBehavior::send_unsent_changes(
    //         &mut reader_locator,
    //         &writer_cache,
    //         |data| data_message_sender.send_data(data),
    //         |gap| gap_message_sender.send_gap(gap),
    //     )
    // }

    // #[test]
    // fn reliable_stateless_writer_send_heartbeat() {
    //     let mut writer_cache = MockHistoryCache::new();
    //     let mut heartbeat_submessage_sender = MockHeartbeatMessageSender::new();
    //     let writer_id = EntityId::new([1; 3], 1);
    //     let heartbeat_count = Count(1);

    //     writer_cache
    //         .expect_get_seq_num_min_()
    //         .once()
    //         .return_const(Some(1));
    //     writer_cache
    //         .expect_get_seq_num_max_()
    //         .once()
    //         .return_const(Some(4));

    //     heartbeat_submessage_sender
    //         .expect_send_heartbeat()
    //         .once()
    //         .with(mockall::predicate::eq(HeartbeatSubmessage {
    //             endianness_flag: true,
    //             final_flag: false,
    //             liveliness_flag: false,
    //             reader_id: EntityIdSubmessageElement {
    //                 value: ENTITYID_UNKNOWN,
    //             },
    //             writer_id: EntityIdSubmessageElement { value: writer_id },
    //             first_sn: SequenceNumberSubmessageElement { value: 1 },
    //             last_sn: SequenceNumberSubmessageElement { value: 4 },
    //             count: CountSubmessageElement {
    //                 value: heartbeat_count,
    //             },
    //         }))
    //         .return_const(());

    //     ReliableStatelessWriterBehavior::send_heartbeat(
    //         &mut writer_cache,
    //         writer_id,
    //         heartbeat_count,
    //         |heartbeat| heartbeat_submessage_sender.send_heartbeat(heartbeat),
    //     );
    // }

    // #[test]
    // fn reliable_stateless_writer_receive_acknack() {
    //     let mut reader_locator = MockReaderLocator::new();
    //     let acknack = AckNackSubmessage {
    //         endianness_flag: true,
    //         final_flag: true,
    //         reader_id: EntityIdSubmessageElement {
    //             value: ENTITYID_UNKNOWN,
    //         },
    //         writer_id: EntityIdSubmessageElement {
    //             value: ENTITYID_UNKNOWN,
    //         },
    //         reader_sn_state: SequenceNumberSetSubmessageElement {
    //             base: 1,
    //             set: vec![2, 3],
    //         },
    //         count: CountSubmessageElement { value: Count(1) },
    //     };

    //     reader_locator
    //         .expect_requested_changes_set()
    //         .with(mockall::predicate::eq(&[2, 3][..]))
    //         .once()
    //         .return_const(());

    //     ReliableStatelessWriterBehavior::receive_acknack(&mut reader_locator, &acknack);
    // }

    // #[test]
    // fn reliable_stateless_writer_send_requested_changes_single_data_submessage() {
    //     let mut seq = mockall::Sequence::new();

    //     let mut reader_locator = MockReaderLocator::new();
    //     let mut writer_cache = MockHistoryCache::new();
    //     let mut data_message_sender = MockDataMessageSender::new();
    //     let mut gap_message_sender = MockGapMessageSender::new();

    //     reader_locator
    //         .expect_next_requested_change()
    //         .once()
    //         .return_const(Some(1))
    //         .in_sequence(&mut seq);
    //     writer_cache
    //         .expect_changes()
    //         .once()
    //         .return_const(vec![MockCacheChange {
    //             kind: ChangeKind::Alive,
    //             writer_guid: Guid::new(GuidPrefix([1; 12]), EntityId::new([1; 3], 1)),
    //             instance_handle: 1,
    //             sequence_number: 1,
    //             data_value: MockData,
    //             inline_qos: MockParameterList,
    //         }])
    //         .in_sequence(&mut seq);
    //     data_message_sender
    //         .expect_send_data()
    //         // Can't use a complete expected DataSubmessage due to issues with the lifetime.
    //         .withf(|data| {
    //             data.data_flag == true
    //                 && data.key_flag == false
    //                 && data.non_standard_payload_flag == false
    //                 && data.writer_sn.value == 1
    //         })
    //         .once()
    //         .return_const(())
    //         .in_sequence(&mut seq);
    //     reader_locator
    //         .expect_next_requested_change()
    //         .once()
    //         .return_const(None)
    //         .in_sequence(&mut seq);

    //     ReliableStatelessWriterBehavior::send_requested_changes(
    //         &mut reader_locator,
    //         &writer_cache,
    //         |data| data_message_sender.send_data(data),
    //         |gap| gap_message_sender.send_gap(gap),
    //     )
    // }

    // #[test]
    // fn reliable_stateless_writer_send_requested_changes_single_gap_submessage() {
    //     let mut seq = mockall::Sequence::new();

    //     let mut reader_locator = MockReaderLocator::new();
    //     let mut writer_cache = MockHistoryCache::new();
    //     let mut data_message_sender = MockDataMessageSender::new();
    //     let mut gap_message_sender = MockGapMessageSender::new();

    //     reader_locator
    //         .expect_next_requested_change()
    //         .once()
    //         .return_const(Some(1))
    //         .in_sequence(&mut seq);
    //     writer_cache
    //         .expect_changes()
    //         .once()
    //         .return_const(vec![])
    //         .in_sequence(&mut seq);
    //     gap_message_sender
    //         .expect_send_gap()
    //         .with(mockall::predicate::eq(GapSubmessage {
    //             endianness_flag: true,
    //             reader_id: EntityIdSubmessageElement {
    //                 value: ENTITYID_UNKNOWN,
    //             },
    //             writer_id: EntityIdSubmessageElement {
    //                 value: ENTITYID_UNKNOWN,
    //             },
    //             gap_start: SequenceNumberSubmessageElement { value: 1 },
    //             gap_list: SequenceNumberSetSubmessageElement {
    //                 base: 1,
    //                 set: vec![],
    //             },
    //         }))
    //         .once()
    //         .return_const(())
    //         .in_sequence(&mut seq);
    //     reader_locator
    //         .expect_next_requested_change()
    //         .once()
    //         .return_const(None)
    //         .in_sequence(&mut seq);

    //     ReliableStatelessWriterBehavior::send_requested_changes(
    //         &mut reader_locator,
    //         &writer_cache,
    //         |data| data_message_sender.send_data(data),
    //         |gap| gap_message_sender.send_gap(gap),
    //     )
    // }
}

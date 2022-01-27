/// This file implements the behaviors described in 8.4.8 RTPS StatelessWriter Behavior
use crate::{
    messages::{
        submessage_elements::{
            CountSubmessageElementConstructor, EntityIdSubmessageElementConstructor,
            SequenceNumberSetSubmessageElementAttributes,
            SequenceNumberSetSubmessageElementConstructor,
            SequenceNumberSubmessageElementConstructor,
        },
        submessages::{
            AckNackSubmessageAttributes, DataSubmessageConstructor, GapSubmessageConstructor,
            HeartbeatSubmessageConstructor,
        },
        types::Count,
    },
    structure::{
        cache_change::RtpsCacheChangeAttributes,
        history_cache::{RtpsHistoryAttributes, RtpsHistoryCacheOperations},
        types::{ChangeKind, EntityId, Guid, SequenceNumber, ENTITYID_UNKNOWN},
    },
};

use super::writer::reader_locator::RtpsReaderLocatorOperations;

pub enum StatelessWriterBehavior<'a, R, C> {
    BestEffort(BestEffortStatelessWriterBehavior<'a, R, C>),
    Reliable(ReliableStatelessWriterBehavior<'a, R, C>),
}

/// This struct is a wrapper for the implementation of the behaviors described in 8.4.8.1 Best-Effort StatelessWriter Behavior
pub struct BestEffortStatelessWriterBehavior<'a, R, C> {
    pub reader_locator: &'a mut R,
    pub writer_cache: &'a C,
    pub last_change_sequence_number: &'a SequenceNumber,
}

impl<'a, R, C> BestEffortStatelessWriterBehavior<'a, R, C> {
    /// Implement 8.4.8.1.4 Transition T4
    pub fn send_unsent_changes<
        Data,
        CacheChange,
        EntityIdElement,
        SequenceNumberElement,
        Gap,
        SequenceNumberSetElement,
    >(
        &mut self,
        mut send_data: impl FnMut(Data),
        mut send_gap: impl FnMut(Gap),
    ) where
        R: RtpsReaderLocatorOperations<CacheChangeType = SequenceNumber, HistoryCacheType = C>,
        Data: DataSubmessageConstructor<
            EntityIdSubmessageElementType = EntityIdElement,
            SequenceNumberSubmessageElementType = SequenceNumberElement,
            ParameterListSubmessageElementType = &'a CacheChange::ParameterListType,
            SerializedDataSubmessageElementType = &'a CacheChange::DataType,
        >,
        C: RtpsHistoryAttributes<CacheChangeType = CacheChange>,
        CacheChange: RtpsCacheChangeAttributes + 'a,
        EntityIdElement: EntityIdSubmessageElementConstructor<EntityIdType = EntityId>,
        SequenceNumberElement:
            SequenceNumberSubmessageElementConstructor<SequenceNumberType = SequenceNumber>,
        SequenceNumberSetElement: SequenceNumberSetSubmessageElementConstructor<
            SequenceNumberType = SequenceNumber,
            SequenceNumberSetType = [SequenceNumber],
        >,
        Gap: GapSubmessageConstructor<
            EntityIdSubmessageElementType = EntityIdElement,
            SequenceNumberSubmessageElementType = SequenceNumberElement,
            SequenceNumberSetSubmessageElementType = SequenceNumberSetElement,
        >,
    {
        while let Some(seq_num) = self
            .reader_locator
            .next_unsent_change(self.writer_cache)
        {
            let change = self
                .writer_cache
                .changes()
                .iter()
                .filter(|cc| cc.sequence_number() == &seq_num)
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
                let reader_id = EntityIdElement::new(&ENTITYID_UNKNOWN);
                let writer_id = EntityIdElement::new(change.writer_guid().entity_id());
                let writer_sn = SequenceNumberElement::new(change.sequence_number());
                let inline_qos = change.inline_qos();
                let serialized_payload = change.data_value();
                let data_submessage = Data::new(
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
                );
                send_data(data_submessage);
            } else {
                let endianness_flag = true;
                let reader_id = EntityIdElement::new(&ENTITYID_UNKNOWN);
                let writer_id = EntityIdElement::new(&ENTITYID_UNKNOWN);
                let gap_start = SequenceNumberElement::new(&seq_num);
                let gap_list = SequenceNumberSetElement::new(&seq_num, &[]);
                let gap_submessage =
                    Gap::new(endianness_flag, reader_id, writer_id, gap_start, gap_list);
                send_gap(gap_submessage);
            }
        }
    }
}

/// This struct is a wrapper for the implementation of the behaviors described in 8.4.8.2 Reliable StatelessWriter Behavior
pub struct ReliableStatelessWriterBehavior<'a, R, C> {
    pub reader_locator: &'a mut R,
    pub writer_cache: &'a C,
    pub last_change_sequence_number: &'a SequenceNumber,
    pub writer_guid: &'a Guid,
}

impl<'a, R, C> ReliableStatelessWriterBehavior<'a, R, C> {
    /// Implement 8.4.8.2.4 Transition T4
    pub fn send_unsent_changes<
        Data,
        EntityIdElement,
        SequenceNumberElement,
        CacheChange,
        Gap,
        SequenceNumberSetElement,
    >(
        &mut self,
        mut send_data: impl FnMut(Data),
        mut send_gap: impl FnMut(Gap),
    ) where
        R: RtpsReaderLocatorOperations<CacheChangeType = SequenceNumber, HistoryCacheType = C>,
        C: RtpsHistoryAttributes<CacheChangeType = CacheChange>,
        Data: DataSubmessageConstructor<
            EntityIdSubmessageElementType = EntityIdElement,
            SequenceNumberSubmessageElementType = SequenceNumberElement,
            ParameterListSubmessageElementType = &'a CacheChange::ParameterListType,
            SerializedDataSubmessageElementType = &'a CacheChange::DataType,
        >,
        EntityIdElement: EntityIdSubmessageElementConstructor<EntityIdType = EntityId>,
        CacheChange: RtpsCacheChangeAttributes + 'a,
        SequenceNumberElement:
            SequenceNumberSubmessageElementConstructor<SequenceNumberType = SequenceNumber>,
        SequenceNumberSetElement: SequenceNumberSetSubmessageElementConstructor<
            SequenceNumberType = SequenceNumber,
            SequenceNumberSetType = [SequenceNumber],
        >,
        Gap: GapSubmessageConstructor<
            EntityIdSubmessageElementType = EntityIdElement,
            SequenceNumberSubmessageElementType = SequenceNumberElement,
            SequenceNumberSetSubmessageElementType = SequenceNumberSetElement,
        >,
    {
        while let Some(seq_num) = self
            .reader_locator
            .next_unsent_change(self.writer_cache)
        {
            let change = self
                .writer_cache
                .changes()
                .iter()
                .filter(|cc| cc.sequence_number() == &seq_num)
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
                let reader_id = EntityIdSubmessageElementConstructor::new(&ENTITYID_UNKNOWN);
                let writer_id =
                    EntityIdSubmessageElementConstructor::new(change.writer_guid().entity_id());
                let writer_sn = SequenceNumberElement::new(change.sequence_number());
                let inline_qos = change.inline_qos();
                let serialized_payload = change.data_value();
                let data_submessage = Data::new(
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
                );
                send_data(data_submessage);
            } else {
                let endianness_flag = true;
                let reader_id = EntityIdElement::new(&ENTITYID_UNKNOWN);
                let writer_id = EntityIdElement::new(&ENTITYID_UNKNOWN);
                let gap_start = SequenceNumberElement::new(&seq_num);
                let gap_list = SequenceNumberSetElement::new(&seq_num, &[]);
                let gap_submessage =
                    Gap::new(endianness_flag, reader_id, writer_id, gap_start, gap_list);
                send_gap(gap_submessage);
            }
        }
    }

    /// Implement 8.4.8.2.5 Transition T5
    pub fn send_heartbeat<Heartbeat, EntityIdElement, CountElement>(
        &mut self,
        heartbeat_count: Count,
        mut send_heartbeat: impl FnMut(Heartbeat),
    ) where
        C: RtpsHistoryCacheOperations,
        Heartbeat: HeartbeatSubmessageConstructor<
            EntityIdSubmessageElementType = EntityIdElement,
            SequenceNumberSubmessageElementType = SequenceNumber,
            CountSubmessageElementType = CountElement,
        >,
        EntityIdElement: EntityIdSubmessageElementConstructor<EntityIdType = EntityId>,
        CountElement: CountSubmessageElementConstructor<CountType = Count>,
    {
        let endianness_flag = true;
        let final_flag = false;
        let liveliness_flag = false;
        let reader_id = EntityIdElement::new(&ENTITYID_UNKNOWN);
        let writer_id = EntityIdElement::new(&self.writer_guid.entity_id);
        let first_sn = self.writer_cache.get_seq_num_min().unwrap_or(0);
        let last_sn = self.writer_cache.get_seq_num_min().unwrap_or(0);
        let count = CountElement::new(&heartbeat_count);
        let heartbeat_submessage = Heartbeat::new(
            endianness_flag,
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        );
        send_heartbeat(heartbeat_submessage)
    }

    /// Implement 8.4.8.2.5 Transition T6
    /// Implementation does not include the part correponding to searching the reader locator
    /// on the stateless writer
    pub fn process_acknack<S>(
        &mut self,
        acknack: &impl AckNackSubmessageAttributes<
            SequenceNumberSetSubmessageElementType = impl SequenceNumberSetSubmessageElementAttributes<
                SequenceNumberType = SequenceNumber,
                SequenceNumberSetType = [SequenceNumber],
            >,
        >,
    ) where
        R: RtpsReaderLocatorOperations<CacheChangeType = SequenceNumber>,
        S: AsRef<[SequenceNumber]>,
    {
        self.reader_locator
            .requested_changes_set(acknack.reader_sn_state().set());
    }

    /// Implement 8.4.9.2.12 Transition T10
    pub fn send_requested_changes<
        P,
        Data,
        EntityIdElement,
        SequenceNumberElement,
        CacheChange,
        Gap,
        SequenceNumberSetElement,
    >(
        &mut self,
        mut send_data: impl FnMut(Data),
        mut send_gap: impl FnMut(Gap),
    ) where
        R: RtpsReaderLocatorOperations<CacheChangeType = SequenceNumber>,
        C: RtpsHistoryAttributes<CacheChangeType = CacheChange>,
        Data: DataSubmessageConstructor<
            EntityIdSubmessageElementType = EntityIdElement,
            SequenceNumberSubmessageElementType = SequenceNumberElement,
            ParameterListSubmessageElementType = &'a CacheChange::ParameterListType,
            SerializedDataSubmessageElementType = &'a CacheChange::DataType,
        >,
        EntityIdElement: EntityIdSubmessageElementConstructor<EntityIdType = EntityId>,
        CacheChange: RtpsCacheChangeAttributes + 'a,
        SequenceNumberElement:
            SequenceNumberSubmessageElementConstructor<SequenceNumberType = SequenceNumber>,
        SequenceNumberSetElement: SequenceNumberSetSubmessageElementConstructor<
            SequenceNumberType = SequenceNumber,
            SequenceNumberSetType = [SequenceNumber],
        >,
        Gap: GapSubmessageConstructor<
            EntityIdSubmessageElementType = EntityIdElement,
            SequenceNumberSubmessageElementType = SequenceNumberElement,
            SequenceNumberSetSubmessageElementType = SequenceNumberSetElement,
        >,
    {
        while let Some(seq_num) = self.reader_locator.next_requested_change() {
            let change = self
                .writer_cache
                .changes()
                .iter()
                .filter(|cc| cc.sequence_number() == &seq_num)
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
                let reader_id = EntityIdElement::new(&ENTITYID_UNKNOWN);
                let writer_id = EntityIdElement::new(change.writer_guid().entity_id());
                let writer_sn = SequenceNumberElement::new(change.sequence_number());
                let inline_qos = change.inline_qos();
                let serialized_payload = change.data_value();
                let data_submessage = Data::new(
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
                );
                send_data(data_submessage)
            } else {
                let endianness_flag = true;
                let reader_id = EntityIdElement::new(&ENTITYID_UNKNOWN);
                let writer_id = EntityIdElement::new(&ENTITYID_UNKNOWN);
                let gap_start = SequenceNumberElement::new(&seq_num);
                let gap_list = SequenceNumberSetElement::new(&seq_num, &[]);
                let gap_submessage =
                    Gap::new(endianness_flag, reader_id, writer_id, gap_start, gap_list);
                send_gap(gap_submessage)
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::structure::types::{InstanceHandle, GUID_UNKNOWN};

//     use super::*;
//     struct MockWriterCache {
//         change: MockCacheChange,
//     }

//     struct MockReaderLocatorOperations(Option<i64>);

//     impl RtpsReaderLocatorOperations for MockReaderLocatorOperations {
//         type CacheChangeType = i64;
//         type HistoryCacheType = MockWriterCache;
//         fn next_requested_change(&mut self) -> Option<Self::CacheChangeType> {
//             todo!()
//         }
//         fn next_unsent_change(
//             &mut self,
//             _last_change: &Self::HistoryCacheType,
//         ) -> Option<Self::CacheChangeType> {
//             self.0.take()
//         }
//         fn requested_changes_set(&mut self, _req_seq_num_set: &[Self::CacheChangeType]) {
//             todo!()
//         }

//     }

//     struct MockEntityIdSubmessageElement;

//     impl EntityIdSubmessageElementConstructor for MockEntityIdSubmessageElement {
//         type EntityIdType = EntityId;

//         fn new(_value: &Self::EntityIdType) -> Self {
//             Self
//         }
//     }

//     struct MockSequenceNumberSubmessageElement;

//     impl SequenceNumberSubmessageElementConstructor for MockSequenceNumberSubmessageElement {
//         type SequenceNumberType = SequenceNumber;

//         fn new(_value: &Self::SequenceNumberType) -> Self {
//             Self
//         }
//     }

//     struct MockSequenceNumberSetSubmessageElement;

//     impl SequenceNumberSetSubmessageElementConstructor for MockSequenceNumberSetSubmessageElement {
//         type SequenceNumberType = SequenceNumber;
//         type SequenceNumberSetType = [SequenceNumber];
//         fn new(_base: &Self::SequenceNumberType, _set: &Self::SequenceNumberSetType) -> Self {
//             Self
//         }
//     }

//     struct MockDataSubmessage<'a>(&'a ());

//     impl<'a> DataSubmessageConstructor for MockDataSubmessage<'a> {
//         type EntityIdSubmessageElementType = MockEntityIdSubmessageElement;
//         type SequenceNumberSubmessageElementType = MockSequenceNumberSubmessageElement;
//         type ParameterListSubmessageElementType = &'a ();
//         type SerializedDataSubmessageElementType = &'a ();

//         fn new(
//             _endianness_flag: crate::messages::types::SubmessageFlag,
//             _inline_qos_flag: crate::messages::types::SubmessageFlag,
//             _data_flag: crate::messages::types::SubmessageFlag,
//             _key_flag: crate::messages::types::SubmessageFlag,
//             _non_standard_payload_flag: crate::messages::types::SubmessageFlag,
//             _reader_id: Self::EntityIdSubmessageElementType,
//             _writer_id: Self::EntityIdSubmessageElementType,
//             _writer_sn: Self::SequenceNumberSubmessageElementType,
//             _inline_qos: Self::ParameterListSubmessageElementType,
//             _serialized_payload: Self::SerializedDataSubmessageElementType,
//         ) -> Self {
//             Self(&())
//         }
//     }
//     struct MockCacheChange {
//         kind: ChangeKind,
//         writer_guid: Guid,
//         sequence_number: SequenceNumber,
//     }

//     impl RtpsCacheChangeAttributes for MockCacheChange {
//         type DataType = ();
//         type ParameterListType = ();

//         fn kind(&self) -> &ChangeKind {
//             &self.kind
//         }

//         fn writer_guid(&self) -> &Guid {
//             &self.writer_guid
//         }

//         fn instance_handle(&self) -> &InstanceHandle {
//             todo!()
//         }

//         fn sequence_number(&self) -> &SequenceNumber {
//             &self.sequence_number
//         }

//         fn data_value(&self) -> &Self::DataType {
//             &()
//         }

//         fn inline_qos(&self) -> &Self::ParameterListType {
//             &()
//         }
//     }

//     struct MockGapSubmessage;

//     impl GapSubmessageConstructor for MockGapSubmessage {
//         type EntityIdSubmessageElementType = MockEntityIdSubmessageElement;

//         type SequenceNumberSubmessageElementType = MockSequenceNumberSubmessageElement;

//         type SequenceNumberSetSubmessageElementType = MockSequenceNumberSetSubmessageElement;

//         fn new(
//             _endianness_flag: crate::messages::types::SubmessageFlag,
//             _reader_id: Self::EntityIdSubmessageElementType,
//             _writer_id: Self::EntityIdSubmessageElementType,
//             _gap_start: Self::SequenceNumberSubmessageElementType,
//             _gap_list: Self::SequenceNumberSetSubmessageElementType,
//         ) -> Self {
//             Self
//         }
//     }

//     #[test]
//     fn best_effort_stateless_writer_send_data() {

//         impl RtpsHistoryAttributes for MockWriterCache {
//             type CacheChangeType = MockCacheChange;

//             fn changes(&self) -> &[Self::CacheChangeType] {
//                 core::slice::from_ref(&self.change)
//             }
//         }
//         let writer_cache = MockWriterCache {
//             change: MockCacheChange {
//                 kind: ChangeKind::Alive,
//                 writer_guid: GUID_UNKNOWN,
//                 sequence_number: 1,
//             },
//         };
//         let mut best_effort_behavior = BestEffortStatelessWriterBehavior {
//             reader_locator: &mut MockReaderLocatorOperations(Some(1)),
//             writer_cache: &writer_cache,
//             last_change_sequence_number: &0,
//         };
//         let mut data_messages = None;
//         best_effort_behavior.send_unsent_changes(
//             |data: MockDataSubmessage| data_messages = Some(data),
//             |_: MockGapSubmessage| assert!(false),
//         );

//         assert!(data_messages.is_some())
//     }

//     #[test]
//     fn best_effort_stateless_writer_send_gap() {
//         struct MockWriterCache;

//         impl RtpsHistoryAttributes for MockWriterCache {
//             type CacheChangeType = MockCacheChange;

//             fn changes(&self) -> &[Self::CacheChangeType] {
//                 &[]
//             }
//         }

//         let mut best_effort_behavior = BestEffortStatelessWriterBehavior {
//             reader_locator: &mut MockReaderLocatorOperations(Some(1)),
//             writer_cache: &MockWriterCache,
//             last_change_sequence_number: &0,
//         };
//         let mut gap_message = None;
//         best_effort_behavior.send_unsent_changes(
//             |_: MockDataSubmessage| assert!(false),
//             |gap: MockGapSubmessage| gap_message = Some(gap),
//         );

//         assert!(gap_message.is_some());
//     }

//     #[test]
//     fn best_effort_stateless_writer_do_nothing() {
//         struct MockWriterCache;

//         impl RtpsHistoryAttributes for MockWriterCache {
//             type CacheChangeType = MockCacheChange;

//             fn changes(&self) -> &[Self::CacheChangeType] {
//                 &[]
//             }
//         }

//         let mut best_effort_behavior = BestEffortStatelessWriterBehavior {
//             reader_locator: &mut MockReaderLocatorOperations(None),
//             writer_cache: &MockWriterCache,
//             last_change_sequence_number: &0,
//         };
//         best_effort_behavior.send_unsent_changes(
//             |_: MockDataSubmessage| assert!(false),
//             |_: MockGapSubmessage| assert!(false),
//         );
//     }
// }

/// This file implements the behaviors described in 8.4.8 RTPS StatelessWriter Behavior
use core::iter::FromIterator;

use crate::{
    messages::{
        submessage_elements::{
            CountSubmessageElement, EntityIdSubmessageElement,
            EntityIdSubmessageElementConstructor, SequenceNumberSetSubmessageElement,
            SequenceNumberSubmessageElement,
        },
        submessages::{
            AckNackSubmessage, DataSubmessageConstructor, GapSubmessage, HeartbeatSubmessage,
        },
        types::Count,
    },
    structure::{
        cache_change::RtpsCacheChangeAttributes,
        history_cache::{RtpsHistoryCacheGetChange, RtpsHistoryCacheOperations},
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
    pub fn send_unsent_changes<Data, EntityIdElement, CacheChange, S>(
        &mut self,
        mut send_data: impl FnMut(Data),
        mut send_gap: impl FnMut(GapSubmessage<S>),
    ) where
        R: RtpsReaderLocatorOperations,
        Data: DataSubmessageConstructor<
            EntityIdSubmessageElementType = EntityIdElement,
            SequenceNumberSubmessageElementType = SequenceNumber,
            ParameterListSubmessageElementType = &'a CacheChange::ParameterListType,
            SerializedDataSubmessageElementType = &'a CacheChange::DataType,
        >,
        C: RtpsHistoryCacheGetChange<CacheChangeType = CacheChange>,
        CacheChange: RtpsCacheChangeAttributes + 'a,
        S: FromIterator<SequenceNumber>,
        EntityIdElement: EntityIdSubmessageElementConstructor<EntityIdType = EntityId>,
    {
        while let Some(seq_num) = self
            .reader_locator
            .next_unsent_change(self.last_change_sequence_number)
        {
            if let Some(change) = self.writer_cache.get_change(&seq_num) {
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
                let writer_sn = *change.sequence_number();
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
                let reader_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let writer_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let gap_start = SequenceNumberSubmessageElement { value: seq_num };
                let set = core::iter::empty().collect();
                let gap_list = SequenceNumberSetSubmessageElement { base: seq_num, set };
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

/// This struct is a wrapper for the implementation of the behaviors described in 8.4.8.2 Reliable StatelessWriter Behavior
pub struct ReliableStatelessWriterBehavior<'a, R, C> {
    pub reader_locator: &'a mut R,
    pub writer_cache: &'a C,
    pub last_change_sequence_number: &'a SequenceNumber,
    pub writer_guid: &'a Guid,
}

impl<'a, R, C> ReliableStatelessWriterBehavior<'a, R, C> {
    /// Implement 8.4.8.2.4 Transition T4
    pub fn send_unsent_changes<Data, EntityIdElement, CacheChange, S>(
        &mut self,
        mut send_data: impl FnMut(Data),
        mut send_gap: impl FnMut(GapSubmessage<S>),
    ) where
        R: RtpsReaderLocatorOperations,
        C: RtpsHistoryCacheGetChange<CacheChangeType = CacheChange>,
        Data: DataSubmessageConstructor<
            EntityIdSubmessageElementType = EntityIdElement,
            SequenceNumberSubmessageElementType = SequenceNumber,
            ParameterListSubmessageElementType = &'a CacheChange::ParameterListType,
            SerializedDataSubmessageElementType = &'a CacheChange::DataType,
        >,
        EntityIdElement: EntityIdSubmessageElementConstructor<EntityIdType = EntityId>,
        CacheChange: RtpsCacheChangeAttributes + 'a,
        S: FromIterator<SequenceNumber>,
    {
        while let Some(seq_num) = self
            .reader_locator
            .next_unsent_change(self.last_change_sequence_number)
        {
            if let Some(change) = self.writer_cache.get_change(&seq_num) {
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
                let writer_sn = *change.sequence_number();
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
                let reader_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let writer_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let gap_start = SequenceNumberSubmessageElement { value: seq_num };
                let set = core::iter::empty().collect();
                let gap_list = SequenceNumberSetSubmessageElement { base: seq_num, set };
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

    /// Implement 8.4.8.2.5 Transition T5
    pub fn send_heartbeat(
        &mut self,
        heartbeat_count: Count,
        send_heartbeat: &mut dyn FnMut(HeartbeatSubmessage),
    ) where
        C: RtpsHistoryCacheOperations,
    {
        let endianness_flag = true;
        let final_flag = false;
        let liveliness_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: ENTITYID_UNKNOWN,
        };
        let writer_id = EntityIdSubmessageElement {
            value: self.writer_guid.entity_id,
        };
        let first_sn = SequenceNumberSubmessageElement {
            value: self.writer_cache.get_seq_num_min().unwrap_or(0),
        };
        let last_sn = SequenceNumberSubmessageElement {
            value: self.writer_cache.get_seq_num_min().unwrap_or(0),
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

    /// Implement 8.4.8.2.5 Transition T6
    /// Implementation does not include the part correponding to searching the reader locator
    /// on the stateless writer
    pub fn process_acknack<S>(&mut self, acknack: &AckNackSubmessage<S>)
    where
        R: RtpsReaderLocatorOperations,
        S: AsRef<[SequenceNumber]>,
    {
        self.reader_locator.requested_changes_set(
            acknack.reader_sn_state.set.as_ref(),
            self.last_change_sequence_number,
        );
    }

    /// Implement 8.4.9.2.12 Transition T10
    pub fn send_requested_changes<P, Data, EntityIdElement, CacheChange, S>(
        &mut self,
        mut send_data: impl FnMut(Data),
        mut send_gap: impl FnMut(GapSubmessage<S>),
    ) where
        R: RtpsReaderLocatorOperations,
        C: RtpsHistoryCacheGetChange<CacheChangeType = CacheChange>,
        Data: DataSubmessageConstructor<
            EntityIdSubmessageElementType = EntityIdElement,
            SequenceNumberSubmessageElementType = SequenceNumber,
            ParameterListSubmessageElementType = &'a CacheChange::ParameterListType,
            SerializedDataSubmessageElementType = &'a CacheChange::DataType,
        >,
        EntityIdElement: EntityIdSubmessageElementConstructor<EntityIdType = EntityId>,
        CacheChange: RtpsCacheChangeAttributes + 'a,
        S: FromIterator<SequenceNumber>,
    {
        while let Some(seq_num) = self.reader_locator.next_requested_change() {
            if let Some(change) = self.writer_cache.get_change(&seq_num) {
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
                let writer_sn = *change.sequence_number();
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
                let reader_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let writer_id = EntityIdSubmessageElement {
                    value: ENTITYID_UNKNOWN,
                };
                let gap_start = SequenceNumberSubmessageElement { value: seq_num };
                let set = core::iter::empty().collect();
                let gap_list = SequenceNumberSetSubmessageElement { base: seq_num, set };
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
    use crate::structure::types::InstanceHandle;

    use super::*;

    struct MockVecSeqNum;

    impl FromIterator<SequenceNumber> for MockVecSeqNum {
        fn from_iter<T: IntoIterator<Item = SequenceNumber>>(_iter: T) -> Self {
            Self
        }
    }

    struct MockReaderLocatorOperations(Option<i64>);

    impl RtpsReaderLocatorOperations for MockReaderLocatorOperations {
        type SequenceNumberVector = ();

        fn next_requested_change(&mut self) -> Option<SequenceNumber> {
            todo!()
        }

        fn next_unsent_change(
            &mut self,
            _last_change_sequence_number: &SequenceNumber,
        ) -> Option<SequenceNumber> {
            self.0.take()
        }

        fn requested_changes(&self) -> Self::SequenceNumberVector {
            todo!()
        }

        fn requested_changes_set(
            &mut self,
            _req_seq_num_set: &[SequenceNumber],
            _last_change_sequence_number: &SequenceNumber,
        ) {
            todo!()
        }

        fn unsent_changes(
            &self,
            _last_change_sequence_number: &SequenceNumber,
        ) -> Self::SequenceNumberVector {
            todo!()
        }
    }

    struct MockEntityIdSubmessageElement;

    impl EntityIdSubmessageElementConstructor for MockEntityIdSubmessageElement {
        type EntityIdType = EntityId;

        fn new(_value: &Self::EntityIdType) -> Self {
            Self
        }
    }

    struct MockDataSubmessage<'a>(&'a ());

    impl<'a> DataSubmessageConstructor for MockDataSubmessage<'a> {
        type EntityIdSubmessageElementType = MockEntityIdSubmessageElement;
        type SequenceNumberSubmessageElementType = SequenceNumber;
        type ParameterListSubmessageElementType = &'a ();
        type SerializedDataSubmessageElementType = &'a ();

        fn new(
            _endianness_flag: crate::messages::types::SubmessageFlag,
            _inline_qos_flag: crate::messages::types::SubmessageFlag,
            _data_flag: crate::messages::types::SubmessageFlag,
            _key_flag: crate::messages::types::SubmessageFlag,
            _non_standard_payload_flag: crate::messages::types::SubmessageFlag,
            _reader_id: Self::EntityIdSubmessageElementType,
            _writer_id: Self::EntityIdSubmessageElementType,
            _writer_sn: Self::SequenceNumberSubmessageElementType,
            _inline_qos: Self::ParameterListSubmessageElementType,
            _serialized_payload: Self::SerializedDataSubmessageElementType,
        ) -> Self {
            Self(&())
        }
    }
    struct MockCacheChange;

    impl RtpsCacheChangeAttributes for MockCacheChange {
        type DataType = ();
        type ParameterListType = ();

        fn kind(&self) -> &ChangeKind {
            todo!()
        }

        fn writer_guid(&self) -> &Guid {
            todo!()
        }

        fn instance_handle(&self) -> &InstanceHandle {
            todo!()
        }

        fn sequence_number(&self) -> &SequenceNumber {
            todo!()
        }

        fn data_value(&self) -> &Self::DataType {
            todo!()
        }

        fn inline_qos(&self) -> &Self::ParameterListType {
            todo!()
        }
    }

    #[test]
    fn best_effort_stateless_writer_send_data() {
        struct MockWriterCache;

        impl RtpsHistoryCacheGetChange for MockWriterCache {
            type CacheChangeType = MockCacheChange;
            fn get_change(&self, _seq_num: &SequenceNumber) -> Option<&Self::CacheChangeType> {
                Some(&MockCacheChange)
            }
        }

        let mut best_effort_behavior = BestEffortStatelessWriterBehavior {
            reader_locator: &mut MockReaderLocatorOperations(Some(1)),
            writer_cache: &MockWriterCache,
            last_change_sequence_number: &1,
        };
        let mut data_messages = None;
        best_effort_behavior.send_unsent_changes(
            |data: MockDataSubmessage| data_messages = Some(data),
            |_: GapSubmessage<MockVecSeqNum>| assert!(false),
        );

        assert!(data_messages.is_some());
    }

    #[test]
    fn best_effort_stateless_writer_send_gap() {
        struct MockWriterCache;

        impl RtpsHistoryCacheGetChange for MockWriterCache {
            type CacheChangeType = MockCacheChange;
            fn get_change(&self, _seq_num: &SequenceNumber) -> Option<&Self::CacheChangeType> {
                None
            }
        }

        let mut best_effort_behavior = BestEffortStatelessWriterBehavior {
            reader_locator: &mut MockReaderLocatorOperations(Some(1)),
            writer_cache: &MockWriterCache,
            last_change_sequence_number: &1,
        };
        let mut gap_message = None;
        best_effort_behavior.send_unsent_changes(
            |_: MockDataSubmessage| assert!(false),
            |gap: GapSubmessage<MockVecSeqNum>| gap_message = Some(gap),
        );

        assert!(gap_message.is_some());
    }

    #[test]
    fn best_effort_stateless_writer_do_nothing() {
        struct MockWriterCache;

        impl RtpsHistoryCacheGetChange for MockWriterCache {
            type CacheChangeType = MockCacheChange;

            fn get_change(&self, _seq_num: &SequenceNumber) -> Option<&Self::CacheChangeType> {
                None
            }
        }

        let mut best_effort_behavior = BestEffortStatelessWriterBehavior {
            reader_locator: &mut MockReaderLocatorOperations(None),
            writer_cache: &MockWriterCache,
            last_change_sequence_number: &1,
        };
        best_effort_behavior.send_unsent_changes(
            |_: MockDataSubmessage| assert!(false),
            |_: GapSubmessage<MockVecSeqNum>| assert!(false),
        );
    }
}

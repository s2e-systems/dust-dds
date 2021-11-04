use core::{
    iter::FromIterator,
    ops::{Deref, DerefMut},
};

use crate::{
    behavior::types::Duration,
    messages::{
        submessage_elements::{
            EntityIdSubmessageElement, ParameterListSubmessageElement,
            SequenceNumberSetSubmessageElement, SequenceNumberSubmessageElement,
            SerializedDataSubmessageElement,
        },
        submessages::{DataSubmessage, GapSubmessage},
    },
    structure::{
        history_cache::{RtpsHistoryCacheConstructor, RtpsHistoryCacheGetChange},
        types::{
            ChangeKind, Guid, Locator, ReliabilityKind, SequenceNumber, TopicKind, ENTITYID_UNKNOWN,
        },
    },
};

use super::{
    reader_locator::{RtpsReaderLocator, RtpsReaderLocatorOperations},
    writer::RtpsWriter,
};

pub struct RtpsStatelessWriter<L, C, R> {
    writer: RtpsWriter<L, C>,
    pub reader_locators: R,
}

impl<L, C, R> Deref for RtpsStatelessWriter<L, C, R> {
    type Target = RtpsWriter<L, C>;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl<L, C, R> DerefMut for RtpsStatelessWriter<L, C, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

impl<L, C, R> RtpsStatelessWriter<L, C, R>
where
    R: Default,
    C: RtpsHistoryCacheConstructor,
{
    pub fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: L,
        multicast_locator_list: L,
        push_mode: bool,
        heartbeat_period: Duration,
        nack_response_delay: Duration,
        nack_suppression_duration: Duration,
        data_max_size_serialized: Option<i32>,
    ) -> Self {
        Self {
            writer: RtpsWriter::new(
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
            ),
            reader_locators: R::default(),
        }
    }
}

pub trait RtpsStatelessWriterOperations {
    fn reader_locator_add(&mut self, a_locator: RtpsReaderLocator);

    fn reader_locator_remove(&mut self, a_locator: &Locator);

    fn unsent_changes_reset(&mut self);
}

pub trait StatelessWriterBehavior<S, P, D> {
    type ReaderLocator;

    fn send_unsent_data(
        &mut self,
        send_data: impl FnMut(&Self::ReaderLocator, DataSubmessage<P, D>),
        send_gap: impl FnMut(&Self::ReaderLocator, GapSubmessage<S>),
    );
}

impl<S, L, C, R, RL, P, D> StatelessWriterBehavior<S, P, D> for RtpsStatelessWriter<L, C, R>
where
    for<'b> &'b mut R: IntoIterator<Item = &'b mut RL>,
    RL: RtpsReaderLocatorOperations,
    C: for<'a> RtpsHistoryCacheGetChange<'a, P, D>,
    S: FromIterator<SequenceNumber>,
{
    type ReaderLocator = RL;

    fn send_unsent_data(
        &mut self,
        mut send_data: impl FnMut(&Self::ReaderLocator, DataSubmessage<P, D>),
        mut send_gap: impl FnMut(&Self::ReaderLocator, GapSubmessage<S>),
    ) {
        let reliability_level = self.writer.reliability_level;
        let last_change_sequence_number = self.writer.last_change_sequence_number;
        for reader_locator in &mut self.reader_locators {
            match reliability_level {
                ReliabilityKind::BestEffort => best_effort_send_unsent_data(
                    reader_locator,
                    &self.writer.writer_cache,
                    &last_change_sequence_number,
                    &mut send_data,
                    &mut send_gap,
                ),
                ReliabilityKind::Reliable => todo!(),
            }
        }
    }
}

fn best_effort_send_unsent_data<RL, S, P, D>(
    reader_locator: &mut RL,
    writer_cache: &impl for<'a> RtpsHistoryCacheGetChange<'a, P, D>,
    last_change_sequence_number: &SequenceNumber,
    send_data: &mut impl FnMut(&RL, DataSubmessage<P, D>),
    send_gap: &mut impl FnMut(&RL, GapSubmessage<S>),
) where
    RL: RtpsReaderLocatorOperations,
    S: FromIterator<SequenceNumber>,
{
    while let Some(seq_num) = reader_locator.next_unsent_change(&last_change_sequence_number) {
        if let Some(change) = writer_cache.get_change(&seq_num) {
            let endianness_flag = true;
            let inline_qos_flag = true;
            let (data_flag, key_flag) = match change.kind {
                ChangeKind::Alive => (true, false),
                ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered => (false, true),
                _ => todo!(),
            };
            let non_standard_payload_flag = false;
            let reader_id = EntityIdSubmessageElement {
                value: ENTITYID_UNKNOWN,
            };
            let writer_id = EntityIdSubmessageElement {
                value: *change.writer_guid.entity_id(),
            };
            let writer_sn = SequenceNumberSubmessageElement {
                value: change.sequence_number,
            };
            let inline_qos = ParameterListSubmessageElement {
                parameter: change.inline_qos,
            };
            let serialized_payload = SerializedDataSubmessageElement {
                value: change.data_value,
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
            send_data(reader_locator, data_submessage)
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
            send_gap(reader_locator, gap_submessage)
        }
    }
}

pub fn reliable_send_unsent_data(
    reader_locator: &mut impl RtpsReaderLocatorOperations,
    last_change_sequence_number: SequenceNumber,
    mut send: impl FnMut(SequenceNumber),
) {
    while let Some(seq_num) = reader_locator.next_unsent_change(&last_change_sequence_number) {
        send(seq_num)
    }
}

// #[cfg(test)]
// mod tests {
//     use core::marker::PhantomData;

//     use crate::{
//         messages::{
//             submessage_elements::{
//                 EntityIdSubmessageElementType, ParameterListSubmessageElementType,
//                 SequenceNumberSetSubmessageElementType, SequenceNumberSubmessageElementType,
//                 SerializedDataSubmessageElementType,
//             },
//             RtpsSubmessageHeaderType, Submessage,
//         },
//         structure::{
//             types::{Locator, GUID, GUID_UNKNOWN, LOCATOR_INVALID},
//             RTPSCacheChange, RtpsHistoryCacheOperations,
//         },
//     };

//     use super::*;

//     #[derive(Clone)]
//     struct MockParameterList;

//     impl ParameterListSubmessageElementType for MockParameterList {
//         type Parameter = ();

//         fn new(_parameter: &[Self::Parameter]) -> Self {
//             todo!()
//         }

//         fn parameter(&self) -> &[Self::Parameter] {
//             todo!()
//         }

//         fn empty() -> Self {
//             MockParameterList
//         }
//     }

//     struct MockPSM;

//     impl ParameterListSubmessageElementPIM for MockPSM {
//         type ParameterListSubmessageElementType = MockParameterList;
//     }

//     impl SubmessageKindPIM for MockPSM {
//         type SubmessageKindType = u8;

//         const DATA: Self::SubmessageKindType = 0;
//         const GAP: Self::SubmessageKindType = 0;
//         const HEARTBEAT: Self::SubmessageKindType = 0;
//         const ACKNACK: Self::SubmessageKindType = 0;
//         const PAD: Self::SubmessageKindType = 0;
//         const INFO_TS: Self::SubmessageKindType = 0;
//         const INFO_REPLY: Self::SubmessageKindType = 0;
//         const INFO_DST: Self::SubmessageKindType = 0;
//         const INFO_SRC: Self::SubmessageKindType = 0;
//         const DATA_FRAG: Self::SubmessageKindType = 0;
//         const NACK_FRAG: Self::SubmessageKindType = 0;
//         const HEARTBEAT_FRAG: Self::SubmessageKindType = 0;
//     }

//     impl<'a> SerializedDataSubmessageElementPIM<'a> for MockPSM {
//         type SerializedDataSubmessageElementType = &'a [u8];
//     }

//     impl SequenceNumberSubmessageElementPIM for MockPSM {
//         type SequenceNumberSubmessageElementType = i64;
//     }

//     impl EntityIdSubmessageElementPIM for MockPSM {
//         type EntityIdSubmessageElementType = [u8; 4];
//     }

//     impl SequenceNumberSetSubmessageElementPIM for MockPSM {
//         type SequenceNumberSetSubmessageElementType = MockSequenceNumberSet;
//     }

//     struct MockSubmessageHeader;

//     impl RtpsSubmessageHeaderType<MockPSM> for MockSubmessageHeader {
//         fn submessage_id(&self) -> u8 {
//             todo!()
//         }

//         fn flags(&self) -> [bool; 8] {
//             todo!()
//         }

//         fn submessage_length(&self) -> u16 {
//             todo!()
//         }
//     }

//     struct MockDataSubmessage<'a>(i64, PhantomData<&'a ()>);

//     impl Submessage for MockDataSubmessage {
//         type RtpsSubmessageHeaderType = MockSubmessageHeader;

//         fn submessage_header(&self) -> MockSubmessageHeader {
//             todo!()
//         }
//     }

//     impl EntityIdSubmessageElementType for [u8; 4] {
//         fn new(value: &[u8; 4]) -> Self {
//             value.clone()
//         }

//         fn value(&self) -> &[u8; 4] {
//             self
//         }
//     }

//     impl SequenceNumberSubmessageElementType for i64 {
//         fn new(value: i64) -> Self {
//             value.clone()
//         }

//         fn value(&self) -> i64 {
//             *self
//         }
//     }

//     impl<'a> SerializedDataSubmessageElementType<'a> for &'a [u8] {
//         fn new(value: &'a [u8]) -> Self {
//             value
//         }

//         fn value(&self) -> &[u8] {
//             self
//         }
//     }

//     impl DataSubmessage for MockDataSubmessage {
//         fn new(
//             _endianness_flag: bool,
//             _inline_qos_flag: bool,
//             _data_flag: bool,
//             _key_flag: bool,
//             _non_standard_payload_flag: bool,
//             _reader_id: [u8; 4],
//             _writer_id: [u8; 4],
//             writer_sn: i64,
//             _inline_qos: MockParameterList,
//             _serialized_payload: &[u8],
//         ) -> Self {
//             Self(writer_sn)
//         }

//         fn endianness_flag(&self) -> bool {
//             todo!()
//         }

//         fn inline_qos_flag(&self) -> bool {
//             todo!()
//         }

//         fn data_flag(&self) -> bool {
//             todo!()
//         }

//         fn key_flag(&self) -> bool {
//             todo!()
//         }

//         fn non_standard_payload_flag(&self) -> bool {
//             todo!()
//         }

//         fn reader_id(&self) -> &[u8; 4] {
//             todo!()
//         }

//         fn writer_id(&self) -> &[u8; 4] {
//             todo!()
//         }

//         fn writer_sn(&self) -> &i64 {
//             &self.0
//         }

//         fn inline_qos(&self) -> &MockParameterList {
//             todo!()
//         }

//         fn serialized_payload(&self) -> &&'static [u8] {
//             todo!()
//         }

//         type EntityIdSubmessageElementType = [u8; 4];
//         type SequenceNumberSubmessageElementType = i64;
//         type ParameterListSubmessageElementType = MockParameterList;
//         type SerializedDataSubmessageElementType = &[u8];
//     }

//     struct MockSequenceNumberSet;

//     impl SequenceNumberSetSubmessageElementType for MockSequenceNumberSet {
//         type IntoIter = core::option::IntoIter<SequenceNumber>;

//         fn new(_base: SequenceNumber, _set: &[SequenceNumber]) -> Self {
//             MockSequenceNumberSet
//         }

//         fn base(&self) -> SequenceNumber {
//             todo!()
//         }

//         fn set(&self) -> Self::IntoIter {
//             todo!()
//         }
//     }

//     struct MockGapSubmessage(i64);

//     impl Submessage for MockGapSubmessage {
//         fn submessage_header(&self) -> MockSubmessageHeader {
//             todo!()
//         }

//         type RtpsSubmessageHeaderType = MockSubmessageHeader;
//     }

//     impl GapSubmessage for MockGapSubmessage {
//         fn new(
//             _endianness_flag: bool,
//             _reader_id: [u8; 4],
//             _writer_id: [u8; 4],
//             gap_start: i64,
//             _gap_list: MockSequenceNumberSet,
//         ) -> Self {
//             MockGapSubmessage(gap_start)
//         }

//         fn endianness_flag(&self) -> bool {
//             todo!()
//         }

//         fn reader_id(&self) -> &[u8; 4] {
//             todo!()
//         }

//         fn writer_id(&self) -> &[u8; 4] {
//             todo!()
//         }

//         fn gap_start(&self) -> &i64 {
//             &self.0
//         }

//         fn gap_list(&self) -> &MockSequenceNumberSet {
//             todo!()
//         }

//         type EntityIdSubmessageElementType = [u8; 4];
//         type SequenceNumberSubmessageElementType = i64;
//         type SequenceNumberSetSubmessageElementType = MockSequenceNumberSet;
//     }

//     struct MockReaderLocator {
//         last_sent_sequence_number: i64,
//     }

//     impl BestEffortBehavior for MockReaderLocator{}

//     impl<'a> RTPSReaderLocator for MockReaderLocator {
//         type SequenceNumberVector = Option<i64>;

//         fn locator(&self) -> &Locator {
//             &LOCATOR_INVALID
//         }

//         fn expects_inline_qos(&self) -> bool {
//             todo!()
//         }

//         fn next_requested_change(&mut self) -> Option<i64> {
//             todo!()
//         }

//         fn next_unsent_change(&mut self, last_change_sequence_number: &i64) -> Option<i64> {
//             if &self.last_sent_sequence_number < last_change_sequence_number {
//                 self.last_sent_sequence_number += 1;
//                 Some(self.last_sent_sequence_number)
//             } else {
//                 None
//             }
//         }

//         fn requested_changes(&self) -> Self::SequenceNumberVector {
//             todo!()
//         }

//         fn requested_changes_set(
//             &mut self,
//             _req_seq_num_set: &[i64],
//             _last_change_sequence_number: &i64,
//         ) {
//             todo!()
//         }

//         fn unsent_changes(&self, _last_change_sequence_number: i64) -> Self::SequenceNumberVector {
//             todo!()
//         }
//     }

//     struct MockCacheChange {
//         kind: ChangeKind,
//         sequence_number: i64,
//     }

//     impl RTPSCacheChange for MockCacheChange {
//         type DataType = [u8; 0];
//         type InstanceHandleType = ();
//         type InlineQosType = MockParameterList;

//         fn kind(&self) -> crate::structure::types::ChangeKind {
//             self.kind
//         }

//         fn writer_guid(&self) -> &GUID {
//             &GUID_UNKNOWN
//         }

//         fn instance_handle(&self) -> &() {
//             todo!()
//         }

//         fn sequence_number(&self) -> &i64 {
//             &self.sequence_number
//         }

//         fn data_value(&self) -> &[u8; 0] {
//             &[]
//         }

//         fn inline_qos(&self) -> &MockParameterList {
//             &MockParameterList
//         }

//     }
//     struct MockHistoryCache<const N: usize> {
//         changes: [MockCacheChange; N],
//     }

//     impl<const N: usize> RtpsHistoryCacheOperations for MockHistoryCache<N> {
//         type CacheChange = MockCacheChange;

//         fn new() -> Self
//         where
//             Self: Sized,
//         {
//             todo!()
//         }

//         fn add_change(&mut self, _change: Self::CacheChange) {
//             todo!()
//         }

//         fn remove_change(&mut self, _seq_num: &i64) {
//             todo!()
//         }

//         fn get_change(&self, seq_num: &i64) -> Option<&Self::CacheChange> {
//             self.changes.iter().find(|&x| &x.sequence_number == seq_num)
//         }

//         fn get_seq_num_min(&self) -> Option<&i64> {
//             todo!()
//         }

//         fn get_seq_num_max(&self) -> Option<&i64> {
//             todo!()
//         }
//     }

//     #[test]
//     fn stateless_writer_best_effort_send_unsent_data_only_data() {
//         let mut sent_data_seq_num = [0, 0];
//         let mut total_data = 0;
//         let mut sent_gap_seq_num = [];
//         let mut total_gap = 0;

//         let expected_total_data = 2;
//         let expected_sent_data_seq_num = [1, 2];
//         let expected_total_gap = 0;
//         let expected_sent_gap_seq_num = [];

//         let mut reader_locator = MockReaderLocator {
//             last_sent_sequence_number: 0,
//         };
//         let writer_cache = MockHistoryCache::<2> {
//             changes: [
//                 MockCacheChange {
//                     kind: ChangeKind::Alive,
//                     sequence_number: 1,
//                 },
//                 MockCacheChange {
//                     kind: ChangeKind::Alive,
//                     sequence_number: 2,
//                 },
//             ],
//         };
//         reader_locator.best_effort_send_unsent_data(
//             &2,
//             &writer_cache,
//             |data_submessage: MockDataSubmessage| {
//                 sent_data_seq_num[total_data] = data_submessage.writer_sn().clone();
//                 total_data += 1;
//             },
//             |gap_submessage: MockGapSubmessage| {
//                 sent_gap_seq_num[total_gap] = gap_submessage.gap_start().clone();
//                 total_gap += 1;
//             },
//         );

//         assert_eq!(total_data, expected_total_data);
//         assert_eq!(sent_data_seq_num, expected_sent_data_seq_num);
//         assert_eq!(total_gap, expected_total_gap);
//         assert_eq!(sent_gap_seq_num, expected_sent_gap_seq_num);
//     }

//     #[test]
//     fn stateless_writer_best_effort_send_unsent_data_only_gap() {
//         let mut sent_data_seq_num = [];
//         let mut total_data = 0;
//         let mut sent_gap_seq_num = [0, 0];
//         let mut total_gap = 0;

//         let expected_total_data = 0;
//         let expected_sent_data_seq_num = [];
//         let expected_total_gap = 2;
//         let expected_sent_gap_seq_num = [1, 2];

//         let mut reader_locator = MockReaderLocator {
//             last_sent_sequence_number: 0,
//         };
//         let writer_cache = MockHistoryCache::<0> { changes: [] };
//         reader_locator.best_effort_send_unsent_data(
//             &2,
//             &writer_cache,
//             |data_submessage: MockDataSubmessage| {
//                 sent_data_seq_num[total_data] = data_submessage.writer_sn().clone();
//                 total_data += 1;
//             },
//             |gap_submessage: MockGapSubmessage| {
//                 sent_gap_seq_num[total_gap] = gap_submessage.gap_start().clone();
//                 total_gap += 1;
//             },
//         );

//         assert_eq!(total_data, expected_total_data);
//         assert_eq!(sent_data_seq_num, expected_sent_data_seq_num);
//         assert_eq!(total_gap, expected_total_gap);
//         assert_eq!(sent_gap_seq_num, expected_sent_gap_seq_num);
//     }

//     #[test]
//     fn stateless_writer_best_effort_send_unsent_data_data_and_gap() {
//         let mut sent_data_seq_num = [0];
//         let mut total_data = 0;
//         let mut sent_gap_seq_num = [0];
//         let mut total_gap = 0;

//         let expected_total_data = 1;
//         let expected_sent_data_seq_num = [2];
//         let expected_total_gap = 1;
//         let expected_sent_gap_seq_num = [1];

//         let mut reader_locator = MockReaderLocator {
//             last_sent_sequence_number: 0,
//         };
//         let writer_cache = MockHistoryCache::<1> {
//             changes: [MockCacheChange {
//                 kind: ChangeKind::Alive,
//                 sequence_number: 2,
//             }],
//         };
//         reader_locator.best_effort_send_unsent_data(
//             &2,
//             &writer_cache,
//             |data_submessage: MockDataSubmessage| {
//                 sent_data_seq_num[total_data] = data_submessage.writer_sn().clone();
//                 total_data += 1;
//             },
//             |gap_submessage: MockGapSubmessage| {
//                 sent_gap_seq_num[total_gap] = gap_submessage.gap_start().clone();
//                 total_gap += 1;
//             },
//         );

//         assert_eq!(total_data, expected_total_data);
//         assert_eq!(sent_data_seq_num, expected_sent_data_seq_num);
//         assert_eq!(total_gap, expected_total_gap);
//         assert_eq!(sent_gap_seq_num, expected_sent_gap_seq_num);
//     }
// }

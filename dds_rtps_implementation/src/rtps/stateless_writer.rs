use rust_rtps_pim::{
    behavior::{
        stateless_writer::{RTPSReaderLocator, RTPSStatelessWriter},
        types::Duration,
        RTPSWriter,
    },
    messages::{submessages::submessage_elements::Parameter, types::ParameterId},
    structure::RTPSHistoryCache,
    types::{EntityId, GuidPrefix, InstanceHandle, Locator, SequenceNumber},
};

// use std::ops::{Deref, DerefMut};

// use rust_rtps::{
//     behavior::{
//         stateless_writer::{
//             BestEffortReaderLocatorBehavior, BestEffortReaderLocatorSendSubmessages,
//             RTPSReaderLocator,
//         },
//         RTPSStatelessWriter, RTPSWriter,
//     },
//     messages::submessages::Submessage,
//     structure::{RTPSCacheChange, RTPSHistoryCache},
//     types::{Locator, SequenceNumber},
// };

// use crate::utils::endpoint_traits::DestinedMessages;

pub struct StatelessWriter<
    GuidPrefixType: GuidPrefix,
    EntityIdType: EntityId,
    LocatorType: Locator,
    LocatorListType: IntoIterator<Item = LocatorType>,
    DurationType: Duration,
    SequenceNumberType: SequenceNumber,
    InstanceHandleType: InstanceHandle,
    DataType,
    ParameterIdType: ParameterId,
    ParameterValueType: AsRef<[u8]> + Clone,
    ParameterListType: IntoIterator<Item = Parameter<ParameterIdType, ParameterValueType>> + Clone,
    HistoryCacheType: RTPSHistoryCache<
        GuidPrefix = GuidPrefixType,
        EntityId = EntityIdType,
        InstanceHandle = InstanceHandleType,
        SequenceNumber = SequenceNumberType,
        Data = DataType,
        ParameterId = ParameterIdType,
        ParameterValue = ParameterValueType,
        ParameterList = ParameterListType,
    >
> {
    writer: RTPSWriter<
        GuidPrefixType,
        EntityIdType,
        LocatorType,
        LocatorListType,
        DurationType,
        SequenceNumberType,
        InstanceHandleType,
        DataType,
        ParameterIdType,
        ParameterValueType,
        ParameterListType,
        HistoryCacheType,
    >,
    reader_locators: Vec<ReaderLocator>,
}

impl<
        GuidPrefixType: GuidPrefix,
        EntityIdType: EntityId,
        LocatorType: Locator,
        LocatorListType: IntoIterator<Item = LocatorType>,
        DurationType: Duration,
        SequenceNumberType: SequenceNumber,
        InstanceHandleType: InstanceHandle,
        DataType,
        ParameterIdType: ParameterId,
        ParameterValueType: AsRef<[u8]> + Clone,
        ParameterListType: IntoIterator<Item = Parameter<ParameterIdType, ParameterValueType>> + Clone,
        HistoryCacheType: RTPSHistoryCache<
            GuidPrefix = GuidPrefixType,
            EntityId = EntityIdType,
            InstanceHandle = InstanceHandleType,
            SequenceNumber = SequenceNumberType,
            Data = DataType,
            ParameterId = ParameterIdType,
            ParameterValue = ParameterValueType,
            ParameterList = ParameterListType,
        >,
        ReaderLocatorType: RTPSReaderLocator<LocatorType = LocatorType>,
    > RTPSStatelessWriter
    for StatelessWriter<
        GuidPrefixType,
        EntityIdType,
        LocatorType,
        LocatorListType,
        DurationType,
        SequenceNumberType,
        InstanceHandleType,
        DataType,
        ParameterIdType,
        ParameterValueType,
        ParameterListType,
        HistoryCacheType,
        ReaderLocatorType,
    >
{
    type GuidPrefixType = GuidPrefixType;
    type EntityIdType = EntityIdType;
    type LocatorType = LocatorType;
    type LocatorListType = LocatorListType;
    type DurationType = DurationType;
    type SequenceNumberType = SequenceNumberType;
    type InstanceHandleType = InstanceHandleType;
    type DataType = DataType;
    type ParameterIdType = ParameterIdType;
    type ParameterValueType = ParameterValueType;
    type ParameterListType = ParameterListType;
    type HistoryCacheType = HistoryCacheType;

    fn reader_locator_add(&mut self, a_locator: Self::LocatorType) {
        let reader_locator = ReaderLocatorType::new(a_locator, false);
        self.reader_locators.push(reader_locator)
    }

    fn reader_locator_remove(&mut self, a_locator: &Self::LocatorType) {
        self.reader_locators.retain(|x| x.)
    }

    fn unsent_changes_reset(&mut self) {
        todo!()
    }
}

impl<
        GuidPrefixType: GuidPrefix,
        EntityIdType: EntityId,
        LocatorType: Locator,
        LocatorListType: IntoIterator<Item = LocatorType>,
        DurationType: Duration,
        SequenceNumberType: SequenceNumber,
        InstanceHandleType: InstanceHandle,
        DataType,
        ParameterIdType: ParameterId,
        ParameterValueType: AsRef<[u8]> + Clone,
        ParameterListType: IntoIterator<Item = Parameter<ParameterIdType, ParameterValueType>> + Clone,
        HistoryCacheType: RTPSHistoryCache<
            GuidPrefix = GuidPrefixType,
            EntityId = EntityIdType,
            InstanceHandle = InstanceHandleType,
            SequenceNumber = SequenceNumberType,
            Data = DataType,
            ParameterId = ParameterIdType,
            ParameterValue = ParameterValueType,
            ParameterList = ParameterListType,
        >,
        ReaderLocator: RTPSReaderLocator<LocatorType = LocatorType>,
    > core::ops::Deref
    for StatelessWriter<
        GuidPrefixType,
        EntityIdType,
        LocatorType,
        LocatorListType,
        DurationType,
        SequenceNumberType,
        InstanceHandleType,
        DataType,
        ParameterIdType,
        ParameterValueType,
        ParameterListType,
        HistoryCacheType,
        ReaderLocator,
    >
{
    type Target = RTPSWriter<
        GuidPrefixType,
        EntityIdType,
        LocatorType,
        LocatorListType,
        DurationType,
        SequenceNumberType,
        InstanceHandleType,
        DataType,
        ParameterIdType,
        ParameterValueType,
        ParameterListType,
        HistoryCacheType,
    >;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl<
        GuidPrefixType: GuidPrefix,
        EntityIdType: EntityId,
        LocatorType: Locator,
        LocatorListType: IntoIterator<Item = LocatorType>,
        DurationType: Duration,
        SequenceNumberType: SequenceNumber,
        InstanceHandleType: InstanceHandle,
        DataType,
        ParameterIdType: ParameterId,
        ParameterValueType: AsRef<[u8]> + Clone,
        ParameterListType: IntoIterator<Item = Parameter<ParameterIdType, ParameterValueType>> + Clone,
        HistoryCacheType: RTPSHistoryCache<
            GuidPrefix = GuidPrefixType,
            EntityId = EntityIdType,
            InstanceHandle = InstanceHandleType,
            SequenceNumber = SequenceNumberType,
            Data = DataType,
            ParameterId = ParameterIdType,
            ParameterValue = ParameterValueType,
            ParameterList = ParameterListType,
        >,
        ReaderLocator: RTPSReaderLocator<LocatorType = LocatorType>,
    > core::ops::DerefMut
    for StatelessWriter<
        GuidPrefixType,
        EntityIdType,
        LocatorType,
        LocatorListType,
        DurationType,
        SequenceNumberType,
        InstanceHandleType,
        DataType,
        ParameterIdType,
        ParameterValueType,
        ParameterListType,
        HistoryCacheType,
        ReaderLocator,
    >
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

// impl<T, R> StatelessWriter<T, R>
//     where T: RTPSWriter,
//     <<<T as RTPSWriter>::HistoryCacheType as RTPSHistoryCache>::CacheChangeType as RTPSCacheChange>::Data: AsRef<[u8]>,
//     R: RTPSReaderLocator<CacheChangeRepresentation = SequenceNumber>, {
//     pub fn produce_messages<'a>(&'a mut self) -> Vec<DestinedMessages> {
//         let mut destined_submessages = Vec::new();

//         for reader_locator in &mut self.reader_locators {

//             let mut submessages: Vec<Box<dyn Submessage + 'a>> = Vec::new();
//             match &self.writer.reliability_level() {
//                 rust_rtps::types::ReliabilityKind::BestEffort => {
//                     while let Some(submessage) = BestEffortReaderLocatorBehavior::produce_message(
//                         reader_locator,
//                         &self.writer,
//                     ) {
//                         match submessage {
//                             BestEffortReaderLocatorSendSubmessages::Data(data) => {
//                                 submessages.push(Box::new(data))
//                             }
//                             BestEffortReaderLocatorSendSubmessages::Gap(gap) => {
//                                 submessages.push(Box::new(gap))
//                             }
//                         }

//                     };
//                 }
//                 rust_rtps::types::ReliabilityKind::Reliable => {
//                    todo!()
//                 }
//             }

//             destined_submessages.push(DestinedMessages::SingleDestination{locator: reader_locator.locator(), messages: submessages});
//         };
//         destined_submessages
//     }

// }

// impl<T: RTPSWriter, R: RTPSReaderLocator> Deref for StatelessWriter<T, R> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         &self.writer
//     }
// }

// impl<T: RTPSWriter, R: RTPSReaderLocator> DerefMut for StatelessWriter<T, R> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.writer
//     }
// }

// impl<T: RTPSWriter, R: RTPSReaderLocator> RTPSStatelessWriter<T> for StatelessWriter<T, R> {
//     type ReaderLocatorType = R;

//     fn new(writer: T) -> Self {
//         Self {
//             writer,
//             reader_locators: Vec::new(),
//         }
//     }

//     fn reader_locators(&self) -> &[Self::ReaderLocatorType] {
//         &self.reader_locators
//     }

//     fn reader_locator_add(&mut self, a_locator: Locator) {
//         let reader_locator = Self::ReaderLocatorType::new(a_locator, false);
//         self.reader_locators.push(reader_locator)
//     }

//     fn reader_locator_remove(&mut self, a_locator: &Locator) {
//         self.reader_locators.retain(|x| &x.locator() != a_locator)
//     }

//     fn unsent_changes_reset(&mut self) {
//         for r in &mut self.reader_locators.iter_mut() {
//             *r = Self::ReaderLocatorType::new(r.locator(), r.expects_inline_qos());
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use rust_rtps::{
//         behavior::stateless_writer::{
//             BestEffortReaderLocatorBehavior, BestEffortReaderLocatorSendSubmessages,
//         },
//         messages::submessages::submessage_elements::ParameterList,
//         structure::{RTPSCacheChange, RTPSEndpoint, RTPSEntity, RTPSHistoryCache},
//         types::SequenceNumber,
//     };

//     use crate::rtps::cache_change::CacheChange;

//     use super::*;

//     struct MockParameterList;

//     impl ParameterList for MockParameterList {
//         fn parameter(
//             &self,
//         ) -> &[Box<dyn rust_rtps::messages::submessages::submessage_elements::Parameter>] {
//             todo!()
//         }
//     }

//     struct MockCacheChange(SequenceNumber);

//     impl RTPSCacheChange for MockCacheChange {
//         type Data = Vec<u8>;
//         type InstanceHandle = ();
//         type ParameterList = MockParameterList;

//         fn new(
//             _kind: rust_rtps::types::ChangeKind,
//             _writer_guid: rust_rtps::types::GUID,
//             _instance_handle: Self::InstanceHandle,
//             _sequence_number: rust_rtps::types::SequenceNumber,
//             _data_value: Self::Data,
//             _inline_qos: Self::ParameterList,
//         ) -> Self {
//             todo!()
//         }

//         fn kind(&self) -> rust_rtps::types::ChangeKind {
//             todo!()
//         }

//         fn writer_guid(&self) -> rust_rtps::types::GUID {
//             todo!()
//         }

//         fn instance_handle(&self) -> &Self::InstanceHandle {
//             todo!()
//         }

//         fn sequence_number(&self) -> rust_rtps::types::SequenceNumber {
//             todo!()
//         }

//         fn data_value(&self) -> &Self::Data {
//             todo!()
//         }

//         fn inline_qos(&self) -> &Self::ParameterList {
//             todo!()
//         }
//     }
//     struct MockHistoryCache(Vec<MockCacheChange>);

//     impl RTPSHistoryCache for MockHistoryCache {
//         type CacheChangeType = MockCacheChange;

//         fn new() -> Self {
//             todo!()
//         }

//         fn add_change(&mut self, _change: Self::CacheChangeType) {
//             todo!()
//         }

//         fn remove_change(&mut self, _seq_num: rust_rtps::types::SequenceNumber) {
//             todo!()
//         }

//         fn get_change(
//             &self,
//             _seq_num: rust_rtps::types::SequenceNumber,
//         ) -> Option<&Self::CacheChangeType> {
//             todo!()
//         }

//         fn get_seq_num_min(&self) -> Option<rust_rtps::types::SequenceNumber> {
//             todo!()
//         }

//         fn get_seq_num_max(&self) -> Option<rust_rtps::types::SequenceNumber> {
//             todo!()
//         }
//     }

//     struct MockWriter;

//     impl RTPSEntity for MockWriter {
//         fn guid(&self) -> rust_rtps::types::GUID {
//             todo!()
//         }
//     }

//     impl RTPSEndpoint for MockWriter {
//         fn unicast_locator_list(&self) -> &[Locator] {
//             todo!()
//         }

//         fn multicast_locator_list(&self) -> &[Locator] {
//             todo!()
//         }

//         fn topic_kind(&self) -> rust_rtps::types::TopicKind {
//             todo!()
//         }

//         fn reliability_level(&self) -> rust_rtps::types::ReliabilityKind {
//             todo!()
//         }
//     }

//     impl RTPSWriter for MockWriter {
//         type HistoryCacheType = MockHistoryCache;

//         fn new(
//             _guid: rust_rtps::types::GUID,
//             _topic_kind: rust_rtps::types::TopicKind,
//             _reliablility_level: rust_rtps::types::ReliabilityKind,
//             _unicast_locator_list: &[Locator],
//             _multicast_locator_list: &[Locator],
//             _push_mode: bool,
//             _heartbeat_period: rust_rtps::behavior::types::Duration,
//             _nack_response_delay: rust_rtps::behavior::types::Duration,
//             _nack_suppression_duration: rust_rtps::behavior::types::Duration,
//             _data_max_sized_serialized: i32,
//         ) -> Self {
//             todo!()
//         }

//         fn push_mode(&self) -> bool {
//             todo!()
//         }

//         fn heartbeat_period(&self) -> rust_rtps::behavior::types::Duration {
//             todo!()
//         }

//         fn nack_response_delay(&self) -> rust_rtps::behavior::types::Duration {
//             todo!()
//         }

//         fn nack_suppression_duration(&self) -> rust_rtps::behavior::types::Duration {
//             todo!()
//         }

//         fn last_change_sequence_number(&self) -> rust_rtps::types::SequenceNumber {
//             todo!()
//         }

//         fn data_max_sized_serialized(&self) -> i32 {
//             todo!()
//         }

//         fn writer_cache(&self) -> &Self::HistoryCacheType {
//             todo!()
//         }

//         fn writer_cache_mut(&mut self) -> &mut Self::HistoryCacheType {
//             todo!()
//         }

//         fn new_change(
//             &mut self,
//             _kind: rust_rtps::types::ChangeKind,
//             _data: <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType as RTPSCacheChange>::Data,
//             _inline_qos:  <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType as RTPSCacheChange>::ParameterList,
//             _handle: <<Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType as RTPSCacheChange>::InstanceHandle,
//         ) -> <Self::HistoryCacheType as RTPSHistoryCache>::CacheChangeType {
//             todo!()
//         }
//     }

//     struct MockReaderLocator {
//         locator: Locator,
//         value: SequenceNumber,
//     }

//     impl RTPSReaderLocator for MockReaderLocator {
//         type CacheChangeRepresentation = SequenceNumber;

//         type CacheChangeRepresentationList = Vec<Self::CacheChangeRepresentation>;

//         fn locator(&self) -> Locator {
//             self.locator
//         }

//         fn expects_inline_qos(&self) -> bool {
//             false
//         }

//         fn new(locator: Locator, _expects_inline_qos: bool) -> Self {
//             Self {
//                 locator,
//                 value: 0.into(),
//             }
//         }

//         fn requested_changes(
//             &self,
//             _writer: &impl RTPSWriter,
//         ) -> Self::CacheChangeRepresentationList {
//             todo!()
//         }

//         fn unsent_changes(&self, _writer: &impl RTPSWriter) -> Self::CacheChangeRepresentationList {
//             todo!()
//         }

//         fn next_requested_change(
//             &mut self,
//             _writer: &impl RTPSWriter,
//         ) -> Option<Self::CacheChangeRepresentation> {
//             todo!()
//         }

//         fn next_unsent_change(
//             &mut self,
//             _writer: &impl RTPSWriter,
//         ) -> Option<Self::CacheChangeRepresentation> {
//             self.value += 1;
//             Some(self.value)
//         }

//         fn requested_changes_set(
//             &mut self,
//             _req_seq_num_set: &[SequenceNumber],
//             _writer: &impl RTPSWriter,
//         ) {
//             todo!()
//         }
//     }

//     #[test]
//     fn reader_locator_add() {
//         let writer = MockWriter;
//         let mut stateless_writer: StatelessWriter<_, MockReaderLocator> =
//             StatelessWriter::new(writer);

//         let locator1 = Locator::new(0, 100, [1; 16]);
//         let locator2 = Locator::new(0, 200, [2; 16]);

//         stateless_writer.reader_locator_add(locator1);
//         stateless_writer.reader_locator_add(locator2);

//         assert_eq!(stateless_writer.reader_locators.len(), 2);
//     }

//     #[test]
//     fn reader_locator_remove() {
//         let writer = MockWriter {};
//         let mut stateless_writer: StatelessWriter<_, MockReaderLocator> =
//             StatelessWriter::new(writer);

//         let locator1 = Locator::new(0, 100, [1; 16]);
//         let locator2 = Locator::new(0, 200, [2; 16]);

//         stateless_writer.reader_locator_add(locator1);
//         stateless_writer.reader_locator_add(locator2);
//         stateless_writer.reader_locator_remove(&locator2);

//         assert_eq!(stateless_writer.reader_locators.len(), 1);
//     }

//     #[test]
//     fn unsent_changes_reset() {
//         let writer = MockWriter;
//         let mut stateless_writer: StatelessWriter<_, MockReaderLocator> =
//             StatelessWriter::new(writer);

//         let locator1 = Locator::new(0, 100, [1; 16]);
//         let locator2 = Locator::new(0, 200, [2; 16]);

//         stateless_writer.reader_locator_add(locator1);
//         stateless_writer.reader_locator_add(locator2);

//         assert_eq!(
//             stateless_writer.reader_locators[0]
//                 .next_unsent_change(&stateless_writer.writer)
//                 .unwrap(),
//             1
//         );
//         assert_eq!(
//             stateless_writer.reader_locators[0]
//                 .next_unsent_change(&stateless_writer.writer)
//                 .unwrap(),
//             2
//         );

//         stateless_writer.unsent_changes_reset();

//         assert_eq!(
//             stateless_writer.reader_locators[0]
//                 .next_unsent_change(&stateless_writer.writer)
//                 .unwrap(),
//             1
//         );
//         assert_eq!(
//             stateless_writer.reader_locators[0]
//                 .next_unsent_change(&stateless_writer.writer)
//                 .unwrap(),
//             2
//         );
//     }

//     #[test]
//     fn reader_locator_behavior() {
//         let writer = MockWriter;
//         let mut stateless_writer: StatelessWriter<_, MockReaderLocator> =
//             StatelessWriter::new(writer);

//         let locator1 = Locator::new(0, 100, [1; 16]);
//         let locator2 = Locator::new(0, 200, [2; 16]);

//         stateless_writer.reader_locator_add(locator1);
//         stateless_writer.reader_locator_add(locator2);

//         // let mut submessages = Vec::new();

//         for reader_locator in &mut stateless_writer.reader_locators {
//             while let Some(message) = BestEffortReaderLocatorBehavior::produce_message(
//                 reader_locator,
//                 &stateless_writer.writer,
//             ) {
//                 match message {
//                     BestEffortReaderLocatorSendSubmessages::Data(_data) => {
//                         // submessages.push(RtpsSubmessage::Data(data))
//                     }
//                     BestEffortReaderLocatorSendSubmessages::Gap(_gap) => {
//                         // submessages.push(RtpsSubmessage::Gap(gap))
//                     }
//                 }
//             }
//         }
//     }
// }

// use rust_rtps_pim::{behavior::{RTPSWriter, stateless_writer::{best_effort_send_unsent_data, RTPSReaderLocator, RTPSStatelessWriter}, types::DurationPIM}, messages::{
//         submessage_elements::{
//             EntityIdSubmessageElementPIM, EntityIdSubmessageElementType,
//             ParameterListSubmessageElementPIM, SequenceNumberSetSubmessageElementPIM,
//             SequenceNumberSetSubmessageElementType, SequenceNumberSubmessageElementPIM,
//             SequenceNumberSubmessageElementType, SerializedDataSubmessageElementPIM,
//             SerializedDataSubmessageElementType,
//         },
//         submessages::{
//             AckNackSubmessagePIM, DataFragSubmessagePIM, DataSubmessage, DataSubmessagePIM,
//             GapSubmessage, GapSubmessagePIM, HeartbeatFragSubmessagePIM, HeartbeatSubmessagePIM,
//             InfoDestinationSubmessagePIM, InfoReplySubmessagePIM, InfoSourceSubmessagePIM,
//             InfoTimestampSubmessagePIM, NackFragSubmessagePIM, PadSubmessagePIM,
//             RtpsSubmessageType,
//         },
//         types::ProtocolIdPIM,
//         RTPSMessage, RTPSMessagePIM, RtpsMessageHeaderPIM, RtpsSubmessageHeaderPIM,
//     }, structure::{
//         types::{
//             DataPIM, EntityIdPIM, GUIDType, GuidPrefixPIM, InstanceHandlePIM, LocatorPIM,
//             ProtocolVersionPIM, SequenceNumberPIM, VendorIdPIM, GUIDPIM,
//         },
//         RTPSCacheChange, RTPSHistoryCache,
//     }};

// use crate::transport::Transport;

// pub fn send_data<PSM, StatelessWriter>(
//     writer: &mut StatelessWriter,
//     transport: &mut dyn Transport<PSM>,
// ) where
//     PSM: SequenceNumberPIM
//         + LocatorPIM
//         + DurationPIM
//         + GUIDPIM
//         + DataPIM
//         + ParameterListSubmessageElementPIM
//         + InstanceHandlePIM
//         + AckNackSubmessagePIM
//         + for<'a> DataSubmessagePIM<'a, PSM>
//         + for<'a> DataFragSubmessagePIM<'a, PSM>
//         + GapSubmessagePIM
//         + HeartbeatSubmessagePIM
//         + HeartbeatFragSubmessagePIM
//         + InfoDestinationSubmessagePIM
//         + InfoReplySubmessagePIM
//         + InfoSourceSubmessagePIM
//         + InfoTimestampSubmessagePIM
//         + NackFragSubmessagePIM
//         + PadSubmessagePIM
//         + EntityIdSubmessageElementPIM
//         + EntityIdPIM
//         + GuidPrefixPIM
//         + SequenceNumberSubmessageElementPIM
//         + for<'a> SerializedDataSubmessageElementPIM<'a>
//         + RtpsSubmessageHeaderPIM
//         + SequenceNumberSetSubmessageElementPIM
//         + ProtocolIdPIM
//         + ProtocolVersionPIM
//         + VendorIdPIM
//         + for<'a> RTPSMessagePIM<'a, PSM>
//         + RtpsMessageHeaderPIM,
//     StatelessWriter: RTPSStatelessWriter<PSM> + RTPSWriter<PSM>,
//     StatelessWriter::ReaderLocatorPIM: RTPSReaderLocator<PSM>,
//     PSM::EntityIdSubmessageElementType: EntityIdSubmessageElementType<PSM>,
//     PSM::GUIDType: GUIDType<PSM>,
//     PSM::SequenceNumberSubmessageElementType: SequenceNumberSubmessageElementType<PSM>,
//     for<'a> <PSM as SerializedDataSubmessageElementPIM<'a>>::SerializedDataSubmessageElementType:
//         SerializedDataSubmessageElementType<'a>,
//     for<'a> <PSM as DataSubmessagePIM<'a, PSM>>::DataSubmessageType: DataSubmessage<'a, PSM>,
//     PSM::SequenceNumberSetSubmessageElementType: SequenceNumberSetSubmessageElementType<PSM>,
//     PSM::GapSubmessageType: GapSubmessage<PSM>,
//     PSM::ParameterListSubmessageElementType: Clone,
//     <StatelessWriter::HistoryCacheType as RTPSHistoryCache<PSM>>::CacheChange: RTPSCacheChange<PSM>,
//     for<'a> <PSM as RTPSMessagePIM<'a, PSM>>::RTPSMessageType: RTPSMessage<'a, PSM>,
//     PSM::SequenceNumberType: Copy,
// {
//     // for writer_group in &rtps_participant_impl.rtps_writer_groups {
//     // let writer_group_lock = writer_group.lock();
//     // let writer_list = rtps_writer_group_impl.writer_list();
//     // for writer in writer_list {
//     let last_change_sequence_number = *writer.last_change_sequence_number();
//     let (reader_locators, writer_cache) = writer.reader_locators();
//     for reader_locator in reader_locators {
//         let mut data_submessage_list: Vec<RtpsSubmessageType<'_, PSM>> = vec![];
//         let mut gap_submessage_list: Vec<RtpsSubmessageType<'_, PSM>> = vec![];

//         // let mut submessages = vec![];
//         best_effort_send_unsent_data(
//             reader_locator,
//             &last_change_sequence_number,
//             writer_cache,
//             |data_submessage| data_submessage_list.push(RtpsSubmessageType::Data(data_submessage)),
//             |gap_submessage| gap_submessage_list.push(RtpsSubmessageType::Gap(gap_submessage)),
//         );

//         let protocol = PSM::PROTOCOL_RTPS;
//         // let version = rtps_participant_impl.protocol_version();
//         // let vendor_id = rtps_participant_impl.vendor_id();
//         // let guid_prefix = rtps_participant_impl.guid().prefix();
//         // for data_submessage in &data_submessage_list {
//         //     submessages.push(RtpsSubmessageType::Data(data_submessage))
//         // }
//         // for gap_submessage in &gap_submessage_list {
//         //     submessages.push(gap_submessage);
//         // }

//         let message = PSM::RTPSMessageType::new(
//             protocol,
//             PSM::PROTOCOLVERSION_2_4,
//             PSM::VENDOR_ID_UNKNOWN,
//             PSM::GUIDPREFIX_UNKNOWN,
//             data_submessage_list,
//         );
//         transport.write(&message, reader_locator.locator());
//     }
// }

// // #[cfg(test)]
// // mod tests {
// //     use rust_rtps_pim::{
// //         messages::{
// //             submessage_elements::{ParameterListSubmessageElementType, ParameterType},
// //             submessages::{NackFragSubmessage, PadSubmessage},
// //             RtpsMessageHeaderType, RtpsSubmessageHeaderType, Submessage,
// //         },
// //         structure::{
// //             types::{GUIDType, LocatorType},
// //             RTPSCacheChange, RTPSEndpoint, RTPSEntity, RTPSHistoryCache,
// //         },
// //     };

// //     use crate::rtps_impl::rtps_history_cache_impl::RTPSHistoryCacheImplTrait;

// //     use super::*;

// //     struct MockPSM;

// //     // impl SequenceNumberPIM for MockPSM {
// //     //     type SequenceNumberType = i64;
// //     //     const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumberType = -1;
// //     // }

// //     impl LocatorPIM for MockPSM {
// //         type LocatorType = MockLocator;

// //         const LOCATOR_INVALID: Self::LocatorType = MockLocator;

// //         const LOCATOR_KIND_INVALID:
// //             <Self::LocatorType as rust_rtps_pim::structure::types::LocatorType>::LocatorKind =
// //             [0; 4];

// //         const LOCATOR_KIND_RESERVED:
// //             <Self::LocatorType as rust_rtps_pim::structure::types::LocatorType>::LocatorKind =
// //             [0; 4];

// //         const LOCATOR_KIND_UDPv4:
// //             <Self::LocatorType as rust_rtps_pim::structure::types::LocatorType>::LocatorKind =
// //             [0; 4];

// //         const LOCATOR_KIND_UDPv6:
// //             <Self::LocatorType as rust_rtps_pim::structure::types::LocatorType>::LocatorKind =
// //             [0; 4];

// //         const LOCATOR_PORT_INVALID:
// //             <Self::LocatorType as rust_rtps_pim::structure::types::LocatorType>::LocatorPort =
// //             [0; 4];

// //         const LOCATOR_ADDRESS_INVALID:
// //             <Self::LocatorType as rust_rtps_pim::structure::types::LocatorType>::LocatorAddress =
// //             [0; 16];
// //     }

// //     // impl ParameterListSubmessageElementPIM for MockPSM {
// //     //     type ParameterListSubmessageElementType = MockParameterList;
// //     // }

// //     // impl ParameterIdPIM for MockPSM {
// //     //     type ParameterIdType = ();
// //     // }

// //     // impl GUIDPIM for MockPSM {
// //     //     type GUIDType = MockGuid;
// //     //     const GUID_UNKNOWN: Self::GUIDType = MockGuid;
// //     // }

// //     // impl GuidPrefixPIM for MockPSM {
// //     //     type GuidPrefixType = [u8; 12];

// //     //     const GUIDPREFIX_UNKNOWN: Self::GuidPrefixType = [0; 12];
// //     // }

// //     // impl EntityIdPIM for MockPSM {
// //     //     type EntityIdType = [u8; 4];

// //     //     const ENTITYID_UNKNOWN: Self::EntityIdType = [0; 4];

// //     //     const ENTITYID_PARTICIPANT: Self::EntityIdType = [0; 4];
// //     // }

// //     // impl InstanceHandlePIM for MockPSM {
// //     //     type InstanceHandleType = ();
// //     // }

// //     // impl DataPIM for MockPSM {
// //     //     type DataType = Vec<u8>;
// //     // }

// //     // impl DurationPIM for MockPSM {
// //     //     type DurationType = ();
// //     // }

// //     // impl ProtocolIdPIM for MockPSM {
// //     //     type ProtocolIdType = ();
// //     //     const PROTOCOL_RTPS: Self::ProtocolIdType = ();
// //     // }

// //     // impl ProtocolVersionPIM for MockPSM {
// //     //     type ProtocolVersionType = ();

// //     //     const PROTOCOLVERSION: Self::ProtocolVersionType = ();
// //     //     const PROTOCOLVERSION_1_0: Self::ProtocolVersionType = ();
// //     //     const PROTOCOLVERSION_1_1: Self::ProtocolVersionType = ();
// //     //     const PROTOCOLVERSION_2_0: Self::ProtocolVersionType = ();
// //     //     const PROTOCOLVERSION_2_1: Self::ProtocolVersionType = ();
// //     //     const PROTOCOLVERSION_2_2: Self::ProtocolVersionType = ();
// //     //     const PROTOCOLVERSION_2_3: Self::ProtocolVersionType = ();
// //     //     const PROTOCOLVERSION_2_4: Self::ProtocolVersionType = ();
// //     // }

// //     // impl<'a> RTPSMessagePIM<'a, Self> for MockPSM {
// //     //     type RTPSMessageType = MockRtpsMessage;
// //     // }

// //     // impl RtpsMessageHeaderPIM for MockPSM {
// //     //     type RtpsMessageHeaderType = MockRtpsMessageHeader;
// //     // }

// //     // impl VendorIdPIM for MockPSM {
// //     //     type VendorIdType = ();
// //     //     const VENDOR_ID_UNKNOWN: Self::VendorIdType = ();
// //     // }

// //     // impl CountPIM for MockPSM {
// //     //     type CountType = ();
// //     // }

// //     // impl SubmessageKindPIM for MockPSM {
// //     //     type SubmessageKindType = ();

// //     //     const DATA: Self::SubmessageKindType = ();
// //     //     const GAP: Self::SubmessageKindType = ();
// //     //     const HEARTBEAT: Self::SubmessageKindType = ();
// //     //     const ACKNACK: Self::SubmessageKindType = ();
// //     //     const PAD: Self::SubmessageKindType = ();
// //     //     const INFO_TS: Self::SubmessageKindType = ();
// //     //     const INFO_REPLY: Self::SubmessageKindType = ();
// //     //     const INFO_DST: Self::SubmessageKindType = ();
// //     //     const INFO_SRC: Self::SubmessageKindType = ();
// //     //     const DATA_FRAG: Self::SubmessageKindType = ();
// //     //     const NACK_FRAG: Self::SubmessageKindType = ();
// //     //     const HEARTBEAT_FRAG: Self::SubmessageKindType = ();
// //     // }

// //     // impl RtpsSubmessageHeaderPIM for MockPSM {
// //     //     type RtpsSubmessageHeaderType = MockRtpsSubmessageHeader;
// //     // }

// //     // impl CountSubmessageElementPIM for MockPSM {
// //     //     type CountSubmessageElementType = MockCountSubmessageElement;
// //     // }

// //     // impl NackFragSubmessagePIM for MockPSM {
// //     //     type NackFragSubmessageType = MockNackFragSubmessage;
// //     // }

// //     // impl PadSubmessagePIM for MockPSM {
// //     //     type PadSubmessageType = MockPadSubmessage;
// //     // }

// //     // struct MockRtpsSubmessageHeader;
// //     // struct MockCountSubmessageElement;

// //     // impl RtpsSubmessageHeaderType<MockPSM> for MockRtpsSubmessageHeader {}

// //     // struct MockNackFragSubmessage;

// //     // impl Submessage<MockPSM> for MockNackFragSubmessage {}
// //     // impl NackFragSubmessage<MockPSM> for MockNackFragSubmessage {}

// //     // struct MockPadSubmessage;

// //     // impl Submessage<MockPSM> for MockPadSubmessage {}
// //     // impl PadSubmessage<MockPSM> for MockPadSubmessage {}

// //     struct MockRtpsMessageHeader;

// //     impl RtpsMessageHeaderType<MockPSM> for MockRtpsMessageHeader {
// //         fn protocol(&self) -> &() {
// //             todo!()
// //         }

// //         fn version(&self) -> &() {
// //             todo!()
// //         }

// //         fn vendor_id(&self) -> &() {
// //             todo!()
// //         }

// //         fn guid_prefix(&self) -> &[u8; 12] {
// //             todo!()
// //         }
// //     }

// //     struct MockRtpsMessage;

// //     impl<'a> RTPSMessage<'a, MockPSM> for MockRtpsMessage {
// //         fn new<T: IntoIterator<Item = RtpsSubmessageType<'a, MockPSM>>>(
// //             protocol: (),
// //             version: (),
// //             vendor_id: (),
// //             guid_prefix: [u8; 12],
// //             submessages: T,
// //         ) -> Self {
// //             todo!()
// //         }

// //         fn header(&self) -> MockRtpsMessageHeader {
// //             todo!()
// //         }

// //         fn submessages(&self) -> &[RtpsSubmessageType<'a, MockPSM>] {
// //             todo!()
// //         }
// //     }

// //     // struct MockParameterList;

// //     // impl ParameterListSubmessageElementType<MockPSM> for MockParameterList {
// //     //     type Parameter = MockParameter;

// //     //     fn new(parameter: &[Self::Parameter]) -> Self {
// //     //         todo!()
// //     //     }

// //     //     fn parameter(&self) -> &[Self::Parameter] {
// //     //         todo!()
// //     //     }
// //     // }

// //     // struct MockParameter;

// //     // impl ParameterType<MockPSM> for MockParameter {
// //     //     fn parameter_id(&self) -> () {
// //     //         todo!()
// //     //     }

// //     //     fn length(&self) -> i16 {
// //     //         todo!()
// //     //     }

// //     //     fn value(&self) -> &[u8] {
// //     //         todo!()
// //     //     }
// //     // }

// //     // struct MockGuid;

// //     // impl From<[u8; 16]> for MockGuid {
// //     //     fn from(_: [u8; 16]) -> Self {
// //     //         todo!()
// //     //     }
// //     // }

// //     // impl From<MockGuid> for [u8; 16] {
// //     //     fn from(_: MockGuid) -> Self {
// //     //         todo!()
// //     //     }
// //     // }

// //     // impl GUIDType<MockPSM> for MockGuid {
// //     //     fn new(prefix: [u8; 12], entity_id: [u8; 4]) -> Self {
// //     //         todo!()
// //     //     }

// //     //     fn prefix(&self) -> &[u8; 12] {
// //     //         todo!()
// //     //     }

// //     //     fn entity_id(&self) -> &[u8; 4] {
// //     //         todo!()
// //     //     }
// //     // }

// //     struct MockLocator;

// //     impl LocatorType for MockLocator {
// //         type LocatorKind = [u8; 4];
// //         type LocatorPort = [u8; 4];
// //         type LocatorAddress = [u8; 16];

// //         fn kind(&self) -> &Self::LocatorKind {
// //             todo!()
// //         }

// //         fn port(&self) -> &Self::LocatorPort {
// //             todo!()
// //         }

// //         fn address(&self) -> &Self::LocatorAddress {
// //             todo!()
// //         }
// //     }

// //     // struct MockReaderLocator;

// //     // impl RTPSReaderLocator<MockPSM> for MockReaderLocator {
// //     //     type SequenceNumberVector = Vec<i64>;

// //     //     fn locator(&self) -> &MockLocator {
// //     //         todo!()
// //     //     }

// //     //     fn expects_inline_qos(&self) -> bool {
// //     //         todo!()
// //     //     }

// //     //     fn next_requested_change(&mut self) -> Option<i64> {
// //     //         todo!()
// //     //     }

// //     //     fn next_unsent_change(&mut self, last_change_sequence_number: &i64) -> Option<i64> {
// //     //         todo!()
// //     //     }

// //     //     fn requested_changes(&self) -> Self::SequenceNumberVector {
// //     //         todo!()
// //     //     }

// //     //     fn requested_changes_set(
// //     //         &mut self,
// //     //         req_seq_num_set: &[i64],
// //     //         last_change_sequence_number: &i64,
// //     //     ) {
// //     //         todo!()
// //     //     }

// //     //     fn unsent_changes(&self, last_change_sequence_number: i64) -> Self::SequenceNumberVector {
// //     //         todo!()
// //     //     }
// //     // }

// //     // struct MockCacheChange;

// //     // impl RTPSCacheChange<MockPSM> for MockCacheChange {
// //     //     fn kind(&self) -> rust_rtps_pim::structure::types::ChangeKind {
// //     //         todo!()
// //     //     }

// //     //     fn writer_guid(&self) -> &MockGuid {
// //     //         todo!()
// //     //     }

// //     //     fn instance_handle(&self) -> &() {
// //     //         todo!()
// //     //     }

// //     //     fn sequence_number(&self) -> &i64 {
// //     //         todo!()
// //     //     }

// //     //     fn data_value(&self) -> &Vec<u8> {
// //     //         todo!()
// //     //     }

// //     //     fn inline_qos(&self) -> &MockParameterList {
// //     //         todo!()
// //     //     }
// //     // }

// //     // struct MockHistoryCache;

// //     // impl RTPSHistoryCache<MockPSM> for MockHistoryCache {
// //     //     type CacheChange = MockCacheChange;

// //     //     fn new() -> Self
// //     //     where
// //     //         Self: Sized,
// //     //     {
// //     //         todo!()
// //     //     }

// //     //     fn add_change(&mut self, change: Self::CacheChange) {
// //     //         todo!()
// //     //     }

// //     //     fn remove_change(&mut self, seq_num: &i64) {
// //     //         todo!()
// //     //     }

// //     //     fn get_change(&self, seq_num: &i64) -> Option<&Self::CacheChange> {
// //     //         todo!()
// //     //     }

// //     //     fn get_seq_num_min(&self) -> Option<&i64> {
// //     //         todo!()
// //     //     }

// //     //     fn get_seq_num_max(&self) -> Option<&i64> {
// //     //         todo!()
// //     //     }
// //     // }

// //     struct MockStatelessWriter;

// //     // impl RTPSStatelessWriter<MockPSM> for MockStatelessWriter {
// //     //     type ReaderLocatorPIM = MockReaderLocator;

// //     //     fn reader_locators(&mut self) -> (&mut [Self::ReaderLocatorPIM], &Self::HistoryCacheType) {
// //     //         todo!()
// //     //     }

// //     //     fn reader_locator_add(&mut self, a_locator: Self::ReaderLocatorPIM) {
// //     //         todo!()
// //     //     }

// //     //     fn reader_locator_remove(&mut self, a_locator: &MockLocator) {
// //     //         todo!()
// //     //     }

// //     //     fn unsent_changes_reset(&mut self) {
// //     //         todo!()
// //     //     }
// //     // }

// //     // impl RTPSWriter<MockPSM> for MockStatelessWriter {
// //     //     type HistoryCacheType = MockHistoryCache;

// //     //     fn push_mode(&self) -> bool {
// //     //         todo!()
// //     //     }

// //     //     fn heartbeat_period(&self) -> &() {
// //     //         todo!()
// //     //     }

// //     //     fn nack_response_delay(&self) -> &() {
// //     //         todo!()
// //     //     }

// //     //     fn nack_suppression_duration(&self) -> &() {
// //     //         todo!()
// //     //     }

// //     //     fn last_change_sequence_number(&self) -> &i64 {
// //     //         todo!()
// //     //     }

// //     //     fn data_max_size_serialized(&self) -> i32 {
// //     //         todo!()
// //     //     }

// //     //     fn writer_cache(&self) -> &Self::HistoryCacheType {
// //     //         todo!()
// //     //     }

// //     //     fn writer_cache_mut(&mut self) -> &mut Self::HistoryCacheType {
// //     //         todo!()
// //     //     }

// //     //     fn new_change(
// //     //     &mut self,
// //     //     _kind: rust_rtps_pim::structure::types::ChangeKind,
// //     //     _data: Vec<u8>,
// //     //     _inline_qos: MockParameterList,
// //     //     _handle: (),
// //     //     ) -> <Self::HistoryCacheType as rust_rtps_pim::structure::RTPSHistoryCache<MockPSM>>::CacheChange{
// //     //         MockCacheChange
// //     //     }
// //     // }

// //     // impl RTPSEndpoint<MockPSM> for MockStatelessWriter {
// //     //     fn topic_kind(&self) -> &rust_rtps_pim::structure::types::TopicKind {
// //     //         todo!()
// //     //     }

// //     //     fn reliability_level(&self) -> &rust_rtps_pim::structure::types::ReliabilityKind {
// //     //         todo!()
// //     //     }

// //     //     fn unicast_locator_list(&self) -> &[MockLocator] {
// //     //         todo!()
// //     //     }

// //     //     fn multicast_locator_list(&self) -> &[MockLocator] {
// //     //         todo!()
// //     //     }
// //     // }

// //     // impl RTPSEntity<MockPSM> for MockStatelessWriter {
// //     //     fn guid(&self) -> &MockGuid {
// //     //         todo!()
// //     //     }
// //     // }

// //     struct MockTransport;

// //     impl Transport<MockPSM> for MockTransport {
// //         fn write<'a>(&mut self, message: &MockRtpsMessage, destination_locator: &MockLocator)
// //         where
// //             MockPSM: RTPSMessagePIM<'a, MockPSM>,
// //         {
// //             todo!()
// //         }

// //         fn read<'a>(&'a self) -> Option<(MockRtpsMessage, MockLocator)>
// //         where
// //             MockPSM: RTPSMessagePIM<'a, MockPSM>,
// //         {
// //             todo!()
// //         }

// //         fn unicast_locator_list(&self) -> &[MockLocator] {
// //             todo!()
// //         }

// //         fn multicast_locator_list(&self) -> &[MockLocator] {
// //             todo!()
// //         }
// //     }

// //     #[test]
// //     fn send_data_message() {
// //         let mut writer = MockStatelessWriter;
// //         let mut transport = MockTransport;
// //         //send_data(&mut writer, &mut transport);
// //     }
// // }

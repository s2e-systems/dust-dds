pub mod submessage_elements;
pub mod submessages;
pub mod types;

use crate::structure::types::{GuidPrefix, ProtocolVersion, VendorId};

use self::{submessages::{RtpsSubmessageType, RtpsSubmessagePIM}, types::{SubmessageFlag, SubmessageKindPIM}};

pub trait RtpsMessageHeaderType {
    type ProtocolIdType;

    const PROTOCOL_RTPS: Self::ProtocolIdType;

    fn new(
        // protocol: Self::ProtocolIdType,
        version: &ProtocolVersion,
        vendor_id: &VendorId,
        guid_prefix: &GuidPrefix,
    ) -> Self;

    fn protocol(&self) -> Self::ProtocolIdType;
    fn version(&self) -> ProtocolVersion;
    fn vendor_id(&self) -> VendorId;
    fn guid_prefix(&self) -> GuidPrefix;
}

pub trait RtpsSubmessageHeaderType<PSM>
where
    PSM: SubmessageKindPIM,
{
    fn submessage_id(&self) -> PSM::SubmessageKindType;
    fn flags(&self) -> [SubmessageFlag; 8];
    fn submessage_length(&self) -> u16;
}

pub trait Submessage {
    type RtpsSubmessageHeaderType;
    fn submessage_header(&self) -> Self::RtpsSubmessageHeaderType;
}

pub trait RTPSMessage<'a> {
    type RtpsMessageHeaderType: RtpsMessageHeaderType;
    type PSM: RtpsSubmessagePIM<'a>;
    type Constructed;

    fn new<T: IntoIterator<Item = RtpsSubmessageType<'a, Self::PSM>>>(
        header: &Self::RtpsMessageHeaderType,
        submessages: T,
    ) -> Self::Constructed;

    fn header(&self) -> Self::RtpsMessageHeaderType;

    fn submessages(&self) -> &[RtpsSubmessageType<'a, Self::PSM>];
}

// #[cfg(test)]
// mod tests {
//     use super::submessages::DataSubmessage;
//     use super::*;

//     struct MockEntityIdSubmessageElement;

//     struct MockSequenceNumberSubmessageElement;
//     struct MockParameterListSubmessageElement;
//     struct MockSerializedDataSubmessageElement;
//     struct MockDataSubmessage;

//     impl Submessage for MockDataSubmessage {
//         type RtpsSubmessageHeaderType = ();
//         fn submessage_header(&self) -> Self::RtpsSubmessageHeaderType {
//             todo!()
//         }
//     }
//     impl<'a> DataSubmessage<'a> for MockDataSubmessage {
//         type EntityIdSubmessageElementType = MockEntityIdSubmessageElement;
//         type SequenceNumberSubmessageElementType = MockSequenceNumberSubmessageElement;
//         type ParameterListSubmessageElementType = MockParameterListSubmessageElement;
//         type SerializedDataSubmessageElementType = MockSerializedDataSubmessageElement;

//         fn new(
//             _endianness_flag: SubmessageFlag,
//             _inline_qos_flag: SubmessageFlag,
//             _data_flag: SubmessageFlag,
//             _key_flag: SubmessageFlag,
//             _non_standard_payload_flag: SubmessageFlag,
//             _reader_id: Self::EntityIdSubmessageElementType,
//             _writer_id: Self::EntityIdSubmessageElementType,
//             _writer_sn: Self::SequenceNumberSubmessageElementType,
//             _inline_qos: Self::ParameterListSubmessageElementType,
//             _serialized_payload: Self::SerializedDataSubmessageElementType,
//         ) -> Self {
//             todo!()
//         }

//         fn endianness_flag(&self) -> SubmessageFlag {
//             todo!()
//         }

//         fn inline_qos_flag(&self) -> SubmessageFlag {
//             todo!()
//         }

//         fn data_flag(&self) -> SubmessageFlag {
//             todo!()
//         }

//         fn key_flag(&self) -> SubmessageFlag {
//             todo!()
//         }

//         fn non_standard_payload_flag(&self) -> SubmessageFlag {
//             todo!()
//         }

//         fn reader_id(&self) -> &Self::EntityIdSubmessageElementType {
//             todo!()
//         }

//         fn writer_id(&self) -> &Self::EntityIdSubmessageElementType {
//             todo!()
//         }

//         fn writer_sn(&self) -> &Self::SequenceNumberSubmessageElementType {
//             todo!()
//         }

//         fn inline_qos(&self) -> &Self::ParameterListSubmessageElementType {
//             todo!()
//         }

//         fn serialized_payload(&self) -> &Self::SerializedDataSubmessageElementType {
//             todo!()
//         }
//     }

//     struct MockPSM;
//     impl AckNackSubmessagePIM for MockPSM {
//         type AckNackSubmessageType = ();
//     }
//     impl<'a> DataSubmessagePIM<'a> for MockPSM {
//         type DataSubmessageType = MockDataSubmessage;
//     }
//     impl<'a> DataFragSubmessagePIM<'a> for MockPSM {
//         type DataFragSubmessageType = ();
//     }
//     impl GapSubmessagePIM for MockPSM {
//         type GapSubmessageType = ();
//     }
//     impl HeartbeatSubmessagePIM for MockPSM {
//         type HeartbeatSubmessageType = ();
//     }
//     impl HeartbeatFragSubmessagePIM for MockPSM {
//         type HeartbeatFragSubmessageType = ();
//     }
//     impl InfoDestinationSubmessagePIM for MockPSM {
//         type InfoDestinationSubmessageType = ();
//     }
//     impl InfoReplySubmessagePIM for MockPSM {
//         type InfoReplySubmessageType = ();
//     }
//     impl InfoSourceSubmessagePIM for MockPSM {
//         type InfoSourceSubmessageType = ();
//     }
//     impl InfoTimestampSubmessagePIM for MockPSM {
//         type InfoTimestampSubmessageType = ();
//     }
//     impl NackFragSubmessagePIM for MockPSM {
//         type NackFragSubmessageType = ();
//     }
//     impl PadSubmessagePIM for MockPSM {
//         type PadSubmessageType = ();
//     }

//     struct MockRTPSMessage<'a> {
//         header: MockRtpsMessageHeader,
//         submessages: Option<RtpsSubmessageType<'a, MockPSM>>,
//     }
//     impl<'a> RTPSMessage<'a> for MockRTPSMessage<'a> {
//         type RtpsMessageHeaderType = MockRtpsMessageHeader;
//         type PSM = MockPSM;
//         type SubmessageVectorType = Option<RtpsSubmessageType<'a, Self::PSM>>;

//         fn new(
//             header: Self::RtpsMessageHeaderType,
//             // protocol: <Self::RtpsMessageHeaderType as RtpsMessageHeaderType>::ProtocolIdType,
//             // version: <Self::RtpsMessageHeaderType as RtpsMessageHeaderType>::ProtocolVersionType,
//             // vendor_id: <Self::RtpsMessageHeaderType as RtpsMessageHeaderType>::VendorIdType,
//             // guid_prefix: <Self::RtpsMessageHeaderType as RtpsMessageHeaderType>::GuidPrefixType,
//             submessages: Self::SubmessageVectorType,
//         ) -> Self {
//             Self {
//                 header,
//                 submessages,
//             }
//         }

//         fn header(&self) -> Self::RtpsMessageHeaderType {
//             todo!()
//         }

//         fn submessages(&self) -> &[RtpsSubmessageType<'a, Self::PSM>] {
//             todo!()
//         }
//     }

//     #[test]
//     fn constructor() {
//         let header = MockRtpsMessageHeader::new();
//         let submessages = None;
//         let message = MockRTPSMessage::new(header, submessages);
//     }

//     struct MockRtpsMessageHeader{
//         protocol: (),
//         version: (),
//         vendor_id: (),
//         guid_prefix: (),
//     }
//     impl RtpsMessageHeaderType for MockRtpsMessageHeader {
//         type ProtocolIdType = ();
//         type ProtocolVersionType = ();
//         type VendorIdType = ();
//         type GuidPrefixType = ();

//         const PROTOCOL_RTPS: Self::ProtocolIdType = ();

//         fn new(
//         // protocol: Self::ProtocolIdType,
//         // version: Self::ProtocolVersionType,
//         // vendor_id: Self::VendorIdType,
//         // guid_prefix: Self::GuidPrefixType,
//     ) -> Self {
//         Self {
//             protocol: (),
//             version: (),
//             vendor_id: (),
//             guid_prefix: (),
//         }
//     }

//         fn protocol(&self) -> &Self::ProtocolIdType {
//         todo!()
//     }

//         fn version(&self) -> &Self::ProtocolVersionType {
//         todo!()
//     }

//         fn vendor_id(&self) -> &Self::VendorIdType {
//         todo!()
//     }

//         fn guid_prefix(&self) -> &Self::GuidPrefixType {
//         todo!()
//     }
//     }
// }

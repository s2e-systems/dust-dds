mod helpers;

mod data_submessage;
mod ack_nack_submessage;
mod data_frag_submessage;
mod gap_submessage;
mod heartbeat_submessage;
mod heartbeat_frag_submessage;
mod info_destination_submessage;
mod info_reply_submessage;
mod info_source_submessage;
mod info_timestamp_submessage;
mod nack_frag_submessage;

use std::collections::VecDeque;
use num_derive::FromPrimitive;
use serde_derive::{Deserialize, Serialize};

use helpers::{deserialize, endianess, EndianessFlag, MINIMUM_RTPS_MESSAGE_SIZE};

use crate::types::*;

use ack_nack_submessage::parse_ack_nack_submessage;
use data_submessage::parse_data_submessage;
use data_frag_submessage::parse_data_frag_submessage;
use gap_submessage::parse_gap_submessage;
use heartbeat_submessage::parse_heartbeat_submessage;
use heartbeat_frag_submessage::parse_heartbeat_frag_submessage;
use info_destination_submessage::parse_info_dst_submessage;
use info_reply_submessage::parse_info_reply_submessage;
use info_source_submessage::parse_info_source_submessage;
use info_timestamp_submessage::parse_info_timestamp_submessage;
use nack_frag_submessage::parse_nack_frag_submessage;

pub use ack_nack_submessage::AckNack;
pub use data_submessage::{Payload, Data};
pub use data_frag_submessage::DataFrag;
pub use gap_submessage::Gap;
pub use heartbeat_submessage::Heartbeat;
pub use heartbeat_frag_submessage::HeartbeatFrag;
pub use info_destination_submessage::InfoDst;
pub use info_reply_submessage::InfoReply;
pub use info_source_submessage::InfoSrc;
pub use info_timestamp_submessage::InfoTs;
pub use nack_frag_submessage::NackFrag;

#[derive(Debug)]
pub enum ErrorMessage {
    MessageTooSmall,
    InvalidHeader,
    RtpsMajorVersionUnsupported,
    RtpsMinorVersionUnsupported,
    InvalidSubmessageHeader,
    InvalidSubmessage,
    InvalidKeyAndDataFlagCombination,
    CdrError(cdr::Error),
    InvalidTypeConversion,
    DeserializationMessageSizeTooSmall,
}

impl From<cdr::Error> for ErrorMessage {
    fn from(error: cdr::Error) -> Self {
        ErrorMessage::CdrError(error)
    }
}

type Result<T> = std::result::Result< T, ErrorMessage>;

pub const RTPS_MAJOR_VERSION: u8 = 2;
pub const RTPS_MINOR_VERSION: u8 = 4;

#[derive(FromPrimitive)]
enum SubmessageKind {
    Pad = 0x01,
    AckNack = 0x06,
    Heartbeat = 0x07,
    Gap = 0x08,
    InfoTimestamp = 0x09,
    InfoSource = 0x0c,
    InfoReplyIP4 = 0x0d,
    InfoDestination = 0x0e,
    InfoReply = 0x0f,
    NackFrag = 0x12,
    HeartbeatFrag = 0x13,
    Data = 0x15,
    DataFrag = 0x16,
}

// enum ProtocolVersionT {
//     PROTOCOLVERSION_1_0,
//     PROTOCOLVERSION_1_1,
//     PROTOCOLVERSION_2_0,
//     PROTOCOLVERSION_2_1,
//     PROTOCOLVERSION_2_2,
//     PROTOCOLVERSION_2_3,
//     PROTOCOLVERSION_2_4,
// }

// TIME_ZERO: seconds = 0, fraction = 0
// TIME_INVALID: seconds = 0xffffffff, fraction = 0xffffffff
// TIME_INFINITE: seconds = 0xffffffff, fraction = 0xfffffffe

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct SubmessageHeader {
    submessage_id: u8,
    flags: u8,
    submessage_length: u16,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum InlineQosPid {
    Pad = 0x0000, 
    Sentinel = 0x0001,
    TopicName = 0x0005,
    Durability = 0x001d,
    Presentation = 0x0021,
    Deadline = 0x0023,
    LatencyBudget = 0x0027,
    Ownership = 0x001f,
    OwnershipStrength = 0x0006,
    Liveliness = 0x001b,
    Partition = 0x0029,
    Reliability = 0x001a,
    TransportPriority = 0x0049,
    Lifespan = 0x002b,
    DestinationOrder = 0x0025,
    ContentFilterInfo = 0x0055,
    CoherentSet = 0x0056,
    DirectedWrite = 0x0057,
    OriginalWriterInfo = 0x0061,
    GroupCoherentSet = 0x0063,
    GroupSeqNum = 0x0064,
    WriterGroupInfo = 0x0065,
    SecureWriterGroupInfo = 0x0066,
    KeyHash = 0x0070,
    StatusInfo = 0x0071,
}

pub enum ParameterId {
    Pad = 0x0000, //N/A
    Sentinel = 0x0001, //N/A
    UserData = 0x002c, //UserDataQosPolicy
    TopicName = 0x0005, //string<256>
    TypeName = 0x0007, //string<256>
    GroupData = 0x002d, //GroupDataQosPolicy
    TopicData = 0x002e, //TopicDataQosPolicy
    Durability = 0x001d, //DurabilityQosPolicy
    DurabilityService = 0x001e, //DurabilityServiceQosPolicy
    Deadline = 0x0023, //DeadlineQosPolicy
    LatencyBudget = 0x0027, //LatencyBudgetQosPolicy
    Liveliness = 0x001b, //LivelinessQosPolicy
    Reliability = 0x001a, //ReliabilityQosPolicy
    Lifespan = 0x002b, //LifespanQosPolicy
    DestinationOrder = 0x0025, //DestinationOrderQosPolicy
    History = 0x0040, //HistoryQosPolicy
    ResourceLimits = 0x0041, //ResourceLimitsQosPolicy
    Ownership = 0x001f, //OwnershipQosPolicy
    OwnershipStrength = 0x0006, //OwnershipStrengthQosPolicy
    Presentation = 0x0021, //PresentationQosPolicy
    Partition = 0x0029, //PartitionQosPolicy
    TimeBasedFilter = 0x0004, //TimeBasedFilterQosPolicy
    TransportPriority = 0x0049, //TransportPriorityQoSPolicy
    DomainId = 0x000f, //DomainId_t
    DomainTag = 0x4014, //string<256>
    ProtocolVersion = 0x0015, //ProtocolVersion_t
    VendorId = 0x0016, //VendorId_t
    UnicastLocator = 0x002f, //Locator_t
    MulticastLocator = 0x0030, //Locator_t
    DefaultUnicastLocator = 0x0031, //Locator_t
    DefaultMulticastLocator = 0x0048, //Locator_t
    MetatrafficUnicastLocator = 0x0032, //Locator_t
    MetatrafficMulticastLocator = 0x0033, //Locator_t
    ExpectsInlineQoS = 0x0043, //boolean
    ParticipantManualLivelinessCount = 0x0034, //Count_t
    ParticipantLeaseDuration = 0x0002, //Duration_t
    ContentFilterProperty = 0x0035, //ContentFilterProperty_t
    ParticipantGuid = 0x0050, //GUID_t
    GroupGuid =  0x0052, //GUID_t
    BuiltinEndpointSet = 0x0058, //BuiltinEndpointSet_t
    BuiltinEndpointQoS = 0x0077, //BuiltinEndpointQos_t
    PropertyList = 0x0059, //sequence<Property_t>
    TypeMaxSizeSerialized = 0x0060, //long
    EntityName = 0x0062, //EntityName_t
    EndpointGuid = 0x005a, //GUID_t
}


#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct MessageHeader {
    protocol_name: [char;4],
    protocol_version: ProtocolVersion,
    vendor_id: VendorId,
    guid_prefix: GuidPrefix,
}

//TODO: InfoReplyIP4

#[derive(PartialEq, Debug)]
pub enum SubMessageType {
    AckNackSubmessage(AckNack),
    DataSubmessage(Data),
    DataFragSubmessage(DataFrag),
    GapSubmessage(Gap),
    HeartbeatSubmessage(Heartbeat),
    HeartbeatFragSubmessage(HeartbeatFrag),
    InfoDstSubmessage(InfoDst),
    InfoReplySubmessage(InfoReply),
    InfoSrcSubmessage(InfoSrc),
    InfoTsSubmessage(InfoTs),
    PadSubmessage(()),
    NackFragSubmessage(NackFrag),
}

pub struct RtpsMessage {
    guid_prefix: GuidPrefix,
    vendor_id: VendorId,
    protocol_version: ProtocolVersion,
    submessages: VecDeque<SubMessageType>,
}

impl RtpsMessage {
    pub fn new(guid_prefix: GuidPrefix, vendor_id: VendorId, protocol_version: ProtocolVersion) -> RtpsMessage {
        RtpsMessage {
            guid_prefix,
            vendor_id,
            protocol_version,
            submessages: VecDeque::new(),
        }
    }

    pub fn add_submessage(&mut self, submessage: SubMessageType) {
        self.submessages.push_back(submessage);
    }

    pub fn get_guid_prefix(&self) -> &GuidPrefix {
        &self.guid_prefix
    }

    pub fn get_vendor_id(&self) -> &VendorId {
        &self.vendor_id
    }

    pub fn get_protocol_version(&self) -> &ProtocolVersion {
        &self.protocol_version
    }

    pub fn get_submessages(&mut self) -> &mut VecDeque<SubMessageType> {
        &mut self.submessages
    }

    pub fn take(self) -> (GuidPrefix, VendorId, ProtocolVersion, VecDeque<SubMessageType>) {
        (self.guid_prefix, self.vendor_id, self.protocol_version, self.submessages)
    }
}

pub fn parse_rtps_message(message : &[u8]) -> Result<RtpsMessage>{
    const MESSAGE_HEADER_FIRST_INDEX: usize = 0;
    const MESSAGE_HEADER_LAST_INDEX: usize = 19;
    const PROTOCOL_VERSION_FIRST_INDEX : usize = 4;
    const PROTOCOL_VERSION_LAST_INDEX : usize = 5;

    if message.len() < MINIMUM_RTPS_MESSAGE_SIZE {
        return Err(ErrorMessage::MessageTooSmall);
    }

    let message_header = deserialize::<MessageHeader>(message, &MESSAGE_HEADER_FIRST_INDEX, &MESSAGE_HEADER_LAST_INDEX, &EndianessFlag::BigEndian /* Endianness not relevant for the header. Only octets */)?;

    if  message_header.protocol_name[0] != 'R' ||
        message_header.protocol_name[1] != 'T' ||
        message_header.protocol_name[2] != 'P' || 
        message_header.protocol_name[3] != 'S' {
        return Err(ErrorMessage::InvalidHeader);
    }

    if message_header.protocol_version.major != 2 {
        return Err(ErrorMessage::RtpsMajorVersionUnsupported);
    }
    if message_header.protocol_version.minor > RTPS_MINOR_VERSION {
        return Err(ErrorMessage::RtpsMinorVersionUnsupported);
    }

    const RTPS_SUBMESSAGE_HEADER_SIZE: usize = 4;

    let mut submessage_vector = VecDeque::with_capacity(4);

    let mut submessage_first_index = MINIMUM_RTPS_MESSAGE_SIZE;
    while submessage_first_index < message.len() {
        const SUBMESSAGE_FLAGS_INDEX_OFFSET : usize = 1;
        
        let submessage_header_first_index = submessage_first_index;
        //In the deserialize library the comparisons are always inclusive of last element (-1 is required)
        let submessage_header_last_index = submessage_header_first_index + RTPS_SUBMESSAGE_HEADER_SIZE - 1;

        if submessage_header_last_index >= message.len() {
            return Err(ErrorMessage::InvalidSubmessageHeader);
        }

        let submessage_endianess = endianess(&message[submessage_header_first_index + SUBMESSAGE_FLAGS_INDEX_OFFSET])?;
        
        let submessage_header = deserialize::<SubmessageHeader>(message, &submessage_header_first_index, &submessage_header_last_index, &submessage_endianess)?;

        let submessage_payload_first_index = submessage_header_last_index + 1;
        let submessage_payload_last_index = submessage_payload_first_index + submessage_header.submessage_length as usize - 1;
        if submessage_payload_last_index >= message.len() {
            return Err(ErrorMessage::MessageTooSmall); // TODO: Replace error by invalid message
        }

        let submessage = 
        match num::FromPrimitive::from_u8(submessage_header.submessage_id).ok_or(ErrorMessage::InvalidSubmessageHeader)? {
            SubmessageKind::AckNack => SubMessageType::AckNackSubmessage(parse_ack_nack_submessage(&message[submessage_payload_first_index..=submessage_payload_last_index], &submessage_header.flags)?),
            SubmessageKind::Data => SubMessageType::DataSubmessage(parse_data_submessage(&message[submessage_payload_first_index..=submessage_payload_last_index], &submessage_header.flags)?),
            SubmessageKind::DataFrag => SubMessageType::DataFragSubmessage(parse_data_frag_submessage(&message[submessage_payload_first_index..=submessage_payload_last_index], &submessage_header.flags)?),
            SubmessageKind::Gap => SubMessageType::GapSubmessage(parse_gap_submessage(&message[submessage_payload_first_index..=submessage_payload_last_index], &submessage_header.flags)?),
            SubmessageKind::Heartbeat => SubMessageType::HeartbeatSubmessage(parse_heartbeat_submessage(&message[submessage_payload_first_index..=submessage_payload_last_index], &submessage_header.flags)?),
            SubmessageKind::HeartbeatFrag => SubMessageType::HeartbeatFragSubmessage(parse_heartbeat_frag_submessage(&message[submessage_payload_first_index..=submessage_payload_last_index], &submessage_header.flags)?),
            SubmessageKind::InfoDestination => SubMessageType::InfoDstSubmessage(parse_info_dst_submessage(&message[submessage_payload_first_index..=submessage_payload_last_index], &submessage_header.flags)?),
            SubmessageKind::InfoReply => SubMessageType::InfoReplySubmessage(parse_info_reply_submessage(&message[submessage_payload_first_index..=submessage_payload_last_index], &submessage_header.flags)?),
            SubmessageKind::InfoSource => SubMessageType::InfoSrcSubmessage(parse_info_source_submessage(&message[submessage_payload_first_index..=submessage_payload_last_index], &submessage_header.flags)?),
            SubmessageKind::InfoTimestamp => SubMessageType::InfoTsSubmessage(parse_info_timestamp_submessage(&message[submessage_payload_first_index..=submessage_payload_last_index], &submessage_header.flags)?),
            SubmessageKind::Pad => SubMessageType::PadSubmessage(()),
            SubmessageKind::NackFrag => SubMessageType::NackFragSubmessage(parse_nack_frag_submessage(&message[submessage_payload_first_index..=submessage_payload_last_index], &submessage_header.flags)?),
            SubmessageKind::InfoReplyIP4 => unimplemented!(),
        };
        
        submessage_vector.push_back(submessage);

        submessage_first_index = submessage_first_index + RTPS_SUBMESSAGE_HEADER_SIZE + submessage_header.submessage_length as usize;
    }

    Ok( RtpsMessage {
        guid_prefix: message_header.guid_prefix,
        vendor_id: message_header.vendor_id,
        protocol_version: message_header.protocol_version,
        submessages: submessage_vector,})
}


#[cfg(test)]
mod tests{
    use super::*;

    use cdr::{BigEndian, Infinite};

    #[test]
    fn test_parse_valid_message_header_only() {
        let message_example = MessageHeader{
            protocol_name: ['R','T','P','S'],
            protocol_version: ProtocolVersion{major: 2, minor: 4},
            vendor_id: [100,210],
            guid_prefix: [10,11,12,13,14,15,16,17,18,19,20,21],};

        let serialized = cdr::ser::serialize_data::<_, _, BigEndian>(&message_example, Infinite).unwrap();

        let parse_result = parse_rtps_message(&serialized).unwrap();
        
        assert_eq!(parse_result.guid_prefix,  [10,11,12,13,14,15,16,17,18,19,20,21]);
        assert_eq!(parse_result.submessages, vec!());
    }

    #[test]
    fn test_parse_too_small_message() {
        let serialized = [0, 1, 2, 3];

        let parse_result = parse_rtps_message(&serialized);

        if let Err(ErrorMessage::MessageTooSmall) = parse_result {
            assert!(true);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_parse_unsupported_version_header() {
        // Unsupported major version
        let message_example = MessageHeader{
            protocol_name: ['R','T','P','S'],
            protocol_version: ProtocolVersion{major: 1, minor: 4},
            vendor_id: [100,210],
            guid_prefix: [10,11,12,13,14,15,16,17,18,19,20,21],};

        let serialized = cdr::ser::serialize_data::<_, _, BigEndian>(&message_example, Infinite).unwrap();

        let parse_result = parse_rtps_message(&serialized);

        if let Err(ErrorMessage::RtpsMajorVersionUnsupported) = parse_result {
            assert!(true);
        } else {
            assert!(false);
        }

        // Unsupported minor version
        let message_example = MessageHeader{
            protocol_name: ['R','T','P','S'],
            protocol_version: ProtocolVersion{major: 2, minor: 5},
            vendor_id: [100,210],
            guid_prefix: [10,11,12,13,14,15,16,17,18,19,20,21],};

        let serialized = cdr::ser::serialize_data::<_, _, BigEndian>(&message_example, Infinite).unwrap();

        let parse_result = parse_rtps_message(&serialized);

        if let Err(ErrorMessage::RtpsMinorVersionUnsupported) = parse_result {
            assert!(true);
        } else {
            assert!(false);
        }
        
        // Unsupported major and minor version
        let message_example = MessageHeader{
            protocol_name: ['R','T','P','S'],
            protocol_version: ProtocolVersion{major: 3, minor: 10},
            vendor_id: [100,210],
            guid_prefix: [10,11,12,13,14,15,16,17,18,19,20,21],};

        let serialized = cdr::ser::serialize_data::<_, _, BigEndian>(&message_example, Infinite).unwrap();

        let parse_result = parse_rtps_message(&serialized);

        if let Err(ErrorMessage::RtpsMajorVersionUnsupported) = parse_result {
            assert!(true);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_parse_different_rtps_messages() {
        let rtps_message_info_ts_and_data = [
            0x52, 0x54, 0x50, 0x53,
            0x02, 0x01, 0x01, 0x02,
            0x7f, 0x20, 0xf7, 0xd7,
            0x00, 0x00, 0x01, 0xbb,
            0x00, 0x00, 0x00, 0x01,
            0x09, 0x01, 0x08, 0x00,
            0x9e, 0x81, 0xbc, 0x5d,
            0x97, 0xde, 0x48, 0x26,
            0x15, 0x07, 0x1c, 0x01,
            0x00, 0x00, 0x10, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x00, 0xc2,
            0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x70, 0x00, 0x10, 0x00,
            0x7f, 0x20, 0xf7, 0xd7,
            0x00, 0x00, 0x01, 0xbb,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x01, 0xc1,
            0x01, 0x00, 0x00, 0x00,
            0x00, 0x03, 0x00, 0x00,
            0x15, 0x00, 0x04, 0x00,
            0x02, 0x01, 0x00, 0x00,
            0x16, 0x00, 0x04, 0x00,
            0x01, 0x02, 0x00, 0x00,
            0x31, 0x00, 0x18, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0xf3, 0x1c, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0xc0, 0xa8, 0x02, 0x04,
            0x32, 0x00, 0x18, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0xf2, 0x1c, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0xc0, 0xa8, 0x02, 0x04,
            0x02, 0x00, 0x08, 0x00,
            0x0b, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x50, 0x00, 0x10, 0x00,
            0x7f, 0x20, 0xf7, 0xd7,
            0x00, 0x00, 0x01, 0xbb,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x01, 0xc1,
            0x58, 0x00, 0x04, 0x00,
            0x15, 0x04, 0x00, 0x00,
            0x00, 0x80, 0x04, 0x00,
            0x15, 0x00, 0x00, 0x00,
            0x07, 0x80, 0x5c, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x2f, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x50, 0x00, 0x00, 0x00,
            0x42, 0x00, 0x00, 0x00,
            0x44, 0x45, 0x53, 0x4b,
            0x54, 0x4f, 0x50, 0x2d,
            0x4f, 0x52, 0x46, 0x44,
            0x4f, 0x53, 0x35, 0x2f,
            0x36, 0x2e, 0x31, 0x30,
            0x2e, 0x32, 0x2f, 0x63,
            0x63, 0x36, 0x66, 0x62,
            0x39, 0x61, 0x62, 0x33,
            0x36, 0x2f, 0x39, 0x30,
            0x37, 0x65, 0x66, 0x66,
            0x30, 0x32, 0x65, 0x33,
            0x2f, 0x22, 0x78, 0x38,
            0x36, 0x5f, 0x36, 0x34,
            0x2e, 0x77, 0x69, 0x6e,
            0x2d, 0x76, 0x73, 0x32,
            0x30, 0x31, 0x35, 0x22,
            0x2f, 0x00, 0x00, 0x00,
            0x25, 0x80, 0x0c, 0x00,
            0xd7, 0xf7, 0x20, 0x7f,
            0xbb, 0x01, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00];

        let parse_result = parse_rtps_message(&rtps_message_info_ts_and_data).unwrap();

        assert_eq!(parse_result.guid_prefix, [0x7f, 0x20, 0xf7, 0xd7, 0x00, 0x00, 0x01, 0xbb, 0x00, 0x00, 0x00, 0x01,]);
        assert_eq!(parse_result.submessages.len(),2);
        if let SubMessageType::InfoTsSubmessage(ts_message) = &parse_result.submessages[0] {
            assert_eq!(*ts_message.get_timestamp(), Some(Time{seconds: 1572635038, fraction: 642309783,}));
        } else {
            assert!(false);
        }
        
        if let SubMessageType::DataSubmessage(data_message) = &parse_result.submessages[1] {
            assert_eq!(*data_message.reader_id(), EntityId::new([0,0,0], 0));
            assert_eq!(*data_message.writer_id(), EntityId::new([0,1,0], 0xc2)); //ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER = {{00,01,00},c2}
            assert_eq!(*data_message.writer_sn(), 1);
            assert_eq!(*data_message.inline_qos(), Some(vec!(Parameter{
                parameter_id: 112,
                value: vec!(127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1, 0, 0, 1, 193),})) );
            assert_eq!(*data_message.serialized_payload(),Payload::Data(vec!(0, 3, 0, 0, 21, 0, 4, 0, 2, 1, 0, 0, 22, 0, 4, 0, 1, 2, 0, 0, 49, 0, 24, 0, 1, 0, 0, 0, 243, 28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 168, 2, 4, 50, 0, 24, 0, 1, 0, 0, 0, 242, 28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 168, 2, 4, 2, 0, 8, 0, 11, 0, 0, 0, 0, 0, 0, 0, 80, 0, 16, 0, 127, 32, 247, 215, 0, 0, 1, 187, 0, 0, 0, 1, 0, 0, 1, 193, 88, 0, 4, 0, 21, 4, 0, 0, 0, 128, 4, 0, 21, 0, 0, 0, 7, 128, 92, 0, 0, 0, 0, 0, 47, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 80, 0, 0, 0, 66, 0, 0, 0, 68, 69, 83, 75, 84, 79, 80, 45, 79, 
                82, 70, 68, 79, 83, 53, 47, 54, 46, 49, 48, 46, 50, 47, 99, 99, 54, 102, 98, 57, 97, 98, 51, 54, 47, 57, 48, 55, 101, 102, 102, 48, 50, 101, 51, 47, 34, 120, 56, 54, 95, 54, 52, 46, 119, 105, 110, 45, 118, 115, 50, 48, 49, 53, 34, 47, 0, 0, 0, 37, 128, 12, 0, 215, 247, 32, 127, 187, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0)));

        } else {
            assert!(false);
        }
    }
}
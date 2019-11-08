extern crate serde;
extern crate serde_derive;
extern crate num;
extern crate num_derive;

use serde_derive::{Deserialize, Serialize};

use num_derive::FromPrimitive;

use cdr::{
    LittleEndian, BigEndian, CdrLe, CdrBe, PlCdrLe, PlCdrBe, Error, Infinite,
};

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
}

impl From<cdr::Error> for ErrorMessage {
    fn from(error: cdr::Error) -> Self {
        ErrorMessage::CdrError(error)
    }
}

type Result<T> = std::result::Result< T, ErrorMessage>;

fn deserialize<'de,T>(message: &[u8], start_index: &usize, end_index: &usize, endianess: &EndianessFlag) -> T 
    where T: serde::de::Deserialize<'de>
{
    if *endianess == EndianessFlag::BigEndian {
        cdr::de::deserialize_data::<T, BigEndian>(&message[*start_index..=*end_index]).unwrap()
    } else {
        cdr::de::deserialize_data::<T, LittleEndian>(&message[*start_index..=*end_index]).unwrap()
    }
}

const RTPS_MINOR_VERSION: u8 = 4;

// All sizes are in octets
const MINIMUM_RTPS_MESSAGE_SIZE: usize = 20;

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

#[derive(FromPrimitive, PartialEq, Debug)]
enum EndianessFlag {
    BigEndian = 0,
    LittleEndian = 1,
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
struct TimeT {
    seconds: u32,
    fraction: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct SubmessageHeader {
    submessage_id: u8,
    flags: u8,
    submessage_length: u16,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Submessage<T> {
    header: SubmessageHeader,
    submessage: T,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct InfoTs {
    timestamp: Option<TimeT>, 
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct InfoSrc {
    protocol_version: ProtocolVersion,
    vendor_id: [u8;2],
    guid_prefix: [u8;12],
}

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

trait ParameterList {}

impl ParameterList for InlineQosPid {}

#[derive(PartialEq, Debug)]
pub struct Data {
    endianess: EndianessFlag,
    inline_qos: Vec<u8>,
    data: Vec<u8>,
}


#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct ProtocolVersion {
    major: u8,
    minor: u8,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct MessageHeader {
    protocol_name: [char;4],
    protocol_version: ProtocolVersion,
    vendor_id: [u8;2],
    guid_prefix: [u8;12],
}

#[derive(PartialEq, Debug)]
pub enum SubMessageType {
    InfoTsSubmessage(InfoTs),
    DataSubmessage(Data),
    InfoSrcSubmessage(InfoSrc),
    PadSubmessage(()),
    None,
}

fn is_valid(message: &[u8]) -> Result<()> {
    const PROTOCOL_VERSION_FIRST_INDEX : usize = 4;
    const PROTOCOL_VERSION_LAST_INDEX : usize = 5;

    if message.len() < MINIMUM_RTPS_MESSAGE_SIZE {
        return Err(ErrorMessage::MessageTooSmall);
    }

    if message[0] != 'R' as u8 || message[1] != 'T' as u8 || message[2] != 'P' as u8 || message[3] != 'S' as u8 {
        return Err(ErrorMessage::InvalidHeader);
    }

    let protocol_version = cdr::de::deserialize_data::<ProtocolVersion, BigEndian>(&message[PROTOCOL_VERSION_FIRST_INDEX..=PROTOCOL_VERSION_LAST_INDEX])?;

    if protocol_version.major != 2 {
        return Err(ErrorMessage::RtpsMajorVersionUnsupported);
    }
    if protocol_version.minor > RTPS_MINOR_VERSION {
        return Err(ErrorMessage::RtpsMinorVersionUnsupported);
    }

    Ok(())
}

fn endianess(flags: &u8) -> Result<EndianessFlag> {
    const ENDIANESS_FLAG_MASK : u8 = 0x01;

    num::FromPrimitive::from_u8((*flags) & ENDIANESS_FLAG_MASK).ok_or(ErrorMessage::InvalidTypeConversion)
}

pub fn parse_rtps_message(message : &[u8]) -> Result< Vec<SubMessageType> >{
    is_valid(message)?;

    const RTPS_SUBMESSAGE_HEADER_SIZE: usize = 4;

    let submessage_vector = Vec::with_capacity(4);

    let mut submessage_first_index = MINIMUM_RTPS_MESSAGE_SIZE;
    while submessage_first_index < message.len() {
        const SUBMESSAGE_FLAGS_INDEX_OFFSET : usize = 1;
        
        let submessage_header_first_index = submessage_first_index;
        //In the deserialize library the comparisons are always inclusive of last element (-1 is required)
        let submessage_header_last_index = submessage_header_first_index + RTPS_SUBMESSAGE_HEADER_SIZE - 1;

        if submessage_header_last_index >= message.len() {
            return Err(ErrorMessage::InvalidSubmessageHeader);
        }

        let submessage_endianess : EndianessFlag = endianess(&message[submessage_header_first_index + SUBMESSAGE_FLAGS_INDEX_OFFSET])?;
        
        let submessage_header = deserialize::<SubmessageHeader>(message, &submessage_header_first_index, &submessage_header_last_index, &submessage_endianess);

        let submessage_payload_first_index = submessage_header_last_index + 1;
        let submessage_payload_last_index = submessage_payload_first_index + submessage_header.submessage_length as usize;
        if submessage_payload_last_index >= message.len() {
            return Err(ErrorMessage::MessageTooSmall); // TODO: Replace error by invalid message
        }

        match num::FromPrimitive::from_u8(submessage_header.submessage_id).ok_or(ErrorMessage::InvalidSubmessageHeader)? {
            SubmessageKind::Pad => SubMessageType::PadSubmessage(()),
            SubmessageKind::AckNack => unimplemented!(),
            SubmessageKind::Heartbeat => unimplemented!(),
            SubmessageKind::Gap => unimplemented!(),
            SubmessageKind::InfoTimestamp => SubMessageType::InfoTsSubmessage(parse_info_timestamp_submessage(&message[submessage_payload_first_index..=submessage_payload_last_index], &submessage_header.flags)?),
            SubmessageKind::InfoSource => SubMessageType::InfoSrcSubmessage(parse_info_source_submessage(&message[submessage_payload_first_index..=submessage_payload_last_index], &submessage_header.flags)?),
            SubmessageKind::InfoReplyIP4 => unimplemented!(),
            SubmessageKind::InfoDestination => unimplemented!(),
            SubmessageKind::InfoReply => unimplemented!(),
            SubmessageKind::NackFrag => unimplemented!(),
            SubmessageKind::HeartbeatFrag => unimplemented!(),
            SubmessageKind::Data => SubMessageType::DataSubmessage(parse_data_submessage(&message[submessage_payload_first_index..=submessage_payload_last_index], &submessage_header.flags)?),
            SubmessageKind::DataFrag => unimplemented!(),
        };

        submessage_first_index = submessage_first_index + submessage_header.submessage_length as usize;
    }

    Ok(submessage_vector)
}

fn parse_info_timestamp_submessage(submessage: &[u8], submessage_flags: &u8) -> Result<InfoTs> {
    const MESSAGE_PAYLOAD_FIRST_INDEX: usize = 0;
    const MESSAGE_PAYLOAD_LAST_INDEX: usize = 7;

    if MESSAGE_PAYLOAD_LAST_INDEX >= submessage.len() {
        return Err(ErrorMessage::InvalidSubmessage);
    }

    let submessage_endianess : EndianessFlag = endianess(submessage_flags)?;

    let timestamp = if *submessage_flags & 0x02 == 0x02 {
        None
    }
    else {
        Some(deserialize::<TimeT>(submessage, &MESSAGE_PAYLOAD_FIRST_INDEX, &MESSAGE_PAYLOAD_LAST_INDEX, &submessage_endianess))
    };

    Ok(InfoTs{timestamp: timestamp})
}

fn parse_info_source_submessage(submessage: &[u8], submessage_flags: &u8) -> Result<InfoSrc> {
    unimplemented!();
}

fn parse_data_submessage(submessage: &[u8], submessage_flags: &u8) -> Result<Data> {
    unimplemented!();

    // let submessage_payload_start = *submessage_payload_index;
    // let submessage_payload_end = submessage_payload_start + submessage_header.submessage_length as usize - 1;

    // if submessage_payload_end >= message.len() {
    //     return Err(ErrorMessage::InvalidSubmessage);
    // }

    // let submessage_endianess : EndianessFlag =
    //     num::FromPrimitive::from_u8(submessage_header.flags & 0x01).ok_or(ErrorMessage::InvalidTypeConversion)?;
    // let inline_qos_flag = submessage_header.flags & 0x02 == 0x02;
    // let data_flag = submessage_header.flags & 0x04 == 0x04;
    // let key_flag = submessage_header.flags & 0x08 == 0x08;
    // let _non_standard_payload_flag = submessage_header.flags & 0x10 == 0x10;

    // let mut submessage_process_index = *submessage_payload_index;

    // let _extra_flags = deserialize::<u16>(message, &submessage_process_index, &(submessage_process_index+1), &submessage_endianess);
    // submessage_process_index = submessage_process_index + 2;

    // let octets_to_inline_qos = deserialize::<u16>(message, &submessage_process_index, &(submessage_process_index+1), &submessage_endianess);
    // submessage_process_index = submessage_process_index + 2;

    // let reader_id = deserialize::<u32>(message, &submessage_process_index, &(submessage_process_index+3), &submessage_endianess);
    // submessage_process_index = submessage_process_index + 4;

    // let writer_id = deserialize::<u32>(message, &submessage_process_index, &(submessage_process_index+3), &submessage_endianess);
    // submessage_process_index = submessage_process_index + 4;

    // // Sequence number is signed 64 bits but it is split into two 32 bit parts, 
    // // a signed i32 representing the 32 msb and an unsigned u32 represing the 32 lsb
    // let writer_seq_number_high = deserialize::<i32>(message, &submessage_process_index, &(submessage_process_index+3), &submessage_endianess);
    // submessage_process_index = submessage_process_index + 4;

    // let writer_seq_number_low = deserialize::<u32>(message, &submessage_process_index, &(submessage_process_index+3), &submessage_endianess);
    // submessage_process_index = submessage_process_index + 4;

    // let writer_seq_number = ((writer_seq_number_high as i64) << 32) + writer_seq_number_low as i64;

    // // In this case we move forward using the information present in the octets to inline qos
    // submessage_process_index = submessage_process_index - 16 + octets_to_inline_qos as usize;

    // if inline_qos_flag {

    //     // TODO: Proces the QOS
    // }

    // if data_flag && key_flag {
    //     return Err(ErrorMessage::InvalidKeyAndDataFlagCombination);
    // } else if data_flag {
    //     // TODO: Process the data
    // } else if key_flag {
    //     // TODO: Process the key
    // }

    Err(ErrorMessage::InvalidSubmessage)
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_parse_valid_message_header_only() {
        let message_example = MessageHeader{
            protocol_name: ['R','T','P','S'],
            protocol_version: ProtocolVersion{major: 2, minor: 4},
            vendor_id: [100,210],
            guid_prefix: [10,11,12,13,14,15,16,17,18,19,20,21],};

        let serialized = cdr::ser::serialize_data::<_, _, BigEndian>(&message_example, Infinite).unwrap();

        let parse_result = parse_rtps_message(&serialized).unwrap();
        
        assert_eq!(parse_result, vec!());
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
    fn test_parse_info_timestamp_submessage() {
        const BIG_ENDIAN_FLAG: u8 = 0x00;
        const LITTLE_ENDIAN_FLAG: u8 = 0x01;
        const INVALID_FLAG : u8 = 0x02;

        // Unix time: 1565525425=>0x5D5005B1
        // Is equivalent to: 08/11/2019 @ 12:10pm (UTC)
        // Seconds fraction: 0x10112243 => 269558339 => 0.0628
        const TEST_TIME : TimeT = TimeT {
            seconds: 1565525425,
            fraction: 269558339,
        };
        
        let timestamp_message_payload_big_endian = [0x5D,0x50,0x05,0xB1,0x10,0x11,0x22,0x43];
        let info_ts_big_endian = parse_info_timestamp_submessage(&timestamp_message_payload_big_endian, &BIG_ENDIAN_FLAG).unwrap();
        assert_eq!(Some(TEST_TIME),info_ts_big_endian.timestamp);

        let timestamp_message_payload_little_endian = [0xB1,0x05,0x50,0x5D,0x43,0x22,0x11,0x10];
        let info_ts_little_endian = parse_info_timestamp_submessage(&timestamp_message_payload_little_endian, &LITTLE_ENDIAN_FLAG).unwrap();
        assert_eq!(Some(TEST_TIME),info_ts_little_endian.timestamp);

        let info_ts_none_big_endian = parse_info_timestamp_submessage(&timestamp_message_payload_big_endian, &(BIG_ENDIAN_FLAG+INVALID_FLAG)).unwrap();
        assert_eq!(None,info_ts_none_big_endian.timestamp);

        let info_ts_none_little_endian = parse_info_timestamp_submessage(&timestamp_message_payload_little_endian, &(LITTLE_ENDIAN_FLAG+INVALID_FLAG)).unwrap();
        assert_eq!(None,info_ts_none_little_endian.timestamp);
    }

    #[test]
    fn test_parse_different_rtps_messages() {
        let rtps_message_info_ts_and_data = [0x52, 0x54, 0x50, 0x53, 0x02, 0x01, 0x01, 0x02, 0x7f, 0x20, 0xf7, 0xd7, 0x00, 0x00, 0x01, 0xbb, 0x00, 0x00, 0x00, 0x01, 0x09, 0x01, 0x08, 0x00, 0x9e, 0x81, 0xbc, 0x5d, 0x97, 0xde, 0x48, 0x26, 0x15, 0x07, 0x1c, 0x01, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0xc2, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x70, 0x00, 0x10, 0x00, 0x7f, 0x20, 0xf7, 0xd7, 0x00, 0x00, 0x01, 0xbb, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0xc1, 0x01, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x15, 0x00, 0x04, 0x00, 0x02, 0x01, 0x00, 0x00, 0x16, 0x00, 0x04, 0x00, 0x01, 0x02, 0x00, 0x00, 0x31, 0x00, 0x18, 0x00, 0x01, 0x00, 0x00, 0x00, 0xf3, 0x1c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0xa8, 0x02, 0x04, 0x32, 0x00, 0x18, 0x00, 0x01, 0x00, 0x00, 0x00, 0xf2, 0x1c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0xa8, 0x02, 0x04, 0x02, 0x00, 0x08, 0x00, 0x0b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50, 0x00, 0x10, 0x00, 0x7f, 0x20, 0xf7, 0xd7, 0x00, 0x00, 0x01, 0xbb, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x01, 0xc1, 0x58, 0x00, 0x04, 0x00, 0x15, 0x04, 0x00, 0x00, 0x00, 0x80, 0x04, 0x00, 0x15, 0x00, 0x00, 0x00, 0x07, 0x80, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x2f, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50, 0x00, 0x00, 0x00, 0x42, 0x00, 0x00, 0x00, 0x44, 0x45, 0x53, 0x4b, 0x54, 0x4f, 0x50, 0x2d, 0x4f, 0x52, 0x46, 0x44, 0x4f, 0x53, 0x35, 0x2f, 0x36, 0x2e, 0x31, 0x30, 0x2e, 0x32, 0x2f, 0x63, 0x63, 0x36, 0x66, 0x62, 0x39, 0x61, 0x62, 0x33, 0x36, 0x2f, 0x39, 0x30, 0x37, 0x65, 0x66, 0x66, 0x30, 0x32, 0x65, 0x33, 0x2f, 0x22, 0x78, 0x38, 0x36, 0x5f, 0x36, 0x34, 0x2e, 0x77, 0x69, 0x6e, 0x2d, 0x76, 0x73, 0x32, 0x30, 0x31, 0x35, 0x22, 0x2f, 0x00, 0x00, 0x00, 0x25, 0x80, 0x0c, 0x00, 0xd7, 0xf7, 0x20, 0x7f, 0xbb, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00];

        let parse_result = parse_rtps_message(&rtps_message_info_ts_and_data);
    }

    // #[test]
    // fn serialize_info_timestamp() {
    //     let ts_submessage = Submessage::<InfoTsSubmessage> {
    //         header: SubmessageHeader{
    //             submessage_id: 0x09,
    //             flags: 0x01,
    //             submessage_length: 8,
    //         },
    //         submessage: InfoTsSubmessage {
    //             timestamp: TimeT{
    //                 seconds: 100000,
    //                 fraction: 500,
    //             },
    //         }
    //     };

    //     let serialized = cdr::ser::serialize_data::<_, _, BigEndian>(&ts_submessage, Infinite).unwrap();

    //     let serialized_message = [9, 1, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 134, 160];

    //     let timemsg = parse_info_ts_submessage(&serialized_message);

    //     println!("{:?}", timemsg);
    // }
}
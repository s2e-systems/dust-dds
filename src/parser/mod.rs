extern crate serde;
extern crate serde_derive;
extern crate num;
extern crate num_derive;

use std::cmp;
use std::mem;
use serde_derive::{Deserialize, Serialize};

use num_derive::FromPrimitive;

use cdr::{
    LittleEndian, BigEndian, CdrLe, CdrBe, PlCdrLe, PlCdrBe, Error, Infinite,
};

use super::EntityIdT;

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

fn deserialize<'de,T>(message: &[u8], start_index: &usize, end_index: &usize, endianess: &EndianessFlag) -> Result<T> 
    where T: serde::de::Deserialize<'de>
{
    if message.len() <= *end_index {
        return Err(ErrorMessage::DeserializationMessageSizeTooSmall);
    }

    if *endianess == EndianessFlag::BigEndian {
        cdr::de::deserialize_data::<T, BigEndian>(&message[*start_index..=*end_index]).map_err(|e|ErrorMessage::CdrError(e))
    } else {
        cdr::de::deserialize_data::<T, LittleEndian>(&message[*start_index..=*end_index]).map_err(|e|ErrorMessage::CdrError(e))
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
struct SequenceNumberSerialization {
    high: i32,
    low: u32,
}

impl From<i64> for SequenceNumberSerialization {
    fn from(value: i64) -> Self {
        SequenceNumberSerialization{
            high: (value >> 32) as i32,
            low: (value & 0x00000000FFFFFFFF) as u32,
        }
    }
}

impl From<SequenceNumberSerialization> for i64 {
    fn from(value: SequenceNumberSerialization) -> Self {
        ((value.high as i64) << 32) + value.low as i64
    }
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

type ParameterList = Vec<Parameter>;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Parameter {
    parameter_id: u16,
    value: Vec<u8>,
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
    guid_prefix: GuidPrefix,
}

type GuidPrefix = [u8;12];
type Count = i32;
type SequenceNumber = i64;
type SequenceNumberSet = Vec<(SequenceNumber, bool)>;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct AckNack {
    final_flag: bool,
    reader_id: EntityIdT,
    writer_id: EntityIdT,
    reader_sn_state: SequenceNumberSet,
    count: Count,
}

#[derive(PartialEq, Debug)]
pub enum Payload {
    None,
    Data(Vec<u8>),
    Key(Vec<u8>),
    NonStandard(Vec<u8>),
}

#[derive(PartialEq, Debug)]
pub struct Data {
    endianess: EndianessFlag,
    reader_id: EntityIdT,
    writer_id: EntityIdT,
    writer_sn: SequenceNumber,
    inline_qos: Option<ParameterList>,
    serialized_payload: Payload,
}

#[derive(PartialEq, Debug)]
pub struct DataFrag {
    endianess: EndianessFlag,
    inline_qos: Vec<u8>,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Gap {
    reader_id: EntityIdT,
    writer_id: EntityIdT,
    gap_start: SequenceNumber,
    gap_list: SequenceNumberSet,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Heartbeat {
    reader_id: EntityIdT,
    writer_id: EntityIdT,
    first_sn: SequenceNumber,
    last_sn: SequenceNumber,
    count: Count,
    final_flag: bool,
    liveliness_flag: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct HeartbeatFrag {

}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct InfoDst {
    guid_prefix: GuidPrefix,

}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct InfoReply {

}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct InfoSrc {
    protocol_version: ProtocolVersion,
    vendor_id: [u8;2],
    guid_prefix: [u8;12],
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct InfoTs {
    timestamp: Option<TimeT>, 
}

// Pad is contentless so it is skipped here

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct NackFrag {
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

fn is_valid(message: &[u8]) -> Result<()> {
    const PROTOCOL_VERSION_FIRST_INDEX : usize = 4;
    const PROTOCOL_VERSION_LAST_INDEX : usize = 5;

    if message.len() < MINIMUM_RTPS_MESSAGE_SIZE {
        return Err(ErrorMessage::MessageTooSmall);
    }

    if message[0] != 'R' as u8 || message[1] != 'T' as u8 || message[2] != 'P' as u8 || message[3] != 'S' as u8 {
        return Err(ErrorMessage::InvalidHeader);
    }

    let protocol_version = deserialize::<ProtocolVersion>(message, &PROTOCOL_VERSION_FIRST_INDEX, &PROTOCOL_VERSION_LAST_INDEX, &EndianessFlag::BigEndian)?;

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
        
        let submessage_header = deserialize::<SubmessageHeader>(message, &submessage_header_first_index, &submessage_header_last_index, &submessage_endianess)?;

        let submessage_payload_first_index = submessage_header_last_index + 1;
        let submessage_payload_last_index = submessage_payload_first_index + submessage_header.submessage_length as usize;
        if submessage_payload_last_index >= message.len() {
            return Err(ErrorMessage::MessageTooSmall); // TODO: Replace error by invalid message
        }

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

        submessage_first_index = submessage_first_index + submessage_header.submessage_length as usize;
    }

    Ok(submessage_vector)
}

fn parse_ack_nack_submessage(submessage: &[u8], submessage_flags: &u8) -> Result<AckNack> {
    const FINAL_FLAG_MASK: u8 = 0x02;
    const READER_ID_FIRST_INDEX: usize = 0;
    const READER_ID_LAST_INDEX: usize = 3;
    const WRITER_ID_FIRST_INDEX: usize = 4;
    const WRITER_ID_LAST_INDEX: usize = 7;
    const SEQUENCE_NUMBER_SET_FIRST_INDEX: usize = 8;
    const COUNT_SIZE: usize = 4;

    let submessage_endianess : EndianessFlag = endianess(submessage_flags)?;
    let final_flag = (submessage_flags & FINAL_FLAG_MASK) == FINAL_FLAG_MASK;

    let reader_id = deserialize::<EntityIdT>(submessage, &READER_ID_FIRST_INDEX, &READER_ID_LAST_INDEX, &submessage_endianess)?;
    
    let writer_id = deserialize::<EntityIdT>(submessage, &WRITER_ID_FIRST_INDEX, &WRITER_ID_LAST_INDEX, &submessage_endianess)?;
    
    let (reader_sn_state, sequence_number_set_size) = parse_sequence_number_set(submessage, &SEQUENCE_NUMBER_SET_FIRST_INDEX, &submessage_endianess)?;

    let count_first_index = SEQUENCE_NUMBER_SET_FIRST_INDEX + sequence_number_set_size;
    let count_last_index = count_first_index + COUNT_SIZE - 1;

    let count = deserialize::<Count>(submessage, &count_first_index, &count_last_index, &submessage_endianess)?;

    Ok( AckNack {
        final_flag,
        reader_id,
        writer_id,
        reader_sn_state,
        count,
    })
}

fn parse_data_submessage(submessage: &[u8], submessage_flags: &u8) -> Result<Data> {

    const INLINE_QOS_FLAG_MASK: u8 = 0x02;
    const DATA_FLAG_MASK: u8 = 0x04;
    const KEY_FLAG_MASK: u8 = 0x08;
    const NON_STANDARD_PAYLOAD_FLAG_MASK: u8 = 0x10;

    const EXTRA_FLAGS_FIRST_INDEX: usize = 0;
    const EXTRA_FLAGS_LAST_INDEX: usize = 1;
    const OCTETS_TO_INLINE_QOS_FIRST_INDEX: usize = 2;
    const OCTETS_TO_INLINE_QOS_LAST_INDEX: usize = 3;
    const READER_ID_FIRST_INDEX: usize = 4;
    const READER_ID_LAST_INDEX: usize = 7;
    const WRITER_ID_FIRST_INDEX: usize = 8;
    const WRITER_ID_LAST_INDEX: usize = 11;
    const WRITER_SN_FIRST_INDEX: usize = 12;
    const WRITER_SN_LAST_INDEX: usize = 19;

    let submessage_endianess : EndianessFlag = endianess(submessage_flags)?;
    let inline_qos_flag = submessage_flags & INLINE_QOS_FLAG_MASK == INLINE_QOS_FLAG_MASK;
    let data_flag = submessage_flags & DATA_FLAG_MASK == DATA_FLAG_MASK;
    let key_flag = submessage_flags & KEY_FLAG_MASK == KEY_FLAG_MASK;

    // TODO: Implement non-standard payload
    let _non_standard_payload_flag = submessage_flags & NON_STANDARD_PAYLOAD_FLAG_MASK == NON_STANDARD_PAYLOAD_FLAG_MASK;

    if data_flag == true && key_flag == true {
        return Err(ErrorMessage::InvalidSubmessage);
    }

    let extra_flags = deserialize::<u16>(submessage, &EXTRA_FLAGS_FIRST_INDEX, &EXTRA_FLAGS_LAST_INDEX, &submessage_endianess)?;
    if extra_flags != 0 {
        return Err(ErrorMessage::InvalidSubmessage);
    }

    let octecs_to_inline_qos = deserialize::<u16>(submessage, &OCTETS_TO_INLINE_QOS_FIRST_INDEX, &OCTETS_TO_INLINE_QOS_LAST_INDEX, &submessage_endianess)? as usize;

    let reader_id = deserialize::<EntityIdT>(submessage, &READER_ID_FIRST_INDEX, &READER_ID_LAST_INDEX, &submessage_endianess)?;
    
    let writer_id = deserialize::<EntityIdT>(submessage, &WRITER_ID_FIRST_INDEX, &WRITER_ID_LAST_INDEX, &submessage_endianess)?;

    let writer_sn : i64 = deserialize::<SequenceNumberSerialization>(submessage, &WRITER_SN_FIRST_INDEX, &WRITER_SN_LAST_INDEX, &submessage_endianess)?.into();

    // Octets to data is considered as having the same meaning as octets to inline qos,
    // i.e. counting from the byte after the octets to inline qos field

    let (inline_qos, octets_to_data) =
        if inline_qos_flag == true {
            let inline_qos_first_index = OCTETS_TO_INLINE_QOS_LAST_INDEX + octecs_to_inline_qos + 1;
            let (parameter_list, parameter_list_size) = parse_parameter_list(submessage, &inline_qos_first_index, &submessage_endianess)?;
            let octets_to_data = octecs_to_inline_qos + parameter_list_size;
            (Some(parameter_list), octets_to_data)
        } else {
            (None, octecs_to_inline_qos)
        };

    let payload_first_index = OCTETS_TO_INLINE_QOS_LAST_INDEX + octets_to_data + 1;

    let serialized_payload = 
        if data_flag == true && key_flag == false {
            Payload::Data(submessage[payload_first_index..].to_vec())
        } else if data_flag == false && key_flag == true {
            Payload::Key(submessage[payload_first_index..].to_vec())
        } else {
            Payload::None
        };

    Ok(Data{
        endianess: submessage_endianess,
        reader_id,
        writer_id,
        writer_sn,
        inline_qos,
        serialized_payload,
    })

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

    // Err(ErrorMessage::InvalidSubmessage)
}

fn parse_data_frag_submessage(_submessage: &[u8], _submessage_flags: &u8) -> Result<DataFrag> {
    unimplemented!()
}

fn parse_gap_submessage(submessage: &[u8], submessage_flags: &u8) -> Result<Gap> {
    const READER_ID_FIRST_INDEX: usize = 0;
    const READER_ID_LAST_INDEX: usize = 3;
    const WRITER_ID_FIRST_INDEX: usize = 4;
    const WRITER_ID_LAST_INDEX: usize = 7;
    const GAP_START_FIRST_INDEX: usize = 8;
    const GAP_START_LAST_INDEX: usize = 15;
    const GAP_LIST_FIRST_INDEX: usize = 16;

    let submessage_endianess : EndianessFlag = endianess(submessage_flags)?;

    let reader_id = deserialize::<EntityIdT>(submessage, &READER_ID_FIRST_INDEX, &READER_ID_LAST_INDEX, &submessage_endianess)?;
    
    let writer_id = deserialize::<EntityIdT>(submessage, &WRITER_ID_FIRST_INDEX, &WRITER_ID_LAST_INDEX, &submessage_endianess)?;

    let gap_start : i64 = deserialize::<SequenceNumberSerialization>(submessage, &GAP_START_FIRST_INDEX, &GAP_START_LAST_INDEX, &submessage_endianess)?.into();
    if gap_start < 1 {
        return Err(ErrorMessage::InvalidSubmessage);
    }
    
    let (gap_list, sequence_number_set_size) = parse_sequence_number_set(submessage, &GAP_LIST_FIRST_INDEX, &submessage_endianess)?;

    // TODO: The GAP message in the PSM is not matching the description given in the PIM. Have to check for that.

    Ok(Gap{
        reader_id,
        writer_id,
        gap_start,
        gap_list,
    })
}

fn parse_heartbeat_submessage(submessage: &[u8], submessage_flags: &u8) -> Result<Heartbeat> {
    const READER_ID_FIRST_INDEX: usize = 0;
    const READER_ID_LAST_INDEX: usize = 3;
    const WRITER_ID_FIRST_INDEX: usize = 4;
    const WRITER_ID_LAST_INDEX: usize = 7;
    const FIRST_SN_FIRST_INDEX: usize = 8;
    const FIRST_SN_LAST_INDEX: usize = 15;
    const LAST_SN_FIRST_INDEX: usize = 16;
    const LAST_SN_LAST_INDEX: usize = 23;
    const COUNT_FIRST_INDEX: usize = 24;
    const COUNT_LAST_INDEX: usize = 27;

    const FINAL_FLAG_MASK: u8 = 0x02;
    const LIVELINESS_FLAG_MASK: u8 = 0x04;

    let submessage_endianess : EndianessFlag = endianess(submessage_flags)?;
    let final_flag = (submessage_flags & FINAL_FLAG_MASK) == FINAL_FLAG_MASK;
    let liveliness_flag = (submessage_flags & LIVELINESS_FLAG_MASK) == LIVELINESS_FLAG_MASK;

    let reader_id = deserialize::<EntityIdT>(submessage, &READER_ID_FIRST_INDEX, &READER_ID_LAST_INDEX, &submessage_endianess)?;
    
    let writer_id = deserialize::<EntityIdT>(submessage, &WRITER_ID_FIRST_INDEX, &WRITER_ID_LAST_INDEX, &submessage_endianess)?;

    let first_sn : i64 = deserialize::<SequenceNumberSerialization>(submessage, &FIRST_SN_FIRST_INDEX, &FIRST_SN_LAST_INDEX, &submessage_endianess)?.into();
    if first_sn < 1 {
        return Err(ErrorMessage::InvalidSubmessage);
    }

    let last_sn : i64 = deserialize::<SequenceNumberSerialization>(submessage, &LAST_SN_FIRST_INDEX, &LAST_SN_LAST_INDEX, &submessage_endianess)?.into();
    if last_sn < 0 {
        return Err(ErrorMessage::InvalidSubmessage);
    }

    if last_sn < first_sn - 1 {
        return Err(ErrorMessage::InvalidSubmessage);
    }

    let count = deserialize::<Count>(submessage, &COUNT_FIRST_INDEX, &COUNT_LAST_INDEX, &submessage_endianess)?;

    Ok( Heartbeat{
        reader_id,
        writer_id,
        first_sn,
        last_sn,
        count,
        final_flag,
        liveliness_flag,
    })
}

fn parse_heartbeat_frag_submessage(_submessage: &[u8], _submessage_flags: &u8) -> Result<HeartbeatFrag> {
    unimplemented!()
}

fn parse_info_dst_submessage(submessage: &[u8], submessage_flags: &u8) -> Result<InfoDst> {
    const GUID_PREFIX_FIRST_INDEX: usize = 0;
    const GUID_PREFIX_LAST_INDEX: usize = 11;
    let submessage_endianess : EndianessFlag = endianess(submessage_flags)?;
    let guid_prefix = deserialize::<GuidPrefix>(submessage, &GUID_PREFIX_FIRST_INDEX, &GUID_PREFIX_LAST_INDEX, &submessage_endianess)?;

    Ok(InfoDst{
        guid_prefix,
    })
}

fn parse_info_reply_submessage(_submessage: &[u8], _submessage_flags: &u8) -> Result<InfoReply> {
    unimplemented!()
}

fn parse_info_source_submessage(_submessage: &[u8], _submessage_flags: &u8) -> Result<InfoSrc> {
    unimplemented!()
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
        Some(deserialize::<TimeT>(submessage, &MESSAGE_PAYLOAD_FIRST_INDEX, &MESSAGE_PAYLOAD_LAST_INDEX, &submessage_endianess)?)
    };

    Ok(InfoTs{timestamp: timestamp})
}

fn parse_nack_frag_submessage(_submessage: &[u8], _submessage_flags: &u8) -> Result<NackFrag> {
    unimplemented!()
}

fn parse_sequence_number_set(submessage: &[u8], sequence_number_set_first_index: &usize, endianess_flag: &EndianessFlag) -> Result<(SequenceNumberSet, usize)> {
    const SEQUENCE_NUMBER_TYPE_SIZE : usize = 8;
    const NUM_BITS_TYPE_SIZE: usize = 4;
    const BITMAP_FIELD_SIZE: usize = 4;

    let bitmap_base_first_index = *sequence_number_set_first_index;
    let bitmap_base_last_index = bitmap_base_first_index + SEQUENCE_NUMBER_TYPE_SIZE - 1;

    let bitmap_base : i64 = deserialize::<SequenceNumberSerialization>(submessage, &bitmap_base_first_index, &bitmap_base_last_index, endianess_flag)?.into();
    if bitmap_base < 1 {
        return Err(ErrorMessage::InvalidSubmessage);
    }

    let num_bits_first_index = bitmap_base_last_index + 1;
    let num_bits_last_index = num_bits_first_index + NUM_BITS_TYPE_SIZE - 1;

    let num_bits = deserialize::<u32>(submessage, &num_bits_first_index, &num_bits_last_index, &endianess_flag)?;
    if num_bits < 1 ||  num_bits > 256 {
        return Err(ErrorMessage::InvalidSubmessage);
    }

    let num_bitmap_fields = ((num_bits + 31) >> 5) as usize;

    let mut sequence_number_set = SequenceNumberSet::with_capacity(num_bitmap_fields);

    for bitmap_field_index in 0..num_bitmap_fields {
        let field_first_index = num_bits_last_index + 1 + bitmap_field_index * BITMAP_FIELD_SIZE;
        let field_last_index = field_first_index + BITMAP_FIELD_SIZE - 1;
        let bitmap_field = deserialize::<u32>(submessage, &field_first_index, &field_last_index, &endianess_flag)?;

        let number_bits_in_field = cmp::min(num_bits as usize - (BITMAP_FIELD_SIZE * 8) * bitmap_field_index,32);
        for sequence_number_index in 0..number_bits_in_field {
            let sequence_number : i64 = bitmap_base + (sequence_number_index + (BITMAP_FIELD_SIZE * 8) * bitmap_field_index) as i64;
            let sequence_bit_mask = 1 << sequence_number_index;
            let sequence_bit = (bitmap_field & sequence_bit_mask) == sequence_bit_mask;
            sequence_number_set.push( (SequenceNumber::from(sequence_number), sequence_bit) );
        }
    }

    Ok( (sequence_number_set, SEQUENCE_NUMBER_TYPE_SIZE+NUM_BITS_TYPE_SIZE+BITMAP_FIELD_SIZE*num_bitmap_fields) )
}

fn parse_parameter_list(submessage: &[u8], parameter_list_first_index: &usize, endianess: &EndianessFlag) -> Result<(ParameterList, usize)>{
    const MINIMUM_PARAMETER_VALUE_LENGTH: usize = 4;
    const PARAMETER_ID_OFFSET: usize = 1;
    const LENGTH_FIRST_OFFSET: usize = 2;
    const LENGTH_LAST_OFFSET: usize = 3;
    const VALUE_FIRST_OFFSET: usize = 4;

    let mut parameter_id_first_index = *parameter_list_first_index;

    let mut parameter_list = Vec::new();
    let parameter_list_size: usize;

    loop {
        let parameter_id_last_index = parameter_id_first_index + PARAMETER_ID_OFFSET;
        let length_first_index = parameter_id_first_index + LENGTH_FIRST_OFFSET;
        let length_last_index = parameter_id_first_index + LENGTH_LAST_OFFSET;

        let value_first_index = parameter_id_first_index + VALUE_FIRST_OFFSET;
        let value_last_index;

        let parameter_id = deserialize::<u16>(submessage, &parameter_id_first_index, &parameter_id_last_index, endianess)?;
        if parameter_id == InlineQosPid::Sentinel as u16{
            parameter_list_size = length_last_index - *parameter_list_first_index + 1;
            break;
        }

        let length = deserialize::<u16>(submessage, &length_first_index, &length_last_index, endianess)? as usize;
        if length < MINIMUM_PARAMETER_VALUE_LENGTH {
            return Err(ErrorMessage::InvalidSubmessage);
        }

        value_last_index = value_first_index + length - 1;
        if value_last_index >= submessage.len() {
            return Err(ErrorMessage::InvalidSubmessage);
        }

        let value = submessage[value_first_index..=value_last_index].to_vec();
        parameter_list.push(Parameter{parameter_id, value,});

        parameter_id_first_index = value_last_index + 1;
    }

    Ok((parameter_list, parameter_list_size))
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
    fn test_parse_sequence_number_set() {
        {
            // Test for example in standard "1234:/12:00110"
            let submessage_test_1_big_endian = [
                0x00,0x00,0x00,0x00,
                0x00,0x00,0x04,0xD2,
                0x00,0x00,0x00,0x0C,
                0x00,0x00,0x00,0x0C];

            let (sequence_set_1, sequence_set_size) = parse_sequence_number_set(&submessage_test_1_big_endian, &0, &EndianessFlag::BigEndian).unwrap();
            assert_eq!(sequence_set_1.len(),12);
            assert_eq!(sequence_set_size,16);
            for (index, item) in sequence_set_1.iter().enumerate() {
                assert_eq!(item.0, 1234 + index as i64);
                if item.0 == 1236 || item.0 == 1237 {
                    assert_eq!(item.1, true);
                } else {
                    assert_eq!(item.1,false);
                }
            }
        }
        
        {
            // Test for example in standard "1234:/12:00110"
            let submessage_test_1_little_endian = [
                0x00,0x00,0x00,0x00,
                0xD2,0x04,0x00,0x00,
                0x0C,0x00,0x00,0x00,
                0x0C,0x00,0x00,0x00];

            let (sequence_set_1, sequence_set_size) = parse_sequence_number_set(&submessage_test_1_little_endian, &0, &EndianessFlag::LittleEndian).unwrap();
            assert_eq!(sequence_set_1.len(),12);
            assert_eq!(sequence_set_size,16);
            for (index, item) in sequence_set_1.iter().enumerate() {
                assert_eq!(item.0, 1234 + index as i64);
                if item.0 == 1236 || item.0 == 1237 {
                    assert_eq!(item.1, true);
                } else {
                    assert_eq!(item.1,false);
                }
            }
        }

        {
            // Test too high num bits
            let submessage_test_high_num_bits = [
                0x00,0x00,0x00,0x00,
                0x00,0x00,0x04,0xD2,
                0x00,0x00,0x02,0x00,
                0x00,0x00,0x00,0x0C];

            let sequence_set_result = parse_sequence_number_set(&submessage_test_high_num_bits, &0, &EndianessFlag::BigEndian);
            if let Err(ErrorMessage::InvalidSubmessage) = sequence_set_result {
                assert!(true);
            } else {
                assert!(false);
            }
        }

        {
            // Negative bitmap base
            let submessage_test_negative_base = [
                0x80,0x00,0x00,0x00,
                0x00,0x00,0x04,0xD2,
                0x00,0x00,0x00,0x0C,
                0x00,0x00,0x00,0x0C];

            let sequence_set_result = parse_sequence_number_set(&submessage_test_negative_base, &0, &EndianessFlag::BigEndian);
            if let Err(ErrorMessage::InvalidSubmessage) = sequence_set_result {
                assert!(true);
            } else {
                assert!(false);
            }
        }

        {
            // Zero bitmap base
            let submessage_test_zero_base = [
                0x00,0x00,0x00,0x00,
                0x00,0x00,0x00,0x00,
                0x00,0x00,0x00,0x0C,
                0x00,0x00,0x00,0x0C];

            let sequence_set_result = parse_sequence_number_set(&submessage_test_zero_base, &0, &EndianessFlag::BigEndian);
            if let Err(ErrorMessage::InvalidSubmessage) = sequence_set_result {
                assert!(true);
            } else {
                assert!(false);
            }
        }

        {
            // Full size bitmap with base > 32bit
            let submessage_test_large = [
                0x00,0x00,0x00,0x01,
                0x00,0x00,0x04,0xD2,
                0x00,0x00,0x01,0x00,
                0xAA,0xAA,0xAA,0xAA,
                0xAA,0xAA,0xAA,0xAA,
                0xAA,0xAA,0xAA,0xAA,
                0xAA,0xAA,0xAA,0xAA,
                0xAA,0xAA,0xAA,0xAA,
                0xAA,0xAA,0xAA,0xAA,
                0xAA,0xAA,0xAA,0xAA,
                0xAA,0xAA,0xAA,0xAA];

            
            let (sequence_set, sequence_set_size) = parse_sequence_number_set(&submessage_test_large, &0, &EndianessFlag::BigEndian).unwrap();
            assert_eq!(sequence_set.len(),256);
            assert_eq!(sequence_set_size,44);
            for (index, item) in sequence_set.iter().enumerate() {
                assert_eq!(item.0, 4294968530i64 + index as i64);
                if (index + 1) % 2 == 0 {
                    assert_eq!(item.1, true);
                } else {
                    assert_eq!(item.1,false);
                }
            }
        }

        {
            // Middle size bitmap with base > 32bit
            let submessage_test_middle = [
                0x00,0x00,0x00,0x01,
                0x00,0x00,0x04,0xD2,
                0x00,0x00,0x00,0x28,
                0xAA,0xAA,0xAA,0xAA,
                0xFF,0x00,0xFF,0xAA];

            
            let (sequence_set, sequence_set_size) = parse_sequence_number_set(&submessage_test_middle, &0, &EndianessFlag::BigEndian).unwrap();
            assert_eq!(sequence_set.len(),40);
            assert_eq!(sequence_set_size,20);
            for (index, item) in sequence_set.iter().enumerate() {
                assert_eq!(item.0, 4294968530i64 + index as i64);
                if (index + 1) % 2 == 0 {
                    assert_eq!(item.1, true);
                } else {
                    assert_eq!(item.1,false);
                }
            }
        }

        {
            // Middle size bitmap with base > 32bit with start not at 0
            let submessage_test_middle = [
                0xFA,0xAF,
                0x00,0x00,0x01,0x01,
                0x00,0x00,0x04,0xD2,
                0x00,0x00,0x00,0x28,
                0xAA,0xAA,0xAA,0xAA,
                0xFF,0x00,0xFF,0xAA,
                0xAB,0x56];

            
            let (sequence_set, sequence_set_size) = parse_sequence_number_set(&submessage_test_middle, &2, &EndianessFlag::BigEndian).unwrap();
            assert_eq!(sequence_set.len(),40);
            assert_eq!(sequence_set_size,20);
            for (index, item) in sequence_set.iter().enumerate() {
                assert_eq!(item.0, 1103806596306i64 + index as i64);
                if (index + 1) % 2 == 0 {
                    assert_eq!(item.1, true);
                } else {
                    assert_eq!(item.1,false);
                }
            }
        }

        {
            let wrong_submessage_test = [0xFA,0xAF];

            let sequence_set_result = parse_sequence_number_set(&wrong_submessage_test, &0, &EndianessFlag::BigEndian);

            if let Err(ErrorMessage::DeserializationMessageSizeTooSmall) = sequence_set_result {
                assert!(true);
            } else {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_parse_parameter_list() {
        {
            let submessage_big_endian = [
                0x00, 0x05, 0x00, 0x04,
                0x01, 0x02, 0x03, 0x04,
                0x00, 0x10, 0x00, 0x08,
                0x10, 0x11, 0x12, 0x13,
                0x14, 0x15, 0x16, 0x17,
                0x00, 0x01, 0x00, 0x00,
            ];

            let (param_list_big_endian, param_list_size) = parse_parameter_list(&submessage_big_endian, &0, &EndianessFlag::BigEndian).unwrap();
            assert_eq!(param_list_size, 24);
            assert_eq!(param_list_big_endian.len(), 2);
            assert_eq!(param_list_big_endian[0].parameter_id, 5);
            assert_eq!(param_list_big_endian[0].value, vec!(1,2,3,4));
            assert_eq!(param_list_big_endian[1].parameter_id, 16);
            assert_eq!(param_list_big_endian[1].value, vec!(16,17,18,19,20,21,22,23));
        }

        {
            let submessage_little_endian = [
                0x05, 0x00, 0x04, 0x00,
                0x01, 0x02, 0x03, 0x04,
                0x10, 0x00, 0x08, 0x00,
                0x10, 0x11, 0x12, 0x13,
                0x14, 0x15, 0x16, 0x17,
                0x01, 0x00, 0x00, 0x00,
            ];

            let (param_list_little_endian, param_list_size) = parse_parameter_list(&submessage_little_endian, &0, &EndianessFlag::LittleEndian).unwrap();
            assert_eq!(param_list_size, 24);
            assert_eq!(param_list_little_endian.len(), 2);
            assert_eq!(param_list_little_endian[0].parameter_id, 5);
            assert_eq!(param_list_little_endian[0].value, vec!(1,2,3,4));
            assert_eq!(param_list_little_endian[1].parameter_id, 16);
            assert_eq!(param_list_little_endian[1].value, vec!(16,17,18,19,20,21,22,23));
        }

        {
            // Test no sentinel message
            let submessage = [
                0x00, 0x05, 0x00, 0x04,
                0x01, 0x02, 0x03, 0x04,
                0x00, 0x10, 0x00, 0x08,
                0x10, 0x11, 0x12, 0x13,
                0x14, 0x15, 0x16, 0x17,
                0x10, 0x11, 0x00, 0x00,
            ];

            let param_list = parse_parameter_list(&submessage, &0, &EndianessFlag::LittleEndian);
            if let Err(ErrorMessage::InvalidSubmessage) = param_list {
                assert!(true);
            } else {
                assert!(false);
            }
        }

        {
            // Test length below minimum
            let submessage = [
                0x00, 0x05, 0x00, 0x03,
                0x01, 0x02, 0x03, 0x00, 0x10, 0x00, 0x08,
                0x10, 0x11, 0x12, 0x13,
                0x14, 0x15, 0x16, 0x17,
                0x00, 0x01, 0x00, 0x00,
            ];

            let param_list = parse_parameter_list(&submessage, &0, &EndianessFlag::BigEndian);
            if let Err(ErrorMessage::InvalidSubmessage) = param_list {
                assert!(true);
            } else {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_parse_ack_nack_submessage() {
        {
            let ack_nack_submessage_big_endian = [
            0x10,0x12,0x14,0x16,
            0x26,0x24,0x22,0x20,
            0x00,0x00,0x00,0x00,
            0x00,0x00,0x04,0xD2,
            0x00,0x00,0x00,0x08,
            0x00,0x00,0x00,0x0C,
            0x00,0x00,0x00,0x0F,
            ];

            let ack_nack_big_endian = parse_ack_nack_submessage(&ack_nack_submessage_big_endian, &0).unwrap();
            assert_eq!(ack_nack_big_endian.final_flag, false);
            assert_eq!(ack_nack_big_endian.reader_id, 269620246);
            assert_eq!(ack_nack_big_endian.writer_id, 639902240);
            assert_eq!(ack_nack_big_endian.count, 15);
            assert_eq!(ack_nack_big_endian.reader_sn_state,
                vec![(1234, false),(1235, false), (1236, true), (1237, true),
                    (1238, false),(1239, false), (1240, false), (1241, false),] );

            let ack_nack_big_endian_final = parse_ack_nack_submessage(&ack_nack_submessage_big_endian, &2).unwrap();
            assert_eq!(ack_nack_big_endian_final.final_flag, true);
            assert_eq!(ack_nack_big_endian_final.reader_id, 269620246);
            assert_eq!(ack_nack_big_endian_final.writer_id, 639902240);
            assert_eq!(ack_nack_big_endian_final.count, 15);
            assert_eq!(ack_nack_big_endian_final.reader_sn_state,
                vec![(1234, false),(1235, false), (1236, true), (1237, true),
                    (1238, false),(1239, false), (1240, false), (1241, false),] );
        }

        {
            let ack_nack_submessage_little_endian = [
            0x16,0x14,0x12,0x10,
            0x20,0x22,0x24,0x26,
            0x00,0x00,0x00,0x00,
            0xD2,0x04,0x00,0x00,
            0x08,0x00,0x00,0x00,
            0x0C,0x00,0x00,0x00,
            0x0F,0x00,0x00,0x00,
            ];

            let ack_nack_little_endian = parse_ack_nack_submessage(&ack_nack_submessage_little_endian, &1).unwrap();
            assert_eq!(ack_nack_little_endian.final_flag, false);
            assert_eq!(ack_nack_little_endian.reader_id, 269620246);
            assert_eq!(ack_nack_little_endian.writer_id, 639902240);
            assert_eq!(ack_nack_little_endian.count, 15);
            assert_eq!(ack_nack_little_endian.reader_sn_state,
                vec![(1234, false),(1235, false), (1236, true), (1237, true),
                    (1238, false),(1239, false), (1240, false), (1241, false),] );

            let ack_nack_little_endian_final = parse_ack_nack_submessage(&ack_nack_submessage_little_endian, &3).unwrap();
            assert_eq!(ack_nack_little_endian_final.final_flag, true);
            assert_eq!(ack_nack_little_endian_final.reader_id, 269620246);
            assert_eq!(ack_nack_little_endian_final.writer_id, 639902240);
            assert_eq!(ack_nack_little_endian_final.count, 15);
            assert_eq!(ack_nack_little_endian_final.reader_sn_state,
                vec![(1234, false),(1235, false), (1236, true), (1237, true),
                    (1238, false),(1239, false), (1240, false), (1241, false),] );
        }


        
    }

    #[test]
    fn test_parse_data_submessage() {
        {
            // Test big endian
            let submessage = [
                0x00, 0x00, 0x00, 0x10,
                0x10, 0x12, 0x14, 0x16,
                0x26, 0x24, 0x22, 0x20,
                0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x04, 0xD1,
                0x00, 0x05, 0x00, 0x04,
                0x01, 0x02, 0x03, 0x04,
                0x00, 0x10, 0x00, 0x08,
                0x10, 0x11, 0x12, 0x13,
                0x14, 0x15, 0x16, 0x17,
                0x00, 0x01, 0x00, 0x00,
                0x20, 0x30, 0x40, 0x50,
            ];

            {
                // Parse message without considering inline qos or data
                let data = parse_data_submessage(&submessage, &0).unwrap();
                assert_eq!(data.endianess, EndianessFlag::BigEndian);
                assert_eq!(data.reader_id, 269620246);
                assert_eq!(data.writer_id, 639902240);
                assert_eq!(data.writer_sn, 1233);
                assert_eq!(data.inline_qos, None);
                assert_eq!(data.serialized_payload, Payload::None);
            }

            {
                // Parse message considering inline qos but no data
                let data = parse_data_submessage(&submessage, &2).unwrap();
                assert_eq!(data.endianess, EndianessFlag::BigEndian);
                assert_eq!(data.reader_id, 269620246);
                assert_eq!(data.writer_id, 639902240);
                assert_eq!(data.writer_sn, 1233);
                assert_eq!(data.serialized_payload, Payload::None);
                let inline_qos = data.inline_qos.unwrap();
                assert_eq!(inline_qos.len(),2);
                assert_eq!(inline_qos[0].parameter_id, 5);
                assert_eq!(inline_qos[0].value, vec!(1,2,3,4));
                assert_eq!(inline_qos[1].parameter_id, 16);
                assert_eq!(inline_qos[1].value, vec!(16,17,18,19,20,21,22,23));
            }

            {
                // Parse message considering serialized data and inline qos
                let data = parse_data_submessage(&submessage, &6).unwrap();
                assert_eq!(data.endianess, EndianessFlag::BigEndian);
                assert_eq!(data.reader_id, 269620246);
                assert_eq!(data.writer_id, 639902240);
                assert_eq!(data.writer_sn, 1233);
                let inline_qos = data.inline_qos.unwrap();
                assert_eq!(inline_qos.len(),2);
                assert_eq!(inline_qos[0].parameter_id, 5);
                assert_eq!(inline_qos[0].value, vec!(1,2,3,4));
                assert_eq!(inline_qos[1].parameter_id, 16);
                assert_eq!(inline_qos[1].value, vec!(16,17,18,19,20,21,22,23));
                if let Payload::Data(serialized_data) = data.serialized_payload {
                    assert_eq!(serialized_data, vec!(0x20, 0x30, 0x40, 0x50,));

                } else {
                    assert!(false);
                }
            }

            {
                // Parse message considering serialized key and inline qos
                let data = parse_data_submessage(&submessage, &10).unwrap();
                assert_eq!(data.endianess, EndianessFlag::BigEndian);
                assert_eq!(data.reader_id, 269620246);
                assert_eq!(data.writer_id, 639902240);
                assert_eq!(data.writer_sn, 1233);
                let inline_qos = data.inline_qos.unwrap();
                assert_eq!(inline_qos.len(),2);
                assert_eq!(inline_qos[0].parameter_id, 5);
                assert_eq!(inline_qos[0].value, vec!(1,2,3,4));
                assert_eq!(inline_qos[1].parameter_id, 16);
                assert_eq!(inline_qos[1].value, vec!(16,17,18,19,20,21,22,23));
                if let Payload::Key(serialized_data) = data.serialized_payload {
                    assert_eq!(serialized_data, vec!(0x20, 0x30, 0x40, 0x50,));

                } else {
                    assert!(false);
                }
            }

            {
                // Parse message considering serialized data and no inline qos
                let data = parse_data_submessage(&submessage, &8).unwrap();
                assert_eq!(data.endianess, EndianessFlag::BigEndian);
                assert_eq!(data.reader_id, 269620246);
                assert_eq!(data.writer_id, 639902240);
                assert_eq!(data.writer_sn, 1233);
                assert_eq!(data.inline_qos, None);
                if let Payload::Key(serialized_data) = data.serialized_payload {
                    assert_eq!(serialized_data, vec!(
                                0x00, 0x05, 0x00, 0x04,
                                0x01, 0x02, 0x03, 0x04,
                                0x00, 0x10, 0x00, 0x08,
                                0x10, 0x11, 0x12, 0x13,
                                0x14, 0x15, 0x16, 0x17,
                                0x00, 0x01, 0x00, 0x00,
                                0x20, 0x30, 0x40, 0x50,));
                } else {
                    assert!(false);
                }
            }
        }
    }

    #[test]
    fn test_parse_data_frag_submessage() {
        parse_data_frag_submessage(&[0,0], &0);
    }

    #[test]
    fn test_parse_gap_submessage() {
        {
            let submessage_big_endian = [ 
                0x10,0x12,0x14,0x16,
                0x26,0x24,0x22,0x20,
                0x00,0x00,0x00,0x00,
                0x00,0x00,0x04,0xD1,
                0x00,0x00,0x00,0x00,
                0x00,0x00,0x04,0xD2,
                0x00,0x00,0x00,0x08,
                0x00,0x00,0x00,0x0C,
            ];

            let gap_big_endian = parse_gap_submessage(&submessage_big_endian, &0).unwrap(); 

            assert_eq!(gap_big_endian.reader_id, 269620246);
            assert_eq!(gap_big_endian.writer_id, 639902240);
            assert_eq!(gap_big_endian.gap_start, 1233);
            assert_eq!(gap_big_endian.gap_list.len(), 8);
            assert_eq!(gap_big_endian.gap_list, 
                vec![(1234, false), (1235, false), (1236, true), (1237, true),
                     (1238, false), (1239, false), (1240, false), (1241, false)])
        }

        {
            let submessage_little_endian = [ 
                0x16,0x14,0x12,0x10,
                0x20,0x22,0x24,0x26,
                0x00,0x00,0x00,0x00,
                0xD1,0x04,0x00,0x00,
                0x00,0x00,0x00,0x00,
                0xD2,0x04,0x00,0x00,
                0x08,0x00,0x00,0x00,
                0x0C,0x00,0x00,0x00,
            ];

            let gap_little_endian = parse_gap_submessage(&submessage_little_endian, &1).unwrap(); 

            assert_eq!(gap_little_endian.reader_id, 269620246);
            assert_eq!(gap_little_endian.writer_id, 639902240);
            assert_eq!(gap_little_endian.gap_start, 1233);
            assert_eq!(gap_little_endian.gap_list.len(), 8);
            assert_eq!(gap_little_endian.gap_list, 
                vec![(1234, false), (1235, false), (1236, true), (1237, true),
                     (1238, false), (1239, false), (1240, false), (1241, false)])
        }

        {
            let submessage_big_endian = [ 
                0x10,0x12,0x14,0x16,
                0x26,0x24,0x22,0x20,
                0x80,0x00,0x00,0x00,
                0x00,0x00,0x04,0xD1,
                0x00,0x00,0x00,0x00,
                0x00,0x00,0x04,0xD2,
                0x00,0x00,0x00,0x08,
                0x00,0x00,0x00,0x0C,
            ];

            let gap_big_endian = parse_gap_submessage(&submessage_big_endian, &0);

            if let Err(ErrorMessage::InvalidSubmessage) = gap_big_endian {
                assert!(true);
            } else {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_parse_heartbeat_submessage() {
        {
            let submessage_big_endian = [ 
                    0x10,0x12,0x14,0x16,
                    0x26,0x24,0x22,0x20,
                    0x00,0x00,0x00,0x00,
                    0x00,0x00,0x04,0xD1,
                    0x00,0x00,0x00,0x00,
                    0x00,0x00,0x04,0xD5,
                    0x00,0x00,0x00,0x08,
                ];

            let heartbeat_big_endian = parse_heartbeat_submessage(&submessage_big_endian, &0).unwrap(); 
            assert_eq!(heartbeat_big_endian.reader_id, 269620246);
            assert_eq!(heartbeat_big_endian.writer_id, 639902240);
            assert_eq!(heartbeat_big_endian.first_sn, 1233);
            assert_eq!(heartbeat_big_endian.last_sn, 1237);
            assert_eq!(heartbeat_big_endian.count,8);
            assert_eq!(heartbeat_big_endian.final_flag, false);
            assert_eq!(heartbeat_big_endian.liveliness_flag, false);
        }

        {
            let submessage_little_endian = [ 
                    0x16,0x14,0x12,0x10,
                    0x20,0x22,0x24,0x26,
                    0x00,0x00,0x00,0x00,
                    0xD1,0x04,0x00,0x00,
                    0x00,0x00,0x00,0x00,
                    0xD5,0x04,0x00,0x00,
                    0x08,0x00,0x00,0x00,
                ];

            let heartbeat_little_endian = parse_heartbeat_submessage(&submessage_little_endian, &7).unwrap(); 
            assert_eq!(heartbeat_little_endian.reader_id, 269620246);
            assert_eq!(heartbeat_little_endian.writer_id, 639902240);
            assert_eq!(heartbeat_little_endian.first_sn, 1233);
            assert_eq!(heartbeat_little_endian.last_sn, 1237);
            assert_eq!(heartbeat_little_endian.count,8);
            assert_eq!(heartbeat_little_endian.final_flag, true);
            assert_eq!(heartbeat_little_endian.liveliness_flag, true);
        }

        {
            // Test first 
            let submessage_big_endian = [ 
                    0x10,0x12,0x14,0x16,
                    0x26,0x24,0x22,0x20,
                    0x00,0x00,0x00,0x00,
                    0x00,0x00,0x00,0x01,
                    0x00,0x00,0x00,0x00,
                    0x00,0x00,0x00,0x00,
                    0x00,0x00,0x00,0x08,
                ];

            let heartbeat_big_endian = parse_heartbeat_submessage(&submessage_big_endian, &2).unwrap(); 
            assert_eq!(heartbeat_big_endian.reader_id, 269620246);
            assert_eq!(heartbeat_big_endian.writer_id, 639902240);
            assert_eq!(heartbeat_big_endian.first_sn, 1);
            assert_eq!(heartbeat_big_endian.last_sn, 0);
            assert_eq!(heartbeat_big_endian.count,8);
            assert_eq!(heartbeat_big_endian.final_flag, true);
            assert_eq!(heartbeat_big_endian.liveliness_flag, false);
        }

        {
            // Test last_sn < first_sn - 1
            let submessage_big_endian = [ 
                    0x10,0x12,0x14,0x16,
                    0x26,0x24,0x22,0x20,
                    0x00,0x00,0x00,0x00,
                    0x00,0x00,0x04,0xD1,
                    0x00,0x00,0x00,0x00,
                    0x00,0x00,0x04,0xCF,
                    0x00,0x00,0x00,0x08,
                ];

            let heartbeat_big_endian = parse_heartbeat_submessage(&submessage_big_endian, &0); 
            if let Err(ErrorMessage::InvalidSubmessage) = heartbeat_big_endian {
                assert!(true);
            } else {
                assert!(false);
            }    
        }

        {
            // Test first_sn = 0
            let submessage_big_endian = [ 
                    0x10,0x12,0x14,0x16,
                    0x26,0x24,0x22,0x20,
                    0x00,0x00,0x00,0x00,
                    0x00,0x00,0x00,0x00,
                    0x00,0x00,0x00,0x00,
                    0x00,0x00,0x04,0xD5,
                    0x00,0x00,0x00,0x08,
                ];

            let heartbeat_big_endian = parse_heartbeat_submessage(&submessage_big_endian, &0); 
            if let Err(ErrorMessage::InvalidSubmessage) = heartbeat_big_endian {
                assert!(true);
            } else {
                assert!(false);
            }    
        }

        {
            // Test last_sn < 0
            let submessage_big_endian = [ 
                    0x10,0x12,0x14,0x16,
                    0x26,0x24,0x22,0x20,
                    0x00,0x00,0x00,0x00,
                    0x00,0x00,0x04,0xD1,
                    0x80,0x00,0x00,0x00,
                    0x00,0x00,0x04,0xD5,
                    0x00,0x00,0x00,0x08,
                ];

            let heartbeat_big_endian = parse_heartbeat_submessage(&submessage_big_endian, &0); 
            if let Err(ErrorMessage::InvalidSubmessage) = heartbeat_big_endian {
                assert!(true);
            } else {
                assert!(false);
            }    
        }
    }

    #[test]
    fn test_heartbeat_frag_submessage() {
        parse_heartbeat_frag_submessage(&[0,0], &0);
    }

    #[test]
    fn test_parse_info_dst_submessage() {
        {
            let submessage_big_endian = [
                10,11,12,13,
                14,15,16,17,
                18,19,20,21,
            ];
            let info_dst_big_endian = parse_info_dst_submessage(&submessage_big_endian, &0).unwrap();
            assert_eq!(info_dst_big_endian.guid_prefix,[10,11,12,13,14,15,16,17,18,19,20,21]);
        }

        {
            let submessage_little_endian = [
                10,11,12,13,
                14,15,16,17,
                18,19,20,21,
            ];
            let info_dst_little_endian = parse_info_dst_submessage(&submessage_little_endian, &1).unwrap();
            assert_eq!(info_dst_little_endian.guid_prefix,[10,11,12,13,14,15,16,17,18,19,20,21]);
        }
    }

    #[test]
    fn test_parse_info_reply_submessage() {
        parse_info_reply_submessage(&[0,0], &0);
    }

    #[test]
    fn test_parse_info_source_submessage() {
        parse_info_source_submessage(&[0,0], &0);
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
    fn test_parse_nack_frag_submessage() {
        parse_nack_frag_submessage(&[0,0], &0);
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
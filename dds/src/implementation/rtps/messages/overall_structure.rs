use std::{io::BufRead, sync::Arc};

use crate::implementation::rtps::{
    messages::{
        submessages::{
            ack_nack::AckNackSubmessageRead, data::DataSubmessageRead,
            data_frag::DataFragSubmessageRead, gap::GapSubmessageRead,
            heartbeat::HeartbeatSubmessageRead, heartbeat_frag::HeartbeatFragSubmessageRead,
            info_destination::InfoDestinationSubmessageRead, info_reply::InfoReplySubmessageRead,
            info_source::InfoSourceSubmessageRead, info_timestamp::InfoTimestampSubmessageRead,
            nack_frag::NackFragSubmessageRead, pad::PadSubmessageRead,
        },
        types::{
            ACKNACK, DATA, DATA_FRAG, GAP, HEARTBEAT, HEARTBEAT_FRAG, INFO_DST, INFO_REPLY,
            INFO_SRC, INFO_TS, NACK_FRAG, PAD,
        },
    },
    types::{GuidPrefix, ProtocolVersion, VendorId},
};

use super::{
    submessage_elements::SubmessageElement,
    submessages::{
        ack_nack::AckNackSubmessageWrite, data::DataSubmessageWrite,
        data_frag::DataFragSubmessageWrite, gap::GapSubmessageWrite,
        heartbeat::HeartbeatSubmessageWrite, heartbeat_frag::HeartbeatFragSubmessageWrite,
        info_destination::InfoDestinationSubmessageWrite, info_reply::InfoReplySubmessageWrite,
        info_source::InfoSourceSubmessageWrite, info_timestamp::InfoTimestampSubmessageWrite,
        nack_frag::NackFragSubmessageWrite, pad::PadSubmessageWrite,
    },
    types::{ProtocolId, SubmessageFlag, SubmessageKind},
};

const BUFFER_SIZE: usize = 65000;

pub trait Submessage {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite;
    fn submessage_elements(&self) -> &[SubmessageElement];
    fn endianness_flag(&self) -> bool;
}

impl<T> WriteBytes for T
where
    T: Submessage,
{
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        let mut len = 4;
        for submessage_element in self.submessage_elements() {
            len += if self.endianness_flag() {
                submessage_element.endian_write_bytes::<byteorder::LittleEndian>(&mut buf[len..])
            } else {
                submessage_element.endian_write_bytes::<byteorder::BigEndian>(&mut buf[len..])
            };
        }
        let octets_to_next_header = len - 4;
        let submessage_header = self.submessage_header(octets_to_next_header as u16);
        if self.endianness_flag() {
            submessage_header.endian_write_bytes::<byteorder::LittleEndian>(&mut buf[0..]);
        } else {
            submessage_header.endian_write_bytes::<byteorder::BigEndian>(&mut buf[0..]);
        }
        len
    }
}

pub trait FromBytes<'a> {
    fn from_bytes<E: byteorder::ByteOrder>(v: &'a [u8]) -> Self;
}

pub trait SubmessageHeader {
    fn submessage_header(&self) -> SubmessageHeaderRead;
}

pub trait RtpsMap<'a>: SubmessageHeader {
    fn map<T: FromBytes<'a>>(&self, data: &'a [u8]) -> T {
        if self.submessage_header().endianness_flag() {
            T::from_bytes::<byteorder::LittleEndian>(data)
        } else {
            T::from_bytes::<byteorder::BigEndian>(data)
        }
    }
}

impl<'a, T: SubmessageHeader> RtpsMap<'a> for T {}

pub trait EndiannessFlag {
    fn endianness_flag(&self) -> bool;
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RtpsMessageRead {
    pub data: Arc<[u8; BUFFER_SIZE]>,
}

impl RtpsMessageRead {
    pub fn new(data: &[u8]) -> Self {
        let mut buf = [0u8; BUFFER_SIZE];
        buf[0..data.len()].copy_from_slice(data);
        Self {
            data: Arc::new(buf),
        }
    }

    pub fn header(&self) -> RtpsMessageHeader {
        let v = &self.data;
        let protocol = ProtocolId::PROTOCOL_RTPS; //([v[0], v[1], v[2], v[3]]);
        let major = v[4];
        let minor = v[5];
        let version = ProtocolVersion::new(major, minor);
        let vendor_id = VendorId::new([v[6], v[7]]);
        let guid_prefix = GuidPrefix::new([
            v[8], v[9], v[10], v[11], v[12], v[13], v[14], v[15], v[16], v[17], v[18], v[19],
        ]);
        RtpsMessageHeader {
            protocol,
            version,
            vendor_id,
            guid_prefix,
        }
    }

    pub fn submessages(&self) -> Vec<RtpsSubmessageReadKind> {
        let mut buf = &self.data[20..];
        const MAX_SUBMESSAGES: usize = 2_usize.pow(16);
        let mut submessages = vec![];
        for _ in 0..MAX_SUBMESSAGES {
            if buf.len() < 4 {
                break;
            }
            let submessage_id = buf[0];
            let endianness_flag = (buf[1] & 0b_0000_0001) != 0;
            let submessage_length = if endianness_flag {
                u16::from_le_bytes([buf[2], buf[3]])
            } else {
                u16::from_be_bytes([buf[2], buf[3]])
            } as usize
                + 4;

            let submessage_data = &buf[..submessage_length];

            let submessage = match submessage_id {
                ACKNACK => {
                    RtpsSubmessageReadKind::AckNack(AckNackSubmessageRead::new(submessage_data))
                }
                DATA => RtpsSubmessageReadKind::Data(DataSubmessageRead::new(submessage_data)),
                DATA_FRAG => {
                    RtpsSubmessageReadKind::DataFrag(DataFragSubmessageRead::new(submessage_data))
                }
                GAP => RtpsSubmessageReadKind::Gap(GapSubmessageRead::new(submessage_data)),
                HEARTBEAT => {
                    RtpsSubmessageReadKind::Heartbeat(HeartbeatSubmessageRead::new(submessage_data))
                }
                HEARTBEAT_FRAG => RtpsSubmessageReadKind::HeartbeatFrag(
                    HeartbeatFragSubmessageRead::new(submessage_data),
                ),
                INFO_DST => RtpsSubmessageReadKind::InfoDestination(
                    InfoDestinationSubmessageRead::new(submessage_data),
                ),
                INFO_REPLY => {
                    RtpsSubmessageReadKind::InfoReply(InfoReplySubmessageRead::new(submessage_data))
                }
                INFO_SRC => RtpsSubmessageReadKind::InfoSource(InfoSourceSubmessageRead::new(
                    submessage_data,
                )),
                INFO_TS => RtpsSubmessageReadKind::InfoTimestamp(InfoTimestampSubmessageRead::new(
                    submessage_data,
                )),
                NACK_FRAG => {
                    RtpsSubmessageReadKind::NackFrag(NackFragSubmessageRead::new(submessage_data))
                }
                PAD => RtpsSubmessageReadKind::Pad(PadSubmessageRead::new(submessage_data)),
                _ => {
                    buf.consume(submessage_length);
                    continue;
                }
            };

            buf.consume(submessage_length);
            submessages.push(submessage);
        }
        submessages
    }
}

pub trait WriteBytes {
    fn write_bytes(&self, buf: &mut [u8]) -> usize;
}

pub trait EndianWriteBytes {
    fn endian_write_bytes<E: byteorder::ByteOrder>(&self, buf: &mut [u8]) -> usize;
}

#[allow(dead_code)]
pub fn into_bytes_le_vec<T: EndianWriteBytes>(value: T) -> Vec<u8> {
    let mut buf = [0u8; BUFFER_SIZE];
    let len = value.endian_write_bytes::<byteorder::LittleEndian>(buf.as_mut_slice());
    Vec::from(&buf[0..len])
}

#[allow(dead_code)]
pub fn into_bytes_vec<T: WriteBytes>(value: T) -> Vec<u8> {
    let mut buf = [0u8; BUFFER_SIZE];
    let len = value.write_bytes(buf.as_mut_slice());
    Vec::from(&buf[0..len])
}

#[derive(Debug, PartialEq, Eq)]
pub struct RtpsMessageWrite {
    buffer: [u8; BUFFER_SIZE],
    len: usize,
}

impl RtpsMessageWrite {
    pub fn new(header: RtpsMessageHeader, submessages: Vec<RtpsSubmessageWriteKind<'_>>) -> Self {
        let mut buffer = [0; BUFFER_SIZE];
        let mut len = header.endian_write_bytes::<byteorder::LittleEndian>(&mut buffer[0..]);
        for submessage in &submessages {
            len += submessage.write_bytes(&mut buffer[len..]);
        }
        Self { buffer, len }
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer[0..self.len]
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RtpsSubmessageReadKind<'a> {
    AckNack(AckNackSubmessageRead<'a>),
    Data(DataSubmessageRead<'a>),
    DataFrag(DataFragSubmessageRead<'a>),
    Gap(GapSubmessageRead<'a>),
    Heartbeat(HeartbeatSubmessageRead<'a>),
    HeartbeatFrag(HeartbeatFragSubmessageRead<'a>),
    InfoDestination(InfoDestinationSubmessageRead<'a>),
    InfoReply(InfoReplySubmessageRead<'a>),
    InfoSource(InfoSourceSubmessageRead<'a>),
    InfoTimestamp(InfoTimestampSubmessageRead<'a>),
    NackFrag(NackFragSubmessageRead<'a>),
    Pad(PadSubmessageRead<'a>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RtpsSubmessageWriteKind<'a> {
    AckNack(AckNackSubmessageWrite<'a>),
    Data(DataSubmessageWrite<'a>),
    DataFrag(DataFragSubmessageWrite<'a>),
    Gap(GapSubmessageWrite<'a>),
    Heartbeat(HeartbeatSubmessageWrite<'a>),
    HeartbeatFrag(HeartbeatFragSubmessageWrite<'a>),
    InfoDestination(InfoDestinationSubmessageWrite<'a>),
    InfoReply(InfoReplySubmessageWrite<'a>),
    InfoSource(InfoSourceSubmessageWrite<'a>),
    InfoTimestamp(InfoTimestampSubmessageWrite<'a>),
    NackFrag(NackFragSubmessageWrite<'a>),
    Pad(PadSubmessageWrite),
}

impl WriteBytes for RtpsSubmessageWriteKind<'_> {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        match self {
            RtpsSubmessageWriteKind::AckNack(s) => s.write_bytes(buf),
            RtpsSubmessageWriteKind::Data(s) => s.write_bytes(buf),
            RtpsSubmessageWriteKind::DataFrag(s) => s.write_bytes(buf),
            RtpsSubmessageWriteKind::Gap(s) => s.write_bytes(buf),
            RtpsSubmessageWriteKind::Heartbeat(s) => s.write_bytes(buf),
            RtpsSubmessageWriteKind::HeartbeatFrag(s) => s.write_bytes(buf),
            RtpsSubmessageWriteKind::InfoDestination(s) => s.write_bytes(buf),
            RtpsSubmessageWriteKind::InfoReply(s) => s.write_bytes(buf),
            RtpsSubmessageWriteKind::InfoSource(s) => s.write_bytes(buf),
            RtpsSubmessageWriteKind::InfoTimestamp(s) => s.write_bytes(buf),
            RtpsSubmessageWriteKind::NackFrag(s) => s.write_bytes(buf),
            RtpsSubmessageWriteKind::Pad(s) => s.write_bytes(buf),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct RtpsMessageHeader {
    protocol: ProtocolId,
    version: ProtocolVersion,
    vendor_id: VendorId,
    guid_prefix: GuidPrefix,
}

impl RtpsMessageHeader {
    pub fn new(version: ProtocolVersion, vendor_id: VendorId, guid_prefix: GuidPrefix) -> Self {
        Self {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version,
            vendor_id,
            guid_prefix,
        }
    }

    pub fn protocol(&self) -> ProtocolId {
        self.protocol
    }

    pub fn version(&self) -> ProtocolVersion {
        self.version
    }

    pub fn vendor_id(&self) -> VendorId {
        self.vendor_id
    }

    pub fn guid_prefix(&self) -> GuidPrefix {
        self.guid_prefix
    }
}

impl EndianWriteBytes for RtpsMessageHeader {
    fn endian_write_bytes<E: byteorder::ByteOrder>(&self, buf: &mut [u8]) -> usize {
        self.protocol.endian_write_bytes::<E>(&mut buf[0..]);
        self.version.endian_write_bytes::<E>(&mut buf[4..]);
        self.vendor_id.endian_write_bytes::<E>(&mut buf[6..]);
        self.guid_prefix.endian_write_bytes::<E>(&mut buf[8..]);
        20
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SubmessageHeaderWrite {
    submessage_id: SubmessageKind,
    flags: [SubmessageFlag; 8],
    submessage_length: u16,
}

impl SubmessageHeaderWrite {
    pub fn new(
        submessage_id: SubmessageKind,
        flags: &[SubmessageFlag],
        submessage_length: u16,
    ) -> Self {
        let mut flags_array = [false; 8];
        flags_array[..flags.len()].copy_from_slice(flags);

        Self {
            submessage_id,
            flags: flags_array,
            submessage_length,
        }
    }
}

impl EndianWriteBytes for SubmessageHeaderWrite {
    fn endian_write_bytes<E: byteorder::ByteOrder>(&self, buf: &mut [u8]) -> usize {
        self.submessage_id.write_bytes(&mut buf[0..]);
        self.flags.write_bytes(&mut buf[1..]);
        self.submessage_length
            .endian_write_bytes::<E>(&mut buf[2..]);
        4
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SubmessageHeaderRead<'a> {
    data: &'a [u8],
}

impl<'a> SubmessageHeaderRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn flags(&self) -> [SubmessageFlag; 8] {
        let flags_byte = self.data[1];
        [
            self.endianness_flag(),
            flags_byte & 0b_0000_0010 != 0,
            flags_byte & 0b_0000_0100 != 0,
            flags_byte & 0b_0000_1000 != 0,
            flags_byte & 0b_0001_0000 != 0,
            flags_byte & 0b_0010_0000 != 0,
            flags_byte & 0b_0100_0000 != 0,
            flags_byte & 0b_1000_0000 != 0,
        ]
    }
    pub fn _submessage_length(&self) -> u16 {
        let length_bytes = [self.data[2], self.data[3]];
        match self.endianness_flag() {
            true => u16::from_le_bytes(length_bytes),
            false => u16::from_be_bytes(length_bytes),
        }
    }

    pub fn endianness_flag(&self) -> bool {
        self.data[1] & 0b_0000_0001 != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::{
        messages::{
            submessage_elements::{Parameter, ParameterList, SerializedPayload},
            submessages::{data::DataSubmessageRead, heartbeat::HeartbeatSubmessageRead},
            types::ParameterId,
        },
        types::{
            EntityId, EntityKey, SequenceNumber, USER_DEFINED_READER_GROUP,
            USER_DEFINED_READER_NO_KEY,
        },
    };

    #[test]
    fn serialize_rtps_message_no_submessage() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion::new(2, 3),
            vendor_id: VendorId::new([9, 8]),
            guid_prefix: GuidPrefix::new([3; 12]),
        };
        let message = RtpsMessageWrite::new(header, Vec::new());
        #[rustfmt::skip]
        assert_eq!(message.buffer(), vec![
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]);
    }

    #[test]
    fn serialize_rtps_message() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion::new(2, 3),
            vendor_id: VendorId::new([9, 8]),
            guid_prefix: GuidPrefix::new([3; 12]),
        };
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::new(5);
        let parameter_1 = Parameter::new(ParameterId(6), vec![10, 11, 12, 13]);
        let parameter_2 = Parameter::new(ParameterId(7), vec![20, 21, 22, 23]);
        let inline_qos = &ParameterList::new(vec![parameter_1, parameter_2]);
        let serialized_payload = SerializedPayload::new(&[]);

        let submessage = RtpsSubmessageWriteKind::Data(DataSubmessageWrite::new(
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        ));
        let value = RtpsMessageWrite::new(header, vec![submessage]);
        #[rustfmt::skip]
        assert_eq!(value.buffer(), vec![
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            0x15, 0b_0000_0011, 40, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            6, 0, 4, 0, // inlineQos: parameterId_1, length_1
            10, 11, 12, 13, // inlineQos: value_1[length_1]
            7, 0, 4, 0, // inlineQos: parameterId_2, length_2
            20, 21, 22, 23, // inlineQos: value_2[length_2]
            1, 0, 0, 0, // inlineQos: Sentinel
        ]);
    }

    #[test]
    fn deserialize_rtps_message_no_submessage() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion::new(2, 3),
            vendor_id: VendorId::new([9, 8]),
            guid_prefix: GuidPrefix::new([3; 12]),
        };

        #[rustfmt::skip]
        let data = &[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ];
        let rtps_message = RtpsMessageRead::new(data);
        assert_eq!(header, rtps_message.header());
    }

    #[test]
    fn deserialize_rtps_message() {
        let expected_header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion::new(2, 3),
            vendor_id: VendorId::new([9, 8]),
            guid_prefix: GuidPrefix::new([3; 12]),
        };

        #[rustfmt::skip]
        let data_submessage = RtpsSubmessageReadKind::Data(DataSubmessageRead::new(
            &[0x15, 0b_0000_0011, 40, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            6, 0, 4, 0, // inlineQos: parameterId_1, length_1
            10, 11, 12, 13, // inlineQos: value_1[length_1]
            7, 0, 4, 0, // inlineQos: parameterId_2, length_2
            20, 21, 22, 23, // inlineQos: value_2[length_2]
            1, 0, 1, 0, // inlineQos: Sentinel
        ]));
        #[rustfmt::skip]
        let heartbeat_submessage = RtpsSubmessageReadKind::Heartbeat(HeartbeatSubmessageRead::new(&[
            0x07, 0b_0000_0101, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // firstSN: SequenceNumber: high
            5, 0, 0, 0, // firstSN: SequenceNumber: low
            0, 0, 0, 0, // lastSN: SequenceNumberSet: high
            7, 0, 0, 0, // lastSN: SequenceNumberSet: low
            2, 0, 0, 0, // count: Count: value (long)
        ]));

        let expected_submessages = vec![data_submessage, heartbeat_submessage];

        #[rustfmt::skip]
        let data = &[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            0x15, 0b_0000_0011, 40, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            6, 0, 4, 0, // inlineQos: parameterId_1, length_1
            10, 11, 12, 13, // inlineQos: value_1[length_1]
            7, 0, 4, 0, // inlineQos: parameterId_2, length_2
            20, 21, 22, 23, // inlineQos: value_2[length_2]
            1, 0, 1, 0, // inlineQos: Sentinel
            0x07, 0b_0000_0101, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // firstSN: SequenceNumber: high
            5, 0, 0, 0, // firstSN: SequenceNumber: low
            0, 0, 0, 0, // lastSN: SequenceNumberSet: high
            7, 0, 0, 0, // lastSN: SequenceNumberSet: low
            2, 0, 0, 0, // count: Count: value (long)
        ];

        let rtps_message = RtpsMessageRead::new(data);
        assert_eq!(expected_header, rtps_message.header());
        assert_eq!(expected_submessages, rtps_message.submessages());
    }

    #[test]
    fn deserialize_rtps_message_unknown_submessage() {
        #[rustfmt::skip]
        let submessage = RtpsSubmessageReadKind::Data(DataSubmessageRead::new(&[
            0x15, 0b_0000_0011, 40, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            6, 0, 4, 0, // inlineQos: parameterId_1, length_1
            10, 11, 12, 13, // inlineQos: value_1[length_1]
            7, 0, 4, 0, // inlineQos: parameterId_2, length_2
            20, 21, 22, 23, // inlineQos: value_2[length_2]
            1, 0, 0, 0, // inlineQos: Sentinel
        ]));
        let expected_submessages = vec![submessage];

        #[rustfmt::skip]
        let data = &[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            0x99, 0b_0101_0011, 4, 0, // Submessage header
            9, 9, 9, 9, // Unkown data
            0x15, 0b_0000_0011, 40, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            6, 0, 4, 0, // inlineQos: parameterId_1, length_1
            10, 11, 12, 13, // inlineQos: value_1[length_1]
            7, 0, 4, 0, // inlineQos: parameterId_2, length_2
            20, 21, 22, 23, // inlineQos: value_2[length_2]
            1, 0, 0, 0, // inlineQos: Sentinel
        ];

        let rtps_message = RtpsMessageRead::new(data);
        assert_eq!(expected_submessages, rtps_message.submessages());
    }
}

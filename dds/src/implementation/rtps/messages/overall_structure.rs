use std::io::BufRead;

use crate::implementation::{
    rtps::{
        messages::submessages::{
            ack_nack::AckNackSubmessageRead, data::DataSubmessageRead,
            data_frag::DataFragSubmessageRead, gap::GapSubmessageRead,
            heartbeat::HeartbeatSubmessageRead, heartbeat_frag::HeartbeatFragSubmessageRead,
            info_destination::InfoDestinationSubmessageRead, info_reply::InfoReplySubmessageRead,
            info_source::InfoSourceSubmessageRead, info_timestamp::InfoTimestampSubmessageRead,
            nack_frag::NackFragSubmessageRead, pad::PadSubmessageRead,
        },
        types::{GuidPrefix, ProtocolVersion, VendorId},
    },
    rtps_udp_psm::mapping_rtps_messages::submessages::submessage_header::{
        ACKNACK, DATA, DATA_FRAG, GAP, HEARTBEAT, HEARTBEAT_FRAG, INFO_DST, INFO_REPLY, INFO_SRC,
        INFO_TS, NACK_FRAG, PAD,
    },
};

use super::{
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

#[derive(Debug, PartialEq, Eq)]
pub struct RtpsMessageRead {
    pub data: [u8; BUFFER_SIZE],
}

impl RtpsMessageRead {
    pub fn new() -> Self {
        Self {
            data: [0; BUFFER_SIZE],
        }
    }

    pub fn header(&self) -> RtpsMessageHeader {
        let v = self.data;
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

    pub fn data_mut(&mut self) -> &mut [u8; BUFFER_SIZE] {
        &mut self.data
    }
}

impl Default for RtpsMessageRead {
    fn default() -> Self {
        Self::new()
    }
}

pub trait IntoBytes {
    fn into_bytes<E: byteorder::ByteOrder>(&self, buf: &mut [u8]) -> usize;
}

pub fn into_bytes_vec<T: IntoBytes>(value: T) -> Vec<u8> {
    let mut buf = [0u8; BUFFER_SIZE];
    let len = value.into_bytes::<byteorder::LittleEndian>(buf.as_mut_slice());
    Vec::from(&buf[0..len])
}

#[derive(Debug, PartialEq, Eq)]
pub struct RtpsMessageWrite<'a> {
    header: RtpsMessageHeader,
    submessages: Vec<RtpsSubmessageWriteKind<'a>>,
}

impl IntoBytes for RtpsMessageWrite<'_> {
    fn into_bytes<E: byteorder::ByteOrder>(&self, buf: &mut [u8]) -> usize {
        let mut len = self.header.into_bytes::<E>(buf);
        for submessage in &self.submessages {
            len += submessage.into_bytes::<E>(buf);
        }
        len
    }
}

impl<'a> RtpsMessageWrite<'a> {
    pub fn new(header: RtpsMessageHeader, submessages: Vec<RtpsSubmessageWriteKind<'a>>) -> Self {
        Self {
            header,
            submessages,
        }
    }

    pub fn header(&self) -> RtpsMessageHeader {
        self.header
    }

    pub fn submessages(&self) -> &[RtpsSubmessageWriteKind] {
        self.submessages.as_ref()
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
    AckNack(AckNackSubmessageWrite),
    Data(DataSubmessageWrite<'a>),
    DataFrag(DataFragSubmessageWrite<'a>),
    Gap(GapSubmessageWrite),
    Heartbeat(HeartbeatSubmessageWrite),
    HeartbeatFrag(HeartbeatFragSubmessageWrite),
    InfoDestination(InfoDestinationSubmessageWrite),
    InfoReply(InfoReplySubmessageWrite),
    InfoSource(InfoSourceSubmessageWrite),
    InfoTimestamp(InfoTimestampSubmessageWrite),
    NackFrag(NackFragSubmessageWrite),
    Pad(PadSubmessageWrite),
}

impl IntoBytes for RtpsSubmessageWriteKind<'_> {
    fn into_bytes<E: byteorder::ByteOrder>(&self, buf: &mut [u8]) -> usize {
        match self {
            // RtpsSubmessageWriteKind::AckNack(s) => {
            //     s.into_bytes(buf)
            // }
            RtpsSubmessageWriteKind::Data(s) => s.into_bytes::<E>(buf),
            // RtpsSubmessageWriteKind::DataFrag(s) => {
            //     s.into_bytes(buf)
            // }
            // RtpsSubmessageWriteKind::Gap(s) => {
            //     s.into_bytes(buf)
            // }
            // RtpsSubmessageWriteKind::Heartbeat(s) => {
            //     s.into_bytes(buf)
            // }
            // RtpsSubmessageWriteKind::HeartbeatFrag(s) => {
            //     s.into_bytes(buf)
            // }
            // RtpsSubmessageWriteKind::InfoDestination(s) => {
            //     s.into_bytes(buf)
            // }
            // RtpsSubmessageWriteKind::InfoReply(s) => {
            //     s.into_bytes(buf)
            // }
            // RtpsSubmessageWriteKind::InfoSource(s) => {
            //     s.into_bytes(buf)
            // }
            // RtpsSubmessageWriteKind::InfoTimestamp(s) => {
            //     s.into_bytes(buf)
            // }
            // RtpsSubmessageWriteKind::NackFrag(s) => {
            //     s.into_bytes(buf)
            // }
            // RtpsSubmessageWriteKind::Pad(s) => {
            //     s.into_bytes(buf)
            // }
            _ => todo!(),
        };
        0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct RtpsMessageHeader {
    pub protocol: ProtocolId,
    pub version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub guid_prefix: GuidPrefix,
}

impl IntoBytes for RtpsMessageHeader {
    fn into_bytes<E: byteorder::ByteOrder>(&self, buf: &mut [u8]) -> usize {
        self.protocol.into_bytes::<E>(&mut buf[0..]);
        self.version.into_bytes::<E>(&mut buf[4..]);
        self.vendor_id.into_bytes::<E>(&mut buf[6..]);
        self.guid_prefix.into_bytes::<E>(&mut buf[8..]);
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
        flags: [SubmessageFlag; 8],
        submessage_length: u16,
    ) -> Self {
        Self {
            submessage_id,
            flags,
            submessage_length,
        }
    }
}

impl IntoBytes for SubmessageHeaderWrite {
    fn into_bytes<E: byteorder::ByteOrder>(&self, buf: &mut [u8]) -> usize {
        self.submessage_id.into_bytes::<E>(buf);
        self.flags.into_bytes::<E>(buf);
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
            submessage_elements::{Parameter, ParameterList},
            submessages::{data::DataSubmessageRead, heartbeat::HeartbeatSubmessageRead},
            types::{ParameterId, SerializedPayload},
        },
        types::{
            EntityId, EntityKey, SequenceNumber, USER_DEFINED_READER_GROUP,
            USER_DEFINED_READER_NO_KEY,
        },
    };

    fn into_bytes_vec<T: IntoBytes>(value: T) -> Vec<u8> {
        let mut buf = [0u8; BUFFER_SIZE];
        let len = value.into_bytes::<byteorder::LittleEndian>(buf.as_mut_slice());
        Vec::from(&buf[0..len])
    }

    #[test]
    fn serialize_rtps_message_no_submessage() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion::new(2, 3),
            vendor_id: VendorId::new([9, 8]),
            guid_prefix: GuidPrefix::new([3; 12]),
        };
        let value = RtpsMessageWrite::new(header, Vec::new());
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(value), vec![
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
        let endianness_flag = true;
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
        ));
        let value = RtpsMessageWrite::new(header, vec![submessage]);
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(value), vec![
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
        let mut rtps_message = RtpsMessageRead::new();
        let data = &[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ];
        rtps_message.data[..data.len()].copy_from_slice(data);
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

        let mut rtps_message = RtpsMessageRead::new();
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
        rtps_message.data[..data.len()].copy_from_slice(data);
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

        let mut rtps_message = RtpsMessageRead::new();
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
        rtps_message.data[..data.len()].copy_from_slice(data);
        assert_eq!(expected_submessages, rtps_message.submessages());
    }
}

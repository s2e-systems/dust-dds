use super::{
    super::{
        error::{RtpsError, RtpsErrorKind, RtpsResult},
        messages::{
            submessages::{
                ack_nack::AckNackSubmessage, data::DataSubmessage, data_frag::DataFragSubmessage,
                gap::GapSubmessage, heartbeat::HeartbeatSubmessage,
                heartbeat_frag::HeartbeatFragSubmessage,
                info_destination::InfoDestinationSubmessage, info_reply::InfoReplySubmessage,
                info_source::InfoSourceSubmessage, info_timestamp::InfoTimestampSubmessage,
                nack_frag::NackFragSubmessage, pad::PadSubmessage,
            },
            types::{
                ACKNACK, DATA, DATA_FRAG, GAP, HEARTBEAT, HEARTBEAT_FRAG, INFO_DST, INFO_REPLY,
                INFO_SRC, INFO_TS, NACK_FRAG, PAD,
            },
        },
        types::{GuidPrefix, ProtocolVersion, VendorId},
    },
    types::{ProtocolId, SubmessageFlag, SubmessageKind},
};
use std::{
    io::{BufRead, Cursor, Write},
    sync::Arc,
};

pub enum Endianness {
    BigEndian,
    LittleEndian,
}

impl Endianness {
    pub fn from_flags(byte: u8) -> Self {
        match byte & 0b_0000_0001 != 0 {
            true => Endianness::LittleEndian,
            false => Endianness::BigEndian,
        }
    }
}

pub trait TryReadFromBytes: Sized {
    fn try_read_from_bytes(data: &mut &[u8], endianness: &Endianness) -> RtpsResult<Self>;
}

pub trait WriteIntoBytes {
    fn write_into_bytes(&self, buf: &mut dyn Write);
}

pub trait Submessage {
    fn write_submessage_header_into_bytes(&self, octets_to_next_header: u16, buf: &mut dyn Write);
    fn write_submessage_elements_into_bytes(&self, buf: &mut dyn Write);
}

impl dyn Submessage + Send + '_ {
    fn write_submessage_into_bytes(&self, buf: &mut Cursor<Vec<u8>>) {
        let header_position: u64 = buf.position();
        let elements_position = header_position + 4;
        buf.set_position(elements_position);
        self.write_submessage_elements_into_bytes(buf);
        let pos = buf.position();
        buf.set_position(header_position);
        let len = pos - elements_position;
        self.write_submessage_header_into_bytes(len as u16, buf);
        buf.set_position(pos);
    }
}

pub struct SubmessageHeaderRead {
    submessage_id: u8,
    flags: [SubmessageFlag; 8],
    submessage_length: u16,
    endianness: Endianness,
}

impl SubmessageHeaderRead {
    pub fn try_read_from_bytes(data: &mut &[u8]) -> RtpsResult<Self> {
        if data.len() >= 4 {
            let submessage_id = data[0];
            let flags_byte = data[1];
            let flags = [
                flags_byte & 0b_0000_0001 != 0,
                flags_byte & 0b_0000_0010 != 0,
                flags_byte & 0b_0000_0100 != 0,
                flags_byte & 0b_0000_1000 != 0,
                flags_byte & 0b_0001_0000 != 0,
                flags_byte & 0b_0010_0000 != 0,
                flags_byte & 0b_0100_0000 != 0,
                flags_byte & 0b_1000_0000 != 0,
            ];
            let endianness = match flags[0] {
                true => Endianness::LittleEndian,
                false => Endianness::BigEndian,
            };
            let submessage_length = u16::try_read_from_bytes(&mut &data[2..], &endianness)?;
            data.consume(4);
            Ok(Self {
                submessage_id,
                flags,
                submessage_length,
                endianness,
            })
        } else {
            Err(RtpsError::new(
                RtpsErrorKind::NotEnoughData,
                "Submessage header",
            ))
        }
    }

    pub fn endianness(&self) -> &Endianness {
        &self.endianness
    }

    pub fn flags(&self) -> [bool; 8] {
        self.flags
    }

    pub fn submessage_length(&self) -> u16 {
        self.submessage_length
    }

    pub fn submessage_id(&self) -> u8 {
        self.submessage_id
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RtpsMessageRead {
    header: RtpsMessageHeader,
    submessages: Vec<RtpsSubmessageReadKind>,
}

impl RtpsMessageRead {
    pub fn header(&self) -> RtpsMessageHeader {
        self.header
    }

    pub fn submessages(self) -> Vec<RtpsSubmessageReadKind> {
        self.submessages
    }
}

impl TryFrom<&[u8]> for RtpsMessageRead {
    type Error = RtpsError;

    fn try_from(mut v: &[u8]) -> RtpsResult<Self> {
        if v.len() >= 20 {
            if b"RTPS" == &[v[0], v[1], v[2], v[3]] {
                let major = v[4];
                let minor = v[5];
                let version = ProtocolVersion::new(major, minor);
                let vendor_id = [v[6], v[7]];
                let guid_prefix = [
                    v[8], v[9], v[10], v[11], v[12], v[13], v[14], v[15], v[16], v[17], v[18],
                    v[19],
                ];
                let header = RtpsMessageHeader {
                    version,
                    vendor_id,
                    guid_prefix,
                };
                v.consume(20);

                const MAX_SUBMESSAGES: usize = 2_usize.pow(16);
                let mut submessages = vec![];
                for _ in 0..MAX_SUBMESSAGES {
                    if v.len() < 4 {
                        break;
                    }
                    if let Ok(submessage_header) = SubmessageHeaderRead::try_read_from_bytes(&mut v)
                    {
                        let submessage_length = submessage_header.submessage_length() as usize;
                        if v.len() < submessage_length {
                            break;
                        }
                        let submessage = match submessage_header.submessage_id() {
                            ACKNACK => AckNackSubmessage::try_from_bytes(&submessage_header, v)
                                .map(RtpsSubmessageReadKind::AckNack),
                            DATA => DataSubmessage::try_from_bytes(&submessage_header, v)
                                .map(RtpsSubmessageReadKind::Data),
                            DATA_FRAG => DataFragSubmessage::try_from_bytes(&submessage_header, v)
                                .map(RtpsSubmessageReadKind::DataFrag),
                            GAP => GapSubmessage::try_from_bytes(&submessage_header, v)
                                .map(RtpsSubmessageReadKind::Gap),
                            HEARTBEAT => HeartbeatSubmessage::try_from_bytes(&submessage_header, v)
                                .map(RtpsSubmessageReadKind::Heartbeat),
                            HEARTBEAT_FRAG => {
                                HeartbeatFragSubmessage::try_from_bytes(&submessage_header, v)
                                    .map(RtpsSubmessageReadKind::HeartbeatFrag)
                            }
                            INFO_DST => {
                                InfoDestinationSubmessage::try_from_bytes(&submessage_header, v)
                                    .map(RtpsSubmessageReadKind::InfoDestination)
                            }
                            INFO_REPLY => {
                                InfoReplySubmessage::try_from_bytes(&submessage_header, v)
                                    .map(RtpsSubmessageReadKind::InfoReply)
                            }
                            INFO_SRC => InfoSourceSubmessage::try_from_bytes(&submessage_header, v)
                                .map(RtpsSubmessageReadKind::InfoSource),
                            INFO_TS => {
                                InfoTimestampSubmessage::try_from_bytes(&submessage_header, v)
                                    .map(RtpsSubmessageReadKind::InfoTimestamp)
                            }
                            NACK_FRAG => NackFragSubmessage::try_from_bytes(&submessage_header, v)
                                .map(RtpsSubmessageReadKind::NackFrag),
                            PAD => PadSubmessage::try_from_bytes(&submessage_header, v)
                                .map(RtpsSubmessageReadKind::Pad),
                            _ => Err(RtpsError::new(
                                RtpsErrorKind::InvalidData,
                                "Unknown message",
                            )),
                        };
                        if let Ok(submessage) = submessage {
                            submessages.push(submessage);
                        }
                        v.consume(submessage_length);
                    }
                }
                Ok(Self {
                    header,
                    submessages,
                })
            } else {
                Err(RtpsError::new(
                    RtpsErrorKind::InvalidData,
                    "RTPS not in data",
                ))
            }
        } else {
            Err(RtpsError::new(
                RtpsErrorKind::NotEnoughData,
                "Rtps message header",
            ))
        }
    }
}

#[allow(dead_code)] // Only used as convenience in tests
pub fn write_into_bytes_vec(value: impl WriteIntoBytes) -> Vec<u8> {
    let mut cursor = Cursor::new(Vec::new());
    value.write_into_bytes(&mut cursor);
    cursor.into_inner()
}

#[allow(dead_code)] // Only used as convenience in tests
pub fn write_submessage_into_bytes_vec(value: &(dyn Submessage + Send)) -> Vec<u8> {
    let mut cursor = Cursor::new(Vec::new());
    value.write_submessage_into_bytes(&mut cursor);
    cursor.into_inner()
}

#[derive(Debug, PartialEq, Eq)]
pub struct RtpsMessageWrite {
    data: Arc<[u8]>,
}

impl RtpsMessageWrite {
    pub fn new(header: &RtpsMessageHeader, submessages: &[Box<dyn Submessage + Send>]) -> Self {
        let buffer = Vec::new();
        let mut cursor = Cursor::new(buffer);
        header.write_into_bytes(&mut cursor);
        for submessage in submessages {
            submessage.write_submessage_into_bytes(&mut cursor);
        }
        Self {
            data: Arc::from(cursor.into_inner().into_boxed_slice()),
        }
    }

    pub fn buffer(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RtpsSubmessageReadKind {
    AckNack(AckNackSubmessage),
    Data(DataSubmessage),
    DataFrag(DataFragSubmessage),
    Gap(GapSubmessage),
    Heartbeat(HeartbeatSubmessage),
    HeartbeatFrag(HeartbeatFragSubmessage),
    InfoDestination(InfoDestinationSubmessage),
    InfoReply(InfoReplySubmessage),
    InfoSource(InfoSourceSubmessage),
    InfoTimestamp(InfoTimestampSubmessage),
    NackFrag(NackFragSubmessage),
    Pad(PadSubmessage),
}
#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct RtpsMessageHeader {
    version: ProtocolVersion,
    vendor_id: VendorId,
    guid_prefix: GuidPrefix,
}

impl RtpsMessageHeader {
    pub fn new(version: ProtocolVersion, vendor_id: VendorId, guid_prefix: GuidPrefix) -> Self {
        Self {
            version,
            vendor_id,
            guid_prefix,
        }
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

impl WriteIntoBytes for RtpsMessageHeader {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        ProtocolId::PROTOCOL_RTPS.write_into_bytes(buf);
        self.version.write_into_bytes(buf);
        self.vendor_id.write_into_bytes(buf);
        self.guid_prefix.write_into_bytes(buf);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SubmessageHeaderWrite {
    submessage_id: SubmessageKind,
    flags_octet: u8,
    submessage_length: u16,
}

impl SubmessageHeaderWrite {
    pub fn new(
        submessage_id: SubmessageKind,
        // flags without endianness
        flags: &[SubmessageFlag],
        submessage_length: u16,
    ) -> Self {
        let mut flags_octet = 0b_0000_0001_u8;
        for (i, &item) in flags.iter().enumerate() {
            if item {
                flags_octet |= 0b_0000_0010 << i
            }
        }

        Self {
            submessage_id,
            flags_octet,
            submessage_length,
        }
    }
}

impl WriteIntoBytes for SubmessageHeaderWrite {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        self.submessage_id.write_into_bytes(buf);
        self.flags_octet.write_into_bytes(buf);
        self.submessage_length.write_into_bytes(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtps::{
        messages::{
            submessage_elements::{Data, Parameter, ParameterList},
            submessages::{data::DataSubmessage, info_timestamp::InfoTimestampSubmessage},
            types::Time,
        },
        types::{EntityId, USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY},
    };

    #[test]
    fn serialize_rtps_message_no_submessage() {
        let header = RtpsMessageHeader {
            version: ProtocolVersion::new(2, 3),
            vendor_id: [9, 8],
            guid_prefix: [3; 12],
        };
        let message = RtpsMessageWrite::new(&header, &[]);
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
            version: ProtocolVersion::new(2, 3),
            vendor_id: [9, 8],
            guid_prefix: [3; 12],
        };
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let writer_sn = 5;
        let parameter_1 = Parameter::new(6, vec![10, 11, 12, 13].into());
        let parameter_2 = Parameter::new(7, vec![20, 21, 22, 23].into());
        let inline_qos = ParameterList::new(vec![parameter_1, parameter_2]);
        let serialized_payload = Data::new(vec![].into());

        let submessage = DataSubmessage::new(
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
        let value = RtpsMessageWrite::new(&header, &[Box::new(submessage)]);
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
    fn serialize_rtps_message_multiple_submessages() {
        let header = RtpsMessageHeader {
            version: ProtocolVersion::new(2, 3),
            vendor_id: [9, 8],
            guid_prefix: [3; 12],
        };
        let info_timestamp_submessage =
            Box::new(InfoTimestampSubmessage::new(false, Time::new(4, 0)));

        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new([1, 2, 3], USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new([6, 7, 8], USER_DEFINED_READER_GROUP);
        let writer_sn = 5;
        let parameter_1 = Parameter::new(6, vec![10, 11, 12, 13].into());
        let parameter_2 = Parameter::new(7, vec![20, 21, 22, 23].into());
        let inline_qos = ParameterList::new(vec![parameter_1, parameter_2]);
        let serialized_payload = Data::new(vec![].into());

        let data_submessage = Box::new(DataSubmessage::new(
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
        let value = RtpsMessageWrite::new(&header, &[info_timestamp_submessage, data_submessage]);
        #[rustfmt::skip]
        assert_eq!(value.buffer(), vec![
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            0x09_u8, 0b_0000_0001, 8, 0, // Submessage header
            4, 0, 0, 0, // Time
            0, 0, 0, 0, // Time
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
            version: ProtocolVersion::new(2, 3),
            vendor_id: [9, 8],
            guid_prefix: [3; 12],
        };

        #[rustfmt::skip]
        let data = [
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ];
        let rtps_message = RtpsMessageRead::try_from(&data[..]).unwrap();
        assert_eq!(rtps_message.header(), header);
        assert_eq!(rtps_message.submessages(), vec![]);
    }

    #[test]
    fn deserialize_rtps_message_too_high_submessage_length() {
        #[rustfmt::skip]
        let data = [
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            0x09_u8, 0b_0000_0001, 8, 0, // Submessage header
            4, 0, 0, 0, // Time (half only)
        ];
        let rtps_message = RtpsMessageRead::try_from(&data[..]).unwrap();
        assert_eq!(rtps_message.submessages(), vec![]);
    }

    #[test]
    fn deserialize_rtps_message() {
        let expected_header = RtpsMessageHeader {
            version: ProtocolVersion::new(2, 3),
            vendor_id: [9, 8],
            guid_prefix: [3; 12],
        };

        #[rustfmt::skip]
        let data = [
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
            0x07, 0b_0000_0101, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // firstSN: SequenceNumber: high
            5, 0, 0, 0, // firstSN: SequenceNumber: low
            0, 0, 0, 0, // lastSN: SequenceNumberSet: high
            7, 0, 0, 0, // lastSN: SequenceNumberSet: low
            2, 0, 0, 0, // count: Count: value (long)
        ];

        let rtps_message = RtpsMessageRead::try_from(&data[..]).unwrap();
        assert_eq!(rtps_message.header(), expected_header);
        let submessages = rtps_message.submessages();
        assert_eq!(submessages.len(), 2);
        assert!(matches!(submessages[0], RtpsSubmessageReadKind::Data(..)));
        assert!(matches!(
            submessages[1],
            RtpsSubmessageReadKind::Heartbeat(..)
        ));
    }

    #[test]
    fn deserialize_rtps_message_unknown_submessage() {
        let expected_data_submessage = RtpsSubmessageReadKind::Data(DataSubmessage::new(
            true,
            false,
            false,
            false,
            EntityId::new([1, 2, 3], 4),
            EntityId::new([6, 7, 8], 9),
            5,
            ParameterList::new(vec![
                Parameter::new(6, vec![10, 11, 12, 13].into()),
                Parameter::new(7, vec![20, 21, 22, 23].into()),
            ]),
            Data::default(),
        ));

        let expected_submessages = vec![expected_data_submessage];

        #[rustfmt::skip]
        let data = [
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

        let rtps_message = RtpsMessageRead::try_from(&data[..]).unwrap();
        assert_eq!(expected_submessages, rtps_message.submessages());
    }
}

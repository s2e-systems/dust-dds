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
use crate::{
    implementation::rtps::{
        messages::{
            submessage_elements::ArcSlice,
            submessages::{
                ack_nack::AckNackSubmessageRead, data::DataSubmessageRead,
                data_frag::DataFragSubmessageRead, gap::GapSubmessageRead,
                heartbeat::HeartbeatSubmessageRead, heartbeat_frag::HeartbeatFragSubmessageRead,
                info_destination::InfoDestinationSubmessageRead,
                info_reply::InfoReplySubmessageRead, info_source::InfoSourceSubmessageRead,
                info_timestamp::InfoTimestampSubmessageRead, nack_frag::NackFragSubmessageRead,
                pad::PadSubmessageRead,
            },
            types::{
                ACKNACK, DATA, DATA_FRAG, GAP, HEARTBEAT, HEARTBEAT_FRAG, INFO_DST, INFO_REPLY,
                INFO_SRC, INFO_TS, NACK_FRAG, PAD,
            },
        },
        types::{GuidPrefix, ProtocolVersion, VendorId},
    },
    infrastructure::error::{DdsError, DdsResult},
};
use std::{marker::PhantomData, sync::Arc};

pub(in crate::implementation::rtps) type WriteEndianness = byteorder::LittleEndian;
const BUFFER_SIZE: usize = 65000;

pub trait Submessage<'a> {
    type SubmessageList;

    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite;
    fn submessage_elements(&'a self) -> Self::SubmessageList;
}

#[inline]
fn write_submessage_bytes<'a>(
    submessage: &'a impl Submessage<
        'a,
        SubmessageList = impl IntoIterator<Item = &'a SubmessageElement<'a>>,
    >,
    buf: &mut [u8],
) -> usize {
    let (header, body) = buf.split_at_mut(4);
    let mut len = 0;
    for submessage_element in submessage.submessage_elements().into_iter() {
        len += submessage_element.write_bytes(&mut body[len..]);
    }
    let submessage_header = submessage.submessage_header(len as u16);
    submessage_header.write_bytes(header) + len
}

pub trait FromBytes {
    fn from_bytes<E: byteorder::ByteOrder>(v: &[u8]) -> Self;
}

pub trait SubmessageHeader {
    fn submessage_header(&self) -> SubmessageHeaderRead;
}

pub trait RtpsMap: SubmessageHeader {
    fn map<T: FromBytes>(&self, data: &[u8]) -> T {
        if self.submessage_header().flags()[0] {
            T::from_bytes::<byteorder::LittleEndian>(data)
        } else {
            T::from_bytes::<byteorder::BigEndian>(data)
        }
    }
}

impl<T: SubmessageHeader> RtpsMap for T {}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RtpsMessageRead {
    data: Arc<[u8]>,
}

impl RtpsMessageRead {
    pub fn new(data: Arc<[u8]>) -> DdsResult<Self> {
        if data.len() >= 20 {
            if b"RTPS" == &[data[0], data[1], data[2], data[3]] {
                Ok(Self { data })
            } else {
                Err(DdsError::Error("".to_string()))
            }
        } else {
            Err(DdsError::Error("".to_string()))
        }
    }

    pub fn header(&self) -> RtpsMessageHeader {
        let v = &self.data;
        let major = v[4];
        let minor = v[5];
        let version = ProtocolVersion::new(major, minor);
        let vendor_id = [v[6], v[7]];
        let guid_prefix = [
            v[8], v[9], v[10], v[11], v[12], v[13], v[14], v[15], v[16], v[17], v[18], v[19],
        ];
        RtpsMessageHeader {
            version,
            vendor_id,
            guid_prefix,
        }
    }

    pub fn submessages(&self) -> Vec<RtpsSubmessageReadKind> {
        let mut offset = 20;
        const MAX_SUBMESSAGES: usize = 2_usize.pow(16);
        let mut submessages = vec![];
        for _ in 0..MAX_SUBMESSAGES {
            if self.data.len() < offset {
                break;
            }
            let buf = &self.data[offset..];
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

            if buf.len() < submessage_length {
                break;
            }

            let submessage_data = &buf[..submessage_length];
            let submessage_arc_slice =
                ArcSlice::new(self.data.clone(), offset..offset + submessage_length);

            let submessage = match submessage_id {
                ACKNACK => AckNackSubmessageRead::try_from_bytes(submessage_data)
                    .map(RtpsSubmessageReadKind::AckNack),
                DATA => DataSubmessageRead::try_from_arc_slice(submessage_arc_slice)
                    .map(RtpsSubmessageReadKind::Data),
                DATA_FRAG => DataFragSubmessageRead::try_from_bytes(submessage_arc_slice)
                    .map(RtpsSubmessageReadKind::DataFrag),
                GAP => GapSubmessageRead::try_from_bytes(submessage_data)
                    .map(RtpsSubmessageReadKind::Gap),
                HEARTBEAT => HeartbeatSubmessageRead::try_from_bytes(submessage_data)
                    .map(RtpsSubmessageReadKind::Heartbeat),
                HEARTBEAT_FRAG => HeartbeatFragSubmessageRead::try_from_bytes(submessage_data)
                    .map(RtpsSubmessageReadKind::HeartbeatFrag),
                INFO_DST => InfoDestinationSubmessageRead::try_from_bytes(submessage_data)
                    .map(RtpsSubmessageReadKind::InfoDestination),
                INFO_REPLY => InfoReplySubmessageRead::try_from_bytes(submessage_data)
                    .map(RtpsSubmessageReadKind::InfoReply),
                INFO_SRC => InfoSourceSubmessageRead::try_from_bytes(submessage_data)
                    .map(RtpsSubmessageReadKind::InfoSource),
                INFO_TS => InfoTimestampSubmessageRead::try_from_bytes(submessage_data)
                    .map(RtpsSubmessageReadKind::InfoTimestamp),
                NACK_FRAG => NackFragSubmessageRead::try_from_bytes(submessage_data)
                    .map(RtpsSubmessageReadKind::NackFrag),
                PAD => PadSubmessageRead::try_from_bytes(submessage_data)
                    .map(RtpsSubmessageReadKind::Pad),
                _ => {
                    offset += submessage_length;
                    continue;
                }
            };
            offset += submessage_length;
            if let Ok(submessage) = submessage {
                submessages.push(submessage);
            }
        }
        submessages
    }
}

pub trait WriteBytes {
    fn write_bytes(&self, buf: &mut [u8]) -> usize;
}

#[allow(dead_code)] // Only used as convenience in tests
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
    pub fn new(header: &RtpsMessageHeader, submessages: &[RtpsSubmessageWriteKind<'_>]) -> Self {
        let mut buffer = [0; BUFFER_SIZE];
        let mut len = header.write_bytes(&mut buffer[0..]);
        for submessage in submessages {
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
    Data(DataSubmessageRead),
    DataFrag(DataFragSubmessageRead),
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

#[allow(dead_code)]
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
    #[inline]
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        match self {
            RtpsSubmessageWriteKind::AckNack(s) => write_submessage_bytes(s, buf),
            RtpsSubmessageWriteKind::Data(s) => write_submessage_bytes(s, buf),
            RtpsSubmessageWriteKind::DataFrag(s) => write_submessage_bytes(s, buf),
            RtpsSubmessageWriteKind::Gap(s) => write_submessage_bytes(s, buf),
            RtpsSubmessageWriteKind::Heartbeat(s) => write_submessage_bytes(s, buf),
            RtpsSubmessageWriteKind::HeartbeatFrag(s) => write_submessage_bytes(s, buf),
            RtpsSubmessageWriteKind::InfoDestination(s) => write_submessage_bytes(s, buf),
            RtpsSubmessageWriteKind::InfoReply(s) => write_submessage_bytes(s, buf),
            RtpsSubmessageWriteKind::InfoSource(s) => write_submessage_bytes(s, buf),
            RtpsSubmessageWriteKind::InfoTimestamp(s) => write_submessage_bytes(s, buf),
            RtpsSubmessageWriteKind::NackFrag(s) => write_submessage_bytes(s, buf),
            RtpsSubmessageWriteKind::Pad(s) => write_submessage_bytes(s, buf),
        }
    }
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

impl WriteBytes for RtpsMessageHeader {
    #[inline]
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        ProtocolId::PROTOCOL_RTPS.write_bytes(&mut buf[0..]);
        self.version.write_bytes(&mut buf[4..]);
        self.vendor_id.write_bytes(&mut buf[6..]);
        self.guid_prefix.write_bytes(&mut buf[8..]);
        20
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SubmessageHeaderWrite {
    submessage_id: SubmessageKind,
    // flags without endianness
    flags: [SubmessageFlag; 7],
    submessage_length: u16,
}

impl SubmessageHeaderWrite {
    pub fn new(
        submessage_id: SubmessageKind,
        // flags without endianness
        flags: &[SubmessageFlag],
        submessage_length: u16,
    ) -> Self {
        let mut flags_array = [false; 7];
        flags_array[..flags.len()].copy_from_slice(flags);

        Self {
            submessage_id,
            flags: flags_array,
            submessage_length,
        }
    }
}

struct EndiannessFlag<E> {
    endianness: PhantomData<E>,
}
impl EndiannessFlag<byteorder::LittleEndian> {
    fn get() -> bool {
        true
    }
}

impl WriteBytes for SubmessageHeaderWrite {
    #[inline]
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        self.submessage_id.write_bytes(&mut buf[0..]);
        let flags = [
            EndiannessFlag::<WriteEndianness>::get(),
            self.flags[0],
            self.flags[1],
            self.flags[2],
            self.flags[3],
            self.flags[4],
            self.flags[5],
            self.flags[6],
        ];
        flags.write_bytes(&mut buf[1..]);
        self.submessage_length.write_bytes(&mut buf[2..]);
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
            flags_byte & 0b_0000_0001 != 0,
            flags_byte & 0b_0000_0010 != 0,
            flags_byte & 0b_0000_0100 != 0,
            flags_byte & 0b_0000_1000 != 0,
            flags_byte & 0b_0001_0000 != 0,
            flags_byte & 0b_0010_0000 != 0,
            flags_byte & 0b_0100_0000 != 0,
            flags_byte & 0b_1000_0000 != 0,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::{
        messages::{
            submessage_elements::{Data, Parameter, ParameterList},
            submessages::{data::DataSubmessageRead, heartbeat::HeartbeatSubmessageRead},
        },
        types::{EntityId, SequenceNumber, USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY},
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
        let writer_sn = SequenceNumber::from(5);
        let parameter_1 = Parameter::new(6, vec![10, 11, 12, 13].into());
        let parameter_2 = Parameter::new(7, vec![20, 21, 22, 23].into());
        let inline_qos = &ParameterList::new(vec![parameter_1, parameter_2]);
        let serialized_payload = &Data::new(vec![].into());

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
        let value = RtpsMessageWrite::new(&header, &[submessage]);
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
            version: ProtocolVersion::new(2, 3),
            vendor_id: [9, 8],
            guid_prefix: [3; 12],
        };

        #[rustfmt::skip]
        let data = Arc::new([
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]);
        let rtps_message = RtpsMessageRead::new(data).unwrap();
        assert_eq!(rtps_message.header(), header);
        assert_eq!(rtps_message.submessages(), vec![]);
    }

    #[test]
    fn deserialize_rtps_message_too_high_submessage_length() {
        #[rustfmt::skip]
        let data = Arc::new([
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            0x15, 0b_0000_0011, 40, 0, // Submessage header
            0, 0, 16, 0,
        ]);
        let rtps_message = RtpsMessageRead::new(data).unwrap();
        assert_eq!(rtps_message.submessages(), vec![]);
    }

    #[test]
    fn deserialize_rtps_message_too_small_submessage_length() {
        #[rustfmt::skip]
        let data = Arc::new([
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            0x07, 0b_0000_0101, 24, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // firstSN: SequenceNumber: high
            5, 0, 0, 0, // firstSN: SequenceNumber: low
            0, 0, 0, 0, // lastSN: SequenceNumberSet: high
            7, 0, 0, 0, // lastSN: SequenceNumberSet: low
            2, 0, 0, 0, // count: Count: value (long)
        ]);
        let rtps_message = RtpsMessageRead::new(data).unwrap();
        assert_eq!(rtps_message.submessages(), vec![]);
    }

    #[test]
    fn deserialize_rtps_message() {
        let expected_header = RtpsMessageHeader {
            version: ProtocolVersion::new(2, 3),
            vendor_id: [9, 8],
            guid_prefix: [3; 12],
        };
        let expected_data_submessage = RtpsSubmessageReadKind::Data(DataSubmessageRead::new(
            true,
            false,
            false,
            false,
            EntityId::new([1, 2, 3], 4),
            EntityId::new([6, 7, 8], 9),
            SequenceNumber::new(0, 5),
            ParameterList::new(vec![
                Parameter::new(6, vec![10, 11, 12, 13].into()),
                Parameter::new(7, vec![20, 21, 22, 23].into()),
            ]),
            Data::empty(),
        ));

        #[rustfmt::skip]
        let expected_heartbeat_submessage = RtpsSubmessageReadKind::Heartbeat(HeartbeatSubmessageRead::try_from_bytes(&[
            0x07, 0b_0000_0101, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // firstSN: SequenceNumber: high
            5, 0, 0, 0, // firstSN: SequenceNumber: low
            0, 0, 0, 0, // lastSN: SequenceNumberSet: high
            7, 0, 0, 0, // lastSN: SequenceNumberSet: low
            2, 0, 0, 0, // count: Count: value (long)
        ]).unwrap());

        let expected_submessages = vec![expected_data_submessage, expected_heartbeat_submessage];

        #[rustfmt::skip]
        let data = Arc::new([
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
        ]);

        let rtps_message = RtpsMessageRead::new(data).unwrap();
        assert_eq!(expected_header, rtps_message.header());
        assert_eq!(expected_submessages, rtps_message.submessages());
    }

    #[test]
    fn deserialize_rtps_message_unknown_submessage() {
        let expected_data_submessage = RtpsSubmessageReadKind::Data(DataSubmessageRead::new(
            true,
            false,
            false,
            false,
            EntityId::new([1, 2, 3], 4),
            EntityId::new([6, 7, 8], 9),
            SequenceNumber::new(0, 5),
            ParameterList::new(vec![
                Parameter::new(6, vec![10, 11, 12, 13].into()),
                Parameter::new(7, vec![20, 21, 22, 23].into()),
            ]),
            Data::empty(),
        ));

        let expected_submessages = vec![expected_data_submessage];

        #[rustfmt::skip]
        let data = Arc::new([
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
        ]);

        let rtps_message = RtpsMessageRead::new(data).unwrap();
        assert_eq!(expected_submessages, rtps_message.submessages());
    }
}

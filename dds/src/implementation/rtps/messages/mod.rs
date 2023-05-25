use std::io::BufRead;

use crate::implementation::{
    rtps::messages::submessages::{AckNackSubmessageRead, GapSubmessageRead},
    rtps_udp_psm::{
        mapping_rtps_messages::submessages::submessage_header::{
            ACKNACK, DATA, DATA_FRAG, GAP, HEARTBEAT, HEARTBEAT_FRAG, INFO_DST, INFO_REPLY,
            INFO_SRC, INFO_TS, NACK_FRAG, PAD,
        },
        mapping_traits::MappingReadByteOrderInfoInData,
    },
};

use self::{
    overall_structure::RtpsMessageHeader,
    submessages::{
        AckNackSubmessageWrite, DataFragSubmessageRead, DataFragSubmessageWrite,
        DataSubmessageRead, DataSubmessageWrite, HeartbeatFragSubmessage, HeartbeatSubmessageRead,
        HeartbeatSubmessageWrite, InfoDestinationSubmessage, InfoReplySubmessage,
        InfoSourceSubmessage, InfoTimestampSubmessage, NackFragSubmessage, PadSubmessage, GapSubmessageWrite,
    },
    types::ProtocolId,
};

use super::types::{GuidPrefix, ProtocolVersion, VendorId};

pub mod overall_structure;
pub mod submessage_elements;
pub mod submessages;
pub mod types;

#[derive(Debug, PartialEq, Eq)]
pub struct RtpsMessageRead<'a> {
    data: &'a [u8],
}

impl<'a> RtpsMessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
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
                DATA_FRAG => RtpsSubmessageReadKind::DataFrag(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(&mut buf)
                        .unwrap(),
                ),
                GAP => RtpsSubmessageReadKind::Gap(GapSubmessageRead::new(submessage_data)),
                HEARTBEAT => {
                    RtpsSubmessageReadKind::Heartbeat(HeartbeatSubmessageRead::new(submessage_data))
                }
                HEARTBEAT_FRAG => RtpsSubmessageReadKind::HeartbeatFrag(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(&mut buf)
                        .unwrap(),
                ),
                INFO_DST => RtpsSubmessageReadKind::InfoDestination(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(&mut buf)
                        .unwrap(),
                ),
                INFO_REPLY => RtpsSubmessageReadKind::InfoReply(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(&mut buf)
                        .unwrap(),
                ),
                INFO_SRC => RtpsSubmessageReadKind::InfoSource(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(&mut buf)
                        .unwrap(),
                ),
                INFO_TS => RtpsSubmessageReadKind::InfoTimestamp(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(&mut buf)
                        .unwrap(),
                ),
                NACK_FRAG => RtpsSubmessageReadKind::NackFrag(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(&mut buf)
                        .unwrap(),
                ),
                PAD => RtpsSubmessageReadKind::Pad(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(&mut buf)
                        .unwrap(),
                ),
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

#[derive(Debug, PartialEq, Eq)]
pub struct RtpsMessageWrite<'a> {
    header: RtpsMessageHeader,
    submessages: Vec<RtpsSubmessageWriteKind<'a>>,
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
    HeartbeatFrag(HeartbeatFragSubmessage),
    InfoDestination(InfoDestinationSubmessage),
    InfoReply(InfoReplySubmessage),
    InfoSource(InfoSourceSubmessage),
    InfoTimestamp(InfoTimestampSubmessage),
    NackFrag(NackFragSubmessage),
    Pad(PadSubmessage),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RtpsSubmessageWriteKind<'a> {
    AckNack(AckNackSubmessageWrite),
    Data(DataSubmessageWrite<'a>),
    DataFrag(DataFragSubmessageWrite<'a>),
    Gap(GapSubmessageWrite),
    Heartbeat(HeartbeatSubmessageWrite),
    HeartbeatFrag(HeartbeatFragSubmessage),
    InfoDestination(InfoDestinationSubmessage),
    InfoReply(InfoReplySubmessage),
    InfoSource(InfoSourceSubmessage),
    InfoTimestamp(InfoTimestampSubmessage),
    NackFrag(NackFragSubmessage),
    Pad(PadSubmessage),
}

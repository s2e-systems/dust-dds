use std::io::{BufRead, Write};

use byteorder::ByteOrder;
use rust_rtps_pim::messages::{submessages::RtpsSubmessageType, RtpsMessageHeader};

use crate::{
    message_header::RtpsMessageHeaderUdp,
    psm::RtpsUdpPsm,
    submessage_header::{
        SubmessageHeaderUdp, ACKNACK, DATA, DATA_FRAG, GAP, HEARTBEAT, HEARTBEAT_FRAG, INFO_DST,
        INFO_REPLY, INFO_SRC, INFO_TS, NACK_FRAG, PAD,
    },
};

#[derive(Debug, PartialEq)]
pub struct RtpsMessageUdp<'a> {
    header: RtpsMessageHeaderUdp,
    submessages: Vec<RtpsSubmessageType<'a, RtpsUdpPsm>>,
}

impl<'a> rust_rtps_pim::messages::RtpsMessageTrait for RtpsMessageUdp<'a> {
    type SubmessageType = RtpsSubmessageType<'a, RtpsUdpPsm>;

    fn new<T: IntoIterator<Item = Self::SubmessageType>>(
        header: &RtpsMessageHeader,
        submessages: T,
    ) -> Self {
        Self {
            header: header.into(),
            submessages: submessages.into_iter().collect(),
        }
    }

    fn header(&self) -> RtpsMessageHeader {
        (&self.header).into()
    }

    fn submessages(&self) -> &[Self::SubmessageType] {
        &self.submessages
    }
}

impl<'a> crate::serialize::Serialize for RtpsMessageUdp<'a> {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        self.header.serialize::<_, B>(&mut writer)?;
        for submessage in &self.submessages {
            match submessage {
                RtpsSubmessageType::AckNack(s) => s.serialize::<_, B>(&mut writer)?,
                RtpsSubmessageType::Data(s) => todo!(), //s.serialize::<_, B>(&mut writer)?,
                RtpsSubmessageType::DataFrag(s) => s.serialize::<_, B>(&mut writer)?,
                RtpsSubmessageType::Gap(s) => s.serialize::<_, B>(&mut writer)?,
                RtpsSubmessageType::Heartbeat(s) => s.serialize::<_, B>(&mut writer)?,
                RtpsSubmessageType::HeartbeatFrag(s) => s.serialize::<_, B>(&mut writer)?,
                RtpsSubmessageType::InfoDestination(s) => s.serialize::<_, B>(&mut writer)?,
                RtpsSubmessageType::InfoReply(s) => s.serialize::<_, B>(&mut writer)?,
                RtpsSubmessageType::InfoSource(s) => s.serialize::<_, B>(&mut writer)?,
                RtpsSubmessageType::InfoTimestamp(s) => s.serialize::<_, B>(&mut writer)?,
                RtpsSubmessageType::NackFrag(s) => s.serialize::<_, B>(&mut writer)?,
                RtpsSubmessageType::Pad(s) => s.serialize::<_, B>(&mut writer)?,
            }
        }
        Ok(())
    }
}
impl<'a, 'de: 'a> crate::deserialize::Deserialize<'de> for RtpsMessageUdp<'a> {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        const MAX_SUBMESSAGES: usize = 2_usize.pow(16);

        let mut submessages = vec![];
        let header = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        for _ in 0..MAX_SUBMESSAGES {
            if buf.len() < 4 {
                break;
            }
            // Preview byte only (to allow full deserialization of submessage header)
            let submessage_id = buf[0];
            let submessage = match submessage_id {
                ACKNACK => RtpsSubmessageType::AckNack(
                    crate::deserialize::Deserialize::deserialize::<B>(buf)?,
                ),
                DATA => todo!(),
                //  RtpsSubmessageType::Data(
                // crate::deserialize::Deserialize::deserialize::<B>(buf)?,
                // ),
                DATA_FRAG => RtpsSubmessageType::DataFrag(
                    crate::deserialize::Deserialize::deserialize::<B>(buf)?,
                ),
                GAP => {
                    RtpsSubmessageType::Gap(crate::deserialize::Deserialize::deserialize::<B>(buf)?)
                }
                HEARTBEAT => RtpsSubmessageType::Heartbeat(
                    crate::deserialize::Deserialize::deserialize::<B>(buf)?,
                ),
                HEARTBEAT_FRAG => RtpsSubmessageType::HeartbeatFrag(
                    crate::deserialize::Deserialize::deserialize::<B>(buf)?,
                ),
                INFO_DST => RtpsSubmessageType::InfoDestination(
                    crate::deserialize::Deserialize::deserialize::<B>(buf)?,
                ),
                INFO_REPLY => RtpsSubmessageType::InfoReply(
                    crate::deserialize::Deserialize::deserialize::<B>(buf)?,
                ),
                INFO_SRC => RtpsSubmessageType::InfoSource(
                    crate::deserialize::Deserialize::deserialize::<B>(buf)?,
                ),
                INFO_TS => RtpsSubmessageType::InfoTimestamp(
                    crate::deserialize::Deserialize::deserialize::<B>(buf)?,
                ),
                NACK_FRAG => RtpsSubmessageType::NackFrag(
                    crate::deserialize::Deserialize::deserialize::<B>(buf)?,
                ),
                PAD => {
                    RtpsSubmessageType::Pad(crate::deserialize::Deserialize::deserialize::<B>(buf)?)
                }
                _ => {
                    let submessage_header: SubmessageHeaderUdp =
                        crate::deserialize::Deserialize::deserialize::<B>(buf)?;
                    buf.consume(submessage_header.submessage_length as usize);
                    continue;
                }
            };
            submessages.push(submessage);
        }
        Ok(Self {
            header,
            submessages,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        deserialize::from_bytes_le,
        parameter_list::ParameterListUdp,
        serialize::to_bytes_le,
        submessage_elements::{
            EntityIdUdp, GuidPrefixUdp, ProtocolVersionUdp, SequenceNumberSetUdp,
            SequenceNumberUdp, SerializedDataUdp, VendorIdUdp,
        },
        submessages,
    };
    use rust_rtps_pim::messages::submessage_elements::SequenceNumberSetSubmessageElementType;
    use rust_rtps_pim::messages::{
        submessage_elements::SequenceNumberSubmessageElementType, submessages::GapSubmessage,
    };

    #[test]
    fn serialize_rtps_message_no_submessage() {
        let header = RtpsMessageHeaderUdp {
            protocol: b"RTPS".to_owned(),
            version: ProtocolVersionUdp { major: 2, minor: 3 },
            vendor_id: VendorIdUdp([9, 8]),
            guid_prefix: GuidPrefixUdp([3; 12]),
        };
        let value = RtpsMessageUdp {
            header,
            submessages: vec![],
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&value).unwrap(), vec![
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]);
    }

    // #[test]
    // fn serialize_rtps_message() {
    //     let header = RtpsMessageHeaderUdp {
    //         protocol: b"RTPS".to_owned(),
    //         version: ProtocolVersionUdp { major: 2, minor: 3 },
    //         vendor_id: VendorIdUdp([9, 8]),
    //         guid_prefix: GuidPrefixUdp([3; 12]),
    //     };
    //     let endianness_flag = true;
    //     let reader_id = EntityIdUdp {
    //         entity_key: [1, 2, 3],
    //         entity_kind: 4,
    //     };
    //     let writer_id = EntityIdUdp {
    //         entity_key: [6, 7, 8],
    //         entity_kind: 9,
    //     };
    //     let gap_start = SequenceNumberUdp::new(&5);
    //     let gap_list = SequenceNumberSetUdp::new(&10, &[]);
    //     let gap_submessage = RtpsSubmessageType::Gap(GapSubmessageTrait::new(
    //         endianness_flag,
    //         reader_id,
    //         writer_id,
    //         gap_start,
    //         gap_list,
    //     ));

    //     let inline_qos_flag = false;
    //     let data_flag = false;
    //     let key_flag = false;
    //     let non_standard_payload_flag = false;
    //     let reader_id = EntityIdUdp {
    //         entity_key: [1, 2, 3],
    //         entity_kind: 4,
    //     };
    //     let writer_id = EntityIdUdp {
    //         entity_key: [6, 7, 8],
    //         entity_kind: 9,
    //     };
    //     let writer_sn = SequenceNumberUdp::new(&5);
    //     let inline_qos = ParameterListUdp {
    //         parameter: vec![].into(),
    //     };
    //     let data = [];
    //     let serialized_payload = SerializedDataUdp(&data[..]);
    //     let data_submessage = RtpsSubmessageType::Data(submessages::data::DataSubmesageUdp::new(
    //         endianness_flag,
    //         inline_qos_flag,
    //         data_flag,
    //         key_flag,
    //         non_standard_payload_flag,
    //         reader_id,
    //         writer_id,
    //         writer_sn,
    //         inline_qos,
    //         serialized_payload,
    //     ));
    //     let value = RtpsMessageUdp {
    //         header,
    //         submessages: vec![gap_submessage, data_submessage],
    //     };
    //     #[rustfmt::skip]
    //     assert_eq!(to_bytes_le(&value).unwrap(), vec![
    //         b'R', b'T', b'P', b'S', // Protocol
    //         2, 3, 9, 8, // ProtocolVersion | VendorId
    //         3, 3, 3, 3, // GuidPrefix
    //         3, 3, 3, 3, // GuidPrefix
    //         3, 3, 3, 3, // GuidPrefix
    //         0x08, 0b_0000_0001, 28, 0, // Submessage header
    //         1, 2, 3, 4, // readerId: value[4]
    //         6, 7, 8, 9, // writerId: value[4]
    //         0, 0, 0, 0, // gapStart: SequenceNumber: high
    //         5, 0, 0, 0, // gapStart: SequenceNumber: low
    //         0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
    //        10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
    //         0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
    //         0x15, 0b_0000_0001, 20, 0, // Submessage header
    //         0, 0, 16, 0, // extraFlags, octetsToInlineQos
    //         1, 2, 3, 4, // readerId: value[4]
    //         6, 7, 8, 9, // writerId: value[4]
    //         0, 0, 0, 0, // writerSN: high
    //         5, 0, 0, 0, // writerSN: low
    //     ]);
    // }

    // #[test]
    // fn deserialize_rtps_message_no_submessage() {
    //     let header = RtpsMessageHeaderUdp {
    //         protocol: b"RTPS".to_owned(),
    //         version: ProtocolVersionUdp { major: 2, minor: 3 },
    //         vendor_id: VendorIdUdp([9, 8]),
    //         guid_prefix: GuidPrefixUdp([3; 12]),
    //     };

    //     let expected = RtpsMessageUdp {
    //         header,
    //         submessages: vec![],
    //     };
    //     #[rustfmt::skip]
    //     let result: RtpsMessageUdp = from_bytes_le(&[
    //         b'R', b'T', b'P', b'S', // Protocol
    //         2, 3, 9, 8, // ProtocolVersion | VendorId
    //         3, 3, 3, 3, // GuidPrefix
    //         3, 3, 3, 3, // GuidPrefix
    //         3, 3, 3, 3, // GuidPrefix
    //     ]).unwrap();
    //     assert_eq!(result, expected);
    // }

    // #[test]
    // fn deserialize_rtps_message() {
    //     let header = RtpsMessageHeaderUdp {
    //         protocol: b"RTPS".to_owned(),
    //         version: ProtocolVersionUdp { major: 2, minor: 3 },
    //         vendor_id: VendorIdUdp([9, 8]),
    //         guid_prefix: GuidPrefixUdp([3; 12]),
    //     };

    //     let endianness_flag = true;
    //     let reader_id = EntityIdUdp {
    //         entity_key: [1, 2, 3],
    //         entity_kind: 4,
    //     };
    //     let writer_id = EntityIdUdp {
    //         entity_key: [6, 7, 8],
    //         entity_kind: 9,
    //     };
    //     let gap_start = SequenceNumberUdp::new(&5);
    //     let gap_list = SequenceNumberSetUdp::new(&10, &[]);
    //     let gap_submessage = RtpsSubmessageType::Gap(GapSubmessageTrait::new(
    //         endianness_flag,
    //         reader_id,
    //         writer_id,
    //         gap_start,
    //         gap_list,
    //     ));

    //     let inline_qos_flag = false;
    //     let data_flag = false;
    //     let key_flag = false;
    //     let non_standard_payload_flag = false;
    //     let reader_id = EntityIdUdp {
    //         entity_key: [1, 2, 3],
    //         entity_kind: 4,
    //     };
    //     let writer_id = EntityIdUdp {
    //         entity_key: [6, 7, 8],
    //         entity_kind: 9,
    //     };
    //     let writer_sn = SequenceNumberUdp::new(&5);
    //     let inline_qos = ParameterListUdp {
    //         parameter: vec![].into(),
    //     };
    //     let data = [];
    //     let serialized_payload = SerializedDataUdp(&data[..]);
    //     let data_submessage = RtpsSubmessageType::Data(submessages::data::DataSubmesageUdp::new(
    //         endianness_flag,
    //         inline_qos_flag,
    //         data_flag,
    //         key_flag,
    //         non_standard_payload_flag,
    //         reader_id,
    //         writer_id,
    //         writer_sn,
    //         inline_qos,
    //         serialized_payload,
    //     ));
    //     let expected = RtpsMessageUdp {
    //         header,
    //         submessages: vec![gap_submessage, data_submessage],
    //     };
    //     #[rustfmt::skip]
    //     let result: RtpsMessageUdp = from_bytes_le(&[
    //         b'R', b'T', b'P', b'S', // Protocol
    //         2, 3, 9, 8, // ProtocolVersion | VendorId
    //         3, 3, 3, 3, // GuidPrefix
    //         3, 3, 3, 3, // GuidPrefix
    //         3, 3, 3, 3, // GuidPrefix
    //         0x08, 0b_0000_0001, 28, 0, // Submessage header (GAP)
    //         1, 2, 3, 4, // readerId: value[4]
    //         6, 7, 8, 9, // writerId: value[4]
    //         0, 0, 0, 0, // gapStart: SequenceNumber: high
    //         5, 0, 0, 0, // gapStart: SequenceNumber: low
    //         0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
    //        10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
    //         0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
    //         0x15, 0b_0000_0001, 20, 0, // Submessage header (DATA)
    //         0, 0, 16, 0, // extraFlags, octetsToInlineQos
    //         1, 2, 3, 4, // readerId: value[4]
    //         6, 7, 8, 9, // writerId: value[4]
    //         0, 0, 0, 0, // writerSN: high
    //         5, 0, 0, 0, // writerSN: low
    //     ]).unwrap();
    //     assert_eq!(result, expected);
    // }

    // #[test]
    // fn deserialize_rtps_message_with_unknown_submessage_kind() {
    //     let header = RtpsMessageHeaderUdp {
    //         protocol: b"RTPS".to_owned(),
    //         version: ProtocolVersionUdp { major: 2, minor: 3 },
    //         vendor_id: VendorIdUdp([9, 8]),
    //         guid_prefix: GuidPrefixUdp([3; 12]),
    //     };

    //     let endianness_flag = true;
    //     let reader_id = EntityIdUdp {
    //         entity_key: [1, 2, 3],
    //         entity_kind: 4,
    //     };
    //     let writer_id = EntityIdUdp {
    //         entity_key: [6, 7, 8],
    //         entity_kind: 9,
    //     };
    //     let gap_start = SequenceNumberUdp::new(&5);
    //     let gap_list = SequenceNumberSetUdp::new(&10, &[]);
    //     let gap_submessage = RtpsSubmessageType::Gap(GapSubmessageTrait::new(
    //         endianness_flag,
    //         reader_id,
    //         writer_id,
    //         gap_start,
    //         gap_list,
    //     ));

    //     let inline_qos_flag = false;
    //     let data_flag = true;
    //     let key_flag = false;
    //     let non_standard_payload_flag = false;
    //     let reader_id = EntityIdUdp {
    //         entity_key: [1, 2, 3],
    //         entity_kind: 4,
    //     };
    //     let writer_id = EntityIdUdp {
    //         entity_key: [6, 7, 8],
    //         entity_kind: 9,
    //     };
    //     let writer_sn = SequenceNumberUdp::new(&5);
    //     let inline_qos = ParameterListUdp {
    //         parameter: vec![].into(),
    //     };
    //     let data = [1, 2, 3, 4];
    //     let serialized_payload = SerializedDataUdp(&data[..]);
    //     let data_submessage = RtpsSubmessageType::Data(submessages::data::DataSubmesageUdp::new(
    //         endianness_flag,
    //         inline_qos_flag,
    //         data_flag,
    //         key_flag,
    //         non_standard_payload_flag,
    //         reader_id,
    //         writer_id,
    //         writer_sn,
    //         inline_qos,
    //         serialized_payload,
    //     ));
    //     let expected = RtpsMessageUdp {
    //         header,
    //         submessages: vec![data_submessage, gap_submessage],
    //     };
    //     #[rustfmt::skip]
    //     let result: RtpsMessageUdp = from_bytes_le(&[
    //         b'R', b'T', b'P', b'S', // Protocol
    //         2, 3, 9, 8, // ProtocolVersion | VendorId
    //         3, 3, 3, 3, // GuidPrefix
    //         3, 3, 3, 3, // GuidPrefix
    //         3, 3, 3, 3, // GuidPrefix
    //      0x99, 0xcc, 8, 0,   // Submessage header (Unknown)
    //         0xcc, 0xcc, 0xcc, 0xcc, // Unknown stuff
    //         0xcc, 0xcc, 0xcc, 0xcc, // Unknown stuff
    //      0x15, 0b_0000_0101, 24, 0, // Submessage header (Data)
    //         0, 0, 16, 0, // extraFlags, octetsToInlineQos
    //         1, 2, 3, 4, // readerId: value[4]
    //         6, 7, 8, 9, // writerId: value[4]
    //         0, 0, 0, 0, // writerSN: high
    //         5, 0, 0, 0, // writerSN: low
    //         1, 2, 3, 4, // serialized payload
    //      0x08, 0b_0000_0001, 28, 0, // Submessage header (Gap)
    //         1, 2, 3, 4, // readerId: value[4]
    //         6, 7, 8, 9, // writerId: value[4]
    //         0, 0, 0, 0, // gapStart: SequenceNumber: high
    //         5, 0, 0, 0, // gapStart: SequenceNumber: low
    //         0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
    //        10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
    //         0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
    //     ]).unwrap();
    //     assert_eq!(result, expected);
    // }
}

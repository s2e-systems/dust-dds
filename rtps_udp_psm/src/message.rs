use std::io::BufRead;

use rust_rtps_pim::messages::{submessages::RtpsSubmessageType, RtpsMessageHeader};
use rust_serde_cdr::{deserializer::RtpsMessageDeserializer, error::Error};
use serde::{de::Deserialize, ser::SerializeStruct};

use crate::{
    message_header::RTPSMessageHeaderUdp,
    psm::RtpsUdpPsm,
    submessage_header::{SubmessageHeaderUdp, DATA, GAP},
};

#[derive(Debug, PartialEq)]
pub struct RTPSMessageUdp<'a> {
    header: RTPSMessageHeaderUdp,
    submessages: Vec<RtpsSubmessageType<'a, RtpsUdpPsm>>,
}

impl<'a> rust_rtps_pim::messages::RTPSMessage for RTPSMessageUdp<'a> {
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

impl<'a> serde::Serialize for RTPSMessageUdp<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let len = self.submessages.len() + 1;
        let mut state = serializer.serialize_struct("RTPSMessage", len)?;
        state.serialize_field("header", &self.header)?;
        for submessage in &self.submessages {
            match submessage {
                //RtpsSubmessageType::AckNack(submessage) => state.serialize_field("submessage", submessage)?,
                RtpsSubmessageType::Data(submessage) => {
                    state.serialize_field("submessage", submessage)?
                }
                //RtpsSubmessageType::DataFrag(submessage) => state.serialize_field("submessage", submessage)?,
                RtpsSubmessageType::Gap(submessage) => {
                    state.serialize_field("submessage", submessage)?
                }
                // RtpsSubmessageType::Heartbeat(submessage) => state.serialize_field("submessage", submessage)?,
                // RtpsSubmessageType::HeartbeatFrag(submessage) => state.serialize_field("submessage", submessage)?,
                // RtpsSubmessageType::InfoDestination(submessage) => state.serialize_field("submessage", submessage)?,
                // RtpsSubmessageType::InfoReply(submessage) => state.serialize_field("submessage", submessage)?,
                // RtpsSubmessageType::InfoSource(submessage) => state.serialize_field("submessage", submessage)?,
                // RtpsSubmessageType::InfoTimestamp(submessage) => state.serialize_field("submessage", submessage)?,
                // RtpsSubmessageType::NackFrag(submessage) => state.serialize_field("submessage", submessage)?,
                //RtpsSubmessageType::Pad(submessage) => state.serialize_field("submessage", submessage)?,
                _ => todo!(),
            }
        }
        state.end()
    }
}

struct RTPSMessageUdpVisitor<'a>(std::marker::PhantomData<&'a ()>);
impl<'a, 'de: 'a> serde::de::Visitor<'de> for RTPSMessageUdpVisitor<'a> {
    type Value = RTPSMessageUdp<'a>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("RTPSMessage")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut submessages = vec![];

        let header: RTPSMessageHeaderUdp = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;

        let buf: &[u8] = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

        let mut header_deserializer = RtpsMessageDeserializer { reader: buf };

        const MAX_SUBMESSAGES: usize = 2 ^ 16;
        for _ in 0..MAX_SUBMESSAGES {
            if header_deserializer.reader.len() < 1 {
                break;
            };
            let mut deserializer = RtpsMessageDeserializer {
                reader: &header_deserializer.reader,
            };
            let submessage_header: SubmessageHeaderUdp =
                Deserialize::deserialize(&mut header_deserializer)
                    .ok()
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
            let submessage_length = submessage_header.submessage_length as usize;

            let typed_submessage = match submessage_header.submessage_id.into() {
                GAP => Some(RtpsSubmessageType::Gap(
                    Deserialize::deserialize(&mut deserializer)
                        .ok()
                        .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?,
                )),
                DATA => Some(RtpsSubmessageType::Data(
                    Deserialize::deserialize(&mut deserializer)
                        .ok()
                        .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?,
                )),
                _ => None,
            };
            if let Some(typed_submessage) = typed_submessage {
                submessages.push(typed_submessage);
            }
            header_deserializer.reader.consume(submessage_length);
        }

        Ok(RTPSMessageUdp {
            header,
            submessages,
        })
    }
}

impl<'a, 'de: 'a> serde::de::Deserialize<'de> for RTPSMessageUdp<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &["header", "submessages"];
        deserializer.deserialize_struct(
            "RTPSMessage",
            FIELDS,
            RTPSMessageUdpVisitor(std::marker::PhantomData),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parameter_list::{ParameterListUdp}, submessage_elements::{
            EntityIdUdp, GuidPrefixUdp, ProtocolVersionUdp, SequenceNumberSetUdp,
            SequenceNumberUdp, SerializedDataUdp, VendorIdUdp,
        }, submessages};
    use rust_rtps_pim::messages::submessage_elements::SequenceNumberSetSubmessageElementType;
    use rust_rtps_pim::messages::{
        submessage_elements::SequenceNumberSubmessageElementType,
        submessages::{DataSubmessage, GapSubmessage},
    };
    use rust_serde_cdr::serializer::RtpsMessageSerializer;

    fn serialize<T: serde::Serialize>(value: T) -> Vec<u8> {
        let mut serializer = RtpsMessageSerializer {
            writer: Vec::<u8>::new(),
        };
        value.serialize(&mut serializer).unwrap();
        serializer.writer
    }

    #[test]
    fn serialize_rtps_message_no_submessage() {
        let header = RTPSMessageHeaderUdp {
            protocol: b"RTPS".to_owned(),
            version: ProtocolVersionUdp { major: 2, minor: 3 },
            vendor_id: VendorIdUdp([9, 8]),
            guid_prefix: GuidPrefixUdp([3; 12]),
        };
        let value = RTPSMessageUdp {
            header,
            submessages: vec![],
        };
        #[rustfmt::skip]
        assert_eq!(serialize(value), vec![
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]);
    }

    #[test]
    fn serialize_rtps_message() {
        let header = RTPSMessageHeaderUdp {
            protocol: b"RTPS".to_owned(),
            version: ProtocolVersionUdp { major: 2, minor: 3 },
            vendor_id: VendorIdUdp([9, 8]),
            guid_prefix: GuidPrefixUdp([3; 12]),
        };
        let endianness_flag = true;
        let reader_id = EntityIdUdp {
            entity_key: [1, 2, 3],
            entity_kind: 4,
        };
        let writer_id = EntityIdUdp {
            entity_key: [6, 7, 8],
            entity_kind: 9,
        };
        let gap_start = SequenceNumberUdp::new(&5);
        let gap_list = SequenceNumberSetUdp::new(&10, &[]);
        let gap_submessage = RtpsSubmessageType::Gap(GapSubmessage::new(
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        ));

        let inline_qos_flag = false;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityIdUdp {
            entity_key: [1, 2, 3],
            entity_kind: 4,
        };
        let writer_id = EntityIdUdp {
            entity_key: [6, 7, 8],
            entity_kind: 9,
        };
        let writer_sn = SequenceNumberUdp::new(&5);
        let inline_qos = ParameterListUdp {
            parameter: vec![].into(),
        };
        let data = [];
        let serialized_payload = SerializedDataUdp(&data[..]);
        let data_submessage = RtpsSubmessageType::Data(submessages::data::DataSubmesageUdp::new(
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
        let value = RTPSMessageUdp {
            header,
            submessages: vec![gap_submessage, data_submessage],
        };
        #[rustfmt::skip]
        assert_eq!(serialize(value), vec![
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            0x08, 0b_0000_0001, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // gapStart: SequenceNumber: high
            5, 0, 0, 0, // gapStart: SequenceNumber: low
            0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
           10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
            0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
            0x15, 0b_0000_0001, 20, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
        ]);
    }

    #[test]
    fn deserialize_rtps_message_no_submessage() {
        let header = RTPSMessageHeaderUdp {
            protocol: b"RTPS".to_owned(),
            version: ProtocolVersionUdp { major: 2, minor: 3 },
            vendor_id: VendorIdUdp([9, 8]),
            guid_prefix: GuidPrefixUdp([3; 12]),
        };

        let expected = RTPSMessageUdp {
            header,
            submessages: vec![],
        };
        #[rustfmt::skip]
        let result: RTPSMessageUdp = rust_serde_cdr::deserializer::from_bytes(&[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_rtps_message() {
        let header = RTPSMessageHeaderUdp {
            protocol: b"RTPS".to_owned(),
            version: ProtocolVersionUdp { major: 2, minor: 3 },
            vendor_id: VendorIdUdp([9, 8]),
            guid_prefix: GuidPrefixUdp([3; 12]),
        };

        let endianness_flag = true;
        let reader_id = EntityIdUdp {
            entity_key: [1, 2, 3],
            entity_kind: 4,
        };
        let writer_id = EntityIdUdp {
            entity_key: [6, 7, 8],
            entity_kind: 9,
        };
        let gap_start = SequenceNumberUdp::new(&5);
        let gap_list = SequenceNumberSetUdp::new(&10, &[]);
        let gap_submessage = RtpsSubmessageType::Gap(GapSubmessage::new(
            endianness_flag,
            reader_id,
            writer_id,
            gap_start,
            gap_list,
        ));

        let inline_qos_flag = false;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityIdUdp {
            entity_key: [1, 2, 3],
            entity_kind: 4,
        };
        let writer_id = EntityIdUdp {
            entity_key: [6, 7, 8],
            entity_kind: 9,
        };
        let writer_sn = SequenceNumberUdp::new(&5);
        let inline_qos = ParameterListUdp {
            parameter: vec![].into(),
        };
        let data = [];
        let serialized_payload = SerializedDataUdp(&data[..]);
        let data_submessage = RtpsSubmessageType::Data(submessages::data::DataSubmesageUdp::new(
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
        let expected = RTPSMessageUdp {
            header,
            submessages: vec![gap_submessage, data_submessage],
        };
        #[rustfmt::skip]
        let result: RTPSMessageUdp = rust_serde_cdr::deserializer::from_bytes(&[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            0x08_u8, 0b_0000_0001, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // gapStart: SequenceNumber: high
            5, 0, 0, 0, // gapStart: SequenceNumber: low
            0, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: high
           10, 0, 0, 0, // gapList: SequenceNumberSet: bitmapBase: low
            0, 0, 0, 0, // gapList: SequenceNumberSet: numBits (ULong)
            0x15, 0b_0000_0001, 20, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
        ]).unwrap();
        assert_eq!(result, expected);
    }

    // #[test]
    // fn deserialize_rtps_message_with_unknown_submessage_kind() {
    //     let header = RTPSMessageHeaderUdp {
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
    //     let gap_submessage = RtpsSubmessageType::Gap(GapSubmessage::new(
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
    //     let expected = RTPSMessageUdp {
    //         header,
    //         submessages: vec![data_submessage, gap_submessage],
    //     };
    //     #[rustfmt::skip]
    //     let result = RTPSMessageUdp::from_bytes(&[
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

    #[test]
    fn serialize_rtps_message_no_submessage_json() {
        let header = RTPSMessageHeaderUdp {
            protocol: b"RTPS".to_owned(),
            version: ProtocolVersionUdp { major: 2, minor: 3 },
            vendor_id: VendorIdUdp([9, 8]),
            guid_prefix: GuidPrefixUdp([3; 12]),
        };
        let value = RTPSMessageUdp {
            header,
            submessages: vec![],
        };
        assert_eq!(
            serde_json::ser::to_string(&value).unwrap(),
            r#"{"header":{"protocol":[82,84,80,83],"version":{"major":2,"minor":3},"vendor_id":[9,8],"guid_prefix":[3,3,3,3,3,3,3,3,3,3,3,3]}}"#
        );
    }
}

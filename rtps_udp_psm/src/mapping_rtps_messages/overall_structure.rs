use std::{io::Write, iter::FromIterator};

use byteorder::ByteOrder;
use rust_rtps_pim::{messages::{RtpsMessage, submessages::RtpsSubmessageType}, structure::types::SequenceNumber};

use crate::{deserialize::{self, Deserialize}, serialize::{self, NumberOfBytes, Serialize}};

//Vec<RtpsSubmessageType<'_, Vec<SequenceNumber>, P, (), ()>>

impl<S, P, L, F> Serialize for RtpsSubmessageType<'_, S, P, L, F> where
    for<'a> &'a S: IntoIterator<Item = &'a SequenceNumber>,
    P: Serialize + NumberOfBytes
{
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        match self {
            RtpsSubmessageType::AckNack(s) => s.serialize::<_, B>(&mut writer)?,
            RtpsSubmessageType::Data(s) => s.serialize::<_, B>(&mut writer)?,
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
        };
        Ok(())
    }
}

impl<M: Serialize> Serialize for RtpsMessage<M> {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.header.serialize::<_, B>(&mut writer)?;
        self.submessages.serialize::<_, B>(&mut writer)
    }
}

// impl<'a, 'de, M, S, P, L, F> Deserialize<'de> for RtpsMessage<M>
// where M: FromIterator<RtpsSubmessageType<'a, S, P, L, F>>,{
//     fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
//         let header = Deserialize::deserialize(&mut buf)?;
//         let submessages = vec![];
//         Ok(Self {
//             header,
//             submessages: M::from_iter(),
//         })
//     }
// }


#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::*;
    use crate::{
        serialize::to_bytes_le,
    };
    use rust_rtps_pim::messages::submessage_elements::{EntityIdSubmessageElement, Parameter, ParameterListSubmessageElement, SequenceNumberSubmessageElement, SerializedDataSubmessageElement};

    use rust_rtps_pim::messages::submessages::DataSubmessage;
    use rust_rtps_pim::messages::types::ParameterId;
    use rust_rtps_pim::messages::{types::ProtocolId, RtpsMessageHeader};
    use rust_rtps_pim::structure::types::{EntityId, EntityKind, ProtocolVersion};

    #[test]
    fn serialize_rtps_message_no_submessage() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: [9, 8],
            guid_prefix: [3; 12],
        };
        let value = RtpsMessage {
            header,
            submessages: Vec::<RtpsSubmessageType<Vec<SequenceNumber>, Vec<Parameter>, (), ()>>::new(),
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

    #[test]
    fn serialize_rtps_message() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: [9, 8],
            guid_prefix: [3; 12],
        };
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityIdSubmessageElement {
            value: EntityId::new([1, 2, 3], EntityKind::UserDefinedReaderNoKey),
        };
        let writer_id = EntityIdSubmessageElement {
            value: EntityId::new([6, 7, 8], EntityKind::UserDefinedReaderGroup),
        };
        let writer_sn = SequenceNumberSubmessageElement { value: 5 };
        let parameter_1 = Parameter::new(ParameterId(6), &[10, 11, 12, 13]);
        let parameter_2 = Parameter::new(ParameterId(7), &[20, 21, 22, 23]);
        let inline_qos = ParameterListSubmessageElement {
            parameter: vec![parameter_1, parameter_2],
            phantom: PhantomData
        };
        let serialized_payload = SerializedDataSubmessageElement { value: &[] };

        let submessage: RtpsSubmessageType<Vec<SequenceNumber>, Vec<Parameter>, (), ()> = RtpsSubmessageType::Data(DataSubmessage{
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
        });
        let value = RtpsMessage {
            header,
            submessages: vec![submessage],
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&value).unwrap(), vec![
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

    // #[test]
    // fn deserialize_rtps_message_no_submessage() {
    //     let header = RtpsMessageHeader {
    //         protocol: ProtocolId::PROTOCOL_RTPS,
    //         version: ProtocolVersion { major: 2, minor: 3 },
    //         vendor_id: [9, 8],
    //         guid_prefix: [3; 12],
    //     };

    //     let expected = RtpsMessage {
    //         header,
    //         submessages: Vec::<()>::new(),
    //     };
    //     #[rustfmt::skip]
    //     let result: RtpsMessage<Vec<()>> = from_bytes_le(&[
    //         b'R', b'T', b'P', b'S', // Protocol
    //         2, 3, 9, 8, // ProtocolVersion | VendorId
    //         3, 3, 3, 3, // GuidPrefix
    //         3, 3, 3, 3, // GuidPrefix
    //         3, 3, 3, 3, // GuidPrefix
    //     ]).unwrap();
    //     assert_eq!(result, expected);
    // }
}
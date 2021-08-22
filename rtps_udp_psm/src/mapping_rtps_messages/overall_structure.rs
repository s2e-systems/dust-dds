use rust_rtps_pim::{messages::{RtpsMessage, submessage_elements::Parameter, submessages::RtpsSubmessageType}, structure::types::SequenceNumber};

use crate::{
    deserialize::{Deserialize, Result},
    serialize::Serialize,
};

impl<'a> Serialize for RtpsMessage<Vec<RtpsSubmessageType<'a, Vec<SequenceNumber>, &'a [Parameter<'a>], (), ()>>> {
    fn serialize<W: std::io::Write, B: byteorder::ByteOrder>(
        &self,
        mut writer: W,
    ) -> crate::serialize::Result {
        self.header.serialize::<_, B>(&mut writer)?;
        for submessage in &self.submessages {
            match submessage {
                RtpsSubmessageType::AckNack(s) => s.serialize::<_, B>(&mut writer)?,
                RtpsSubmessageType::Data(s) => todo!(),//s.serialize::<_, B>(&mut writer)?,
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

impl<'de, M> Deserialize<'de> for RtpsMessage<M> {
    fn deserialize<B>(buf: &mut &'de [u8]) -> Result<Self>
    where
        B: byteorder::ByteOrder,
    {
        todo!()
        // let header = crate::deserialize::Deserialize::deserialize(&mut buf)?;
        // let submessages = ();
        // Self {
        //     header,
        //     submessages,
        // };
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
    use rust_rtps_pim::messages::{types::ProtocolId, RtpsMessageHeader};
    use rust_rtps_pim::structure::types::ProtocolVersion;

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
            submessages: Vec::new(),
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
    fn deserialize_rtps_message_no_submessage() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: [9, 8],
            guid_prefix: [3; 12],
        };

        let expected = RtpsMessage {
            header,
            submessages: Vec::<()>::new(),
        };
        #[rustfmt::skip]
        let result: RtpsMessage<Vec<()>> = from_bytes_le(&[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]).unwrap();
        assert_eq!(result, expected);
    }
}
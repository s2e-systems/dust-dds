use crate::implementation::{
    rtps::messages::overall_structure::{RtpsMessageWrite, RtpsSubmessageWriteKind},
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrderInfoInData, MappingWriteByteOrdered},
};
use byteorder::LittleEndian;
use std::io::{Error, Write};

impl MappingWriteByteOrderInfoInData for RtpsSubmessageWriteKind<'_> {
    fn mapping_write_byte_order_info_in_data<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        match self {
            RtpsSubmessageWriteKind::AckNack(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::Data(s) => {
                //s.mapping_write_byte_order_info_in_data(&mut writer)?
                todo!()
            }
            RtpsSubmessageWriteKind::DataFrag(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::Gap(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::Heartbeat(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::HeartbeatFrag(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::InfoDestination(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::InfoReply(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::InfoSource(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::InfoTimestamp(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::NackFrag(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::Pad(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
        };
        Ok(())
    }
}

impl MappingWriteByteOrderInfoInData for RtpsMessageWrite<'_> {
    fn mapping_write_byte_order_info_in_data<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        // The byteorder is determined by each submessage individually. Hence
        // decide here for a byteorder for the header
        self.header()
            .mapping_write_byte_ordered::<_, LittleEndian>(&mut writer)?;
        for submessage in self.submessages() {
            submessage.mapping_write_byte_order_info_in_data(&mut writer)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::{
        rtps::{
            messages::{
                overall_structure::RtpsMessageHeader,
                submessage_elements::{Parameter, ParameterList},
                submessages::data::DataSubmessageWrite,
                types::{ParameterId, ProtocolId, SerializedPayload},
            },
            types::{
                EntityId, EntityKey, GuidPrefix, ProtocolVersion, SequenceNumber, VendorId,
                USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY,
            },
        },
        rtps_udp_psm::mapping_traits::to_bytes,
    };

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
        assert_eq!(to_bytes(&value).unwrap(), vec![
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]);
    }

    // #[test]
    // fn serialize_rtps_message() {
    //     let header = RtpsMessageHeader {
    //         protocol: ProtocolId::PROTOCOL_RTPS,
    //         version: ProtocolVersion::new(2, 3),
    //         vendor_id: VendorId::new([9, 8]),
    //         guid_prefix: GuidPrefix::new([3; 12]),
    //     };
    //     let endianness_flag = true;
    //     let inline_qos_flag = true;
    //     let data_flag = false;
    //     let key_flag = false;
    //     let non_standard_payload_flag = false;
    //     let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
    //     let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
    //     let writer_sn = SequenceNumber::new(5);
    //     let parameter_1 = Parameter::new(ParameterId(6), vec![10, 11, 12, 13]);
    //     let parameter_2 = Parameter::new(ParameterId(7), vec![20, 21, 22, 23]);
    //     let inline_qos = &ParameterList::new(vec![parameter_1, parameter_2]);
    //     let serialized_payload = SerializedPayload::new(&[]);

    //     let submessage = RtpsSubmessageWriteKind::Data(DataSubmessageWrite {
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
    //     });
    //     let value = RtpsMessageWrite::new(header, vec![submessage]);
    //     #[rustfmt::skip]
    //     assert_eq!(to_bytes(&value).unwrap(), vec![
    //         b'R', b'T', b'P', b'S', // Protocol
    //         2, 3, 9, 8, // ProtocolVersion | VendorId
    //         3, 3, 3, 3, // GuidPrefix
    //         3, 3, 3, 3, // GuidPrefix
    //         3, 3, 3, 3, // GuidPrefix
    //         0x15, 0b_0000_0011, 40, 0, // Submessage header
    //         0, 0, 16, 0, // extraFlags, octetsToInlineQos
    //         1, 2, 3, 4, // readerId: value[4]
    //         6, 7, 8, 9, // writerId: value[4]
    //         0, 0, 0, 0, // writerSN: high
    //         5, 0, 0, 0, // writerSN: low
    //         6, 0, 4, 0, // inlineQos: parameterId_1, length_1
    //         10, 11, 12, 13, // inlineQos: value_1[length_1]
    //         7, 0, 4, 0, // inlineQos: parameterId_2, length_2
    //         20, 21, 22, 23, // inlineQos: value_2[length_2]
    //         1, 0, 0, 0, // inlineQos: Sentinel
    //     ]);
    // }
}

use super::submessage_elements;
use super::{Submessage, SubmessageHeader};
use super::{SubmessageFlag, SubmessageKind};
use crate::types;

#[derive(Debug, PartialEq)]
pub struct Data<'a> {
    pub endianness_flag: SubmessageFlag,
    pub inline_qos_flag: SubmessageFlag,
    pub data_flag: SubmessageFlag,
    pub key_flag: SubmessageFlag,
    pub non_standard_payload_flag: SubmessageFlag,
    pub reader_id: submessage_elements::EntityId,
    pub writer_id: submessage_elements::EntityId,
    pub writer_sn: submessage_elements::SequenceNumber,
    pub inline_qos: submessage_elements::ParameterList,
    pub serialized_payload: &'a submessage_elements::SerializedData,
}

impl<'a> Submessage for Data<'a> {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeader {
        let submessage_id = SubmessageKind::Data;

        let x = false;
        let e = self.endianness_flag; // Indicates endianness.
        let q = self.inline_qos_flag; //Indicates to the Reader the presence of a ParameterList containing QoS parameters that should be used to interpret the message.
        let d = self.data_flag; //Indicates to the Reader that the dataPayload submessage element contains the serialized value of the data-object.
        let k = self.key_flag; //Indicates to the Reader that the dataPayload submessage element contains the serialized value of the key of the data-object.
        let n = self.non_standard_payload_flag; //Indicates to the Reader that the serializedPayload submessage element is not formatted according to Section 10.
                                                // X|X|X|N|K|D|Q|E
        let flags = [e, q, d, k, n, x, x, x];

        SubmessageHeader::new(submessage_id, flags, octets_to_next_header)
    }

    fn is_valid(&self) -> bool {
        if self.writer_sn < 1 || self.writer_sn == types::constants::SEQUENCE_NUMBER_UNKNOWN {
            //TODO: Check validity of inline_qos
            false
        } else {
            true
        }
    }
}

impl<'a> serde::Serialize for Data<'a> {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

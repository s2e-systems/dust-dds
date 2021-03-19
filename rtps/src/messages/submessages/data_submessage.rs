use serde::ser::SerializeStruct;

use super::submessage_elements;
use super::SubmessageFlag;
use super::{Submessage, SubmessageHeader};
use crate::messages::types::constants;
use crate::types;

pub struct Data<'a> {
    pub endianness_flag: SubmessageFlag,
    pub inline_qos_flag: SubmessageFlag,
    pub data_flag: SubmessageFlag,
    pub key_flag: SubmessageFlag,
    pub non_standard_payload_flag: SubmessageFlag,
    pub reader_id: submessage_elements::EntityId,
    pub writer_id: submessage_elements::EntityId,
    pub writer_sn: submessage_elements::SequenceNumber,
    pub inline_qos: &'a dyn submessage_elements::ParameterList,
    pub serialized_payload: &'a submessage_elements::SerializedData,
}

impl<'a> Submessage for Data<'a> {
    fn submessage_header(&self) -> SubmessageHeader {
        let x = false;
        let e = self.endianness_flag; // Indicates endianness.
        let q = self.inline_qos_flag; //Indicates to the Reader the presence of a ParameterList containing QoS parameters that should be used to interpret the message.
        let d = self.data_flag; //Indicates to the Reader that the dataPayload submessage element contains the serialized value of the data-object.
        let k = self.key_flag; //Indicates to the Reader that the dataPayload submessage element contains the serialized value of the key of the data-object.
        let n = self.non_standard_payload_flag; //Indicates to the Reader that the serializedPayload submessage element is not formatted according to Section 10.
                                                // X|X|X|N|K|D|Q|E
        let flags = [e, q, d, k, n, x, x, x];

        SubmessageHeader::new(constants::SUBMESSAGE_KIND_DATA, flags, 0)
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Data", 1)?;
        state.serialize_field("header", &self.submessage_header())?;
        state.end()
    }
}

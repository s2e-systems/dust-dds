use super::{SubmessageKind, SubmessageFlag, };
use super::{Submessage, SubmessageHeader, };
use super::submessage_elements;
use crate::types;
use crate::messages::types::Endianness;

#[derive(Debug, PartialEq)]
pub struct Data {
    endianness_flag: SubmessageFlag,
    inline_qos_flag: SubmessageFlag,    
    data_flag: SubmessageFlag, 
    key_flag: SubmessageFlag,
    non_standard_payload_flag: SubmessageFlag,
    reader_id: submessage_elements::EntityId,
    writer_id: submessage_elements::EntityId,
    writer_sn: submessage_elements::SequenceNumber,
    inline_qos: submessage_elements::ParameterList,
    serialized_payload: submessage_elements::SerializedData,
}

#[derive(PartialEq, Debug)]
pub enum Payload {
    None,
    Data(Vec<u8>),
    Key(Vec<u8>),
    NonStandard(Vec<u8>),
}


impl Data {
    /// Inline_qos_flag is inferred from option of inline_qos
    /// data_flag, key_flag and non_standard_payload_flag are inferred from the kind of payload
    pub fn new(
        endianness: Endianness,
        reader_id: types::EntityId,
        writer_id: types::EntityId,
        writer_sn: rust_dds_interface::types::SequenceNumber,
        inline_qos: Option<rust_dds_interface::types::ParameterList>,
        payload: Payload,) -> Self {
            let endianness_flag = endianness.into();
            let inline_qos_flag = inline_qos.is_some();
            let mut data_flag = false;
            let mut key_flag = false;
            let mut non_standard_payload_flag = false;
            let inline_qos = match inline_qos {
                Some(inline_qos_parameter_list) => inline_qos_parameter_list,
                None => rust_dds_interface::types::ParameterList::new(),
            };
            let serialized_payload = match  payload {
                Payload::Data(serialized_payload) => {data_flag = true; serialized_payload},
                Payload::Key(serialized_payload) => {key_flag = true; serialized_payload},
                Payload::NonStandard(serialized_payload) => {non_standard_payload_flag = true; serialized_payload},
                Payload::None => {Vec::new()},
            };
        
        Data {
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
        }
    }

    pub fn from_raw_parts(endianness_flag: SubmessageFlag,
        inline_qos_flag: SubmessageFlag,    
        data_flag: SubmessageFlag, 
        key_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        reader_id: submessage_elements::EntityId,
        writer_id: submessage_elements::EntityId,
        writer_sn: submessage_elements::SequenceNumber,
        inline_qos: submessage_elements::ParameterList,
        serialized_payload: submessage_elements::SerializedData,) -> Self {
            
            Data {
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
            }


    }

    pub fn reader_id(&self) -> submessage_elements::EntityId {
        self.reader_id
    }

    pub fn writer_id(&self) -> submessage_elements::EntityId {
        self.writer_id
    }

    pub fn writer_sn(&self) -> submessage_elements::SequenceNumber {
        self.writer_sn
    }

    pub fn serialized_payload(&self) -> &submessage_elements::SerializedData {
        &self.serialized_payload
    }
    
    pub fn inline_qos(&self) -> &rust_dds_interface::types::ParameterList {
        &self.inline_qos
    }

    pub fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    pub fn inline_qos_flag(&self) -> SubmessageFlag {
        self.inline_qos_flag
    }

    pub fn data_flag(&self) -> SubmessageFlag {
        self.data_flag
    }

    pub fn key_flag(&self) -> SubmessageFlag {
        self.key_flag
    }

    pub fn non_standard_payload_flag(&self) -> SubmessageFlag {
        self.non_standard_payload_flag
    }

    pub fn take_payload_and_qos(self) -> (submessage_elements::SerializedData, rust_dds_interface::types::ParameterList) {
        (self.serialized_payload, self.inline_qos)
    }
}

impl Submessage for Data {
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
        if self.writer_sn < 1 || self.writer_sn == rust_dds_interface::types::SEQUENCE_NUMBER_UNKNOWN {
            //TODO: Check validity of inline_qos
            false
        } else {
            true
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::constants::{ENTITYID_UNKNOWN, ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, };

    // E: EndiannessFlag - Indicates endianness.
    // Q: InlineQosFlag - Indicates to the Reader the presence of a ParameterList containing QoS parameters that should be used to interpret the message.
    // D: DataFlag - Indicates to the Reader that the dataPayload submessage element contains the serialized value of the data-object.
    // K: KeyFlag - Indicates to the Reader that the dataPayload submessage element contains the serialized value of the key of the data-object. 
    // N: NonStandardPayloadFlag  -Indicates to the Reader that the serializedPayload submessage element is not formatted according to Section 10.
    // X|X|X|N|K|D|Q|E
    #[test]
    fn test_data_contructor() {
        let data = Data::new(
            Endianness::LittleEndian,
            ENTITYID_UNKNOWN, 
            ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER, 
            1, 
            Some(rust_dds_interface::types::ParameterList::new()),
            Payload::Data(vec![])
        );
        assert_eq!(data.endianness_flag, true);
        assert_eq!(data.inline_qos_flag, true);
        assert_eq!(data.data_flag, true);
        assert_eq!(data.key_flag, false);
        assert_eq!(data.non_standard_payload_flag, false);
    }
}
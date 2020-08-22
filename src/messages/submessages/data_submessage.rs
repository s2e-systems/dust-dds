use std::convert::From;

use crate::messages::serdes::{SubmessageElement, Endianness, RtpsSerdesResult, };
use super::{SubmessageKind, SubmessageFlag, UdpPsmMapping, };
use super::{Submessage, SubmessageHeader, };
use super::submessage_elements;
use crate::types;
use crate::types::constants::SEQUENCE_NUMBER_UNKNOWN;
use crate::messages::parameter_list::ParameterList;


#[derive(PartialEq, Debug)]
pub struct Data {
    endianness_flag: SubmessageFlag,
    inline_qos_flag: SubmessageFlag,    
    data_flag: SubmessageFlag, 
    key_flag: SubmessageFlag,
    non_standard_payload_flag: SubmessageFlag,
    reader_id: submessage_elements::EntityId,
    writer_id: submessage_elements::EntityId,
    writer_sn: submessage_elements::SequenceNumber,
    inline_qos: ParameterList,
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
        endianness_flag: Endianness,
        reader_id: types::EntityId,
        writer_id: types::EntityId,
        writer_sn: types::SequenceNumber,
        inline_qos: Option<ParameterList>,
        payload: Payload,) -> Self {
            let inline_qos_flag = inline_qos.is_some();
            let mut data_flag = false;
            let mut key_flag = false;
            let mut non_standard_payload_flag = false;
            let inline_qos = match inline_qos {
                Some(inline_qos_parameter_list) => inline_qos_parameter_list,
                None => ParameterList::new(),
            };
            let serialized_payload = match  payload {
                Payload::Data(serialized_payload) => {data_flag = true; submessage_elements::SerializedData(serialized_payload)},
                Payload::Key(serialized_payload) => {key_flag = true; submessage_elements::SerializedData(serialized_payload)},
                Payload::NonStandard(serialized_payload) => {non_standard_payload_flag = true; submessage_elements::SerializedData(serialized_payload)},
                Payload::None => {submessage_elements::SerializedData(Vec::new())},
            };
        
        Data {
            endianness_flag: endianness_flag.into(),
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id: submessage_elements::EntityId(reader_id),
            writer_id: submessage_elements::EntityId(writer_id),
            writer_sn: submessage_elements::SequenceNumber(writer_sn),
            inline_qos, 
            serialized_payload, 
        }
    }

    pub fn reader_id(&self) -> types::EntityId {
        self.reader_id.0
    }

    pub fn writer_id(&self) -> types::EntityId {
        self.writer_id.0
    }

    pub fn writer_sn(&self) -> types::SequenceNumber {
        self.writer_sn.0
    }

    pub fn serialized_payload(&self) -> &Vec<u8> {
        &self.serialized_payload.0
    }
    
    pub fn inline_qos(&self) -> &ParameterList {
        &self.inline_qos
    }

    pub fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
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

    pub fn take_payload_and_qos(self) -> (submessage_elements::SerializedData, ParameterList) {
        (self.serialized_payload, self.inline_qos)
    }
}

impl Submessage for Data {
    fn submessage_flags(&self) -> [SubmessageFlag; 8] {
        let x = false;
        let e = self.endianness_flag; // Indicates endianness.
        let q = self.inline_qos_flag; //Indicates to the Reader the presence of a ParameterList containing QoS parameters that should be used to interpret the message.
        let d = self.data_flag; //Indicates to the Reader that the dataPayload submessage element contains the serialized value of the data-object.
        let k = self.key_flag; //Indicates to the Reader that the dataPayload submessage element contains the serialized value of the key of the data-object. 
        let n = self.non_standard_payload_flag; //Indicates to the Reader that the serializedPayload submessage element is not formatted according to Section 10.
        // X|X|X|N|K|D|Q|E
        [e, q, d, k, n, x, x, x]
    }

    fn is_valid(&self) -> bool {
        if self.writer_sn.0 < 1 || self.writer_sn.0 == SEQUENCE_NUMBER_UNKNOWN {
            //TODO: Check validity of inline_qos
            false
        } else {
            true
        }
    }
}
use std::io::BufRead;

use crate::implementation::{
    data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
    rtps::{
        messages::{
            overall_structure::SubmessageHeaderRead,
            submessage_elements::ParameterList,
            types::{FragmentNumber, SerializedPayload, SubmessageFlag, ULong, UShort},
            RtpsMap, SubmessageHeader,
        },
        types::{EntityId, SequenceNumber},
    },
};

#[derive(Debug, PartialEq, Eq)]
pub struct DataFragSubmessageRead<'a> {
    data: &'a [u8],
}

impl SubmessageHeader for DataFragSubmessageRead<'_> {
    fn submessage_header(&self) -> SubmessageHeaderRead {
        SubmessageHeaderRead::new(self.data)
    }
}

impl<'a> DataFragSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    fn octets_to_inline_qos(&self) -> u16 {
        self.map(&self.data[6..])
    }

    fn inline_qos_len(&self) -> usize {
        if self.inline_qos_flag() {
            let mut parameter_list_buf = &self.data[8 + self.octets_to_inline_qos() as usize..];
            let parameter_list_buf_length = parameter_list_buf.len();
            loop {
                let pid: u16 = self.map(parameter_list_buf);
                parameter_list_buf.consume(2);
                let length: i16 = self.map(parameter_list_buf);
                parameter_list_buf.consume(2);
                if pid == PID_SENTINEL {
                    break;
                } else {
                    parameter_list_buf.consume(length as usize);
                }
            }
            parameter_list_buf_length - parameter_list_buf.len()
        } else {
            0
        }
    }

    pub fn endianness_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0001) != 0
    }

    pub fn inline_qos_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0010) != 0
    }

    pub fn key_flag(&self) -> bool {
        (self.data[1] & 0b_0000_0100) != 0
    }

    pub fn non_standard_payload_flag(&self) -> bool {
        (self.data[1] & 0b_0000_1000) != 0
    }

    pub fn reader_id(&self) -> EntityId {
        self.map(&self.data[8..])
    }

    pub fn writer_id(&self) -> EntityId {
        self.map(&self.data[12..])
    }

    pub fn writer_sn(&self) -> SequenceNumber {
        self.map(&self.data[16..])
    }

    pub fn fragment_starting_num(&self) -> FragmentNumber {
        self.map(&self.data[24..])
    }

    pub fn fragments_in_submessage(&self) -> u16 {
        self.map(&self.data[28..])
    }

    pub fn fragment_size(&self) -> u16 {
        self.map(&self.data[30..])
    }

    pub fn data_size(&self) -> u32 {
        self.map(&self.data[32..])
    }

    pub fn inline_qos(&self) -> ParameterList {
        if self.inline_qos_flag() {
            self.map(&self.data[self.octets_to_inline_qos() as usize + 8..])
        } else {
            ParameterList::empty()
        }
    }

    pub fn serialized_payload(&self) -> SerializedPayload<'a> {
        self.map(&self.data[8 + self.octets_to_inline_qos() as usize + self.inline_qos_len()..])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DataFragSubmessageWrite<'a> {
    pub endianness_flag: SubmessageFlag,
    pub inline_qos_flag: SubmessageFlag,
    pub non_standard_payload_flag: SubmessageFlag,
    pub key_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub fragment_starting_num: FragmentNumber,
    pub fragments_in_submessage: UShort,
    pub data_size: ULong,
    pub fragment_size: UShort,
    pub inline_qos: &'a ParameterList,
    pub serialized_payload: SerializedPayload<'a>,
}

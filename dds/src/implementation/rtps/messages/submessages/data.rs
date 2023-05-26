use byteorder::ReadBytesExt;

use crate::implementation::{
    data_representation_builtin_endpoints::parameter_id_values::PID_SENTINEL,
    rtps::{
        messages::{
            overall_structure::SubmessageHeaderRead,
            submessage_elements::ParameterList,
            types::{SerializedPayload, SubmessageFlag},
            RtpsMap, SubmessageHeader,
        },
        types::{EntityId, SequenceNumber},
    },
};

#[derive(Debug, PartialEq, Eq)]
pub struct DataSubmessageRead<'a> {
    data: &'a [u8],
}

impl SubmessageHeader for DataSubmessageRead<'_> {
    fn submessage_header(&self) -> SubmessageHeaderRead {
        SubmessageHeaderRead::new(self.data)
    }
}

impl<'a> DataSubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    fn octets_to_inline_qos(&self) -> usize {
        (&self.data[6..])
            .read_u16::<byteorder::LittleEndian>()
            .unwrap() as usize
    }

    fn inline_qos_len(&self) -> usize {
        let mut parameter_list_buf = &self.data[8 + self.octets_to_inline_qos()..];
        let parameter_list_buf_length = parameter_list_buf.len();

        if self.inline_qos_flag() {
            loop {
                let pid = parameter_list_buf
                    .read_u16::<byteorder::LittleEndian>()
                    .expect("pid read failed");
                let length = parameter_list_buf
                    .read_i16::<byteorder::LittleEndian>()
                    .expect("length read failed");
                if pid == PID_SENTINEL {
                    break;
                } else {
                    (_, parameter_list_buf) = parameter_list_buf.split_at(length as usize);
                }
            }
            parameter_list_buf_length - parameter_list_buf.len()
        } else {
            0
        }
    }

    pub fn inline_qos_flag(&self) -> bool {
        self.submessage_header().flags()[1]
    }

    pub fn data_flag(&self) -> bool {
        self.submessage_header().flags()[2]
    }

    pub fn key_flag(&self) -> bool {
        self.submessage_header().flags()[3]
    }

    pub fn non_standard_payload_flag(&self) -> bool {
        self.submessage_header().flags()[4]
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

    pub fn inline_qos(&self) -> ParameterList {
        if self.inline_qos_flag() {
            self.map(&self.data[self.octets_to_inline_qos() + 8..])
        } else {
            ParameterList::empty()
        }
    }

    pub fn serialized_payload(&self) -> SerializedPayload<'a> {
        self.map(&self.data[8 + self.octets_to_inline_qos() + self.inline_qos_len()..])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DataSubmessageWrite<'a> {
    pub endianness_flag: SubmessageFlag,
    pub inline_qos_flag: SubmessageFlag,
    pub data_flag: SubmessageFlag,
    pub key_flag: SubmessageFlag,
    pub non_standard_payload_flag: SubmessageFlag,
    pub reader_id: EntityId,
    pub writer_id: EntityId,
    pub writer_sn: SequenceNumber,
    pub inline_qos: &'a ParameterList,
    pub serialized_payload: SerializedPayload<'a>,
}

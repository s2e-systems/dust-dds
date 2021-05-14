use crate::{messages::submessage_elements, PIM};

pub struct Data<PSM: PIM, SerializedData> {
    pub endianness_flag: PSM::SubmessageFlag,
    pub inline_qos_flag: PSM::SubmessageFlag,
    pub data_flag: PSM::SubmessageFlag,
    pub key_flag: PSM::SubmessageFlag,
    pub non_standard_payload_flag: PSM::SubmessageFlag,
    pub reader_id: submessage_elements::EntityId<PSM>,
    pub writer_id: submessage_elements::EntityId<PSM>,
    pub writer_sn: submessage_elements::SequenceNumber<PSM>,
    // pub inline_qos: <Self::PSM as structure::Types>::ParameterVector,
    pub serialized_payload: SerializedData,
}

impl<PSM: PIM, SerializedData> Data<PSM, SerializedData> {
    pub fn new(
        endianness_flag: PSM::SubmessageFlag,
        inline_qos_flag: PSM::SubmessageFlag,
        data_flag: PSM::SubmessageFlag,
        key_flag: PSM::SubmessageFlag,
        non_standard_payload_flag: PSM::SubmessageFlag,
        reader_id: PSM::EntityId,
        writer_id: PSM::EntityId,
        writer_sn: PSM::SequenceNumber,
        serialized_payload: SerializedData,
    ) -> Self {
        Self {
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id: submessage_elements::EntityId { value: reader_id },
            writer_id: submessage_elements::EntityId { value: writer_id },
            writer_sn: submessage_elements::SequenceNumber { value: writer_sn },
            serialized_payload,
        }
    }
}

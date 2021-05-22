use crate::{
    messages::{self, submessage_elements, Submessage},
    structure,
};

pub trait Data<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn new(
        endianness_flag: PSM::SubmessageFlag,
        inline_qos_flag: PSM::SubmessageFlag,
        data_flag: PSM::SubmessageFlag,
        key_flag: PSM::SubmessageFlag,
        non_standard_payload_flag: PSM::SubmessageFlag,
        reader_id: PSM::EntityId,
        writer_id: PSM::EntityId,
        writer_sn: PSM::SequenceNumber,
        serialized_payload: &PSM::Data,
    ) -> Self;
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn inline_qos_flag(&self) -> PSM::SubmessageFlag;
    fn data_flag(&self) -> PSM::SubmessageFlag;
    fn key_flag(&self) -> PSM::SubmessageFlag;
    fn non_standard_payload_flag(&self) -> PSM::SubmessageFlag;
    fn reader_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_sn(&self) -> submessage_elements::SequenceNumber<PSM>;
    // pub inline_qos: <Self::PSM as structure::Types>::ParameterVector,
    fn serialized_payload(&self) -> &PSM::Data;
}

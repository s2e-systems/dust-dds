use crate::{
    messages::{self, submessage_elements, Submessage},
    structure,
};

pub trait DataFrag<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn inline_qos_flag(&self) -> PSM::SubmessageFlag;
    fn non_standard_payload_flag(&self) -> PSM::SubmessageFlag;
    fn key_flag(&self) -> PSM::SubmessageFlag;
    fn reader_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_id(&self) -> submessage_elements::EntityId<PSM>;
    fn writer_sn(&self) -> submessage_elements::SequenceNumber<PSM>;
    fn fragment_starting_num(&self) -> submessage_elements::FragmentNumber<PSM>;
    fn fragments_in_submessage(&self) -> submessage_elements::UShort;
    fn data_size(&self) -> submessage_elements::ULong;
    fn fragment_size(&self) -> submessage_elements::UShort;
    fn inline_qos(&self) -> submessage_elements::ParameterList<PSM>;
    fn serialized_payload(&self) -> &PSM::Data;
}

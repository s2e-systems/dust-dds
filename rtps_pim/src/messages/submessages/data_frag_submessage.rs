use super::submessage_elements;
use super::SubmessageFlag;

pub trait DataFrag {
    type EntityId: submessage_elements::EntityId;
    type SequenceNumber: submessage_elements::SubmessageElement;
    type FragmentNumber: submessage_elements::FragmentNumber;
    type ParameterList: submessage_elements::ParameterList;
    type SerializedDataFragment: submessage_elements::SerializedDataFragment;

    fn endianness_flag(&self) -> SubmessageFlag;
    fn inline_qos_flag(&self) -> SubmessageFlag;
    fn non_standard_payload_flag(&self) -> SubmessageFlag;
    fn key_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn writer_sn(&self) -> &Self::SequenceNumber;
    fn fragment_starting_num(&self) -> &Self::FragmentNumber;
    fn fragments_in_submessage(&self) -> submessage_elements::UShort;
    fn data_size(&self) -> submessage_elements::ULong;
    fn fragment_size(&self) -> submessage_elements::UShort;
    fn inline_qos(&self) -> &Self::ParameterList;
    fn serialized_payload(&self) -> &Self::SerializedDataFragment;
}

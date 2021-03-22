use super::{submessage_elements, Submessage, SubmessageHeader};

pub trait DataFrag : Submessage {
    type EntityId: submessage_elements::EntityId;
    type SequenceNumber: submessage_elements::SubmessageElement;
    type FragmentNumber: submessage_elements::FragmentNumber;
    type ParameterList: submessage_elements::ParameterList;
    type SerializedDataFragment: submessage_elements::SerializedDataFragment;

    fn endianness_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    fn inline_qos_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    fn non_standard_payload_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    fn key_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn writer_sn(&self) -> &Self::SequenceNumber;
    fn fragment_starting_num(&self) -> &Self::FragmentNumber;
    fn fragments_in_submessage(&self) -> u16;
    fn data_size(&self) -> u32;
    fn fragment_size(&self) -> u16;
    fn inline_qos(&self) -> &Self::ParameterList;
    fn serialized_payload(&self) -> &Self::SerializedDataFragment;
}

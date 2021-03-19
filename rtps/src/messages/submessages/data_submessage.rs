use super::submessage_elements;
use super::SubmessageFlag;

pub trait Data {
    type EntityId: submessage_elements::EntityId;
    type SequenceNumber: submessage_elements::SequenceNumber;
    type ParameterList: submessage_elements::ParameterList;
    type SerializedData: submessage_elements::SerializedData;

    fn endianness_flag(&self) -> SubmessageFlag;
    fn inline_qos_flag(&self) -> SubmessageFlag;
    fn data_flag(&self) -> SubmessageFlag;
    fn key_flag(&self) -> SubmessageFlag;
    fn non_standard_payload_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn writer_sn(&self) -> &Self::SequenceNumber;
    fn inline_qos(&self) -> &Self::ParameterList;
    fn serialized_payload(&self) -> &Self::SerializedData;
}

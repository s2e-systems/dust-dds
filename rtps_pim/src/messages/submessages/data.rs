use crate::messages::{self, submessage_elements, Submessage};

pub trait Data: Submessage {
    type SerializedData: AsRef<[u8]>;

    fn new(
        endianness_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        inline_qos_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        data_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        key_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        non_standard_payload_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        reader_id: submessage_elements::EntityId<Self::PSM>,
        writer_id: submessage_elements::EntityId<Self::PSM>,
        writer_sn: submessage_elements::SequenceNumber<Self::PSM>,
        inline_qos: submessage_elements::ParameterList<Self::PSM>,
        serialized_payload: submessage_elements::SerializedData<Self::SerializedData>,
    ) -> Self;

    fn endianness_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn inline_qos_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn data_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn key_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn non_standard_payload_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn reader_id(&self) -> &submessage_elements::EntityId<Self::PSM>;
    fn writer_id(&self) -> &submessage_elements::EntityId<Self::PSM>;
    fn writer_sn(&self) -> &submessage_elements::SequenceNumber<Self::PSM>;
    fn inline_qos(&self) -> &submessage_elements::ParameterList<Self::PSM>;
    fn serialized_payload(&self) -> &submessage_elements::SerializedData<Self::SerializedData>;
}

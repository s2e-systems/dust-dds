use crate::messages::{self, submessage_elements, Submessage};

pub trait DataFrag: Submessage {
    type SerializedDataFragment: AsRef<[u8]>;

    fn new(
        endianness_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        inline_qos_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        non_standard_payload_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        key_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        reader_id: submessage_elements::EntityId<Self::PSM>,
        writer_id: submessage_elements::EntityId<Self::PSM>,
        writer_sn: submessage_elements::SequenceNumber<Self::PSM>,
        fragment_starting_num: submessage_elements::FragmentNumber<Self::PSM>,
        fragments_in_submessage: submessage_elements::UShort,
        data_size: submessage_elements::ULong,
        fragment_size: submessage_elements::UShort,
        inline_qos: submessage_elements::ParameterList<Self::PSM>,
        serialized_payload: submessage_elements::SerializedDataFragment<
            Self::SerializedDataFragment,
        >,
    ) -> Self;

    fn endianness_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn inline_qos_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn non_standard_payload_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn key_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn reader_id(&self) -> &submessage_elements::EntityId<Self::PSM>;
    fn writer_id(&self) -> &submessage_elements::EntityId<Self::PSM>;
    fn writer_sn(&self) -> &submessage_elements::SequenceNumber<Self::PSM>;
    fn fragment_starting_num(&self) -> &submessage_elements::FragmentNumber<Self::PSM>;
    fn fragments_in_submessage(&self) -> &submessage_elements::UShort;
    fn data_size(&self) -> &submessage_elements::ULong;
    fn fragment_size(&self) -> &submessage_elements::UShort;
    fn inline_qos(&self) -> &submessage_elements::ParameterList<Self::PSM>;
    fn serialized_payload(
        &self,
    ) -> &submessage_elements::SerializedDataFragment<Self::SerializedDataFragment>;
}

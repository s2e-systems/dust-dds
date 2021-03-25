use super::{submessage_elements, Submessage};
use crate::{messages, types};

pub trait DataFrag: Submessage {
    type EntityId: types::EntityId;
    type SequenceNumber: types::SequenceNumber;
    type FragmentNumber: messages::types::FragmentNumber;
    type ParameterId: messages::types::ParameterId;
    type ParameterValue: AsRef<[u8]> + Clone;
    type ParameterList: IntoIterator<Item = submessage_elements::Parameter<Self::ParameterId, Self::ParameterValue>>
        + Clone;
    type SerializedDataFragment: AsRef<[u8]>;

    fn new(
        endianness_flag: <Self as Submessage>::SubmessageFlag,
        inline_qos_flag: <Self as Submessage>::SubmessageFlag,
        non_standard_payload_flag: <Self as Submessage>::SubmessageFlag,
        key_flag: <Self as Submessage>::SubmessageFlag,
        reader_id: submessage_elements::EntityId<Self::EntityId>,
        writer_id: submessage_elements::EntityId<Self::EntityId>,
        writer_sn: submessage_elements::SequenceNumber<Self::SequenceNumber>,
        fragment_starting_num: submessage_elements::FragmentNumber<Self::FragmentNumber>,
        fragments_in_submessage: submessage_elements::UShort,
        data_size: submessage_elements::ULong,
        fragment_size: submessage_elements::UShort,
        inline_qos: submessage_elements::ParameterList<
            Self::ParameterId,
            Self::ParameterValue,
            Self::ParameterList,
        >,
        serialized_payload: submessage_elements::SerializedDataFragment<
            Self::SerializedDataFragment,
        >,
    ) -> Self;

    fn endianness_flag(&self) -> <Self as Submessage>::SubmessageFlag;
    fn inline_qos_flag(&self) -> <Self as Submessage>::SubmessageFlag;
    fn non_standard_payload_flag(&self) -> <Self as Submessage>::SubmessageFlag;
    fn key_flag(&self) -> <Self as Submessage>::SubmessageFlag;
    fn reader_id(&self) -> &submessage_elements::EntityId<Self::EntityId>;
    fn writer_id(&self) -> &submessage_elements::EntityId<Self::EntityId>;
    fn writer_sn(&self) -> &submessage_elements::SequenceNumber<Self::SequenceNumber>;
    fn fragment_starting_num(&self) -> &submessage_elements::FragmentNumber<Self::FragmentNumber>;
    fn fragments_in_submessage(&self) -> &submessage_elements::UShort;
    fn data_size(&self) -> &submessage_elements::ULong;
    fn fragment_size(&self) -> &submessage_elements::UShort;
    fn inline_qos(
        &self,
    ) -> &submessage_elements::ParameterList<
        Self::ParameterId,
        Self::ParameterValue,
        Self::ParameterList,
    >;
    fn serialized_payload(&self) -> &Self::SerializedDataFragment;
}

use rust_rtps_pim::messages::submessages::{
    submessage_elements::{self, Parameter},
    SubmessageHeader,
};

use crate::{
    messages::types::{FragmentNumber, ParameterId, SubmessageFlag, SubmessageKind},
    types::{EntityId, SequenceNumber},
};
pub struct DataFrag<'a> {
    endianness_flag: SubmessageFlag,
    inline_qos_flag: SubmessageFlag,
    non_standard_payload_flag: SubmessageFlag,
    key_flag: SubmessageFlag,
    reader_id: submessage_elements::EntityId<EntityId>,
    writer_id: submessage_elements::EntityId<EntityId>,
    writer_sn: submessage_elements::SequenceNumber<SequenceNumber>,
    fragment_starting_num: submessage_elements::FragmentNumber<FragmentNumber>,
    fragments_in_submessage: submessage_elements::UShort,
    data_size: submessage_elements::ULong,
    fragment_size: submessage_elements::UShort,
    inline_qos: submessage_elements::ParameterList<
        ParameterId,
        Vec<u8>,
        Vec<Parameter<ParameterId, Vec<u8>>>,
    >,
    serialized_payload: submessage_elements::SerializedDataFragment<&'a [u8]>,
}

impl<'a> rust_rtps_pim::messages::submessages::data_frag_submessage::DataFrag for DataFrag<'a> {
    type EntityId = EntityId;
    type SequenceNumber = SequenceNumber;
    type FragmentNumber = FragmentNumber;
    type ParameterId = ParameterId;
    type ParameterValue = Vec<u8>;
    type ParameterList = Vec<Parameter<Self::ParameterId, Self::ParameterValue>>;
    type SerializedDataFragment = &'a [u8];

    fn new(
        endianness_flag: SubmessageFlag,
        inline_qos_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
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
    ) -> Self {
        Self {
            endianness_flag,
            inline_qos_flag,
            non_standard_payload_flag,
            key_flag,
            reader_id,
            writer_id,
            writer_sn,
            fragment_starting_num,
            fragments_in_submessage,
            data_size,
            fragment_size,
            inline_qos,
            serialized_payload,
        }
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    fn inline_qos_flag(&self) -> SubmessageFlag {
        self.inline_qos_flag
    }

    fn non_standard_payload_flag(&self) -> SubmessageFlag {
        self.non_standard_payload_flag
    }

    fn key_flag(&self) -> SubmessageFlag {
        self.key_flag
    }

    fn reader_id(&self) -> &submessage_elements::EntityId<Self::EntityId> {
        &self.reader_id
    }

    fn writer_id(&self) -> &submessage_elements::EntityId<Self::EntityId> {
        &self.writer_id
    }

    fn writer_sn(&self) -> &submessage_elements::SequenceNumber<Self::SequenceNumber> {
        &self.writer_sn
    }

    fn fragment_starting_num(&self) -> &submessage_elements::FragmentNumber<Self::FragmentNumber> {
        &self.fragment_starting_num
    }

    fn fragments_in_submessage(&self) -> &submessage_elements::UShort {
        &self.fragments_in_submessage
    }

    fn data_size(&self) -> &submessage_elements::ULong {
        &self.data_size
    }

    fn fragment_size(&self) -> &submessage_elements::UShort {
        &self.fragment_size
    }

    fn inline_qos(
        &self,
    ) -> &submessage_elements::ParameterList<
        Self::ParameterId,
        Self::ParameterValue,
        Self::ParameterList,
    > {
        &self.inline_qos
    }

    fn serialized_payload(&self) -> &submessage_elements::SerializedDataFragment<Self::SerializedDataFragment> {
        &self.serialized_payload
    }
}

impl<'a> rust_rtps_pim::messages::submessages::Submessage for DataFrag<'a> {
    type SubmessageKind = SubmessageKind;
    type SubmessageFlag = SubmessageFlag;

    fn submessage_header(&self) -> SubmessageHeader<Self::SubmessageKind, Self::SubmessageFlag> {
        todo!()
    }
}

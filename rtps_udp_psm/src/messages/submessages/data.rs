use rust_rtps_pim::messages::submessages::{
    submessage_elements::{self, Parameter},
    SubmessageHeader,
};

use crate::{
    messages::types::{ParameterId, SubmessageFlag, SubmessageKind},
    types::{EntityId, SequenceNumber},
};

pub struct Data<'a> {
    endianness_flag: SubmessageFlag,
    inline_qos_flag: SubmessageFlag,
    data_flag: SubmessageFlag,
    key_flag: SubmessageFlag,
    non_standard_payload_flag: SubmessageFlag,

    reader_id: submessage_elements::EntityId<EntityId>,
    writer_id: submessage_elements::EntityId<EntityId>,
    writer_sn: submessage_elements::SequenceNumber<SequenceNumber>,
    inline_qos: submessage_elements::ParameterList<
        ParameterId,
        Vec<u8>,
        Vec<Parameter<ParameterId, Vec<u8>>>,
    >,
    serialized_payload: submessage_elements::SerializedData<&'a [u8]>,
}

impl<'a> rust_rtps_pim::messages::submessages::data_submessage::Data for Data<'a> {
    type EntityId = EntityId;
    type SequenceNumber = SequenceNumber;
    type ParameterId = ParameterId;
    type ParameterValue = Vec<u8>;
    type ParameterList = Vec<Parameter<Self::ParameterId, Self::ParameterValue>>;
    type SerializedData = &'a [u8];

    fn new(
        endianness_flag: SubmessageFlag,
        inline_qos_flag: SubmessageFlag,
        data_flag: SubmessageFlag,
        key_flag: SubmessageFlag,
        non_standard_payload_flag: SubmessageFlag,
        reader_id: submessage_elements::EntityId<Self::EntityId>,
        writer_id: submessage_elements::EntityId<Self::EntityId>,
        writer_sn: submessage_elements::SequenceNumber<Self::SequenceNumber>,
        inline_qos: submessage_elements::ParameterList<
            Self::ParameterId,
            Self::ParameterValue,
            Self::ParameterList,
        >,
        serialized_payload: submessage_elements::SerializedData<Self::SerializedData>,
    ) -> Self {
        Self {
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
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

    fn data_flag(&self) -> SubmessageFlag {
        self.data_flag
    }

    fn key_flag(&self) -> SubmessageFlag {
        self.key_flag
    }

    fn non_standard_payload_flag(&self) -> SubmessageFlag {
        self.non_standard_payload_flag
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

    fn inline_qos(
        &self,
    ) -> &submessage_elements::ParameterList<
        Self::ParameterId,
        Self::ParameterValue,
        Self::ParameterList,
    > {
        &self.inline_qos
    }

    fn serialized_payload(&self) -> &submessage_elements::SerializedData<Self::SerializedData> {
        &self.serialized_payload
    }
}

impl<'a> rust_rtps_pim::messages::submessages::Submessage for Data<'a> {
    type SubmessageKind = SubmessageKind;
    type SubmessageFlag = SubmessageFlag;

    fn submessage_header(&self) -> SubmessageHeader<Self::SubmessageKind, Self::SubmessageFlag> {
        todo!()
    }
}

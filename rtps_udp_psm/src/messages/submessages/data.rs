use super::SubmessageHeader;
use crate::messages::{submessage_elements, types::SubmessageFlag};

pub struct Data<'a> {
    endianness_flag: SubmessageFlag,
    inline_qos_flag: SubmessageFlag,
    data_flag: SubmessageFlag,
    key_flag: SubmessageFlag,
    non_standard_payload_flag: SubmessageFlag,

    reader_id: <Self as rust_rtps_pim::messages::submessages::data_submessage::Data<'a>>::EntityId,
    writer_id: <Self as rust_rtps_pim::messages::submessages::data_submessage::Data<'a>>::EntityId,
    writer_sn:
        <Self as rust_rtps_pim::messages::submessages::data_submessage::Data<'a>>::SequenceNumber,
    inline_qos:
        &'a <Self as rust_rtps_pim::messages::submessages::data_submessage::Data<'a>>::ParameterList,
    serialized_payload:
        <Self as rust_rtps_pim::messages::submessages::data_submessage::Data<'a>>::SerializedData,
}

impl<'a> rust_rtps_pim::messages::submessages::Submessage for Data<'a> {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }
}

impl<'a> rust_rtps_pim::messages::submessages::data_submessage::Data<'a> for Data<'a> {
    type EntityId = submessage_elements::EntityId;
    type SequenceNumber = submessage_elements::SequenceNumber;
    type ParameterList = submessage_elements::ParameterList;
    type SerializedData = submessage_elements::SerializedData<'a>;

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

    fn reader_id(&self) -> &Self::EntityId {
        &self.reader_id
    }

    fn writer_id(&self) -> &Self::EntityId {
        &self.writer_id
    }

    fn writer_sn(&self) -> &Self::SequenceNumber {
        &self.writer_sn
    }

    fn inline_qos(&self) -> &Self::ParameterList {
        &self.inline_qos
    }

    fn serialized_payload(&self) -> &Self::SerializedData {
        &self.serialized_payload
    }

    fn new(
        _endianness_flag: SubmessageFlag,
        _inline_qos_flag: SubmessageFlag,
        _data_flag: SubmessageFlag,
        _key_flag: SubmessageFlag,
        _non_standard_payload_flag: SubmessageFlag,
        _reader_id: Self::EntityId,
        _writer_id: Self::EntityId,
        _writer_sn: Self::SequenceNumber,
        _inline_qos: &Self::ParameterList,
        _serialized_payload: Self::SerializedData,
    ) -> Self {
        todo!()
    }
}

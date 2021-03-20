use super::SubmessageHeader;
use crate::messages::submessage_elements;
use rust_rtps_pim::messages::{submessages::Submessage, types::SubmessageFlag};
pub struct DataFrag<'a> {
    endianness_flag: SubmessageFlag,
    inline_qos_flag: SubmessageFlag,
    non_standard_payload_flag: SubmessageFlag,
    key_flag: SubmessageFlag,
    reader_id:
        <Self as rust_rtps_pim::messages::submessages::data_frag_submessage::DataFrag>::EntityId,
    writer_id:
        <Self as rust_rtps_pim::messages::submessages::data_frag_submessage::DataFrag>::EntityId,
    writer_sn:
        <Self as rust_rtps_pim::messages::submessages::data_frag_submessage::DataFrag>::SequenceNumber,
    fragment_starting_num:
        <Self as rust_rtps_pim::messages::submessages::data_frag_submessage::DataFrag>::FragmentNumber,
    fragments_in_submessage: u16,
    data_size: u32,
    fragment_size: u16,
    inline_qos: &'a <Self as rust_rtps_pim::messages::submessages::data_frag_submessage::DataFrag>::ParameterList,
    serialized_payload: <Self as rust_rtps_pim::messages::submessages::data_frag_submessage::DataFrag>::SerializedDataFragment
}

impl<'a> Submessage for DataFrag<'a> {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }

    fn is_valid(&self) -> bool {
        todo!()
    }
}

impl<'a> rust_rtps_pim::messages::submessages::data_frag_submessage::DataFrag for DataFrag<'a> {
    type EntityId = submessage_elements::EntityId;
    type SequenceNumber = submessage_elements::SequenceNumber;
    type FragmentNumber = submessage_elements::FragmentNumber;
    type ParameterList = submessage_elements::ParameterList;
    type SerializedDataFragment = submessage_elements::SerializedDataFragment<'a>;

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

    fn reader_id(&self) -> &Self::EntityId {
        &self.reader_id
    }

    fn writer_id(&self) -> &Self::EntityId {
        &self.writer_id
    }

    fn writer_sn(&self) -> &Self::SequenceNumber {
        &self.writer_sn
    }

    fn fragment_starting_num(&self) -> &Self::FragmentNumber {
        &self.fragment_starting_num
    }

    fn fragments_in_submessage(&self) -> u16 {
        self.fragments_in_submessage
    }

    fn data_size(&self) -> u32 {
        self.data_size
    }

    fn fragment_size(&self) -> u16 {
        self.fragment_size
    }

    fn inline_qos(&self) -> &Self::ParameterList {
        self.inline_qos
    }

    fn serialized_payload(&self) -> &Self::SerializedDataFragment {
        &self.serialized_payload
    }
}

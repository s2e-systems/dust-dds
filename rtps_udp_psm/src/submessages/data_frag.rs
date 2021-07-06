use rust_rtps_pim::messages::types::SubmessageFlag;

use crate::{
    psm::RtpsUdpPsm,
    submessage_elements::{
        EntityId, FragmentNumber, ParameterList, SequenceNumber, SerializedData, ULong, UShort,
    },
    submessage_header::SubmessageHeader,
};

#[derive(Debug, PartialEq)]
pub struct DataFrag<'a> {
    pub serialized_data: SerializedData<'a>,
}

impl<'a> rust_rtps_pim::messages::submessages::DataFragSubmessage<'a, RtpsUdpPsm> for DataFrag<'a> {
    fn new(
        _endianness_flag: SubmessageFlag,
        _inline_qos_flag: SubmessageFlag,
        _non_standard_payload_flag: SubmessageFlag,
        _key_flag: SubmessageFlag,
        _reader_id: EntityId,
        _writer_id: EntityId,
        _writer_sn: SequenceNumber,
        _fragment_starting_num: FragmentNumber,
        _fragments_in_submessage: UShort,
        _data_size: ULong,
        _fragment_size: UShort,
        _inline_qos: ParameterList,
        _serialized_payload: SerializedData,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn inline_qos_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn non_standard_payload_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn key_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn reader_id(&self) -> &EntityId {
        todo!()
    }

    fn writer_id(&self) -> &EntityId {
        todo!()
    }

    fn writer_sn(&self) -> &SequenceNumber {
        todo!()
    }

    fn fragment_starting_num(&self) -> &FragmentNumber {
        todo!()
    }

    fn fragments_in_submessage(&self) -> &UShort {
        todo!()
    }

    fn data_size(&self) -> &ULong {
        todo!()
    }

    fn fragment_size(&self) -> &UShort {
        todo!()
    }

    fn inline_qos(&self) -> &ParameterList {
        todo!()
    }

    fn serialized_payload(&self) -> &SerializedData<'a> {
        todo!()
    }
}

impl<'a> rust_rtps_pim::messages::Submessage for DataFrag<'a> {
    type RtpsSubmessageHeaderType = SubmessageHeader;
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}

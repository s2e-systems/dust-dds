use crate::{
    EntityId, FragmentNumber, ParameterList, RtpsUdpPsm, SequenceNumber, SerializedData,
    SubmessageFlag, ULong, UShort,
};

use super::SubmessageHeader;

pub struct DataFrag<'a> {
    pub serialized_data: SerializedData<'a>,
}

impl<'a> rust_rtps_pim::messages::submessages::DataFragSubmessage<'a, RtpsUdpPsm> for DataFrag<'a> {
    type EntityId = EntityId;
    type SequenceNumber = SequenceNumber;
    type FragmentNumber = FragmentNumber;
    type UShort = UShort;
    type ULong = ULong;
    type SerializedData = SerializedData<'a>;
    type ParameterList = ParameterList;

    fn new(
        _endianness_flag: SubmessageFlag,
        _inline_qos_flag: SubmessageFlag,
        _non_standard_payload_flag: SubmessageFlag,
        _key_flag: SubmessageFlag,
        _reader_id: Self::EntityId,
        _writer_id: Self::EntityId,
        _writer_sn: Self::SequenceNumber,
        _fragment_starting_num: Self::FragmentNumber,
        _fragments_in_submessage: Self::UShort,
        _data_size: Self::ULong,
        _fragment_size: Self::UShort,
        _inline_qos: Self::ParameterList,
        _serialized_payload: Self::SerializedData,
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

    fn reader_id(&self) -> &Self::EntityId {
        todo!()
    }

    fn writer_id(&self) -> &Self::EntityId {
        todo!()
    }

    fn writer_sn(&self) -> &Self::SequenceNumber {
        todo!()
    }

    fn fragment_starting_num(&self) -> &Self::FragmentNumber {
        todo!()
    }

    fn fragments_in_submessage(&self) -> &Self::UShort {
        todo!()
    }

    fn data_size(&self) -> &Self::ULong {
        todo!()
    }

    fn fragment_size(&self) -> &Self::UShort {
        todo!()
    }

    fn inline_qos(&self) -> &Self::ParameterList {
        todo!()
    }

    fn serialized_payload(&self) -> &Self::SerializedData {
        todo!()
    }
}

impl<'a> rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for DataFrag<'a> {
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}

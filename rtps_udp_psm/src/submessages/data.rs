use rust_rtps_pim::messages::Submessage;

use crate::RtpsUdpPsm;

pub struct Data<'a> {
    endianness_flag: <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    inline_qos_flag: <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    data_flag: <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    key_flag: <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    non_standard_payload_flag:
        <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    reader_id: rust_rtps_pim::messages::submessage_elements::EntityId<<Self as Submessage>::PSM>,
    writer_id: rust_rtps_pim::messages::submessage_elements::EntityId<<Self as Submessage>::PSM>,
    writer_sn:
        rust_rtps_pim::messages::submessage_elements::SequenceNumber<<Self as Submessage>::PSM>,
    inline_qos:
        rust_rtps_pim::messages::submessage_elements::ParameterList<<Self as Submessage>::PSM>,
    serialized_payload: rust_rtps_pim::messages::submessage_elements::SerializedData<&'a Vec<u8>>,
}

impl<'a> Submessage for Data<'a> {
    type PSM = RtpsUdpPsm;

    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<Self::PSM> {
        todo!()
    }
}

impl<'a> rust_rtps_pim::messages::submessages::Data for Data<'a> {
    type SerializedData = &'a Vec<u8>;

    fn new(
        _endianness_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        _inline_qos_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        _data_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        _key_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        _non_standard_payload_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        _reader_id: <<Self::PSM as rust_rtps_pim::structure::Types>::Guid as rust_rtps_pim::structure::types::Guid>::EntityId,
        _writer_id: <<Self::PSM as rust_rtps_pim::structure::Types>::Guid as rust_rtps_pim::structure::types::Guid>::EntityId,
        _writer_sn: <Self::PSM as rust_rtps_pim::structure::Types>::SequenceNumber,
        _inline_qos: <Self::PSM as rust_rtps_pim::structure::Types>::ParameterVector,
        _serialized_payload: Self::SerializedData,
    ) -> Self {
        todo!()
        // Self {
        //     endianness_flag,
        //     inline_qos_flag,
        //     data_flag,
        //     key_flag,
        //     non_standard_payload_flag,
        //     reader_id,
        //     writer_id,
        //     writer_sn,
        //     inline_qos,
        //     serialized_payload,
        // }
    }

    fn endianness_flag(&self) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.endianness_flag
    }

    fn inline_qos_flag(&self) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.inline_qos_flag
    }

    fn data_flag(&self) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.data_flag
    }

    fn key_flag(&self) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.key_flag
    }

    fn non_standard_payload_flag(
        &self,
    ) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.non_standard_payload_flag
    }

    fn reader_id(&self) -> &rust_rtps_pim::messages::submessage_elements::EntityId<Self::PSM> {
        &self.reader_id
    }

    fn writer_id(&self) -> &rust_rtps_pim::messages::submessage_elements::EntityId<Self::PSM> {
        &self.writer_id
    }

    fn writer_sn(
        &self,
    ) -> &rust_rtps_pim::messages::submessage_elements::SequenceNumber<Self::PSM> {
        &self.writer_sn
    }

    fn inline_qos(
        &self,
    ) -> &rust_rtps_pim::messages::submessage_elements::ParameterList<Self::PSM> {
        &self.inline_qos
    }

    fn serialized_payload(
        &self,
    ) -> &rust_rtps_pim::messages::submessage_elements::SerializedData<Self::SerializedData> {
        &self.serialized_payload
    }
}

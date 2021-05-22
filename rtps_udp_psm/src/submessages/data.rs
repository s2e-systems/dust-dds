use rust_rtps_pim::messages::Submessage;

use crate::RtpsUdpPsm;

pub struct Data {}

impl rust_rtps_pim::messages::submessages::Data<RtpsUdpPsm> for Data {
    fn new(
        _endianness_flag: <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag,
        _inline_qos_flag: <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag,
        _data_flag: <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag,
        _key_flag: <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag,
        _non_standard_payload_flag: <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag,
        _reader_id: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::EntityId,
        _writer_id: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::EntityId,
        _writer_sn: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::SequenceNumber,
        _serialized_payload: &<RtpsUdpPsm as rust_rtps_pim::structure::Types>::Data,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn inline_qos_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn data_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn key_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn non_standard_payload_flag(
        &self,
    ) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn reader_id(&self) -> rust_rtps_pim::messages::submessage_elements::EntityId<RtpsUdpPsm> {
        todo!()
    }

    fn writer_id(&self) -> rust_rtps_pim::messages::submessage_elements::EntityId<RtpsUdpPsm> {
        todo!()
    }

    fn writer_sn(
        &self,
    ) -> rust_rtps_pim::messages::submessage_elements::SequenceNumber<RtpsUdpPsm> {
        todo!()
    }

    fn serialized_payload(&self) -> &<RtpsUdpPsm as rust_rtps_pim::structure::Types>::Data {
        todo!()
    }
}

impl Submessage<RtpsUdpPsm> for Data {
    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<RtpsUdpPsm> {
        todo!()
    }
}

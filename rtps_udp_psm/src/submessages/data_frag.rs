use crate::RtpsUdpPsm;

pub struct DataFrag;

impl rust_rtps_pim::messages::submessages::DataFrag<RtpsUdpPsm> for DataFrag {
    fn endianness_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn inline_qos_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn non_standard_payload_flag(
        &self,
    ) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn key_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
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

    fn fragment_starting_num(
        &self,
    ) -> rust_rtps_pim::messages::submessage_elements::FragmentNumber<RtpsUdpPsm> {
        todo!()
    }

    fn fragments_in_submessage(&self) -> rust_rtps_pim::messages::submessage_elements::UShort {
        todo!()
    }

    fn data_size(&self) -> rust_rtps_pim::messages::submessage_elements::ULong {
        todo!()
    }

    fn fragment_size(&self) -> rust_rtps_pim::messages::submessage_elements::UShort {
        todo!()
    }

    fn inline_qos(
        &self,
    ) -> rust_rtps_pim::messages::submessage_elements::ParameterList<RtpsUdpPsm> {
        todo!()
    }

    fn serialized_payload(&self) -> &<RtpsUdpPsm as rust_rtps_pim::structure::Types>::Data {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for DataFrag {
    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<RtpsUdpPsm> {
        todo!()
    }
}

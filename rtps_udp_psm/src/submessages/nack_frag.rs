use rust_rtps_pim::messages::Submessage;

use crate::RtpsUdpPsm;

pub struct NackFrag {
    endianness_flag: <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    reader_id: rust_rtps_pim::messages::submessage_elements::EntityId<<Self as Submessage>::PSM>,
    writer_id: rust_rtps_pim::messages::submessage_elements::EntityId<<Self as Submessage>::PSM>,
    writer_sn:
        rust_rtps_pim::messages::submessage_elements::SequenceNumber<<Self as Submessage>::PSM>,
    fragment_number_state:
        rust_rtps_pim::messages::submessage_elements::FragmentNumberSet<<Self as Submessage>::PSM>,
    count: rust_rtps_pim::messages::submessage_elements::Count<<Self as Submessage>::PSM>,
}

impl Submessage for NackFrag {
    type PSM = RtpsUdpPsm;

    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<Self::PSM> {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessages::NackFrag for NackFrag {
    fn new(
        endianness_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        reader_id: rust_rtps_pim::messages::submessage_elements::EntityId<Self::PSM>,
        writer_id: rust_rtps_pim::messages::submessage_elements::EntityId<Self::PSM>,
        writer_sn: rust_rtps_pim::messages::submessage_elements::SequenceNumber<Self::PSM>,
        fragment_number_state: rust_rtps_pim::messages::submessage_elements::FragmentNumberSet<
            Self::PSM,
        >,
        count: rust_rtps_pim::messages::submessage_elements::Count<Self::PSM>,
    ) -> Self {
        Self {
            endianness_flag,
            reader_id,
            writer_id,
            writer_sn,
            fragment_number_state,
            count,
        }
    }

    fn endianness_flag(&self) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.endianness_flag
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

    fn fragment_number_state(
        &self,
    ) -> &rust_rtps_pim::messages::submessage_elements::FragmentNumberSet<Self::PSM> {
        &self.fragment_number_state
    }

    fn count(&self) -> &rust_rtps_pim::messages::submessage_elements::Count<Self::PSM> {
        &self.count
    }
}

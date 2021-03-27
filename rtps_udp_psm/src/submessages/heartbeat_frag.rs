use rust_rtps_pim::messages::Submessage;

use crate::RtpsUdpPsm;

pub struct HeartbeatFrag {
    endianness_flag: <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    reader_id: rust_rtps_pim::messages::submessage_elements::EntityId<<Self as Submessage>::PSM>,
    writer_id: rust_rtps_pim::messages::submessage_elements::EntityId<<Self as Submessage>::PSM>,
    writer_sn:
        rust_rtps_pim::messages::submessage_elements::SequenceNumber<<Self as Submessage>::PSM>,
    last_fragment_num:
        rust_rtps_pim::messages::submessage_elements::FragmentNumber<<Self as Submessage>::PSM>,
    count: rust_rtps_pim::messages::submessage_elements::Count<<Self as Submessage>::PSM>,
}

impl Submessage for HeartbeatFrag {
    type PSM = RtpsUdpPsm;

    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<Self::PSM> {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessages::HeartbeatFrag for HeartbeatFrag {
    fn new(
        endianness_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        reader_id: rust_rtps_pim::messages::submessage_elements::EntityId<Self::PSM>,
        writer_id: rust_rtps_pim::messages::submessage_elements::EntityId<Self::PSM>,
        writer_sn: rust_rtps_pim::messages::submessage_elements::SequenceNumber<Self::PSM>,
        last_fragment_num: rust_rtps_pim::messages::submessage_elements::FragmentNumber<Self::PSM>,
        count: rust_rtps_pim::messages::submessage_elements::Count<Self::PSM>,
    ) -> Self {
        Self {
            endianness_flag,
            reader_id,
            writer_id,
            writer_sn,
            last_fragment_num,
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

    fn last_fragment_num(
        &self,
    ) -> &rust_rtps_pim::messages::submessage_elements::FragmentNumber<Self::PSM> {
        &self.last_fragment_num
    }

    fn count(&self) -> &rust_rtps_pim::messages::submessage_elements::Count<Self::PSM> {
        &self.count
    }
}

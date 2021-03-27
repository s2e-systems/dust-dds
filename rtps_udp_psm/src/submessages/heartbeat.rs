use rust_rtps_pim::messages::Submessage;

use crate::RtpsUdpPsm;

pub struct Heartbeat {
    endianness_flag: <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    final_flag: <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    liveliness_flag: <<Self as Submessage>::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
    reader_id: rust_rtps_pim::messages::submessage_elements::EntityId<<Self as Submessage>::PSM>,
    writer_id: rust_rtps_pim::messages::submessage_elements::EntityId<<Self as Submessage>::PSM>,
    first_sn:
        rust_rtps_pim::messages::submessage_elements::SequenceNumber<<Self as Submessage>::PSM>,
    last_sn:
        rust_rtps_pim::messages::submessage_elements::SequenceNumber<<Self as Submessage>::PSM>,
    count: rust_rtps_pim::messages::submessage_elements::Count<<Self as Submessage>::PSM>,
}

impl Submessage for Heartbeat {
    type PSM = RtpsUdpPsm;

    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<Self::PSM> {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessages::Heartbeat for Heartbeat {
    fn new(
        endianness_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        final_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        liveliness_flag: <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag,
        reader_id: rust_rtps_pim::messages::submessage_elements::EntityId<Self::PSM>,
        writer_id: rust_rtps_pim::messages::submessage_elements::EntityId<Self::PSM>,
        first_sn: rust_rtps_pim::messages::submessage_elements::SequenceNumber<Self::PSM>,
        last_sn: rust_rtps_pim::messages::submessage_elements::SequenceNumber<Self::PSM>,
        count: rust_rtps_pim::messages::submessage_elements::Count<Self::PSM>,
    ) -> Self {
        Self {
            endianness_flag,
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        }
    }

    fn endianness_flag(&self) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.endianness_flag
    }

    fn final_flag(&self) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.final_flag
    }

    fn liveliness_flag(&self) -> <Self::PSM as rust_rtps_pim::messages::Types>::SubmessageFlag {
        self.liveliness_flag
    }

    fn reader_id(&self) -> &rust_rtps_pim::messages::submessage_elements::EntityId<Self::PSM> {
        &self.reader_id
    }

    fn writer_id(&self) -> &rust_rtps_pim::messages::submessage_elements::EntityId<Self::PSM> {
        &self.writer_id
    }

    fn first_sn(&self) -> &rust_rtps_pim::messages::submessage_elements::SequenceNumber<Self::PSM> {
        &self.first_sn
    }

    fn last_sn(&self) -> &rust_rtps_pim::messages::submessage_elements::SequenceNumber<Self::PSM> {
        &self.last_sn
    }

    fn count(&self) -> &rust_rtps_pim::messages::submessage_elements::Count<Self::PSM> {
        &self.count
    }
}

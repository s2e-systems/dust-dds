use rust_rtps_pim::messages::Submessage;

use crate::{EntityId, RtpsUdpPsm, SequenceNumber, SerializedData, SubmessageFlag};

use super::SubmessageHeader;

pub struct Data<'a> {
    pub serialized_data: SerializedData<'a>,
}

impl<'a> rust_rtps_pim::messages::submessages::DataSubmessage<RtpsUdpPsm> for Data<'a> {
    type EntityId = EntityId;
    type SequenceNumber = SequenceNumber;
    type SerializedData = SerializedData<'a>;

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn inline_qos_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn data_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn key_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn non_standard_payload_flag(&self) -> SubmessageFlag {
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

    fn serialized_payload(&self) -> &Self::SerializedData {
        todo!()
    }
}

impl<'a> Submessage<RtpsUdpPsm> for Data<'a> {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }
}

use crate::messages::{submessage_elements, types::SubmessageFlag};
use serde::ser::SerializeStruct;

use super::SubmessageHeader;

struct AckNack {
    submessage_header: SubmessageHeader,
    reader_id: submessage_elements::EntityId,
    writer_id: submessage_elements::EntityId,
    reader_sn_state: submessage_elements::SequenceNumberSet,
    count: submessage_elements::Count,
}

impl rust_rtps_pim::messages::submessages::ack_nack_submessage::AckNack for AckNack {
    type EntityId = submessage_elements::EntityId;
    type SequenceNumberSet = submessage_elements::SequenceNumberSet;
    type Count = submessage_elements::Count;

    fn endianness_flag(&self) -> SubmessageFlag {
        rust_rtps_pim::messages::submessages::SubmessageHeader::flags(&self.submessage_header)[0]
    }

    fn final_flag(&self) -> SubmessageFlag {
        rust_rtps_pim::messages::submessages::SubmessageHeader::flags(&self.submessage_header)[1]
    }

    fn reader_id(&self) -> &Self::EntityId {
        &self.reader_id
    }

    fn writer_id(&self) -> &Self::EntityId {
        &self.writer_id
    }

    fn reader_sn_state(&self) -> &Self::SequenceNumberSet {
        &self.reader_sn_state
    }

    fn count(&self) -> &Self::Count {
        &self.count
    }
}

impl rust_rtps_pim::messages::submessages::Submessage for AckNack {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        self.submessage_header
    }
}

impl serde::Serialize for AckNack {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // let value = 10i32;
        let state = serializer.serialize_struct("AckNack", 3)?;
        // // state.serialize_field("reader_id", &self.0.reader_id.value)?;
        // if endianness_flag == true {
        //     state.serialize_field("sequence_number", &value.to_be_bytes())?;
        // } else {
        //     state.serialize_field("sequence_number", &value.to_le_bytes())?;
        // }
        // state.serialize_field("key", &self.0.reader_sn_state)?;
        state.end()
    }
}

// pub struct AckNack {
//     endianness_flag: SubmessageFlag,
//     final_flag: SubmessageFlag,
//     reader_id: <Self as rust_rtps_pim::messages::submessages::ack_nack_submessage::AckNack>::EntityId,
//     writer_id: <Self as rust_rtps_pim::messages::submessages::ack_nack_submessage::AckNack>::EntityId,
//     sequence_number_set: <Self as rust_rtps_pim::messages::submessages::ack_nack_submessage::AckNack>::SequenceNumberSet,
//     count: <Self as rust_rtps_pim::messages::submessages::ack_nack_submessage::AckNack>::Count,
// }

// impl Submessage for AckNack {
//     type SubmessageHeader = SubmessageHeader;

//     fn submessage_header(&self) -> Self::SubmessageHeader {
//         todo!()
//     }

//     fn is_valid(&self) -> bool {
//         todo!()
//     }
// }

// impl rust_rtps_pim::messages::submessages::ack_nack_submessage::AckNack for AckNack {
//     type EntityId = submessage_elements::EntityId;
//     type SequenceNumberSet = submessage_elements::SequenceNumberSet;
//     type Count = submessage_elements::Count;

//     fn endianness_flag(&self) -> SubmessageFlag {
//         self.endianness_flag
//     }

//     fn final_flag(&self) -> SubmessageFlag {
//         self.final_flag
//     }

//     fn reader_id(&self) -> &Self::EntityId {
//         &self.reader_id
//     }

//     fn writer_id(&self) -> &Self::EntityId {
//         &self.writer_id
//     }

//     fn reader_sn_state(&self) -> &Self::SequenceNumberSet {
//         &self.sequence_number_set
//     }

//     fn count(&self) -> &Self::Count {
//         &self.count
//     }
// }

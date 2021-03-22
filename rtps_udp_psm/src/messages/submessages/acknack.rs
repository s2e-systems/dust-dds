use serde::ser::SerializeStruct;

use crate::{
    messages::types::{Count, SubmessageFlag, SubmessageKind},
    types::{EntityId, SequenceNumber},
};

// use super::SubmessageHeader;
// use crate::messages::submessage_elements;
// use rust_rtps_pim::messages::{submessages::Submessage, types::SubmessageFlag};

struct AckNack(
    rust_rtps_pim::messages::submessages::ack_nack_submessage::AckNack<
        SubmessageKind,
        SubmessageFlag,
        EntityId,
        SequenceNumber,
        Vec<SequenceNumber>,
        Count,
    >,
);

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

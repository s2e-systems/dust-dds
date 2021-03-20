use super::submessage_elements;
use super::Submessage;
use super::SubmessageFlag;

pub trait AckNack: Submessage {
    type EntityId: submessage_elements::EntityId;
    type SequenceNumberSet: submessage_elements::SequenceNumberSet;
    type Count: submessage_elements::Count;

    fn endianness_flag(&self) -> SubmessageFlag;
    fn final_flag(&self) -> SubmessageFlag;
    fn reader_id(&self) -> &Self::EntityId;
    fn writer_id(&self) -> &Self::EntityId;
    fn reader_sn_state(&self) -> &Self::SequenceNumberSet;
    fn count(&self) -> &Self::Count;
}

// impl Submessage for AckNack {
//     fn submessage_header(&self) -> SubmessageHeader {
//         const X: SubmessageFlag = false;
//         let e = self.endianness_flag;
//         let f = self.final_flag;
//         let flags = [e, f, X, X, X, X, X, X];

//         SubmessageHeader::new(constants::SUBMESSAGE_KIND_ACK_NACK, flags, 0)
//     }

//     fn is_valid(&self) -> bool {
//         todo!()
//         // self.reader_sn_state.is_valid()
//     }
// }

// impl serde::Serialize for AckNack {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let mut state = serializer.serialize_struct("AckNack", 4)?;
//         state.serialize_field("header", &self.submessage_header())?;
//         state.serialize_field("reader_id", &self.reader_id)?;
//         state.serialize_field("writer_id", &self.writer_id)?;
//         state.serialize_field("reader_sn_state", &self.reader_sn_state)?;
//         state.serialize_field("count", &self.count)?;
//         state.end()
//     }
// }

// #[cfg(test)]
// mod tests {
//     use serde_test::{assert_ser_tokens, Token};

//     use crate::{
//         messages::submessages::submessage_elements::SequenceNumberSet,
//         types::constants::ENTITYID_PARTICIPANT,
//     };

//     use super::*;

//     #[test]
//     fn serialize() {
//         assert_ser_tokens(
//             &AckNack {
//                 endianness_flag: false,
//                 final_flag: true,
//                 reader_id: ENTITYID_PARTICIPANT,
//                 writer_id: ENTITYID_PARTICIPANT,
//                 reader_sn_state: SequenceNumberSet::new(10.into(), 5, [10; 8]),
//                 count: 10,
//             },
//             &[
//                 Token::Struct {
//                     name: "AckNack",
//                     len: 4,
//                 },
//                 Token::StructEnd,
//             ],
//         )
//     }
// }

// use std::io::Write;
// use byteorder::ByteOrder;

// use crate::{serialize::Serialize, submessage_elements::{CountUdp, EntityIdUdp, SequenceNumberSetUdp}};
// use rust_rtps_pim::messages::{types::SubmessageFlag, RtpsSubmessageHeader, Submessage};

// #[derive(Debug, PartialEq)]


// impl Serialize for AckNack {
//     fn serialize<W: Write, B: ByteOrder>(&self, mut _writer: W) -> crate::serialize::Result {
//         todo!()
//     }
// }
// impl<'de> crate::deserialize::Deserialize<'de> for AckNackUdp {
//     fn deserialize<B>(_buf: &mut &'de[u8]) -> crate::deserialize::Result<Self> where B: ByteOrder {
//         todo!()
//     }
// }

// impl<'a> rust_rtps_pim::messages::submessages::AckNackSubmessage for AckNackUdp {
//     type EntityIdSubmessageElementType = EntityIdUdp;
//     type SequenceNumberSetSubmessageElementType = SequenceNumberSetUdp;
//     type CountSubmessageElementType = CountUdp;

//     fn new(
//         _endianness_flag: SubmessageFlag,
//         _final_flag: SubmessageFlag,
//         _reader_id: EntityIdUdp,
//         _writer_id: EntityIdUdp,
//         _reader_sn_state: SequenceNumberSetUdp,
//         _count: CountUdp,
//     ) -> Self {
//         todo!()
//     }

//     fn endianness_flag(&self) -> SubmessageFlag {
//         todo!()
//     }

//     fn final_flag(&self) -> SubmessageFlag {
//         todo!()
//     }

//     fn reader_id(&self) -> &EntityIdUdp {
//         todo!()
//     }

//     fn writer_id(&self) -> &EntityIdUdp {
//         todo!()
//     }

//     fn reader_sn_state(&self) -> &SequenceNumberSetUdp {
//         todo!()
//     }

//     fn count(&self) -> &CountUdp {
//         todo!()
//     }
// }

// impl Submessage for AckNackUdp {
//     fn submessage_header(&self) -> RtpsSubmessageHeader {
//         todo!()
//     }
// }

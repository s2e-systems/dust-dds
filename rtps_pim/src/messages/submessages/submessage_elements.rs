///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///
use crate::{messages, types};

pub trait SubmessageElement {}

pub type Long = i32;
pub type ULong = u32;
pub type Short = i16;
pub type UShort = u16;

pub trait GuidPrefix: SubmessageElement {
    fn value(&self) -> types::GuidPrefix;
}

pub trait EntityId: SubmessageElement {
    fn value(&self) -> types::EntityId;
}

pub trait VendorId: SubmessageElement {
    fn value(&self) -> types::VendorId;
}

pub trait ProtocolVersion: SubmessageElement {
    fn value(&self) -> types::ProtocolVersion;
}

pub trait SequenceNumber: SubmessageElement {
    fn value(&self) -> types::SequenceNumber;
}

pub trait SequenceNumberSet: SubmessageElement {
    type SequenceNumberList: IntoIterator<Item = types::SequenceNumber>;

    fn base(&self) -> types::SequenceNumber;

    fn set(&self) -> Self::SequenceNumberList;
}

pub trait FragmentNumber: SubmessageElement {
    fn value(&self) -> messages::types::FragmentNumber;
}

pub trait FragmentNumberSet: SubmessageElement {
    type FragmentNumberList: IntoIterator<Item = messages::types::FragmentNumber>;

    fn base(&self) -> messages::types::FragmentNumber;

    fn set(&self) -> Self::FragmentNumberList;
}

pub trait Timestamp: SubmessageElement {
    fn value(&self) -> messages::types::Time;
}

pub trait Parameter {
    fn parameter_id(&self) -> messages::types::ParameterId;
    fn length(&self) -> i16;
    fn value(&self) -> &[u8];
}

pub trait ParameterList: SubmessageElement {
    type Parameter: AsRef<dyn Parameter>;
    type ParameterList: IntoIterator<Item = Self::Parameter>;
    fn parameter(&self) -> &Self::ParameterList;
}

pub trait Count: SubmessageElement {
    fn value(&self) -> messages::types::Count;
}

pub trait LocatorList: SubmessageElement {
    type LocatorList: IntoIterator<Item = types::Locator>;
    fn value(&self) -> Self::LocatorList;
}

pub trait SerializedData: SubmessageElement {
    type SerializedData: IntoIterator<Item = u8>;
    fn value(&self) -> Self::SerializedData;
}

pub trait SerializedDataFragment: SubmessageElement {
    type SerializedData: IntoIterator<Item = u8>;
    fn value(&self) -> Self::SerializedData;
}

// pub type GroupDigest = TBD

// #[cfg(test)]
// mod tests {
//     use serde_test::{assert_ser_tokens, Token};

//     use super::*;

//     #[test]
//     fn serialize_sequence_number_set() {
//         assert_ser_tokens(
//             &SequenceNumberSet::new(8.into(), 16, [8, 0, 0, 0, 0, 0, 0, 0]),
//             &[
//                 Token::Struct {
//                     name: "SequenceNumberSet",
//                     len: 3,
//                 },
//                 Token::Str("bitmap_base"),
//                 Token::Struct {
//                     name: "SequenceNumber",
//                     len: 2,
//                 },
//                 Token::Str("high"),
//                 Token::I32(0),
//                 Token::Str("low"),
//                 Token::U32(8),
//                 Token::StructEnd,
//                 Token::Str("num_bits"),
//                 Token::U32(16),
//                 Token::Str("bitmap"),
//                 Token::I32(8),
//                 Token::StructEnd,
//             ],
//         );

//         assert_ser_tokens(
//             &SequenceNumberSet::new(8.into(), 128, [8, 9, 10, 11, 0, 0, 0, 0]),
//             &[
//                 Token::Struct {
//                     name: "SequenceNumberSet",
//                     len: 3,
//                 },
//                 Token::Str("bitmap_base"),
//                 Token::Struct {
//                     name: "SequenceNumber",
//                     len: 2,
//                 },
//                 Token::Str("high"),
//                 Token::I32(0),
//                 Token::Str("low"),
//                 Token::U32(8),
//                 Token::StructEnd,
//                 Token::Str("num_bits"),
//                 Token::U32(128),
//                 Token::Str("bitmap"),
//                 Token::I32(8),
//                 Token::Str("bitmap"),
//                 Token::I32(9),
//                 Token::Str("bitmap"),
//                 Token::I32(10),
//                 Token::Str("bitmap"),
//                 Token::I32(11),
//                 Token::StructEnd,
//             ],
//         )
//     }
// }

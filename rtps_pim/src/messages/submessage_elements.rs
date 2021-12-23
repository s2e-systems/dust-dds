///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///
pub trait UShortSubmessageElementConstructor {
    fn new(value: &u16) -> Self;
}

pub trait UShortSubmessageElementAttributes {
    fn value(&self) -> &u16;
}

pub trait ShortSubmessageElementConstructor {
    fn new(value: &i16) -> Self;
}

pub trait ShortSubmessageElementAttributes {
    fn value(&self) -> &i16;
}

pub trait ULongSubmessageElementConstructor {
    fn new(value: &u32) -> Self;
}

pub trait ULongSubmessageElementAttributes {
    fn value(&self) -> &u32;
}

pub trait LongSubmessageElementConstructor {
    fn new(value: &i32) -> Self;
}

pub trait LongSubmessageElementAttributes {
    fn value(&self) -> &i32;
}

pub trait GuidPrefixSubmessageElementConstructor {
    type GuidPrefixType: ?Sized;
    fn new(value: &Self::GuidPrefixType) -> Self;
}

pub trait GuidPrefixSubmessageElementAttributes {
    type GuidPrefixType: ?Sized;
    fn value(&self) -> &Self::GuidPrefixType;
}

pub trait EntityIdSubmessageElementConstructor {
    type EntityIdType: ?Sized;
    fn new(value: &Self::EntityIdType) -> Self;
}

pub trait EntityIdSubmessageElementAttributes {
    type EntityIdType: ?Sized;
    fn value(&self) -> &Self::EntityIdType;
}

pub trait VendorIdSubmessageElementConstructor {
    type VendorIdType: ?Sized;
    fn new(value: &Self::VendorIdType) -> Self;
}

pub trait VendorIdSubmessageElementAttributes {
    type VendorIdType: ?Sized;
    fn value(&self) -> &Self::VendorIdType;
}

pub trait ProtocolVersionSubmessageElementConstructor {
    type ProtocolVersionType: ?Sized;
    fn new(value: &Self::ProtocolVersionType) -> Self;
}

pub trait ProtocolVersionSubmessageElementAttributes {
    type ProtocolVersionType: ?Sized;
    fn value(&self) -> &Self::ProtocolVersionType;
}

pub trait SequenceNumberSubmessageElementConstructor {
    type SequenceNumberType: ?Sized;
    fn new(value: &Self::SequenceNumberType) -> Self;
}

pub trait SequenceNumberSubmessageElementAttributes {
    type SequenceNumberType: ?Sized;
    fn value(&self) -> &Self::SequenceNumberType;
}

pub trait SequenceNumberSetSubmessageElementConstructor {
    type SequenceNumberType: ?Sized;
    type SequenceNumberSetType: ?Sized;
    fn new(base: &Self::SequenceNumberType, set: &Self::SequenceNumberSetType) -> Self;
}

pub trait SequenceNumberSetSubmessageElementAttributes {
    type SequenceNumberType: ?Sized;
    type SequenceNumberSetType: ?Sized;
    fn base(&self) -> &Self::SequenceNumberType;
    fn set(&self) -> &Self::SequenceNumberSetType;
}

pub trait FragmentNumberSubmessageElementConstructor {
    type FragmentNumberType: ?Sized;
    fn new(value: &Self::FragmentNumberType) -> Self;
}

pub trait FragmentNumberSubmessageElementAttributes {
    type FragmentNumberType: ?Sized;
    fn new(&self) -> &Self::FragmentNumberType;
}

pub trait FragmentNumberSetSubmessageElementConstructor {
    type FragmentNumberType: ?Sized;
    type FragmentNumberSetType: ?Sized;

    fn new(base: &Self::FragmentNumberType, set: &Self::FragmentNumberSetType) -> Self;
}

pub trait FragmentNumberSetSubmessageElementAttributes {
    type FragmentNumberType: ?Sized;
    type FragmentNumberSetType: ?Sized;

    fn base(&self) -> &Self::FragmentNumberType;
    fn set(&self) -> &Self::FragmentNumberSetType;
}

pub trait TimestampSubmessageElementConstructor {
    type TimeType: ?Sized;
    fn new(value: &Self::TimeType) -> Self;
}

pub trait TimestampSubmessageElementAttributes {
    type TimeType: ?Sized;
    fn value(&self) -> &Self::TimeType;
}

pub trait ParameterConstructor {
    type ParameterIdType: ?Sized;
    type ParameterValueType: ?Sized;

    fn new(
        parameter_id: &Self::ParameterIdType,
        length: &i16,
        value: &Self::ParameterValueType,
    ) -> Self;
}

pub trait ParameterAttributes {
    type ParameterIdType: ?Sized;
    type ParameterValueType: ?Sized;

    fn parameter_id(&self) -> &Self::ParameterIdType;
    fn length(&self) -> &i16;
    fn value(&self) -> &Self::ParameterValueType;
}

pub trait ParameterListSubmessageElementConstructor {
    type ParameterListType: ?Sized;
    fn new(parameter: &Self::ParameterListType) -> Self;
}

pub trait ParameterListSubmessageElementAttributes {
    type ParameterListType: ?Sized;
    fn parameter(&self) -> &Self::ParameterListType;
}

pub trait CountSubmessageElementConstructor {
    type CountType: ?Sized;
    fn new(value: &Self::CountType) -> Self;
}

pub trait CountSubmessageElementAttributes {
    type CountType: ?Sized;
    fn value(&self) -> &Self::CountType;
}

pub trait LocatorListSubmessageElementConstructor {
    type LocatorListType: ?Sized;
    fn new(value: &Self::LocatorListType) -> Self;
}

pub trait LocatorListSubmessageElementAttributes {
    type LocatorListType: ?Sized;
    fn value(&self) -> &Self::LocatorListType;
}

pub trait SerializedDataSubmessageElementConstructor {
    type SerializedDataType: ?Sized;
    fn new(value: &Self::SerializedDataType) -> Self;
}

pub trait SerializedDataSubmessageElementAttributes {
    type SerializedDataType: ?Sized;
    fn value(&self) -> &Self::SerializedDataType;
}

pub trait SerializedDataFragmentSubmessageElementConstructor {
    type SerializedDataFragmentType: ?Sized;
    fn new(value: &Self::SerializedDataFragmentType) -> Self;
}

pub trait SerializedDataFragmentSubmessageElementAttributes {
    type SerializedDataFragmentType: ?Sized;
    fn value(&self) -> &Self::SerializedDataFragmentType;
}

pub trait GroupDigestSubmessageElementConstructor {
    type GroupDigestType: ?Sized;
    fn new(value: &Self::GroupDigestType) -> Self;
}

pub trait GroupDigestSubmessageElementAttributes {
    type GroupDigestType: ?Sized;
    fn value(&self) -> &Self::GroupDigestType;
}

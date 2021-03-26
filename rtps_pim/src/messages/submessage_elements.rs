///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///
use crate::{messages, structure, RtpsPim};

pub struct UShort {
    pub value: u16,
}

pub struct Short {
    pub value: i16,
}

pub struct ULong {
    pub value: u32,
}

pub struct Long {
    pub value: i32,
}

pub struct GuidPrefix<PIM: RtpsPim> {
    pub value: <PIM as structure::Types>::GuidPrefix,
}

pub struct EntityId<PIM: RtpsPim> {
    pub value: <PIM as structure::Types>::EntityId,
}

pub struct VendorId<PIM: RtpsPim> {
    pub value: <PIM as structure::Types>::VendorId,
}

pub struct ProtocolVersion<PIM: RtpsPim> {
    pub value: <PIM as structure::Types>::ProtocolVersion,
}

pub struct SequenceNumber<PIM: RtpsPim> {
    pub value: <PIM as structure::Types>::SequenceNumber,
}

pub struct SequenceNumberSet<
    PIM: RtpsPim,
    SequenceNumberList: IntoIterator<Item = <PIM as structure::Types>::SequenceNumber>,
> {
    pub base: <PIM as structure::Types>::SequenceNumber,
    pub set: SequenceNumberList,
}

pub struct FragmentNumber<PIM: RtpsPim> {
    pub value: <PIM as messages::Types>::FragmentNumber,
}

pub struct FragmentNumberSet<
    PIM: RtpsPim,
    FragmentNumberList: IntoIterator<Item = <PIM as messages::Types>::FragmentNumber>,
> {
    pub base: <PIM as messages::Types>::FragmentNumber,
    pub set: FragmentNumberList,
}

pub struct Timestamp<PIM: RtpsPim> {
    pub value: <PIM as messages::Types>::Time,
}

pub struct Parameter<PIM: RtpsPim, Value: AsRef<[u8]>> {
    pub parameter_id: <PIM as messages::Types>::ParameterId,
    pub length: i16,
    pub value: Value,
}

pub struct ParameterList<
    PIM: RtpsPim,
    Value: AsRef<[u8]> + Clone,
    ParameterList: IntoIterator<Item = Parameter<PIM, Value>> + Clone,
> {
    pub parameter: ParameterList,
}

pub struct Count<PIM: RtpsPim> {
    pub value: <PIM as messages::Types>::Count,
}

pub struct LocatorList<
    PIM: RtpsPim,
    LocatorList: IntoIterator<Item = <PIM as structure::Types>::Locator>,
> {
    pub phantom: core::marker::PhantomData<<PIM as structure::Types>::Locator>,
    pub value: LocatorList,
}

pub struct SerializedData<SerializedData: AsRef<[u8]>> {
    pub value: SerializedData,
}

pub struct SerializedDataFragment<SerializedDataFragment: AsRef<[u8]>> {
    pub value: SerializedDataFragment,
}

pub struct GroupDigest<PIM: RtpsPim> {
    pub value: <PIM as messages::Types>::GroupDigest,
}

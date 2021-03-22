///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// 8.3.5 RTPS SubmessageElements
///
use crate::{messages, types};

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

pub struct GuidPrefix<GuidPrefix: types::GuidPrefix> {
    pub value: GuidPrefix,
}

pub struct EntityId<EntityId: types::EntityId> {
    pub value: EntityId,
}

pub struct VendorId<VendorId: types::VendorId> {
    pub value: VendorId,
}

impl<T: types::VendorId> VendorId<T> {
    pub const VENDORID_UNKNOWN: Self = Self {
        value: T::VENDOR_ID_UNKNOWN,
    };
}

pub struct ProtocolVersion<ProtocolVersion: types::ProtocolVersion> {
    pub value: ProtocolVersion,
}

impl<T: types::ProtocolVersion> ProtocolVersion<T> {
    pub const PROTOCOLVERSION_1_0: Self = Self {
        value: T::PROTOCOLVERSION_1_0,
    };
    pub const PROTOCOLVERSION_1_1: Self = Self {
        value: T::PROTOCOLVERSION_1_1,
    };
    pub const PROTOCOLVERSION_2_0: Self = Self {
        value: T::PROTOCOLVERSION_2_0,
    };
    pub const PROTOCOLVERSION_2_1: Self = Self {
        value: T::PROTOCOLVERSION_2_1,
    };
    pub const PROTOCOLVERSION_2_2: Self = Self {
        value: T::PROTOCOLVERSION_2_2,
    };
    pub const PROTOCOLVERSION_2_3: Self = Self {
        value: T::PROTOCOLVERSION_2_3,
    };
    pub const PROTOCOLVERSION_2_4: Self = Self {
        value: T::PROTOCOLVERSION_2_4,
    };
}

pub struct SequenceNumber<SequenceNumber: types::SequenceNumber> {
    pub value: SequenceNumber,
}

impl<T: types::SequenceNumber> SequenceNumber<T> {
    pub const SEQUENCENUMBER_UNKNOWN: Self = Self {
        value: T::SEQUENCE_NUMBER_UNKNOWN,
    };
}
pub struct SequenceNumberSet<
    SequenceNumber: types::SequenceNumber,
    SequenceNumberList: IntoIterator<Item = SequenceNumber>,
> {
    pub base: SequenceNumber,
    pub set: SequenceNumberList,
}

pub struct FragmentNumber<FragmentNumber: messages::types::FragmentNumber> {
    pub value: FragmentNumber,
}

pub struct FragmentNumberSet<
    FragmentNumber: messages::types::FragmentNumber,
    FragmentNumberList: IntoIterator<Item = FragmentNumber>,
> {
    pub base: FragmentNumber,
    pub set: FragmentNumberList,
}

pub struct Timestamp<Time: messages::types::Time> {
    pub value: Time,
}

impl<T: messages::types::Time> Timestamp<T> {
    pub const TIME_ZERO: Self = Self {
        value: T::TIME_ZERO,
    };
    pub const TIME_INVALID: Self = Self {
        value: T::TIME_INVALID,
    };
    pub const TIME_INFINITE: Self = Self {
        value: T::TIME_INFINITE,
    };
}
pub struct Parameter<
    ParameterId: messages::types::ParameterId,
    OctetList: IntoIterator<Item = [u8]>,
> {
    pub parameter_id: ParameterId,
    pub length: i16,
    pub value: OctetList,
}

pub struct ParameterList<
    ParameterId: messages::types::ParameterId,
    OctetList: IntoIterator<Item = [u8]>,
    ParameterList: IntoIterator<Item = Parameter<ParameterId, OctetList>>,
> {
    pub parameter: ParameterList,
}

pub struct Count<Count: messages::types::Count> {
    pub value: Count,
}

pub struct LocatorList<Locator: types::Locator, LocatorList: IntoIterator<Item = Locator>> {
    pub value: LocatorList,
}

pub struct SerializedData<OctetList: IntoIterator<Item = [u8]>> {
    pub value: OctetList,
}

pub struct SerializedDataFragment<OctetList: IntoIterator<Item = [u8]>> {
    pub value: OctetList,
}

pub struct GroupDigest<GroupDigest: messages::types::GroupDigest> {
    pub value :GroupDigest,
}

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

pub struct ProtocolVersion<ProtocolVersion: types::ProtocolVersion> {
    pub value: ProtocolVersion,
}

pub struct SequenceNumber<SequenceNumber: types::SequenceNumber> {
    pub value: SequenceNumber,
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

pub struct Parameter<ParameterId: messages::types::ParameterId, Value: AsRef<[u8]> + Clone> {
    pub parameter_id: ParameterId,
    pub length: i16,
    pub value: Value,
}

impl<ParameterId: messages::types::ParameterId, Value: AsRef<[u8]> + Clone> Clone
    for Parameter<ParameterId, Value>
{
    fn clone(&self) -> Self {
        Self {
            parameter_id: self.parameter_id.clone(),
            length: self.length,
            value: self.value.clone(),
        }
    }
}

pub struct ParameterList<
    ParameterId: messages::types::ParameterId,
    Value: AsRef<[u8]> + Clone,
    ParameterList: IntoIterator<Item = Parameter<ParameterId, Value>> + Clone,
> {
    pub parameter: ParameterList,
}

impl<
        ParameterId: messages::types::ParameterId,
        Value: AsRef<[u8]> + Clone,
        List: IntoIterator<Item = Parameter<ParameterId, Value>> + Clone,
    > Clone for ParameterList<ParameterId, Value, List>
{
    fn clone(&self) -> Self {
        Self {
            parameter: self.parameter.clone(),
        }
    }
}

pub struct Count<Count: messages::types::Count> {
    pub value: Count,
}

pub struct LocatorList<
    Locator: types::Locator,
    LocatorList: IntoIterator<Item = Locator>,
> {
    pub value: LocatorList,
}

pub struct SerializedData<SerializedData: AsRef<[u8]>> {
    pub value: SerializedData,
}

pub struct SerializedDataFragment<SerializedDataFragment: AsRef<[u8]>> {
    pub value: SerializedDataFragment,
}

pub struct GroupDigest<GroupDigest: messages::types::GroupDigest> {
    pub value: GroupDigest,
}

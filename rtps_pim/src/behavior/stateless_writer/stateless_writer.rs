use core::ops::{Deref, DerefMut};

use crate::{
    behavior::{types::Duration, RTPSWriter},
    messages::{submessages::submessage_elements::Parameter, types::ParameterId},
    structure::RTPSHistoryCache,
    types::{EntityId, GuidPrefix, InstanceHandle, Locator, SequenceNumber},
};

pub trait RTPSStatelessWriter:
    Deref<
        Target = RTPSWriter<
            Self::GuidPrefixType,
            Self::EntityIdType,
            Self::LocatorType,
            Self::LocatorListType,
            Self::DurationType,
            Self::SequenceNumberType,
            Self::InstanceHandleType,
            Self::DataType,
            Self::ParameterIdType,
            Self::ParameterValueType,
            Self::ParameterListType,
            Self::HistoryCacheType,
        >,
    > + DerefMut
{
    type GuidPrefixType: GuidPrefix;
    type EntityIdType: EntityId;
    type LocatorType: Locator;
    type LocatorListType: IntoIterator<Item = Self::LocatorType>;
    type DurationType: Duration;
    type SequenceNumberType: SequenceNumber;
    type InstanceHandleType: InstanceHandle;
    type DataType;
    type ParameterIdType: ParameterId;
    type ParameterValueType: AsRef<[u8]> + Clone;
    type ParameterListType: IntoIterator<Item = Parameter<Self::ParameterIdType, Self::ParameterValueType>>
        + Clone;
    type HistoryCacheType: RTPSHistoryCache<
        GuidPrefix = Self::GuidPrefixType,
        EntityId = Self::EntityIdType,
        InstanceHandle = Self::InstanceHandleType,
        SequenceNumber = Self::SequenceNumberType,
        Data = Self::DataType,
        ParameterId = Self::ParameterIdType,
        ParameterValue = Self::ParameterValueType,
        ParameterList = Self::ParameterListType,
    >;

    fn reader_locator_add(&mut self, a_locator: Self::LocatorType);
    fn reader_locator_remove(&mut self, a_locator: &Self::LocatorType);
    fn unsent_changes_reset(&mut self);
}

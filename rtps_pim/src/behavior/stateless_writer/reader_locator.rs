use crate::{
    behavior::{data_submessage_from_cache_change, types::Duration, RTPSWriter},
    messages::{
        submessages::{self, submessage_elements::Parameter},
        types::ParameterId,
    },
    structure::RTPSHistoryCache,
    types::{EntityId, GuidPrefix, InstanceHandle, Locator, SequenceNumber},
};

pub trait RTPSReaderLocator {
    type GuidPrefixType: GuidPrefix;
    type EntityIdType: EntityId;
    type LocatorType: Locator;
    type LocatorListType: IntoIterator<Item = Self::LocatorType>;
    type DurationType: Duration;
    type SequenceNumberType: SequenceNumber;
    type InstanceHandleType: InstanceHandle;
    type DataType: AsRef<[u8]>;
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
    type SequenceNumberListType: IntoIterator<Item = Self::SequenceNumberType>;

    fn requested_changes(&self) -> Self::SequenceNumberListType;
    fn unsent_changes(&self) -> Self::SequenceNumberListType;
    fn next_requested_change(&mut self) -> Option<Self::SequenceNumberType>;
    fn next_unsent_change(&mut self) -> Option<Self::SequenceNumberType>;
    fn requested_changes_set(
        &mut self,
        req_seq_num_set: &[Self::SequenceNumberType],
        writer: &RTPSWriter<
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
    );
}

impl<
        GuidPrefixType: GuidPrefix,
        EntityIdType: EntityId,
        LocatorType: Locator,
        LocatorListType: IntoIterator<Item = LocatorType>,
        DurationType: Duration,
        SequenceNumberType: SequenceNumber,
        InstanceHandleType: InstanceHandle,
        DataType: AsRef<[u8]>,
        ParameterIdType: ParameterId,
        ParameterValueType: AsRef<[u8]> + Clone,
        ParameterListType: IntoIterator<Item = Parameter<ParameterIdType, ParameterValueType>> + Clone,
        HistoryCacheType: RTPSHistoryCache<
            GuidPrefix = GuidPrefixType,
            EntityId = EntityIdType,
            InstanceHandle = InstanceHandleType,
            SequenceNumber = SequenceNumberType,
            Data = DataType,
            ParameterId = ParameterIdType,
            ParameterValue = ParameterValueType,
            ParameterList = ParameterListType,
        >,
        SequenceNumberListType: IntoIterator<Item = SequenceNumberType>,
    >
    dyn RTPSReaderLocator<
        GuidPrefixType = GuidPrefixType,
        EntityIdType = EntityIdType,
        LocatorType = LocatorType,
        LocatorListType = LocatorListType,
        DurationType = DurationType,
        SequenceNumberType = SequenceNumberType,
        InstanceHandleType = InstanceHandleType,
        DataType = DataType,
        ParameterIdType = ParameterIdType,
        ParameterValueType = ParameterValueType,
        ParameterListType = ParameterListType,
        HistoryCacheType = HistoryCacheType,
        SequenceNumberListType = SequenceNumberListType,
    >
{
    fn transition_t4<
        'a,
        DataSubmessage: submessages::data_submessage::Data<
            EntityId = EntityIdType,
            SequenceNumber = SequenceNumberType,
            SerializedData = &'a [u8],
        >,
    >(
        &mut self,
        the_writer: &'a RTPSWriter<
            GuidPrefixType,
            EntityIdType,
            LocatorType,
            LocatorListType,
            DurationType,
            SequenceNumberType,
            InstanceHandleType,
            DataType,
            ParameterIdType,
            ParameterValueType,
            ParameterListType,
            HistoryCacheType,
        >,
    ) -> Option<DataSubmessage> {
        if let Some(next_unsent_sequence_number) = self.next_unsent_change() {
            if let Some(next_unsent_cache_change) = the_writer
                .writer_cache
                .get_change(&next_unsent_sequence_number)
            {
                Some(data_submessage_from_cache_change(
                    next_unsent_cache_change,
                    <EntityIdType as EntityId>::ENTITYID_UNKNOWN,
                ))
            } else {
                todo!()
            }
        } else {
            None
        }
    }
}

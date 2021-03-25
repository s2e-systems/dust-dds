use crate::{
    messages::{
        submessages::submessage_elements::{Parameter, ParameterList},
        types::ParameterId,
    },
    structure::{RTPSCacheChange, RTPSEndpoint, RTPSEntity, RTPSHistoryCache},
    types::{
        ChangeKind, EntityId, GuidPrefix, InstanceHandle, Locator, ReliabilityKind, SequenceNumber,
        TopicKind, GUID,
    },
};

use super::types::Duration;

pub struct RTPSWriter<
    GuidPrefixType: GuidPrefix,
    EntityIdType: EntityId,
    LocatorType: Locator,
    LocatorListType: IntoIterator<Item = LocatorType>,
    DurationType: Duration,
    SequenceNumberType: SequenceNumber,
    InstanceHandleType: InstanceHandle,
    DataType,
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
> {
    pub endpoint: RTPSEndpoint<GuidPrefixType, EntityIdType, LocatorType, LocatorListType>,
    pub push_mode: bool,
    pub heartbeat_period: DurationType,
    pub nack_response_delay: DurationType,
    pub nack_suppression_duration: DurationType,
    pub last_change_sequence_number: SequenceNumberType,
    pub data_max_size_serialized: i32,
    pub writer_cache: HistoryCacheType,
}

impl<
        GuidPrefixType: GuidPrefix,
        EntityIdType: EntityId,
        LocatorType: Locator,
        LocatorListType: IntoIterator<Item = LocatorType>,
        DurationType: Duration,
        SequenceNumberType: SequenceNumber,
        InstanceHandleType: InstanceHandle,
        DataType,
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
    >
    RTPSWriter<
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
    >
{
    pub fn new(
        guid_prefix: GuidPrefixType,
        entity_id: EntityIdType,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: LocatorListType,
        multicast_locator_list: LocatorListType,
        push_mode: bool,
        heartbeat_period: DurationType,
        nack_response_delay: DurationType,
        nack_suppression_duration: DurationType,
        data_max_size_serialized: i32,
    ) -> Self {
        let entity = RTPSEntity {
            guid: GUID {
                guid_prefix,
                entity_id,
            },
        };
        let endpoint = RTPSEndpoint {
            entity,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
        };

        Self {
            endpoint,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_suppression_duration,
            last_change_sequence_number: 0.into(),
            data_max_size_serialized,
            writer_cache: HistoryCacheType::new(),
        }
    }

    pub fn new_change(
        &mut self,
        kind: ChangeKind,
        data: DataType,
        inline_qos: ParameterList<ParameterIdType, ParameterValueType, ParameterListType>,
        handle: InstanceHandleType,
    ) -> RTPSCacheChange<
        GuidPrefixType,
        EntityIdType,
        InstanceHandleType,
        SequenceNumberType,
        DataType,
        ParameterIdType,
        ParameterValueType,
        ParameterListType,
    > {
        self.last_change_sequence_number = (self.last_change_sequence_number.into() + 1).into();

        RTPSCacheChange {
            kind,
            writer_guid: self.endpoint.guid,
            instance_handle: handle,
            sequence_number: self.last_change_sequence_number,
            data_value: data,
            inline_qos,
        }
    }
}

// }

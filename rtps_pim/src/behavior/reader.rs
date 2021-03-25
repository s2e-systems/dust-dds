use crate::{
    messages::{submessages::submessage_elements::Parameter, types::ParameterId},
    structure::{RTPSEndpoint, RTPSEntity, RTPSHistoryCache},
    types::{
        EntityId, GuidPrefix, InstanceHandle, Locator, ReliabilityKind, SequenceNumber, TopicKind,
        GUID,
    },
};

use super::types::Duration;

pub struct RTPSReader<
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
    pub expects_inline_qos: bool,
    pub heartbeat_response_delay: DurationType,
    pub heartbeat_supression_duration: DurationType,
    pub reader_cache: HistoryCacheType,
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
    RTPSReader<
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
        expects_inline_qos: bool,
        heartbeat_response_delay: DurationType,
        heartbeat_supression_duration: DurationType,
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
            expects_inline_qos,
            heartbeat_response_delay,
            heartbeat_supression_duration,
            reader_cache: HistoryCacheType::new(),
        }
    }
}

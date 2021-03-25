use crate::{
    messages::{self, submessages::submessage_elements::Parameter},
    types,
};

use super::RTPSCacheChange;

pub trait RTPSHistoryCache {
    type GuidPrefix: types::GuidPrefix;
    type EntityId: types::EntityId;
    type InstanceHandle: types::InstanceHandle;
    type SequenceNumber: types::SequenceNumber;
    type Data;
    type ParameterId: messages::types::ParameterId;
    type ParameterValue: AsRef<[u8]>;
    type ParameterList: IntoIterator<Item = Parameter<Self::ParameterId, Self::ParameterValue>>;

    /// This operation creates a new RTPS HistoryCache. The newly-created history cache is initialized with an empty list of changes.
    fn new() -> Self;

    /// This operation inserts the CacheChange a_change into the HistoryCache.
    /// This operation will only fail if there are not enough resources to add the change to the HistoryCache. It is the responsibility
    /// of the DDS service implementation to configure the HistoryCache in a manner consistent with the DDS Entity RESOURCE_LIMITS QoS
    /// and to propagate any errors to the DDS-user in the manner specified by the DDS specification.
    fn add_change(
        &mut self,
        change: RTPSCacheChange<
            Self::GuidPrefix,
            Self::EntityId,
            Self::InstanceHandle,
            Self::SequenceNumber,
            Self::Data,
            Self::ParameterId,
            Self::ParameterValue,
            Self::ParameterList,
        >,
    );

    /// This operation indicates that a previously-added CacheChange has become irrelevant and the details regarding the CacheChange need
    /// not be maintained in the HistoryCache. The determination of irrelevance is made based on the QoS associated with the related DDS
    /// entity and on the acknowledgment status of the CacheChange. This is described in 8.4.1.
    fn remove_change(&mut self, seq_num: &Self::SequenceNumber);

    fn get_change(
        &self,
        seq_num: &Self::SequenceNumber,
    ) -> Option<
        &RTPSCacheChange<
            Self::GuidPrefix,
            Self::EntityId,
            Self::InstanceHandle,
            Self::SequenceNumber,
            Self::Data,
            Self::ParameterId,
            Self::ParameterValue,
            Self::ParameterList,
        >,
    >;

    /// This operation retrieves the smallest value of the CacheChange::sequenceNumber attribute among the CacheChange stored in the HistoryCache.
    fn get_seq_num_min(&self) -> Option<&Self::SequenceNumber>;

    /// This operation retrieves the largest value of the CacheChange::sequenceNumber attribute among the CacheChange stored in the HistoryCache.
    fn get_seq_num_max(&self) -> Option<&Self::SequenceNumber>;
}

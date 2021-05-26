use crate::{
    messages::types::ParameterIdType,
    structure::{
        types::{
            ChangeKind, DataType, EntityIdType, GuidPrefixType, InstanceHandleType, LocatorType,
            ParameterListType, SequenceNumberType,
        },
        RTPSCacheChange, RTPSEndpoint, RTPSHistoryCache,
    },
};

use super::types::DurationType;

pub trait RTPSWriter<
    PSM: GuidPrefixType
        + EntityIdType
        + LocatorType
        + DurationType
        + SequenceNumberType
        + DataType
        + ParameterIdType
        + ParameterListType<PSM>
        + InstanceHandleType,
>: RTPSEndpoint<PSM>
{
    fn push_mode(&self) -> bool;
    fn heartbeat_period(&self) -> PSM::Duration;
    fn nack_response_delay(&self) -> PSM::Duration;
    fn nack_suppression_duration(&self) -> PSM::Duration;
    fn last_change_sequence_number(&self) -> PSM::SequenceNumber;
    fn data_max_size_serialized(&self) -> i32;
    fn writer_cache(&self) -> &dyn RTPSHistoryCache<PSM>;
    fn writer_cache_mut(&mut self) -> &mut dyn RTPSHistoryCache<PSM>;

    fn new_change(
        &mut self,
        kind: ChangeKind,
        data: PSM::Data,
        inline_qos: PSM::ParameterList,
        handle: PSM::InstanceHandle,
    ) -> RTPSCacheChange<PSM>;
}

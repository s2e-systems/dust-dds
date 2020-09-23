use crate::types::{InstanceHandle, Data, Duration, ReliabilityKind, Time, ResourceLimits, HistoryKind, TopicKind};

pub trait WriterInterface{
    fn new(topic_kind: TopicKind, reliability: ReliabilityKind, max_blocking_time: Duration, history_kind: HistoryKind,  resource_limits: ResourceLimits ) -> Self;
    
    fn write(&self, instance_handle: InstanceHandle, data: Data, timestamp: Time);

    fn dispose(&self, instance_handle: InstanceHandle);

    fn unregister(&self, instance_handle: InstanceHandle);

    fn register(&self, instance_handle: InstanceHandle);
}

pub trait ReaderProtocolInterface {

}
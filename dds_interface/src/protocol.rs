use crate::types::{InstanceHandle, Data, Time, TopicKind, EntityType, ReturnCode};
use crate::qos::DataWriterQos;

pub trait WriterInterface{
    fn new(
        parent_instance_handle: InstanceHandle,
        entity_type: EntityType,
        topic_kind: TopicKind,
        writer_qos: DataWriterQos,
    ) -> Self;
    
    fn write(&self, instance_handle: InstanceHandle, data: Data, timestamp: Time) -> ReturnCode<()>;

    fn dispose(&self, instance_handle: InstanceHandle) -> ReturnCode<()>;

    fn unregister(&self, instance_handle: InstanceHandle) -> ReturnCode<()>;

    fn register(&self, instance_handle: InstanceHandle) -> ReturnCode<()>;
}

pub trait ReaderProtocolInterface {

}
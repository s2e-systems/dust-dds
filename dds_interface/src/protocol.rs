use std::sync::Weak;
use crate::types::{InstanceHandle, Data, Time, TopicKind, EntityType, ReturnCode};
use crate::qos::DataWriterQos;

pub trait ProtocolEntity : Send + Sync {
    fn enable(&self) -> ReturnCode<()>;
    fn get_instance_handle(&self) -> InstanceHandle;
}

pub trait ProtocolParticipant : ProtocolEntity {
    fn create_group(&self) -> Weak<dyn ProtocolGroup>;
}

pub trait ProtocolGroup : ProtocolEntity {
    fn create_writer(&self) -> Weak<dyn ProtocolWriter>;
    fn create_reader(&self) -> Weak<dyn ProtocolReader>;
}

pub trait ProtocolWriter : ProtocolEntity {
    // fn new(
    //     parent_instance_handle: InstanceHandle,
    //     entity_type: EntityType,
    //     topic_kind: TopicKind,
    //     writer_qos: DataWriterQos,
    // ) -> Self;
    
    fn write(&self, instance_handle: InstanceHandle, data: Data, timestamp: Time) -> ReturnCode<()>;

    fn dispose(&self, instance_handle: InstanceHandle, timestamp: Time) -> ReturnCode<()>;

    fn unregister(&self, instance_handle: InstanceHandle, timestamp: Time) -> ReturnCode<()>;

    fn register(&self, instance_handle: InstanceHandle, timestamp: Time) -> ReturnCode<()>;

    fn is_registered(&self, instance_handle: InstanceHandle) -> bool;
}

pub trait ProtocolReader: ProtocolEntity {

}
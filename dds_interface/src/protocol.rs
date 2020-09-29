use std::sync::{Arc, Weak, };
use crate::types::{InstanceHandle, Data, Time, ReturnCode};

pub trait ProtocolEntity : Send + Sync {
    fn enable(&self) -> ReturnCode<()>;
    fn get_instance_handle(&self) -> InstanceHandle;
}

pub trait ProtocolEndpoint : ProtocolEntity {}

pub trait ProtocolParticipant : ProtocolEntity {
    fn create_publisher(&self) -> Arc<dyn ProtocolPublisher>;
    fn create_subscriber(&self) -> Arc<dyn ProtocolSubscriber>;
    fn delete_publisher(&self, publisher: Weak<dyn ProtocolPublisher>);
    fn delete_subscriber(&self, subscriber: Weak<dyn ProtocolPublisher>);
}

pub trait ProtocolSubscriber : ProtocolEntity {
    fn create_reader(&self) -> Arc<dyn ProtocolReader>;
    fn delete_reader(&self, reader: Weak<dyn ProtocolReader>);
}
pub trait ProtocolPublisher : ProtocolEntity {
    fn create_writer(&self) -> Arc<dyn ProtocolWriter>;
    fn delete_writer(&self, writer: Weak<dyn ProtocolWriter>);
}

pub trait ProtocolWriter : ProtocolEndpoint {    
    fn write(&self, instance_handle: InstanceHandle, data: Data, timestamp: Time) -> ReturnCode<()>;

    fn dispose(&self, instance_handle: InstanceHandle, timestamp: Time) -> ReturnCode<()>;

    fn unregister(&self, instance_handle: InstanceHandle, timestamp: Time) -> ReturnCode<()>;

    fn register(&self, instance_handle: InstanceHandle, timestamp: Time) -> ReturnCode<Option<InstanceHandle>>;

    fn lookup_instance(&self, instance_handle: InstanceHandle) -> Option<InstanceHandle>;
}

pub trait ProtocolReader: ProtocolEndpoint {

}
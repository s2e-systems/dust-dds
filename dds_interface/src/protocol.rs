use crate::types::{InstanceHandle, Data, Time, ReturnCode, TopicKind};
use crate::qos::{DataWriterQos, DataReaderQos};

pub trait ProtocolEntity {
    fn enable(&self) -> ReturnCode<()>;
    fn get_instance_handle(&self) -> InstanceHandle;
}

pub trait ProtocolParticipant : ProtocolEntity + 'static {
    fn create_publisher(&mut self) -> Box<dyn ProtocolPublisher>;
    // fn delete_publisher(&mut self, publisher: &Arc<Mutex<dyn ProtocolPublisher>>);
    fn create_subscriber(&mut self) -> Box<dyn ProtocolSubscriber>;
    fn get_builtin_subscriber(&self) -> Box<dyn ProtocolSubscriber>;   
}

pub trait ProtocolSubscriber : ProtocolEntity {
    fn create_reader(&mut self, topic_kind: TopicKind, data_reader_qos: &DataReaderQos) -> Box<dyn ProtocolReader>;
}
pub trait ProtocolPublisher : ProtocolEntity {
    fn create_writer(&mut self, topic_kind: TopicKind, data_writer_qos: &DataWriterQos) -> Box<dyn ProtocolWriter>;
}

pub trait ProtocolWriter : ProtocolEntity  {    
    fn write(&mut self, instance_handle: InstanceHandle, data: Data, timestamp: Time) -> ReturnCode<()>;

    fn dispose(&self, instance_handle: InstanceHandle, timestamp: Time) -> ReturnCode<()>;

    fn unregister(&self, instance_handle: InstanceHandle, timestamp: Time) -> ReturnCode<()>;

    fn register(&self, instance_handle: InstanceHandle, timestamp: Time) -> ReturnCode<Option<InstanceHandle>>;

    fn lookup_instance(&self, instance_handle: InstanceHandle) -> Option<InstanceHandle>;
}

pub trait ProtocolReader: ProtocolEntity {

}

pub trait ProtocolDiscovery{

}

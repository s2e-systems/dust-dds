use crate::types::{InstanceHandle, Data, TopicKind, ChangeKind, ParameterList, ReturnCode};
use crate::cache_change::CacheChange;
use crate::history_cache::HistoryCache;
use crate::qos::{DataWriterQos, DataReaderQos};

pub trait ProtocolEntity {
    fn get_instance_handle(&self) -> InstanceHandle;
    fn enable(&self);
}

pub trait ProtocolParticipant : ProtocolEntity {
    fn create_publisher<'a>(&'a self) -> ReturnCode<Box<dyn ProtocolPublisher + 'a>>;
    fn create_subscriber<'a>(&'a self) -> ReturnCode<Box<dyn ProtocolSubscriber + 'a>>;
    
    fn get_builtin_subscriber(&self) -> ReturnCode<InstanceHandle>;
}

pub trait ProtocolPublisher : ProtocolEntity {
    fn create_writer<'a>(&'a self, parent_publisher: InstanceHandle, topic_kind: TopicKind, data_writer_qos: &DataWriterQos) -> ReturnCode<Box<dyn ProtocolWriter + 'a>>;
}

pub trait ProtocolSubscriber : ProtocolEntity {
    fn create_reader<'a>(&'a self, parent_subscriber: InstanceHandle, topic_kind: TopicKind, data_reader_qos: &DataReaderQos) -> ReturnCode<Box<dyn ProtocolReader + 'a>>;
}

pub trait ProtocolWriter : ProtocolEntity  {
    fn new_change(&mut self, kind: ChangeKind, data: Option<Data>, inline_qos: Option<ParameterList>, handle: InstanceHandle) -> CacheChange;
    fn writer_cache(&mut self) -> &mut HistoryCache;
}

pub trait ProtocolReader: ProtocolEntity {
}
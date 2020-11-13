use std::sync::Mutex;

use crate::types::{InstanceHandle, Data, Time, ReturnCode, TopicKind, ChangeKind, ParameterList};
use crate::cache_change::CacheChange;
use crate::history_cache::HistoryCache;
use crate::qos::{DataWriterQos, DataReaderQos};

pub trait ProtocolEntity {
    fn enable(&self) -> ReturnCode<()>;
    fn get_instance_handle(&self) -> InstanceHandle;
}

pub trait ProtocolParticipant : ProtocolEntity + 'static {
    fn create_publisher(&mut self) -> Box<dyn ProtocolPublisher>;
    fn delete_publisher(&mut self, publisher: &Box<dyn ProtocolPublisher>);

    fn create_subscriber(&mut self) -> Box<dyn ProtocolSubscriber>;
    fn delete_subscriber(&mut self, subscriber: &Box<dyn ProtocolSubscriber>);
    fn get_builtin_subscriber(&self) -> Box<dyn ProtocolSubscriber>;   
}

pub trait ProtocolSubscriber : ProtocolEntity {
    fn create_reader(&mut self, topic_kind: TopicKind, data_reader_qos: &DataReaderQos) -> Box<dyn ProtocolReader>;
}
pub trait ProtocolPublisher : ProtocolEntity {
    fn create_writer(&mut self, topic_kind: TopicKind, data_writer_qos: &DataWriterQos) -> Box<dyn ProtocolWriter>;
    fn delete_writer(&mut self, writer: &Box<dyn ProtocolWriter>);
}

pub trait ProtocolWriter : ProtocolEntity  {
    fn new_change(&self, kind: ChangeKind, data: Option<Data>, inline_qos: Option<ParameterList>, handle: InstanceHandle) -> CacheChange;
    fn writer_cache(&self) -> &Mutex<HistoryCache>;
}

pub trait ProtocolReader: ProtocolEntity {
}
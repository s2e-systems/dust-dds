use crate::types::{InstanceHandle, Data, TopicKind, ChangeKind, ParameterList};
use crate::cache_change::CacheChange;
use crate::history_cache::HistoryCache;
use crate::qos::{DataWriterQos, DataReaderQos};

pub trait ProtocolEntity {
    fn get_instance_handle(&self) -> InstanceHandle;
}

pub trait ProtocolParticipant : ProtocolEntity + 'static {
    fn create_publisher(&mut self) -> Box<dyn ProtocolPublisher>;

    fn create_subscriber(&mut self) -> Box<dyn ProtocolSubscriber>;
    fn get_builtin_subscriber(&self) -> Box<dyn ProtocolSubscriber>;

    fn run(&self);
    fn receive(&self, publisher_list: &[&dyn ProtocolPublisher], subscriber_list: &[&dyn ProtocolSubscriber]);
    fn send(&self, publisher_list: &[&dyn ProtocolPublisher], subscriber_list: &[&dyn ProtocolSubscriber]);
}

pub trait ProtocolSubscriber : ProtocolEntity {
    fn create_reader(&mut self, topic_kind: TopicKind, data_reader_qos: &DataReaderQos) -> Box<dyn ProtocolReader>;
}
pub trait ProtocolPublisher : ProtocolEntity {
    fn create_writer(&mut self, topic_kind: TopicKind, data_writer_qos: &DataWriterQos) -> Box<dyn ProtocolWriter>;
}

pub trait ProtocolWriter : ProtocolEntity  {
    fn new_change(&mut self, kind: ChangeKind, data: Option<Data>, inline_qos: Option<ParameterList>, handle: InstanceHandle) -> CacheChange;
    fn writer_cache(&mut self) -> &mut HistoryCache;
}

pub trait ProtocolReader: ProtocolEntity {
}
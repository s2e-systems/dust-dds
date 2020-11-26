use crate::types::{InstanceHandle, Data, TopicKind, ChangeKind, ParameterList, ReturnCode};
use crate::cache_change::CacheChange;
use crate::history_cache::HistoryCache;
use crate::qos::{DataWriterQos, DataReaderQos};

pub trait ProtocolEntity {
    fn get_instance_handle(&self) -> InstanceHandle;
}

pub trait ProtocolParticipant : ProtocolEntity + 'static {
    fn create_publisher(&self) -> ReturnCode<InstanceHandle>;
    fn create_writer(&self, parent_publisher: InstanceHandle, topic_kind: TopicKind, data_writer_qos: &DataWriterQos) -> ReturnCode<Box<dyn ProtocolWriter>>;

    fn create_subscriber(&self) -> ReturnCode<InstanceHandle>;
    fn create_reader(&self, parent_subscriber: InstanceHandle, topic_kind: TopicKind, data_reader_qos: &DataReaderQos) -> ReturnCode<Box<dyn ProtocolReader>>;
    fn get_builtin_subscriber(&self) -> ReturnCode<InstanceHandle>;

    fn enable(&self);
    // fn receive(&self, publisher_list: &[&dyn ProtocolPublisher], subscriber_list: &[&dyn ProtocolSubscriber]);
    // fn send(&self, publisher_list: &[&dyn ProtocolPublisher], subscriber_list: &[&dyn ProtocolSubscriber]);
}

pub trait ProtocolWriter : ProtocolEntity  {
    fn new_change(&mut self, kind: ChangeKind, data: Option<Data>, inline_qos: Option<ParameterList>, handle: InstanceHandle) -> CacheChange;
    fn writer_cache(&mut self) -> &mut HistoryCache;
}

pub trait ProtocolReader: ProtocolEntity {
}
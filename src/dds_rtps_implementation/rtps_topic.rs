use crate::dds_infrastructure::qos::TopicQos;
use crate::dds_rtps_implementation::rtps_object::RtpsObject;
use crate::rtps::structure::Entity;
use crate::rtps::types::GUID;
use crate::types::ReturnCode;
use std::sync::{atomic, Mutex, RwLockReadGuard};

pub struct RtpsTopicInner {
    entity: Entity,
    topic_name: String,
    type_name: &'static str,
    qos: Mutex<TopicQos>,
    reader_writer_reference_count: atomic::AtomicUsize, /* Reference count to this topic from DataReaders and DataWriters */
}

impl RtpsTopicInner {
    pub fn new(guid: GUID, topic_name: String, type_name: &'static str, qos: TopicQos) -> Self {
        Self {
            entity: Entity { guid },
            topic_name,
            type_name,
            qos: Mutex::new(qos),
            reader_writer_reference_count: atomic::AtomicUsize::new(0),
        }
    }
 
    // These two functions are defined on the inner object since they are meant as an
    // internal interface for this library
    pub fn increment_reference_count(&self) {
        self.reader_writer_reference_count.fetch_add(1, atomic::Ordering::Relaxed);
    }

    pub fn decrement_reference_count(&self) {
        self.reader_writer_reference_count.fetch_sub(1, atomic::Ordering::Relaxed);
    }
}

pub type RtpsTopic<'a> = RwLockReadGuard<'a, RtpsObject<RtpsTopicInner>>;

impl RtpsObject<RtpsTopicInner> {
    pub fn get_name(&self) -> ReturnCode<String> {
        Ok(self.value()?.topic_name.clone())
    }
}

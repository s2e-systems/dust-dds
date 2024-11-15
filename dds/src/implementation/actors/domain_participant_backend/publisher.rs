use crate::{
    implementation::{actor::Actor, actors::status_condition_actor::StatusConditionActor},
    infrastructure::{
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos},
        status::StatusKind,
    },
};

use super::{data_writer::DataWriterActor, publisher_listener::PublisherListenerThread};

pub struct PublisherActor {
    pub qos: PublisherQos,
    pub instance_handle: InstanceHandle,
    pub data_writer_list: Vec<DataWriterActor>,
    pub enabled: bool,
    pub default_datawriter_qos: DataWriterQos,
    pub publisher_listener_thread: Option<PublisherListenerThread>,
    pub status_kind: Vec<StatusKind>,
    pub status_condition: Actor<StatusConditionActor>,
}

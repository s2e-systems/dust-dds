use crate::{
    implementation::{actor::Actor, actors::status_condition_actor::StatusConditionActor},
    infrastructure::{
        instance::InstanceHandle,
        qos::{DataReaderQos, SubscriberQos},
        status::StatusKind,
    },
};

use super::{data_reader::DataReaderActor, subscriber_listener::SubscriberListenerThread};

pub struct SubscriberActor {
    pub instance_handle: InstanceHandle,
    pub qos: SubscriberQos,
    pub data_reader_list: Vec<DataReaderActor>,
    pub enabled: bool,
    pub default_data_reader_qos: DataReaderQos,
    pub status_condition: Actor<StatusConditionActor>,
    pub subscriber_listener_thread: Option<SubscriberListenerThread>,
    pub subscriber_status_kind: Vec<StatusKind>,
}

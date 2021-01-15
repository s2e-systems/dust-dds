use std::sync::{Mutex, atomic};

use crate::{dds::{infrastructure::{qos::{DataReaderQos, SubscriberQos}, status::StatusMask}, subscription::subscriber_listener::SubscriberListener}, rtps::{structure::Group, types::GUID}, utils::maybe_valid::MaybeValidList};

use super::rtps_datareader::AnyRtpsReader;

pub struct RtpsSubscriber {
    pub group: Group,
    pub reader_list: MaybeValidList<Box<dyn AnyRtpsReader>>,
    pub reader_count: atomic::AtomicU8,
    pub default_datareader_qos: Mutex<DataReaderQos>,
    pub qos: SubscriberQos,
    pub listener: Option<Box<dyn SubscriberListener>>,
    pub status_mask: StatusMask,
}

impl RtpsSubscriber {
    pub fn new(guid: GUID, qos: SubscriberQos, listener: Option<Box<dyn SubscriberListener>>, status_mask: StatusMask) -> Self {
        Self {
            group: Group::new(guid),
            reader_list: Default::default(),
            reader_count: atomic::AtomicU8::new(0),
            default_datareader_qos: Mutex::new(DataReaderQos::default()),
            qos,
            listener,
            status_mask
        }
    }
}
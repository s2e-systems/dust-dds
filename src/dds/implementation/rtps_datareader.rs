use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use crate::{dds::{
        infrastructure::{
            qos::DataReaderQos, qos_policy::ReliabilityQosPolicyKind, status::StatusMask,
        },
        subscription::data_reader_listener::DataReaderListener,
    }, rtps::{
        behavior::{self, StatefulReader},
        types::{ReliabilityKind, GUID},
    }, types::{DDSType, ReturnCode, ReturnCodes}, utils::maybe_valid::MaybeValidRef};

use super::rtps_topic::AnyRtpsTopic;

pub struct RtpsDataReader<T: DDSType> {
    pub reader: StatefulReader,
    pub qos: Mutex<DataReaderQos>,
    pub topic: Mutex<Option<Arc<dyn AnyRtpsTopic>>>,
    pub listener: Option<Box<dyn DataReaderListener<T>>>,
    pub status_mask: StatusMask,
}

impl<T: DDSType> RtpsDataReader<T> {
    pub fn new(
        guid: GUID,
        topic: Arc<dyn AnyRtpsTopic>,
        qos: DataReaderQos,
        listener: Option<Box<dyn DataReaderListener<T>>>,
        status_mask: StatusMask,
    ) -> Self {
        assert!(
            qos.is_consistent().is_ok(),
            "RtpsDataReader can only be created with consistent QoS"
        );

        let topic_kind = topic.topic_kind();
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };
        let expects_inline_qos = false;
        let heartbeat_response_delay = behavior::types::constants::DURATION_ZERO;
        let reader = StatefulReader::new(
            guid,
            topic_kind,
            reliability_level,
            expects_inline_qos,
            heartbeat_response_delay,
        );
        Self {
            reader,
            qos: Mutex::new(qos),
            topic: Mutex::new(Some(topic)),
            listener,
            status_mask,
        }
    }
}

pub trait AnyRtpsReader: Send + Sync {
    fn reader(&self) -> &StatefulReader;
    fn qos(&self) -> &Mutex<DataReaderQos>;
    fn topic(&self) -> &Mutex<Option<Arc<dyn AnyRtpsTopic>>>;
    fn status_mask(&self) -> &StatusMask;
    fn as_any(&self) -> &dyn Any;
}

impl<T: DDSType + Sized> AnyRtpsReader for RtpsDataReader<T> {
    fn reader(&self) -> &StatefulReader {
        &self.reader
    }

    fn qos(&self) -> &Mutex<DataReaderQos> {
        &self.qos
    }

    fn topic(&self) -> &Mutex<Option<Arc<dyn AnyRtpsTopic>>> {
        &self.topic
    }

    fn status_mask(&self) -> &StatusMask {
        &self.status_mask
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub type RtpsAnyDataReaderRef<'a> = MaybeValidRef<'a, Box<dyn AnyRtpsReader>>;


impl<'a> RtpsAnyDataReaderRef<'a> {
    pub fn value(&self) -> ReturnCode<&Box<dyn AnyRtpsReader>> {
        self.get().ok_or(ReturnCodes::AlreadyDeleted)
    }

    pub fn value_as<U: 'static>(&self) -> ReturnCode<&U> {
        self.value()?
            .as_ref()
            .as_any()
            .downcast_ref::<U>()
            .ok_or(ReturnCodes::Error)
    }
}
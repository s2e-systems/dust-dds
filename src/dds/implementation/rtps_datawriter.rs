use std::{
    any::Any,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use crate::{dds::{
        infrastructure::{
            qos::DataWriterQos, qos_policy::ReliabilityQosPolicyKind, status::StatusMask,
        },
        publication::data_writer_listener::DataWriterListener,
    }, rtps::{
        behavior::{
            self, endpoint_traits::CacheChangeSender, StatefulWriter, StatelessWriter, Writer,
        },
        types::{ReliabilityKind, GUID},
    }, types::{DDSType, InstanceHandle, ReturnCode, ReturnCodes, Time}, utils::{as_any::AsAny, maybe_valid::{MaybeValid, MaybeValidRef}}};

use super::rtps_topic::{AnyRtpsTopic, RtpsAnyTopicRef};

pub enum WriterFlavor {
    Stateful(StatefulWriter),
    Stateless(StatelessWriter),
}
impl WriterFlavor {
    pub fn try_get_stateless(&mut self) -> Option<&mut StatelessWriter> {
        match self {
            WriterFlavor::Stateless(writer) => Some(writer),
            _ => None,
        }
    }
}
impl Deref for WriterFlavor {
    type Target = Writer;

    fn deref(&self) -> &Self::Target {
        match self {
            WriterFlavor::Stateful(writer) => writer,
            WriterFlavor::Stateless(writer) => writer,
        }
    }
}
impl DerefMut for WriterFlavor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            WriterFlavor::Stateful(writer) => writer,
            WriterFlavor::Stateless(writer) => writer,
        }
    }
}

impl CacheChangeSender for WriterFlavor {
    fn produce_messages(&mut self) -> Vec<behavior::endpoint_traits::DestinedMessages> {
        match self {
            WriterFlavor::Stateful(writer) => writer.produce_messages(),
            WriterFlavor::Stateless(writer) => writer.produce_messages(),
        }
    }
}

pub struct RtpsDataWriter<T: DDSType> {
    pub writer: Mutex<WriterFlavor>,
    pub qos: Mutex<DataWriterQos>,
    pub topic: Mutex<Option<Arc<dyn AnyRtpsTopic>>>,
    pub listener: Option<Box<dyn DataWriterListener<T>>>,
    pub status_mask: StatusMask,
}

impl<T: DDSType> RtpsDataWriter<T> {
    pub fn new_stateful(
        guid: GUID,
        topic: &RtpsAnyTopicRef,
        qos: DataWriterQos,
        listener: Option<Box<dyn DataWriterListener<T>>>,
        status_mask: StatusMask,
    ) -> Self {
        assert!(
            qos.is_consistent().is_ok(),
            "RtpsDataWriter can only be created with consistent QoS"
        );
        let topic = topic.get().unwrap().clone();
        let topic_kind = topic.topic_kind();
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };
        let push_mode = true;
        let data_max_sized_serialized = None;
        let heartbeat_period = behavior::types::Duration::from_millis(500);
        let nack_response_delay = behavior::types::constants::DURATION_ZERO;
        let nack_supression_duration = behavior::types::constants::DURATION_ZERO;
        let writer = StatefulWriter::new(
            guid,
            topic_kind,
            reliability_level,
            push_mode,
            data_max_sized_serialized,
            heartbeat_period,
            nack_response_delay,
            nack_supression_duration,
        );

        Self {
            writer: Mutex::new(WriterFlavor::Stateful(writer)),
            qos: Mutex::new(qos),
            topic: Mutex::new(Some(topic)),
            listener,
            status_mask,
        }
    }

    pub fn new_stateless(
        guid: GUID,
        topic: &RtpsAnyTopicRef,
        qos: DataWriterQos,
        listener: Option<Box<dyn DataWriterListener<T>>>,
        status_mask: StatusMask,
    ) -> Self {
        assert!(
            qos.is_consistent().is_ok(),
            "RtpsDataWriter can only be created with consistent QoS"
        );
        let topic = topic.get().unwrap().clone();
        let topic_kind = topic.topic_kind();
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };
        let push_mode = true;
        let data_max_sized_serialized = None;
        let writer = StatelessWriter::new(
            guid,
            topic_kind,
            reliability_level,
            push_mode,
            data_max_sized_serialized,
        );

        Self {
            writer: Mutex::new(WriterFlavor::Stateless(writer)),
            qos: Mutex::new(qos),
            topic: Mutex::new(Some(topic)),
            listener,
            status_mask,
        }
    }

    pub fn register_instance_w_timestamp(&self) {}

    pub fn write_w_timestamp(
        &self,
        data: T,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> ReturnCode<()> {
        let writer = &mut self.writer.lock().unwrap();
        let kind = crate::types::ChangeKind::Alive;
        let inline_qos = None;
        let change = writer.new_change(
            kind,
            Some(data.serialize()),
            inline_qos,
            data.instance_handle(),
        );
        writer.writer_cache.add_change(change);

        Ok(())
    }

    pub fn set_qos(&self, qos: Option<DataWriterQos>) -> ReturnCode<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        *self.qos.lock().unwrap() = qos;
        Ok(())
    }

    pub fn get_qos(&self) -> ReturnCode<DataWriterQos> {
        Ok(self.qos.lock().unwrap().clone())
    }
}

pub trait AnyRtpsWriter: AsAny + Send + Sync {
    fn writer(&self) -> &Mutex<WriterFlavor>;
}

impl<T: DDSType + Sized> AnyRtpsWriter for RtpsDataWriter<T> {
    fn writer(&self) -> &Mutex<WriterFlavor> {
        &self.writer
    }
}

impl<T: DDSType + Sized> AsAny for RtpsDataWriter<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub type RtpsAnyDataWriterRef<'a> = MaybeValidRef<'a, Box<dyn AnyRtpsWriter>>;

impl<'a> RtpsAnyDataWriterRef<'a> {
    pub fn get(&self) -> ReturnCode<&dyn AnyRtpsWriter> {
        Ok(MaybeValid::get(self)
            .ok_or(ReturnCodes::AlreadyDeleted)?
            .as_ref())
    }

    pub fn get_as<U: DDSType>(&self) -> ReturnCode<&RtpsDataWriter<U>> {
        MaybeValid::get(self)
            .ok_or(ReturnCodes::AlreadyDeleted)?
            .as_ref()
            .as_any()
            .downcast_ref()
            .ok_or(ReturnCodes::Error)
    }
    
    pub fn delete(&self) {
        MaybeValid::delete(self)
    }
}

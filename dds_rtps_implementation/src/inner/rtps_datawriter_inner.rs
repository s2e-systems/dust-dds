use std::{any::Any, convert::TryInto, ops::{Deref, DerefMut}, sync::{Arc, Mutex, MutexGuard}};

use behavior::types::{Duration, constants::DURATION_ZERO};
use rust_dds_api::{dcps_psm::{InstanceHandle, StatusMask, Time}, dds_type::DDSType, infrastructure::{qos::DataWriterQos, qos_policy::ReliabilityQosPolicyKind}, publication::data_writer_listener::DataWriterListener, return_type::{DDSError, DDSResult}};
use rust_rtps::{
    behavior::{self, StatefulWriter, StatelessWriter, Writer},
    types::{
        constants::{
            ENTITY_KIND_BUILT_IN_WRITER_NO_KEY, ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
            ENTITY_KIND_USER_DEFINED_WRITER_NO_KEY, ENTITY_KIND_USER_DEFINED_WRITER_WITH_KEY,
        },
        ChangeKind, EntityId, GuidPrefix, ReliabilityKind, GUID,
    },
};

use crate::utils::{
    as_any::AsAny,
    maybe_valid::{MaybeValid, MaybeValidRef},
};

use super::rtps_topic_inner::{RtpsTopicInner, RtpsTopicInnerRef};

pub enum WriterFlavor {
    Stateful(StatefulWriter),
    Stateless(StatelessWriter),
}
impl WriterFlavor {
    pub fn try_get_stateless(&mut self) -> Option<&mut StatelessWriter> {
        match self {
            WriterFlavor::Stateless(writer) => Some(writer),
            WriterFlavor::Stateful(_) => None,
        }
    }

    pub fn try_get_stateful(&mut self) -> Option<&mut StatefulWriter> {
        match self {
            WriterFlavor::Stateless(_) => None,
            WriterFlavor::Stateful(writer) => Some(writer),
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

pub struct RtpsDataWriterInner<T: DDSType> {
    writer: Mutex<WriterFlavor>,
    qos: Mutex<DataWriterQos>,
    topic: Mutex<Option<Arc<RtpsTopicInner>>>,
    listener: Option<Box<dyn DataWriterListener<DataType=T>>>,
    status_mask: StatusMask,
}

impl<T: DDSType> RtpsDataWriterInner<T> {
    pub fn new_builtin_stateless(
        guid_prefix: GuidPrefix,
        entity_key: [u8;3],
        topic: &RtpsTopicInnerRef,
        qos: DataWriterQos,
        listener: Option<Box<dyn DataWriterListener<DataType=T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let entity_kind = match T::has_key() {
            false => ENTITY_KIND_BUILT_IN_WRITER_NO_KEY,
            true => ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);
        Self::new_stateless(guid, topic, qos, listener, status_mask)
    }

    pub fn new_user_defined_stateless(
        guid_prefix: GuidPrefix,
        entity_key: [u8;3],
        topic: &RtpsTopicInnerRef,
        qos: DataWriterQos,
        listener: Option<Box<dyn DataWriterListener<DataType=T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let entity_kind = match T::has_key() {
            false => ENTITY_KIND_USER_DEFINED_WRITER_NO_KEY,
            true => ENTITY_KIND_USER_DEFINED_WRITER_WITH_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);
        Self::new_stateless(guid, topic, qos, listener, status_mask)
    }

    pub fn new_builtin_stateful(
        guid_prefix: GuidPrefix,
        entity_key: [u8;3],
        topic: &RtpsTopicInnerRef,
        qos: DataWriterQos,
        listener: Option<Box<dyn DataWriterListener<DataType=T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let entity_kind = match T::has_key() {
            false => ENTITY_KIND_BUILT_IN_WRITER_NO_KEY,
            true => ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);
        Self::new_stateful(guid, topic, qos, listener, status_mask)
    }

    pub fn new_user_defined_stateful(
        guid_prefix: GuidPrefix,
        entity_key: [u8;3],
        topic: &RtpsTopicInnerRef,
        qos: DataWriterQos,
        listener: Option<Box<dyn DataWriterListener<DataType=T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let entity_kind = match T::has_key() {
            false => ENTITY_KIND_BUILT_IN_WRITER_NO_KEY,
            true => ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = GUID::new(guid_prefix, entity_id);
        Self::new_stateful(guid, topic, qos, listener, status_mask)
    }

    fn new_stateful(
        guid: GUID,
        topic: &RtpsTopicInnerRef,
        qos: DataWriterQos,
        listener: Option<Box<dyn DataWriterListener<DataType=T>>>,
        status_mask: StatusMask,
    ) -> Self {
        assert!(
            qos.is_consistent().is_ok(),
            "RtpsDataWriter can only be created with consistent QoS"
        );
        let topic_kind = topic.topic_kind().unwrap();
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
        let topic = topic.get().unwrap().clone();

        Self {
            writer: Mutex::new(WriterFlavor::Stateful(writer)),
            qos: Mutex::new(qos),
            topic: Mutex::new(Some(topic)),
            listener,
            status_mask,
        }
    }

    fn new_stateless(
        guid: GUID,
        topic: &RtpsTopicInnerRef,
        qos: DataWriterQos,
        listener: Option<Box<dyn DataWriterListener<DataType=T>>>,
        status_mask: StatusMask,
    ) -> Self {
        assert!(
            qos.is_consistent().is_ok(),
            "RtpsDataWriter can only be created with consistent QoS"
        );
        let topic_kind = topic.topic_kind().unwrap();
        let reliability_level = match qos.reliability.kind {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
            ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
        };
        let push_mode = true;
        let data_max_sized_serialized = None;
        let heartbeat_period = Duration::from_millis(200);
        let nack_response_delay = DURATION_ZERO;
        let nack_supression_duration = DURATION_ZERO;
        let writer = StatelessWriter::new(
            guid,
            topic_kind,
            reliability_level,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_supression_duration,
            data_max_sized_serialized,
        );
        let topic = topic.get().unwrap().clone();

        Self {
            writer: Mutex::new(WriterFlavor::Stateless(writer)),
            qos: Mutex::new(qos),
            topic: Mutex::new(Some(topic)),
            listener,
            status_mask,
        }
    }
}

pub trait RtpsAnyDataWriterInner: AsAny + Send + Sync {
    fn writer(&self) -> MutexGuard<WriterFlavor>;

    fn topic(&self) -> MutexGuard<Option<Arc<RtpsTopicInner>>>;

    fn qos(&self) -> MutexGuard<DataWriterQos>;
}

impl<T: DDSType + Sized> RtpsAnyDataWriterInner for RtpsDataWriterInner<T> {
    fn writer(&self) -> MutexGuard<WriterFlavor> {
        self.writer.lock().unwrap()
    }

    fn topic(&self) -> MutexGuard<Option<Arc<RtpsTopicInner>>> {
        self.topic.lock().unwrap()
    }

    fn qos(&self) -> MutexGuard<DataWriterQos> {
        self.qos.lock().unwrap()
    }
}

impl<T: DDSType + Sized> AsAny for RtpsDataWriterInner<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn instance_handle_from_dds_type<T: DDSType>(data: T) -> rust_rtps::types::InstanceHandle {
    if data.key().is_empty() {
        [0; 16]
    } else {
        let mut key = data.key();
        key.resize_with(16, Default::default);
        key.try_into().unwrap()
    }
}

pub type RtpsAnyDataWriterInnerRef<'a> = MaybeValidRef<'a, Box<dyn RtpsAnyDataWriterInner>>;

impl<'a> RtpsAnyDataWriterInnerRef<'a> {
    pub fn get(&self) -> DDSResult<&Box<dyn RtpsAnyDataWriterInner>> {
        MaybeValid::get(self).ok_or(DDSError::AlreadyDeleted)
    }

    //     pub fn get_as<U: DDSType>(&self) -> DDSResult<&RtpsDataWriter<U>> {
    //         self.get()?
    //             .as_ref()
    //             .as_any()
    //             .downcast_ref()
    //             .ok_or(DDSError::Error)
    //     }

    pub fn delete(&self) -> DDSResult<()> {
        self.get()?.topic().take(); // Drop the topic
        MaybeValid::delete(self);
        Ok(())
    }

    pub fn write_w_timestamp<T: DDSType>(
        &self,
        data: T,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DDSResult<()> {
        let writer = &mut self.get()?.writer();
        let kind = ChangeKind::Alive;
        let inline_qos = None;
        let change = writer.new_change(
            kind,
            Some(data.serialize()),
            inline_qos,
            instance_handle_from_dds_type(data),
        );
        writer.writer_cache.add_change(change);

        Ok(())
    }

    //     pub fn get_qos(&self) -> DDSResult<DataWriterQos> {
    //         Ok(self.get()?.qos().clone())
    //     }

    //     pub fn set_qos(&self, qos: Option<DataWriterQos>) -> DDSResult<()> {
    //         let qos = qos.unwrap_or_default();
    //         qos.is_consistent()?;
    //         *self.get()?.qos() = qos;
    //         Ok(())
    //     }

    pub fn produce_messages(&self) -> Vec<behavior::endpoint_traits::DestinedMessages> {
        todo!()
        // if let Some(rtps_writer) = self.get().ok() {
        //     match &mut *rtps_writer.writer() {
        //         WriterFlavor::Stateful(writer) => writer.produce_messages(),
        //         WriterFlavor::Stateless(writer) => writer.produce_messages(),
        //     }
        // } else {
        //     vec![]
        // }
    }
}

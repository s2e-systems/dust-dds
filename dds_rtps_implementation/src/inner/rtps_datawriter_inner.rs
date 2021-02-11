use std::{any::Any, convert::TryInto, sync::{Arc, Mutex, MutexGuard}};

use behavior::StatelessWriter;
use rust_dds_api::{
    dcps_psm::{InstanceHandle, StatusMask, Time},
    dds_type::DDSType,
    infrastructure::{qos::DataWriterQos, qos_policy::ReliabilityQosPolicyKind},
    publication::data_writer_listener::DataWriterListener,
    return_type::{DDSError, DDSResult},
};
use rust_rtps::{behavior::{self, Writer}, types::{ChangeKind, EntityId, GUID, GuidPrefix, ReliabilityKind}};

use crate::utils::{
    as_any::AsAny,
    maybe_valid::{MaybeValid, MaybeValidRef},
};

use super::rtps_topic_inner::RtpsTopicInner;

pub struct RtpsDataWriterInner<T: DDSType> {
    writer: Mutex<Writer>,
    qos: Mutex<DataWriterQos>,
    topic: Mutex<Option<Arc<RtpsTopicInner>>>,
    listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
    status_mask: StatusMask,
}

impl<T: DDSType> RtpsDataWriterInner<T> {
    pub fn new(
        guid_prefix: GuidPrefix,
        entity_id: EntityId,
        topic: &Arc<RtpsTopicInner>,
        qos: DataWriterQos,
        listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let topic = topic.clone();

        assert!(
            qos.is_consistent().is_ok(),
            "RtpsDataWriter can only be created with consistent QoS"
        );
        let guid = GUID::new(guid_prefix, entity_id);
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
        let writer = Writer::new(
            guid,
            topic_kind,
            reliability_level,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_supression_duration,
            data_max_sized_serialized,
        );

        Self {
            writer: Mutex::new(writer),
            qos: Mutex::new(qos),
            topic: Mutex::new(Some(topic)),
            listener,
            status_mask,
        }
    }

    pub fn writer(&self) -> &Mutex<Writer> {
        &self.writer
    }

    pub fn qos(&self) -> &Mutex<DataWriterQos> {
        &self.qos
    }

    pub fn topic(&self) -> &Mutex<Option<Arc<RtpsTopicInner>>> {
        &self.topic
    }

    pub fn listener(&self) -> &Option<Box<dyn DataWriterListener<DataType = T>>> {
        &self.listener
    }

    pub fn status_mask(&self) -> &StatusMask {
        &self.status_mask
    }
}

pub trait RtpsAnyDataWriterInner: AsAny + Send + Sync {
    fn writer(&self) -> MutexGuard<Writer>;

    fn topic(&self) -> MutexGuard<Option<Arc<RtpsTopicInner>>>;

    fn qos(&self) -> MutexGuard<DataWriterQos>;
}


impl<T: DDSType + Sized> RtpsAnyDataWriterInner for RtpsDataWriterInner<T> {
    fn writer(&self) -> MutexGuard<Writer> {
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

    pub fn try_get_stateless(&self) -> DDSResult<Option<&Box<dyn StatelessWriter>>> {
        Ok(self.get()?.as_ref().as_any().downcast_ref())
    }

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
        let this = self.get()?;
        let kind = ChangeKind::Alive;
        let inline_qos = None;
        let change = this.writer().new_change(
            kind,
            Some(data.serialize()),
            inline_qos,
            instance_handle_from_dds_type(data),
        );
        this.writer().writer_cache.add_change(change);

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

    // pub fn produce_messages(&self) -> Vec<behavior::endpoint_traits::DestinedMessages> {
    // todo!()
    // if let Some(rtps_writer) = self.get().ok() {
    //     match &mut *rtps_writer.writer() {
    //         WriterFlavor::Stateful(writer) => writer.produce_messages(),
    //         WriterFlavor::Stateless(writer) => writer.produce_messages(),
    //     }
    // } else {
    //     vec![]
    // }
    // }
}

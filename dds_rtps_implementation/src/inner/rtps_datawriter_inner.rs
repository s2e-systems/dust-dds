use std::{
    convert::TryInto,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use rust_dds_api::{
    dcps_psm::{InstanceHandle, StatusMask, Time},
    dds_type::DDSType,
    infrastructure::qos::DataWriterQos,
    publication::data_writer_listener::DataWriterListener,
    return_type::{DDSError, DDSResult},
};
use rust_rtps::{behavior::Writer, types::ChangeKind};

use crate::utils::maybe_valid::{MaybeValid, MaybeValidRef};

use super::{rtps_stateful_datawriter_inner::RtpsStatefulDataWriterInner, rtps_stateless_datawriter_inner::RtpsStatelessDataWriterInner, rtps_topic_inner::RtpsTopicInner};

pub trait AnyDataWriterListener: Send + Sync {}

impl<T: DDSType> AnyDataWriterListener for dyn DataWriterListener<DataType = T> {}

pub struct RtpsDataWriterInner {
    qos: DataWriterQos,
    topic: Option<Arc<RtpsTopicInner>>,
    listener: Option<Box<dyn AnyDataWriterListener>>,
    status_mask: StatusMask,
}

impl RtpsDataWriterInner {
    pub fn new(
        topic: &Arc<RtpsTopicInner>,
        qos: DataWriterQos,
        listener: Option<Box<dyn AnyDataWriterListener>>,
        status_mask: StatusMask,
    ) -> Self {
        let topic = Some(topic.clone());

        Self {
            qos,
            topic,
            listener,
            status_mask,
        }
    }
}

pub enum RtpsDataWriterFlavor {
    Stateful(RtpsStatefulDataWriterInner),
    Stateless(RtpsStatelessDataWriterInner),
}

impl RtpsDataWriterFlavor {
    pub fn topic(&mut self) -> &mut Option<Arc<RtpsTopicInner>> {
        match self {
            Self::Stateful(stateful) => &mut stateful.inner.topic,
            Self::Stateless(stateless) => &mut stateless.inner.topic,
        }
    }
}

impl Deref for RtpsDataWriterFlavor {
    type Target = Writer;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Stateful(stateful) => &stateful.stateful_writer,
            Self::Stateless(stateless) => &stateless.stateless_writer,
        }
    }
}

impl DerefMut for RtpsDataWriterFlavor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Stateful(stateful) => &mut stateful.stateful_writer,
            Self::Stateless(stateless) => &mut stateless.stateless_writer,
        }
    }
}

// impl RtpsDataWriterFlavor {
//     pub fn new_stateless(guid_prefix: GuidPrefix,
//         entity_id: EntityId,
//         topic: &Arc<RtpsTopicInner>,
//         qos: DataWriterQos,
//         listener: Option<Box<dyn AnyDataWriterListener>>,
//         status_mask: StatusMask,) -> Self {
//         assert!(
//             qos.is_consistent().is_ok(),
//             "RtpsDataWriter can only be created with consistent QoS"
//         );
//         let guid = GUID::new(guid_prefix, entity_id);
//         let topic_kind = topic.topic_kind();
//         let reliability_level = match qos.reliability.kind {
//             ReliabilityQosPolicyKind::BestEffortReliabilityQos => ReliabilityKind::BestEffort,
//             ReliabilityQosPolicyKind::ReliableReliabilityQos => ReliabilityKind::Reliable,
//         };
//         let push_mode = true;
//         let data_max_sized_serialized = None;
//         let heartbeat_period = behavior::types::Duration::from_millis(500);
//         let nack_response_delay = behavior::types::constants::DURATION_ZERO;
//         let nack_supression_duration = behavior::types::constants::DURATION_ZERO;
//         let writer = Writer::new(
//             guid,
//             topic_kind,
//             reliability_level,
//             push_mode,
//             heartbeat_period,
//             nack_response_delay,
//             nack_supression_duration,
//             data_max_sized_serialized,
//         );
//     }
// }

fn instance_handle_from_dds_type<T: DDSType>(data: T) -> rust_rtps::types::InstanceHandle {
    if data.key().is_empty() {
        [0; 16]
    } else {
        let mut key = data.key();
        key.resize_with(16, Default::default);
        key.try_into().unwrap()
    }
}

pub type RtpsAnyDataWriterInnerRef<'a> = MaybeValidRef<'a, Mutex<RtpsDataWriterFlavor>>;

impl<'a> RtpsAnyDataWriterInnerRef<'a> {
    pub fn get(&self) -> DDSResult<&Mutex<RtpsDataWriterFlavor>> {
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
        self.get()?.lock().unwrap().topic().take(); // Drop the topic
        MaybeValid::delete(self);
        Ok(())
    }

    pub fn write_w_timestamp<T: DDSType>(
        &self,
        data: T,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DDSResult<()> {
        let mut this = self.get()?.lock().unwrap();
        let kind = ChangeKind::Alive;
        let inline_qos = None;
        let change = this.new_change(
            kind,
            Some(data.serialize()),
            inline_qos,
            instance_handle_from_dds_type(data),
        );
        this.writer_cache.add_change(change);

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

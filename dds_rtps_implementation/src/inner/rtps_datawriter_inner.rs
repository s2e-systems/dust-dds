use std::{convert::TryInto, ops::{Deref, DerefMut}, sync::{Arc, Mutex, MutexGuard}};

use rust_dds_api::{
    dcps_psm::{InstanceHandle, StatusMask, Time},
    dds_type::DDSType,
    infrastructure::qos::DataWriterQos,
    publication::data_writer_listener::DataWriterListener,
    return_type::{DDSError, DDSResult},
};
use rust_rtps::{behavior::Writer, types::ChangeKind};

use crate::utils::maybe_valid::{MaybeValid, MaybeValidRef};

use super::{
    rtps_stateful_datawriter_inner::RtpsStatefulDataWriterInner,
    rtps_stateless_datawriter_inner::RtpsStatelessDataWriterInner,
    rtps_topic_inner::RtpsTopicInner,
};

pub struct RtpsDataWriterInner<T: DDSType> {
    qos: DataWriterQos,
    topic: Option<Arc<RtpsTopicInner>>,
    listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
    status_mask: StatusMask,
}

impl<T: DDSType> RtpsDataWriterInner<T> {
    pub fn new(
        topic: &Arc<RtpsTopicInner>,
        qos: DataWriterQos,
        listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
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

pub trait AnyRtpsDataWriterInner: Send + Sync {
    fn topic(&mut self) -> &mut Option<Arc<RtpsTopicInner>>;

    fn qos(&mut self) -> &mut DataWriterQos;
}

impl<T: DDSType> AnyRtpsDataWriterInner for RtpsDataWriterInner<T> {
    fn topic(&mut self) -> &mut Option<Arc<RtpsTopicInner>> {
        &mut self.topic
    }

    fn qos(&mut self) -> &mut DataWriterQos {
        &mut self.qos
    }
}

pub enum RtpsDataWriterFlavor {
    Stateful(RtpsStatefulDataWriterInner),
    Stateless(RtpsStatelessDataWriterInner),
}

impl RtpsDataWriterFlavor {
    fn inner(&mut self) -> &mut dyn AnyRtpsDataWriterInner {
        match self {
            Self::Stateful(stateful) => stateful.inner.as_mut(),
            Self::Stateless(stateless) => stateless.inner.as_mut(),
        }
    }
}

impl AnyRtpsDataWriterInner for RtpsDataWriterFlavor {
    fn topic(&mut self) -> &mut Option<Arc<RtpsTopicInner>> {
        self.inner().topic()
    }

    fn qos(&mut self) -> &mut DataWriterQos {
        self.inner().qos()
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
    pub fn get(&self) -> DDSResult<MutexGuard<RtpsDataWriterFlavor>> {
        Ok(MaybeValid::get(self).ok_or(DDSError::AlreadyDeleted)?.lock().unwrap())
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
        let mut this = self.get()?;
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

    pub fn get_qos(&self) -> DDSResult<DataWriterQos> {
        Ok(self.get()?.qos().clone())
    }

    pub fn set_qos(&self, qos: Option<DataWriterQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        *self.get()?.qos() = qos;
        Ok(())
    }

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

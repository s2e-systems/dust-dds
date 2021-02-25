use std::ops::{Deref, DerefMut};

use rust_dds_api::{
    dcps_psm::StatusMask, dds_type::DDSType, infrastructure::qos::DataWriterQos,
    publication::data_writer_listener::DataWriterListener,
};
use rust_rtps::behavior::{StatefulWriter, StatelessWriter, Writer};

use super::mask_listener::MaskListener;
struct RtpsDataWriterListener<T: DDSType>(Box<dyn DataWriterListener<DataType = T>>);
trait AnyDataWriterListener: Send + Sync {}

impl<T: DDSType> AnyDataWriterListener for RtpsDataWriterListener<T> {}

pub enum RtpsWriterFlavor {
    Stateful(StatefulWriter),
    Stateless(StatelessWriter),
}

impl Deref for RtpsWriterFlavor {
    type Target = Writer;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Stateful(stateful_writer) => stateful_writer,
            Self::Stateless(stateless_writer) => stateless_writer,
        }
    }
}

impl DerefMut for RtpsWriterFlavor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Stateful(stateful_writer) => stateful_writer,
            Self::Stateless(stateless_writer) => stateless_writer,
        }
    }
}

pub struct RtpsDataWriterImpl {
    rtps_writer_flavor: RtpsWriterFlavor,
    qos: DataWriterQos,
    mask_listener: MaskListener<Box<dyn AnyDataWriterListener>>,
}

impl RtpsDataWriterImpl {
    pub fn new<T: DDSType>(
        rtps_writer_flavor: RtpsWriterFlavor,
        qos: DataWriterQos,
        listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let listener: Option<Box<dyn AnyDataWriterListener>> = match listener {
            Some(listener) => Some(Box::new(RtpsDataWriterListener(listener))),
            None => None,
        };
        let mask_listener = MaskListener::new(listener, status_mask);
        Self {
            rtps_writer_flavor,
            qos,
            mask_listener,
        }
    }
}
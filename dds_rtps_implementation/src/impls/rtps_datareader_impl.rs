use std::ops::{Deref, DerefMut};

use rust_dds_api::{
    dcps_psm::StatusMask, dds_type::DDSType, infrastructure::qos::DataReaderQos,
    subscription::data_reader_listener::DataReaderListener,
};
use rust_rtps::behavior::{Reader, StatefulReader, StatelessReader};

use super::mask_listener::MaskListener;

struct RtpsDataReaderListener<T: DDSType>(Box<dyn DataReaderListener<DataType = T>>);
trait AnyDataReaderListener: Send + Sync {}

impl<T: DDSType> AnyDataReaderListener for RtpsDataReaderListener<T> {}

pub enum RtpsReaderFlavor {
    Stateful(StatefulReader),
    Stateless(StatelessReader),
}

impl Deref for RtpsReaderFlavor {
    type Target = Reader;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Stateful(stateful_reader) => stateful_reader,
            Self::Stateless(stateless_reader) => stateless_reader,
        }
    }
}

impl DerefMut for RtpsReaderFlavor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Stateful(stateful_reader) => stateful_reader,
            Self::Stateless(stateless_reader) => stateless_reader,
        }
    }
}
pub struct RtpsDataReaderImpl {
    rtps_reader_flavor: RtpsReaderFlavor,
    qos: DataReaderQos,
    mask_listener: MaskListener<Box<dyn AnyDataReaderListener>>,
}

impl RtpsDataReaderImpl {
    pub fn new<T: DDSType>(
        rtps_reader_flavor: RtpsReaderFlavor,
        qos: DataReaderQos,
        listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
        status_mask: StatusMask,
    ) -> Self {
        let listener: Option<Box<dyn AnyDataReaderListener>> = match listener {
            Some(listener) => Some(Box::new(RtpsDataReaderListener(listener))),
            None => None,
        };
        let mask_listener = MaskListener::new(listener, status_mask);
        Self {
            rtps_reader_flavor,
            qos,
            mask_listener,
        }
    }
}
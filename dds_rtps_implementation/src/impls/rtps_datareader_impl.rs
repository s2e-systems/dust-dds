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

// pub struct RtpsDataReaderInner<T: DDSType> {
//     qos: DataReaderQos,
//     topic: Option<Arc<RtpsTopicInner>>,
//     listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
//     status_mask: StatusMask,
// }

// impl<T: DDSType> RtpsDataReaderInner<T> {
//     pub fn new(
//         topic: &Arc<RtpsTopicInner>,
//         qos: DataReaderQos,
//         listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
//         status_mask: StatusMask,
//     ) -> Self {
//         let topic = Some(topic.clone());

//         Self {
//             qos,
//             topic,
//             listener,
//             status_mask,
//         }
//     }
// }

// pub trait AnyRtpsDataReaderInner: Send + Sync {
//     fn topic(&mut self) -> &mut Option<Arc<RtpsTopicInner>>;

//     fn qos(&mut self) -> &mut DataReaderQos;

//     fn on_data_available(&self, data_reader_ref: RtpsAnyDataReaderInnerRef);
// }

// impl<T: DDSType> AnyRtpsDataReaderInner for RtpsDataReaderInner<T> {
//     fn topic(&mut self) -> &mut Option<Arc<RtpsTopicInner>> {
//         &mut self.topic
//     }

//     fn qos(&mut self) -> &mut DataReaderQos {
//         &mut self.qos
//     }

//     fn on_data_available(&self, _data_reader_ref: RtpsAnyDataReaderInnerRef) {
//         // let the_reader = RtpsDataReader {
//         //     parent_subscriber: None,
//         //     data_reader_ref,
//         //     phantom_data: PhantomData,
//         // };
//         // self.listener
//         //     .as_ref()
//         //     .unwrap()
//         //     .on_data_available(&the_reader)
//         todo!()
//     }
// }

// pub enum RtpsDataReaderFlavor {
//     Stateful(RtpsStatefulDataReaderInner),
//     Stateless(RtpsStatelessDataReaderInner),
// }

// impl RtpsDataReaderFlavor {
//     fn inner(&mut self) -> &mut dyn AnyRtpsDataReaderInner {
//         match self {
//             Self::Stateful(stateful) => stateful.inner.as_mut(),
//             Self::Stateless(stateless) => stateless.inner.as_mut(),
//         }
//     }
// }

// impl AnyRtpsDataReaderInner for RtpsDataReaderFlavor {
//     fn topic(&mut self) -> &mut Option<Arc<RtpsTopicInner>> {
//         self.inner().topic()
//     }

//     fn qos(&mut self) -> &mut DataReaderQos {
//         self.inner().qos()
//     }

//     fn on_data_available(&self, _data_reader_ref: RtpsAnyDataReaderInnerRef) {
//         todo!()
//     }
// }

// impl Deref for RtpsDataReaderFlavor {
//     type Target = Reader;

//     fn deref(&self) -> &Self::Target {
//         match self {
//             Self::Stateful(stateful) => &stateful.stateful_reader,
//             Self::Stateless(stateless) => &stateless.stateless_reader,
//         }
//     }
// }

// impl DerefMut for RtpsDataReaderFlavor {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         match self {
//             Self::Stateful(stateful) => &mut stateful.stateful_reader,
//             Self::Stateless(stateless) => &mut stateless.stateless_reader,
//         }
//     }
// }

// pub type RtpsAnyDataReaderInnerRef<'a> = MaybeValidRef<'a, Mutex<RtpsDataReaderFlavor>>;

// impl<'a> RtpsAnyDataReaderInnerRef<'a> {
//     pub fn get(&self) -> DDSResult<MutexGuard<RtpsDataReaderFlavor>> {
//         Ok(MaybeValid::get(self).ok_or(DDSError::AlreadyDeleted)?.lock().unwrap())
//     }

//     pub fn delete(&self) -> DDSResult<()> {
//         self.get()?.topic().take(); // Drop the topic
//         MaybeValid::delete(self);
//         Ok(())
//     }
// }

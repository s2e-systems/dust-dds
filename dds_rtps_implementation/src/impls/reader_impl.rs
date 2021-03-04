use std::sync::{Arc, Mutex};

use rust_dds_api::{
    dcps_psm::StatusMask, dds_type::DDSType, infrastructure::qos::DataReaderQos,
    subscription::data_reader_listener::DataReaderListener,
};
use rust_rtps::{
    behavior::{Reader, StatefulReader},
    structure::{Endpoint, Entity},
};

use super::{history_cache_impl::HistoryCacheImpl, mask_listener::MaskListener, topic_impl::TopicImpl};

struct RtpsDataReaderListener<T: DDSType>(Box<dyn DataReaderListener<DataType = T>>);
trait AnyDataReaderListener: Send + Sync {}

impl<T: DDSType> AnyDataReaderListener for RtpsDataReaderListener<T> {}

pub struct ReaderImpl {
    qos: DataReaderQos,
    mask_listener: MaskListener<Box<dyn AnyDataReaderListener>>,
    topic: Arc<Mutex<TopicImpl>>,
}

impl ReaderImpl {
    pub fn new<T: DDSType>(
        topic: Arc<Mutex<TopicImpl>>,
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
            qos,
            mask_listener,
            topic,
        }
    }
}

impl Entity for ReaderImpl {
    fn guid(&self) -> rust_rtps::types::GUID {
        todo!()
    }
}

impl Endpoint for ReaderImpl {
    fn unicast_locator_list(&self) -> &[rust_rtps::types::Locator] {
        todo!()
    }

    fn multicast_locator_list(&self) -> &[rust_rtps::types::Locator] {
        todo!()
    }

    fn topic_kind(&self) -> rust_rtps::types::TopicKind {
        todo!()
    }

    fn reliability_level(&self) -> rust_rtps::types::ReliabilityKind {
        todo!()
    }
}

impl Reader for ReaderImpl {
    type HistoryCacheType = HistoryCacheImpl;

    fn heartbeat_response_delay(&self) -> rust_rtps::behavior::types::Duration {
        todo!()
    }

    fn heartbeat_supression_duration(&self) -> rust_rtps::behavior::types::Duration {
        todo!()
    }

    fn reader_cache(&mut self) -> &mut HistoryCacheImpl {
        todo!()
    }

    fn expects_inline_qos(&self) -> bool {
        todo!()
    }
}
use std::sync::{Arc, Mutex};

use rust_dds_api::{
    dcps_psm::StatusMask, dds_type::DDSType, infrastructure::qos::DataReaderQos,
    subscription::data_reader_listener::DataReaderListener,
};
use rust_rtps::{
    behavior::{Reader, StatefulReader},
    structure::{Endpoint, Entity},
};

use super::{mask_listener::MaskListener, rtps_topic_impl::RtpsTopicImpl};

struct RtpsDataReaderListener<T: DDSType>(Box<dyn DataReaderListener<DataType = T>>);
trait AnyDataReaderListener: Send + Sync {}

impl<T: DDSType> AnyDataReaderListener for RtpsDataReaderListener<T> {}

pub struct RtpsStatefulDataReaderImpl {
    qos: DataReaderQos,
    mask_listener: MaskListener<Box<dyn AnyDataReaderListener>>,
    topic: Arc<Mutex<RtpsTopicImpl>>,
}

impl RtpsStatefulDataReaderImpl {
    pub fn new<T: DDSType>(
        topic: Arc<Mutex<RtpsTopicImpl>>,
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

impl Entity for RtpsStatefulDataReaderImpl {
    fn guid(&self) -> rust_rtps::types::GUID {
        todo!()
    }
}

impl Endpoint for RtpsStatefulDataReaderImpl {
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

impl Reader for RtpsStatefulDataReaderImpl {
    fn heartbeat_response_delay(&self) -> rust_rtps::behavior::types::Duration {
        todo!()
    }

    fn heartbeat_supression_duration(&self) -> rust_rtps::behavior::types::Duration {
        todo!()
    }

    fn reader_cache(&mut self) -> &mut rust_rtps::structure::HistoryCache {
        todo!()
    }

    fn expects_inline_qos(&self) -> bool {
        todo!()
    }
}

impl StatefulReader for RtpsStatefulDataReaderImpl {
    fn matched_writers(&self) -> &[rust_rtps::behavior::WriterProxy] {
        todo!()
    }

    fn matched_writer_add(&mut self, _a_writer_proxy: rust_rtps::behavior::WriterProxy) {
        todo!()
    }

    fn matched_writer_remove(&mut self, _writer_proxy_guid: &rust_rtps::types::GUID) {
        todo!()
    }

    fn matched_writer_lookup(
        &self,
        _a_writer_guid: rust_rtps::types::GUID,
    ) -> Option<&rust_rtps::behavior::WriterProxy> {
        todo!()
    }
}

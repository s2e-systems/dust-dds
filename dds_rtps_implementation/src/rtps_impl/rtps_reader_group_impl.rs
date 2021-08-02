use rust_dds_api::{
    dcps_psm::StatusMask, infrastructure::qos::SubscriberQos,
    subscription::subscriber_listener::SubscriberListener,
};
use rust_rtps_pim::structure::types::GUID;

use crate::utils::shared_object::{RtpsLock, RtpsShared};

use super::rtps_reader_impl::RtpsReaderImpl;

pub struct RtpsReaderGroupImpl {
    _guid: GUID,
    _qos: SubscriberQos,
    _listener: Option<&'static dyn SubscriberListener>,
    _status_mask: StatusMask,
}

impl RtpsReaderGroupImpl {
    pub fn new(
        guid: GUID,
        qos: SubscriberQos,
        listener: Option<&'static dyn SubscriberListener>,
        status_mask: StatusMask,
    ) -> Self {
        Self {
            _guid: guid,
            _qos: qos,
            _listener: listener,
            _status_mask: status_mask,
        }
    }
}

pub struct RTPSReaderIterator<'a>(std::slice::Iter<'a, RtpsShared<RtpsReaderImpl>>);

impl<'a> Iterator for RTPSReaderIterator<'a> {
    type Item = RtpsLock<'a, RtpsReaderImpl>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some(reader) => Some(reader.lock()),
            None => None,
        }
    }
}

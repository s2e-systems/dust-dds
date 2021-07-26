use rust_dds_api::{dcps_psm::{InstanceHandle, StatusMask}, infrastructure::qos::SubscriberQos, subscription::subscriber_listener::SubscriberListener};
use rust_rtps_pim::structure::{types::GUID, RTPSGroup};

use crate::utils::shared_object::{RtpsLock, RtpsShared};

use super::rtps_reader_impl::RTPSReaderImpl;

pub struct RTPSReaderGroupImpl {
    _guid: GUID,
    _qos: SubscriberQos,
    _listener: Option<&'static dyn SubscriberListener>,
    _status_mask: StatusMask,
    reader_list: Vec<RtpsShared<RTPSReaderImpl>>,
}

impl RTPSReaderGroupImpl {
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
            reader_list: Vec::new(),
        }
    }

    pub fn reader_list(&self) -> &[RtpsShared<RTPSReaderImpl>] {
        &self.reader_list
    }

    pub fn add_reader(&mut self, reader: RtpsShared<RTPSReaderImpl> ) {
        self.reader_list.push(reader)
    }

    pub fn delete_reader(&mut self, _reader: InstanceHandle) {
        todo!()
    }
}

impl<'a> RTPSGroup for &'a RTPSReaderGroupImpl {
    type Endpoints = RTPSReaderIterator<'a>;

    fn endpoints(self) -> Self::Endpoints {
        RTPSReaderIterator((&self.reader_list).into_iter())
    }
}

pub struct RTPSReaderIterator<'a>(std::slice::Iter<'a, RtpsShared<RTPSReaderImpl>>);

impl<'a> Iterator for RTPSReaderIterator<'a> {
    type Item = RtpsLock<'a, RTPSReaderImpl>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some(reader) => Some(reader.lock()),
            None => None,
        }
    }
}

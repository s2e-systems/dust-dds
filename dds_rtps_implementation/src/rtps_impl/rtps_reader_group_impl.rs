use rust_dds_api::{
    dcps_psm::StatusMask, infrastructure::qos::SubscriberQos,
    subscription::subscriber_listener::SubscriberListener,
};
use rust_rtps_pim::structure::{types::GUID, RTPSGroup};

use crate::utils::shared_object::{RtpsLock, RtpsShared};

use super::rtps_reader_impl::RTPSReaderImpl;

pub struct RTPSReaderGroupImpl {
    guid: GUID,
    qos: SubscriberQos,
    listener: Option<&'static dyn SubscriberListener>,
    status_mask: StatusMask,
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
            guid,
            qos,
            listener,
            status_mask,
            reader_list: Vec::new(),
        }
    }

    pub fn reader_list(&self) -> &[RtpsShared<RTPSReaderImpl>] {
        &self.reader_list
    }
}

impl<'a> RTPSGroup for &'a RTPSReaderGroupImpl {
    type Endpoints = RTPSReaderIterator<'a>;

    fn endpoints(self) -> Self::Endpoints {
        todo!()
        // RTPSReaderIterator {
        //     lock: self,
        //     index: 0,
        // }
    }
}

pub struct RTPSReaderIterator<'a> {
    lock: RtpsLock<'a, RTPSReaderGroupImpl>,
    index: usize,
}

impl<'a> Iterator for RTPSReaderIterator<'a> {
    type Item = RtpsLock<'a, RTPSReaderImpl>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
        // match self.0.next() {
        //     Some(reader) => Some(reader.lock()),
        //     None => None,
        // }
    }
}

use std::ops::{Deref, DerefMut};

use rust_rtps_pim::behavior::{
    stateless_writer::{RTPSReaderLocator, RTPSStatelessWriter},
    RTPSWriter,
};
use rust_rtps_udp_psm::RtpsUdpPsm;

use super::history_cache_impl::HistoryCacheImpl;

pub struct StatelessDataWriterImpl {
    writer: RTPSWriter<RtpsUdpPsm, HistoryCacheImpl>,
    reader_locators: Vec<RTPSReaderLocator<RtpsUdpPsm>>,
}

impl StatelessDataWriterImpl {
    pub fn new(writer: RTPSWriter<RtpsUdpPsm, HistoryCacheImpl>) -> Self {
        Self {
            writer,
            reader_locators: Vec::new(),
        }
    }
}

impl Deref for StatelessDataWriterImpl {
    type Target = RTPSWriter<RtpsUdpPsm, HistoryCacheImpl>;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl DerefMut for StatelessDataWriterImpl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

impl RTPSStatelessWriter<RtpsUdpPsm, HistoryCacheImpl> for StatelessDataWriterImpl {
    fn reader_locator_add(
        &mut self,
        a_locator: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::Locator,
    ) {
        let expects_inline_qos = false;
        self.reader_locators
            .push(RTPSReaderLocator::new(a_locator, expects_inline_qos));
    }

    fn reader_locator_remove(
        &mut self,
        a_locator: &<RtpsUdpPsm as rust_rtps_pim::structure::Types>::Locator,
    ) {
        self.reader_locators.retain(|x| x.locator() != a_locator)
    }

    fn reader_locators(
        &mut self,
    ) -> &mut [rust_rtps_pim::behavior::stateless_writer::RTPSReaderLocator<RtpsUdpPsm>] {
        &mut self.reader_locators
    }
}

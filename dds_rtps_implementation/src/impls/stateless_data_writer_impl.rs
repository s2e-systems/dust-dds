use std::ops::{Deref, DerefMut};

use rust_dds_api::{
    dcps_psm::{InstanceHandle, Time},
    dds_type::DDSType,
    return_type::DDSResult,
};
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

    pub fn register_instance_w_timestamp<T: DDSType>(
        &mut self,
        _instance: T,
        _timestamp: Time,
    ) -> DDSResult<Option<InstanceHandle>> {
        todo!()
    }

    pub fn write_w_timestamp<T: DDSType>(
        &mut self,
        _data: T,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DDSResult<()> {
        // let kind = ChangeKind::Alive;
        // let data = data.serialize();
        // let inline_qos = MyParameterList::new();
        // let handle = handle.unwrap_or(0);
        // let change = self.writer.new_change(kind, data, inline_qos, handle);

        // self.writer.writer_cache_mut().add_change(change);

        Ok(())
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

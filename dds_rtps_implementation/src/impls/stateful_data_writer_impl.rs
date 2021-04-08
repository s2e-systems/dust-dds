use std::ops::{Deref, DerefMut};

use rust_dds_api::return_type::DDSResult;
use rust_rtps_pim::behavior::RTPSWriter;
use rust_rtps_pim::structure::RTPSHistoryCache;
use rust_rtps_udp_psm::types::ChangeKind;
use rust_rtps_udp_psm::RtpsUdpPsm;

use super::history_cache_impl::HistoryCacheImpl;

pub struct StatefulDataWriterImpl {
    writer: RTPSWriter<RtpsUdpPsm, HistoryCacheImpl>,
}

impl StatefulDataWriterImpl {
    pub fn new(writer: RTPSWriter<RtpsUdpPsm, HistoryCacheImpl>) -> Self {
        Self { writer }
    }

    pub fn write_w_timestamp(&mut self) -> DDSResult<()> {
        let kind = ChangeKind::Alive;
        let data = vec![0, 1, 2];
        let inline_qos = vec![];
        let handle = 1;
        let change = self.new_change(kind, data, inline_qos, handle);
        self.writer_cache.add_change(change);
        Ok(())
    }
}

impl Deref for StatefulDataWriterImpl {
    type Target = RTPSWriter<RtpsUdpPsm, HistoryCacheImpl>;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl DerefMut for StatefulDataWriterImpl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

use rust_dds_api::{
    dcps_psm::{InstanceHandle, Time},
    dds_type::DDSType,
    return_type::DDSResult,
};
use rust_rtps::{
    behavior::RTPSWriter, messages::submessages::submessage_elements::ParameterList,
    structure::RTPSHistoryCache, types::ChangeKind,
};

use crate::rtps::{
    cache_change::CacheChange, history_cache::HistoryCache, reader_locator::ReaderLocator,
    stateless_writer::StatelessWriter, writer::Writer,
};

pub struct DataWriterImpl {
    writer: StatelessWriter<
        Writer<HistoryCache<CacheChange>>,
        ReaderLocator<Writer<HistoryCache<CacheChange>>>,
    >,
}

impl DataWriterImpl {
    pub fn new(
        writer: StatelessWriter<
            Writer<HistoryCache<CacheChange>>,
            ReaderLocator<Writer<HistoryCache<CacheChange>>>,
        >,
    ) -> Self {
        Self { writer }
    }

    pub fn register_instance_w_timestamp<T: DDSType>(
        &mut self,
        _instance: T,
        _timestamp: Time,
    ) -> DDSResult<Option<InstanceHandle>> {
        todo!()
    }

    pub fn write_w_timestamp<T: DDSType>(
        &self,
        data: T,
        handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DDSResult<()> {
        let kind = ChangeKind::Alive;
        let data = data.serialize();
        let inline_qos = ParameterList::new();
        let handle = handle.unwrap_or(0);
        let change = self.writer.new_change(kind, data, inline_qos, handle);

        self.writer.writer_cache().add_change(change);

        Ok(())
    }
}

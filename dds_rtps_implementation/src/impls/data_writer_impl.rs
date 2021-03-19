use std::ops::{Deref, DerefMut};

use rust_dds_api::{
    dcps_psm::{InstanceHandle, Time},
    dds_type::DDSType,
    return_type::DDSResult,
};
use rust_rtps::{behavior::RTPSWriter, structure::RTPSHistoryCache, types::ChangeKind};

use crate::rtps::{
    cache_change::{CacheChange, MyParameterList},
    history_cache::HistoryCache,
    reader_locator::ReaderLocator,
    stateless_writer::StatelessWriter,
    writer::Writer,
};

pub struct DataWriterImpl {
    writer: StatelessWriter<Writer<HistoryCache<CacheChange>>, ReaderLocator>,
}

impl DataWriterImpl {
    pub fn new(writer: StatelessWriter<Writer<HistoryCache<CacheChange>>, ReaderLocator>) -> Self {
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
        &mut self,
        data: T,
        handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DDSResult<()> {
        let kind = ChangeKind::Alive;
        let data = data.serialize();
        let inline_qos = MyParameterList::new();
        let handle = handle.unwrap_or(0);
        let change = self.writer.new_change(kind, data, inline_qos, handle);

        self.writer.writer_cache_mut().add_change(change);

        Ok(())
    }
}

impl Deref for DataWriterImpl {
    type Target = StatelessWriter<Writer<HistoryCache<CacheChange>>, ReaderLocator>;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl DerefMut for DataWriterImpl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

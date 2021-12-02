use std::ops::Deref;

use rust_rtps_pim::{behavior::{stateful_writer_behavior::ReliableStatefulWriterBehavior, writer::{
        reader_proxy::RtpsReaderProxy,
        stateful_writer::{RtpsStatefulWriter, RtpsStatefulWriterOperations},
        writer::RtpsWriterOperations,
    }}, messages::{submessage_elements::Parameter, submessages::HeartbeatSubmessage, types::Count}, structure::{
        cache_change::RtpsCacheChange,
        history_cache::RtpsHistoryCacheAddChange,
        types::{ChangeKind, Guid, InstanceHandle, Locator},
    }};

use crate::dds_type::DdsSerialize;

use super::{
    rtps_reader_proxy_impl::RtpsReaderProxyImpl,
    rtps_writer_history_cache_impl::{WriterHistoryCache, WriterHistoryCacheAddChangeMut},
};

pub struct RtpsStatefulWriterImpl {
    pub stateful_writer:
        RtpsStatefulWriter<Vec<Locator>, WriterHistoryCache, Vec<RtpsReaderProxyImpl>>,
    last_sent_heartbeat_instant: std::time::Instant,
    pub heartbeat_count: Count,
}
impl RtpsStatefulWriterImpl {
    pub fn new(
        stateful_writer: RtpsStatefulWriter<
            Vec<Locator>,
            WriterHistoryCache,
            Vec<RtpsReaderProxyImpl>,
        >,
    ) -> Self {
        Self {
            stateful_writer,
            last_sent_heartbeat_instant: std::time::Instant::now(),
            heartbeat_count: Count(0)
        }
    }

    pub fn try_create_heartbeat_submessage(&mut self) -> Option<HeartbeatSubmessage> {
        let mut heartbeat_submessage = None;
        if self.is_after_heartbeat_period() {
            ReliableStatefulWriterBehavior::send_heartbeat(
                &self.stateful_writer.writer,
                self.heartbeat_count,
                &mut |heartbeat| {
                    heartbeat_submessage = Some(heartbeat);
                },
            );
            self.increment_heartbeat_count();
            self.reset_heartbeat_instant();
        }
        heartbeat_submessage
    }

    fn is_after_heartbeat_period(&self) -> bool {
        if self.last_sent_heartbeat_instant.elapsed()
            > std::time::Duration::new(
                self.stateful_writer.writer.heartbeat_period.seconds as u64,
                self.stateful_writer.writer.heartbeat_period.fraction,
            )
        {
            true
        } else {
            false
        }
    }
    fn reset_heartbeat_instant(&mut self) {
        self.last_sent_heartbeat_instant = std::time::Instant::now();
    }
    fn increment_heartbeat_count(&mut self) {
        self.heartbeat_count += Count(1);
    }
}

impl RtpsStatefulWriterOperations<Vec<Locator>> for RtpsStatefulWriterImpl {
    fn matched_reader_add(&mut self, a_reader_proxy: RtpsReaderProxy<Vec<Locator>>) {
        let reader_proxy = RtpsReaderProxyImpl::new(a_reader_proxy);
        self.stateful_writer.matched_readers.push(reader_proxy)
    }

    fn matched_reader_remove(&mut self, reader_proxy_guid: &Guid) {
        self.stateful_writer
            .matched_readers
            .retain(|x| &x.remote_reader_guid != reader_proxy_guid);
    }

    fn matched_reader_lookup(
        &self,
        a_reader_guid: &Guid,
    ) -> Option<&RtpsReaderProxy<Vec<Locator>>> {
        self.stateful_writer
            .matched_readers
            .iter()
            .find(|&x| &x.remote_reader_guid == a_reader_guid)
            .map(|x| x.deref())
    }

    fn is_acked_by_all(&self) -> bool {
        todo!()
    }
}

impl RtpsWriterOperations for RtpsStatefulWriterImpl {
    fn new_change<'a, P, D>(
        &mut self,
        kind: ChangeKind,
        data: D,
        inline_qos: P,
        handle: InstanceHandle,
    ) -> RtpsCacheChange<P, D> {
        self.stateful_writer
            .writer
            .new_change(kind, data, inline_qos, handle)
    }
}

impl<T> WriterHistoryCacheAddChangeMut<'_, T> for RtpsStatefulWriterImpl
where
    T: DdsSerialize,
{
    fn get_writer_history_cache_add_change_mut(
        &'_ mut self,
    ) -> &mut dyn RtpsHistoryCacheAddChange<Vec<Parameter<Vec<u8>>>, &'_ T> {
        &mut self.stateful_writer.writer.writer_cache
    }
}

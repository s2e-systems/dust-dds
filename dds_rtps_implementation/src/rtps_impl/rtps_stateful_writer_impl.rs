use std::{cell::RefCell, ops::Deref};

use rust_rtps_pim::{
    behavior::{
        stateful_writer_behavior::ReliableStatefulWriterBehavior,
        writer::{
            reader_proxy::RtpsReaderProxy,
            stateful_writer::{RtpsStatefulWriter, RtpsStatefulWriterOperations},
            writer::RtpsWriterOperations,
        },
    },
    messages::{submessage_elements::Parameter, types::Count},
    structure::{
        cache_change::RtpsCacheChange,
        history_cache::RtpsHistoryCacheAddChange,
        types::{ChangeKind, Guid, InstanceHandle, Locator},
    },
};
use rust_rtps_psm::messages::{
    overall_structure::RtpsSubmessageTypeWrite,
    submessages::{DataSubmessageWrite, GapSubmessageWrite},
};

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
            heartbeat_count: Count(0),
        }
    }

    pub fn produce_submessages(&mut self) -> Vec<(&Locator, Vec<RtpsSubmessageTypeWrite<'_>>)> {
        let mut destined_submessages = Vec::new();

        let mut heartbeat_submessage = None;
        if self.last_sent_heartbeat_instant.elapsed()
            > std::time::Duration::new(
                self.stateful_writer.writer.heartbeat_period.seconds as u64,
                self.stateful_writer.writer.heartbeat_period.fraction,
            )
        {
            {
                ReliableStatefulWriterBehavior::send_heartbeat(
                    &self.stateful_writer.writer,
                    self.heartbeat_count,
                    &mut |heartbeat| {
                        heartbeat_submessage = Some(heartbeat);
                    },
                );
                self.heartbeat_count += Count(1);
                self.last_sent_heartbeat_instant = std::time::Instant::now();
            }
        }

        for reader_proxy in &mut self.stateful_writer.matched_readers {
            let submessages = RefCell::new(Vec::new());
            ReliableStatefulWriterBehavior::send_unsent_changes(
                reader_proxy,
                &self.stateful_writer.writer,
                |data| {
                    submessages.borrow_mut().push(RtpsSubmessageTypeWrite::Data(
                        DataSubmessageWrite::new(
                            data.endianness_flag,
                            data.inline_qos_flag,
                            data.data_flag,
                            data.key_flag,
                            data.non_standard_payload_flag,
                            data.reader_id,
                            data.writer_id,
                            data.writer_sn,
                            data.inline_qos,
                            data.serialized_payload,
                        ),
                    ))
                },
                |gap| {
                    submessages.borrow_mut().push(RtpsSubmessageTypeWrite::Gap(
                        GapSubmessageWrite::new(
                            gap.endianness_flag,
                            gap.reader_id,
                            gap.writer_id,
                            gap.gap_start,
                            gap.gap_list,
                        ),
                    ))
                },
            );
            let submessages = submessages.take();
            if !submessages.is_empty() {
                destined_submessages.push((&reader_proxy.unicast_locator_list[0], submessages));
            }
        }
        destined_submessages
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

use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

use rust_rtps_pim::{
    behavior::{
        writer::{
            reader_locator::{RtpsReaderLocator, RtpsReaderLocatorOperations},
            stateless_writer::{RtpsStatelessWriter, RtpsStatelessWriterOperations},
            writer::RtpsWriterOperations,
        },
    },
    messages::submessage_elements::Parameter,
    structure::{
        cache_change::RtpsCacheChange,
        history_cache::RtpsHistoryCacheAddChange,
        types::{ChangeKind, InstanceHandle, Locator, SequenceNumber},
    },
};
use rust_rtps_psm::messages::{
    overall_structure::RtpsSubmessageTypeWrite,
    submessages::{DataSubmessageWrite, GapSubmessageWrite},
};

use crate::dds_type::DdsSerialize;

use super::{
    rtps_reader_locator_impl::RtpsReaderLocatorImpl,
    rtps_writer_history_cache_impl::{WriterHistoryCache, WriterHistoryCacheAddChangeMut},
};

pub struct RtpsStatelessWriterImpl(
    pub RtpsStatelessWriter<Vec<Locator>, WriterHistoryCache, Vec<RtpsReaderLocatorImpl>>,
);

impl RtpsStatelessWriterImpl {
    pub fn new(
        stateless_writer: RtpsStatelessWriter<
            Vec<Locator>,
            WriterHistoryCache,
            Vec<RtpsReaderLocatorImpl>,
        >,
    ) -> Self {
        Self(stateless_writer)
    }

    pub fn produce_submessages(&mut self) -> Vec<RtpsSubmessageTypeWrite<'_>> {
        let destined_submessages = RefCell::new(Vec::new());

        for reader_locator in &mut self.0.reader_locators {
            let reader_locator_operations: &mut dyn RtpsReaderLocatorOperations<
                SequenceNumberVector = Vec<SequenceNumber>,
            > = reader_locator;
            reader_locator_operations.send_unsent_data(
                &self.0.writer.writer_cache,
                self.0.writer.last_change_sequence_number,
                &mut |data| {
                    destined_submessages
                        .borrow_mut()
                        .push(RtpsSubmessageTypeWrite::Data(DataSubmessageWrite::new(
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
                        )))
                },
                &mut |gap| {
                    destined_submessages
                        .borrow_mut()
                        .push(RtpsSubmessageTypeWrite::Gap(GapSubmessageWrite::new(
                            gap.endianness_flag,
                            gap.reader_id,
                            gap.writer_id,
                            gap.gap_start,
                            gap.gap_list,
                        )))
                },
            )
        }
        destined_submessages.take()
    }
}

impl Deref for RtpsStatelessWriterImpl {
    type Target = RtpsStatelessWriter<Vec<Locator>, WriterHistoryCache, Vec<RtpsReaderLocatorImpl>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RtpsStatelessWriterImpl {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RtpsStatelessWriterOperations for RtpsStatelessWriterImpl {
    fn reader_locator_add(&mut self, a_locator: RtpsReaderLocator) {
        let reader_locator_impl = RtpsReaderLocatorImpl::new(a_locator);
        self.reader_locators.push(reader_locator_impl);
    }

    fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.reader_locators.retain(|x| &x.locator != a_locator)
    }

    fn unsent_changes_reset(&mut self) {
        for reader_locator in &mut self.reader_locators {
            reader_locator.unsent_changes_reset()
        }
    }
}

impl RtpsWriterOperations for RtpsStatelessWriterImpl {
    fn new_change<'a, P, D>(
        &mut self,
        kind: ChangeKind,
        data: D,
        inline_qos: P,
        handle: InstanceHandle,
    ) -> RtpsCacheChange<P, D> {
        self.0.writer.new_change(kind, data, inline_qos, handle)
    }
}

impl<T> WriterHistoryCacheAddChangeMut<'_, T> for RtpsStatelessWriterImpl
where
    T: DdsSerialize,
{
    fn get_writer_history_cache_add_change_mut(
        &'_ mut self,
    ) -> &mut dyn RtpsHistoryCacheAddChange<Vec<Parameter<Vec<u8>>>, &'_ T> {
        &mut self.0.writer.writer_cache
    }
}

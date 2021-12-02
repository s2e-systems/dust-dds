use std::cell::RefCell;

use rust_rtps_pim::{
    behavior::{
        stateless_writer_behavior::BestEffortStatelessWriterBehavior,
        writer::{
            reader_locator::RtpsReaderLocator,
            stateless_writer::{RtpsStatelessWriter, RtpsStatelessWriterOperations},
            writer::RtpsWriterOperations,
        },
    },
    messages::submessage_elements::Parameter,
    structure::{
        cache_change::RtpsCacheChange,
        history_cache::RtpsHistoryCacheAddChange,
        types::{ChangeKind, InstanceHandle, Locator},
    },
};
use rust_rtps_psm::messages::overall_structure::RtpsSubmessageTypeWrite;

use crate::{dds_impl::publisher_impl::SubmessageProducer, dds_type::DdsSerialize};

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
}

impl<'a> SubmessageProducer<'a> for RtpsStatelessWriterImpl {
    type DestinedSubmessages = Vec<(&'a RtpsReaderLocator, Vec<RtpsSubmessageTypeWrite<'a>>)>;

    fn produce_submessages(&'a mut self) -> Self::DestinedSubmessages {
        let mut destined_submessages = Vec::new();

        for reader_locator_impl in &mut self.0.reader_locators {
            let submessages = RefCell::new(Vec::new());
            BestEffortStatelessWriterBehavior::send_unsent_changes(
                reader_locator_impl,
                &self.0.writer,
                |data| {
                    submessages
                        .borrow_mut()
                        .push(RtpsSubmessageTypeWrite::from(data))
                },
                |gap| {
                    submessages
                        .borrow_mut()
                        .push(RtpsSubmessageTypeWrite::from(gap))
                },
            );
            let submessages = submessages.take();
            if !submessages.is_empty() {
                destined_submessages.push((&reader_locator_impl.reader_locator, submessages));
            }
        }
        destined_submessages
    }
}

impl RtpsStatelessWriterOperations for RtpsStatelessWriterImpl {
    fn reader_locator_add(&mut self, a_locator: RtpsReaderLocator) {
        let reader_locator_impl = RtpsReaderLocatorImpl::new(a_locator);
        self.0.reader_locators.push(reader_locator_impl);
    }

    fn reader_locator_remove(&mut self, a_locator: &Locator) {
        self.0.reader_locators.retain(|x| &x.locator != a_locator)
    }

    fn unsent_changes_reset(&mut self) {
        for reader_locator in &mut self.0.reader_locators {
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

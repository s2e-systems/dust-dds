use std::cell::RefCell;

use rust_dds_api::{
    dcps_psm::InstanceHandle,
    infrastructure::{entity::Entity, qos::DataWriterQos},
    publication::{
        data_writer::DataWriter, data_writer_listener::DataWriterListener, publisher::Publisher,
    },
    return_type::DDSResult,
    topic::topic::Topic,
};
use rust_rtps_pim::{
    behavior::{
        stateful_writer_behavior::ReliableStatefulWriterBehavior,
        stateless_writer_behavior::BestEffortStatelessWriterBehavior,
        writer::{
            reader_locator::RtpsReaderLocator, reader_proxy::RtpsReaderProxy,
            writer::RtpsWriterOperations,
        },
    },
    messages::types::Count,
    structure::types::{ChangeKind, Locator},
};
use rust_rtps_psm::messages::overall_structure::RtpsSubmessageTypeWrite;

use crate::{
    dds_type::DdsSerialize,
    rtps_impl::{
        rtps_stateful_writer_impl::RtpsStatefulWriterImpl,
        rtps_stateless_writer_impl::RtpsStatelessWriterImpl,
        rtps_writer_history_cache_impl::WriterHistoryCacheAddChangeMut,
    },
};

use super::publisher_impl::{StatefulWriterSubmessageProducer, StatelessWriterSubmessageProducer};

impl<T, W> AsRef<W> for DataWriterImpl<T, W> {
    fn as_ref(&self) -> &W {
        &self.rtps_writer_impl
    }
}

impl<T, W> AsMut<W> for DataWriterImpl<T, W> {
    fn as_mut(&mut self) -> &mut W {
        &mut self.rtps_writer_impl
    }
}

pub struct DataWriterImpl<T, W> {
    _qos: DataWriterQos,
    rtps_writer_impl: W,
    _listener: Option<Box<dyn DataWriterListener<DataType = T> + Send + Sync>>,
    last_sent_heartbeat_instant: std::time::Instant,
    heartbeat_count: Count,
}

impl<T, W> DataWriterImpl<T, W>
where
    T: Send + 'static,
{
    pub fn new(qos: DataWriterQos, rtps_writer_impl: W) -> Self {
        Self {
            _qos: qos,
            rtps_writer_impl,
            _listener: None,
            last_sent_heartbeat_instant: std::time::Instant::now(),
            heartbeat_count: Count(1),
        }
    }
}

impl<T, W> DataWriter<T> for DataWriterImpl<T, W>
where
    T: DdsSerialize,
    W: RtpsWriterOperations + for<'a> WriterHistoryCacheAddChangeMut<'a, T>,
{
    fn register_instance(&mut self, _instance: T) -> DDSResult<Option<InstanceHandle>> {
        unimplemented!()
    }

    fn register_instance_w_timestamp(
        &mut self,
        _instance: T,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<Option<InstanceHandle>> {
        todo!()
    }

    fn unregister_instance(
        &mut self,
        _instance: T,
        _handle: Option<InstanceHandle>,
    ) -> DDSResult<()> {
        unimplemented!()
    }

    fn unregister_instance_w_timestamp(
        &mut self,
        _instance: T,
        _handle: Option<InstanceHandle>,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_key_value(&self, _key_holder: &mut T, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn lookup_instance(&self, _instance: &T) -> DDSResult<Option<InstanceHandle>> {
        todo!()
    }

    fn write(&mut self, _data: &T, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        unimplemented!()
    }

    fn write_w_timestamp(
        &mut self,
        data: &T,
        _handle: Option<InstanceHandle>,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<()> {
        let change = self
            .rtps_writer_impl
            .new_change(ChangeKind::Alive, data, vec![], 0);
        self.rtps_writer_impl
            .get_writer_history_cache_add_change_mut()
            .add_change(change);
        Ok(())
    }

    fn dispose(&mut self, _data: T, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        unimplemented!()
    }

    fn dispose_w_timestamp(
        &mut self,
        _data: T,
        _handle: Option<InstanceHandle>,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<()> {
        todo!()
    }

    fn wait_for_acknowledgments(
        &self,
        _max_wait: rust_dds_api::dcps_psm::Duration,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_liveliness_lost_status(
        &self,
        _status: &mut rust_dds_api::dcps_psm::LivelinessLostStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_offered_deadline_missed_status(
        &self,
        _status: &mut rust_dds_api::dcps_psm::OfferedDeadlineMissedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_offered_incompatible_qos_status(
        &self,
        _status: &mut rust_dds_api::dcps_psm::OfferedIncompatibleQosStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_publication_matched_status(
        &self,
        _status: &mut rust_dds_api::dcps_psm::PublicationMatchedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_topic(&self) -> &dyn Topic<T> {
        unimplemented!()
    }

    fn get_publisher(&self) -> &dyn Publisher {
        unimplemented!()
    }

    fn assert_liveliness(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_matched_subscription_data(
        &self,
        _subscription_data: rust_dds_api::builtin_topics::SubscriptionBuiltinTopicData,
        _subscription_handle: InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_matched_subscriptions(
        &self,
        _subscription_handles: &mut [InstanceHandle],
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<T, W> Entity for DataWriterImpl<T, W> {
    type Qos = DataWriterQos;
    type Listener = Box<dyn DataWriterListener<DataType = T>>;

    fn set_qos(&mut self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        // let qos = qos.unwrap_or_default();
        // qos.is_consistent()?;
        // self.qos = qos;
        // Ok(())
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // &self.qos
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: rust_dds_api::dcps_psm::StatusMask,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(
        &self,
    ) -> DDSResult<rust_dds_api::infrastructure::entity::StatusCondition> {
        todo!()
    }

    fn get_status_changes(&self) -> DDSResult<rust_dds_api::dcps_psm::StatusMask> {
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
    }
}

impl<T> StatelessWriterSubmessageProducer for DataWriterImpl<T, RtpsStatelessWriterImpl> {
    fn produce_submessages(
        &mut self,
    ) -> Vec<(&'_ RtpsReaderLocator, Vec<RtpsSubmessageTypeWrite<'_>>)> {
        let mut destined_submessages = Vec::new();

        for reader_locator_impl in &mut self.rtps_writer_impl.0.reader_locators {
            let submessages = RefCell::new(Vec::new());
            BestEffortStatelessWriterBehavior::send_unsent_changes(
                reader_locator_impl,
                &self.rtps_writer_impl.0.writer,
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

impl<T> StatefulWriterSubmessageProducer for DataWriterImpl<T, RtpsStatefulWriterImpl> {
    fn produce_submessages(
        &mut self,
    ) -> Vec<(
        &'_ RtpsReaderProxy<Vec<Locator>>,
        Vec<RtpsSubmessageTypeWrite<'_>>,
    )> {
        let mut destined_submessages = Vec::new();

        let mut heartbeat_submessage = None;
        if self.last_sent_heartbeat_instant.elapsed()
            > std::time::Duration::new(
                self.rtps_writer_impl
                    .stateful_writer
                    .writer
                    .heartbeat_period
                    .seconds as u64,
                self.rtps_writer_impl
                    .stateful_writer
                    .writer
                    .heartbeat_period
                    .fraction,
            )
        {
            {
                ReliableStatefulWriterBehavior::send_heartbeat(
                    &self.rtps_writer_impl.stateful_writer.writer,
                    self.heartbeat_count,
                    &mut |heartbeat| {
                        heartbeat_submessage = Some(heartbeat);
                    },
                );
                self.heartbeat_count += Count(1);
                self.last_sent_heartbeat_instant = std::time::Instant::now();
            }
        }

        for reader_proxy in &mut self.rtps_writer_impl.stateful_writer.matched_readers {
            let submessages = RefCell::new(Vec::new());
            ReliableStatefulWriterBehavior::send_unsent_changes(
                reader_proxy,
                &self.rtps_writer_impl.stateful_writer.writer,
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
            let mut submessages = submessages.take();

            // Add heartbeat to the submessages to be sent to every proxy
            if let Some(heartbeat_submessage) = heartbeat_submessage.clone() {
                submessages.push(RtpsSubmessageTypeWrite::from(heartbeat_submessage));
            }

            if !submessages.is_empty() {
                destined_submessages.push((&reader_proxy.reader_proxy, submessages));
            }
        }
        destined_submessages
    }
}

#[cfg(test)]
mod tests {

    use rust_rtps_pim::{
        messages::submessage_elements::Parameter,
        structure::{
            cache_change::RtpsCacheChange,
            history_cache::RtpsHistoryCacheAddChange,
            types::{InstanceHandle, GUID_UNKNOWN},
        },
    };

    use super::*;

    #[test]
    fn write_w_timestamp() {
        struct MockData(Vec<u8>);

        impl DdsSerialize for MockData {
            fn serialize<W: std::io::Write, R: crate::dds_type::Endianness>(
                &self,
                mut writer: W,
            ) -> DDSResult<()> {
                writer.write(&self.0).unwrap();
                Ok(())
            }
        }

        struct MockWriterCache;

        impl<T> RtpsHistoryCacheAddChange<Vec<Parameter<Vec<u8>>>, &'_ T> for MockWriterCache {
            fn add_change(&mut self, _change: RtpsCacheChange<Vec<Parameter<Vec<u8>>>, &'_ T>) {}
        }

        struct MockWriter {
            cache: MockWriterCache,
        }

        impl RtpsWriterOperations for MockWriter {
            fn new_change<'a, P, D>(
                &mut self,
                kind: ChangeKind,
                data: D,
                inline_qos: P,
                handle: InstanceHandle,
            ) -> RtpsCacheChange<P, D> {
                RtpsCacheChange {
                    data_value: data,
                    kind,
                    instance_handle: handle,
                    inline_qos,
                    sequence_number: 1,
                    writer_guid: GUID_UNKNOWN,
                }
            }
        }

        impl<T> WriterHistoryCacheAddChangeMut<'_, T> for MockWriter {
            fn get_writer_history_cache_add_change_mut(
                &'_ mut self,
            ) -> &mut dyn RtpsHistoryCacheAddChange<Vec<Parameter<Vec<u8>>>, &'_ T> {
                &mut self.cache
            }
        }

        let mut dds_data_writer = DataWriterImpl::new(
            DataWriterQos::default(),
            MockWriter {
                cache: MockWriterCache,
            },
        );

        let data_value = MockData(vec![0, 1, 0, 0, 7, 3]);
        dds_data_writer
            .write_w_timestamp(
                &data_value,
                None,
                rust_dds_api::dcps_psm::Time { sec: 0, nanosec: 0 },
            )
            .unwrap();
    }
}

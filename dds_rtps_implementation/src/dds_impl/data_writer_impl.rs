use rust_dds_api::{
    dcps_psm::InstanceHandle,
    infrastructure::qos::DataWriterQos,
    publication::{data_writer::DataWriter, publisher::Publisher},
    return_type::DDSResult,
    topic::topic::Topic,
};
use rust_rtps_pim::{
    behavior::writer::writer::{RtpsWriter, RtpsWriterOperations},
    structure::{types::ChangeKind, RtpsHistoryCache},
};

use crate::{dds_type::DDSType, rtps_impl::rtps_writer_impl::RtpsWriterImpl};

pub struct DataWriterImpl {
    qos: DataWriterQos,
    rtps_writer_impl: RtpsWriterImpl,
}

impl DataWriterImpl {
    pub fn new(qos: DataWriterQos, rtps_writer_impl: RtpsWriterImpl) -> Self {
        Self {
            qos,
            rtps_writer_impl,
        }
    }

    /// Get a reference to the data writer storage's rtps data writer.
    pub fn rtps_data_writer(&self) -> &RtpsWriterImpl {
        &self.rtps_writer_impl
    }

    /// Get a mutable reference to the data writer storage's rtps data writer.
    pub fn rtps_data_writer_mut(&mut self) -> &mut RtpsWriterImpl {
        &mut self.rtps_writer_impl
    }

    pub fn set_qos(&mut self, qos: Option<DataWriterQos>) -> DDSResult<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        self.qos = qos;
        Ok(())
    }

    pub fn get_qos(&self) -> &DataWriterQos {
        &self.qos
    }

    pub fn write_w_timestamp<T: DDSType + 'static>(
        &mut self,
        _data: T,
        _handle: Option<InstanceHandle>,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<()> {
        todo!()
        // let data = cdr::serialize::<_, _, cdr::CdrLe>(&data, cdr::Infinite).unwrap();
        // let change = self
        //     .rtps_writer_impl
        //     .new_change(ChangeKind::Alive, data.as_slice(), &[], 0);
        // let writer_cache = self.rtps_writer_impl.writer_cache_mut();
        // let time = rust_rtps_pim::messages::types::Time(0);
        // writer_cache.set_source_timestamp(Some(time));
        // writer_cache.add_change(&change);
        // Ok(())
    }
}

impl<T> DataWriter<T> for DataWriterImpl {
    fn register_instance(&self, _instance: T) -> DDSResult<Option<InstanceHandle>> {
        unimplemented!()
    }

    fn register_instance_w_timestamp(
        &self,
        _instance: T,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<Option<InstanceHandle>> {
        todo!()
    }

    fn unregister_instance(&self, _instance: T, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        unimplemented!()
    }

    fn unregister_instance_w_timestamp(
        &self,
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

    fn write(&self, _data: T, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        unimplemented!()
    }

    fn write_w_timestamp(
        &self,
        _data: T,
        _handle: Option<InstanceHandle>,
        _timestamp: rust_dds_api::dcps_psm::Time,
    ) -> DDSResult<()> {
        todo!()
    }

    fn dispose(&self, _data: T, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        unimplemented!()
    }

    fn dispose_w_timestamp(
        &self,
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

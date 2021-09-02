use rust_dds_api::{
    infrastructure::{entity::Entity, qos::DataReaderQos},
    return_type::DDSResult,
    subscription::{data_reader::DataReader, data_reader_listener::DataReaderListener},
};

use crate::rtps_impl::rtps_reader_impl::RtpsReaderImpl;

pub struct DataReaderImpl {
    rtps_reader: RtpsReaderImpl,
    qos: DataReaderQos,
}

impl DataReaderImpl {
    pub fn new(rtps_reader: RtpsReaderImpl, qos: DataReaderQos) -> Self {
        Self { rtps_reader, qos }
    }

    /// Get a reference to the data reader storage's reader.
    pub fn rtps_reader(&self) -> &RtpsReaderImpl {
        &self.rtps_reader
    }

    /// Get a mutable reference to the data reader storage's reader.
    pub fn rtps_reader_mut(&mut self) -> &mut RtpsReaderImpl {
        &mut self.rtps_reader
    }

    pub fn set_qos(&mut self, qos: Option<DataReaderQos>) -> DDSResult<()> {
        self.qos = qos.unwrap_or_default();
        Ok(())
    }

    pub fn get_qos(&self) -> DDSResult<&DataReaderQos> {
        Ok(&self.qos)
    }
}

// let shared_reader = self.reader.upgrade()?;
// let mut reader = shared_reader.lock();
// let reader_cache = reader.rtps_reader_mut().reader_cache_mut();
// Ok(reader_cache
//     .changes_mut()
//     .iter()
//     .map(|cc| {
//         let data = cc.data();
//         let value = cdr::deserialize(data).unwrap();
//         let sample_info = SampleInfo {
//             sample_state: *cc.sample_state_kind(),
//             view_state: *cc.view_state_kind(),
//             instance_state: *cc.instance_state_kind(),
//             disposed_generation_count: 0,
//             no_writers_generation_count: 0,
//             sample_rank: 0,
//             generation_rank: 0,
//             absolute_generation_rank: 0,
//             source_timestamp: Time { sec: 0, nanosec: 0 },
//             instance_handle: 0,
//             publication_handle: 0,
//             valid_data: true,
//         };
//         (value, sample_info)
//     })
//     .collect())

impl<T> DataReader<T> for DataReaderImpl {
    type Samples = Vec<()>;

    fn read(
        &self,
        _max_samples: i32,
        _sample_states: &[rust_dds_api::dcps_psm::SampleStateKind],
        _view_states: &[rust_dds_api::dcps_psm::ViewStateKind],
        _instance_states: &[rust_dds_api::dcps_psm::InstanceStateKind],
    ) -> DDSResult<Self::Samples> {
        todo!()
    }

    fn take(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
        _max_samples: i32,
        _sample_states: &[rust_dds_api::dcps_psm::SampleStateKind],
        _view_states: &[rust_dds_api::dcps_psm::ViewStateKind],
        _instance_states: &[rust_dds_api::dcps_psm::InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    fn read_w_condition(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
        _max_samples: i32,
        _a_condition: rust_dds_api::infrastructure::read_condition::ReadCondition,
    ) -> DDSResult<()> {
        todo!()
    }

    fn take_w_condition(
        &self,
        data_values: &mut [T],
        sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
        max_samples: i32,
        a_condition: rust_dds_api::infrastructure::read_condition::ReadCondition,
    ) -> DDSResult<()> {
        todo!()
    }

    fn read_next_sample(
        &self,
        data_value: &mut [T],
        sample_info: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
    ) -> DDSResult<()> {
        todo!()
    }

    fn take_next_sample(
        &self,
        data_value: &mut [T],
        sample_info: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
    ) -> DDSResult<()> {
        todo!()
    }

    fn read_instance(
        &self,
        data_values: &mut [T],
        sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
        max_samples: i32,
        a_handle: rust_dds_api::dcps_psm::InstanceHandle,
        sample_states: &[rust_dds_api::dcps_psm::SampleStateKind],
        view_states: &[rust_dds_api::dcps_psm::ViewStateKind],
        instance_states: &[rust_dds_api::dcps_psm::InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    fn take_instance(
        &self,
        data_values: &mut [T],
        sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
        max_samples: i32,
        a_handle: rust_dds_api::dcps_psm::InstanceHandle,
        sample_states: &[rust_dds_api::dcps_psm::SampleStateKind],
        view_states: &[rust_dds_api::dcps_psm::ViewStateKind],
        instance_states: &[rust_dds_api::dcps_psm::InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    fn read_next_instance(
        &self,
        data_values: &mut [T],
        sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
        max_samples: i32,
        previous_handle: rust_dds_api::dcps_psm::InstanceHandle,
        sample_states: &[rust_dds_api::dcps_psm::SampleStateKind],
        view_states: &[rust_dds_api::dcps_psm::ViewStateKind],
        instance_states: &[rust_dds_api::dcps_psm::InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    fn take_next_instance(
        &self,
        data_values: &mut [T],
        sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
        max_samples: i32,
        previous_handle: rust_dds_api::dcps_psm::InstanceHandle,
        sample_states: &[rust_dds_api::dcps_psm::SampleStateKind],
        view_states: &[rust_dds_api::dcps_psm::ViewStateKind],
        instance_states: &[rust_dds_api::dcps_psm::InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    fn read_next_instance_w_condition(
        &self,
        data_values: &mut [T],
        sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
        max_samples: i32,
        previous_handle: rust_dds_api::dcps_psm::InstanceHandle,
        a_condition: rust_dds_api::infrastructure::read_condition::ReadCondition,
    ) -> DDSResult<()> {
        todo!()
    }

    fn take_next_instance_w_condition(
        &self,
        data_values: &mut [T],
        sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
        max_samples: i32,
        previous_handle: rust_dds_api::dcps_psm::InstanceHandle,
        a_condition: rust_dds_api::infrastructure::read_condition::ReadCondition,
    ) -> DDSResult<()> {
        todo!()
    }

    fn return_loan(
        &self,
        data_values: &mut [T],
        sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_key_value(
        &self,
        key_holder: &mut T,
        handle: rust_dds_api::dcps_psm::InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn lookup_instance(&self, instance: &T) -> rust_dds_api::dcps_psm::InstanceHandle {
        todo!()
    }

    fn create_readcondition(
        &self,
        sample_states: &[rust_dds_api::dcps_psm::SampleStateKind],
        view_states: &[rust_dds_api::dcps_psm::ViewStateKind],
        instance_states: &[rust_dds_api::dcps_psm::InstanceStateKind],
    ) -> rust_dds_api::infrastructure::read_condition::ReadCondition {
        todo!()
    }

    fn create_querycondition(
        &self,
        sample_states: &[rust_dds_api::dcps_psm::SampleStateKind],
        view_states: &[rust_dds_api::dcps_psm::ViewStateKind],
        instance_states: &[rust_dds_api::dcps_psm::InstanceStateKind],
        query_expression: &'static str,
        query_parameters: &[&'static str],
    ) -> rust_dds_api::subscription::query_condition::QueryCondition {
        todo!()
    }

    fn delete_readcondition(
        &self,
        a_condition: rust_dds_api::infrastructure::read_condition::ReadCondition,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_liveliness_changed_status(
        &self,
        status: &mut rust_dds_api::dcps_psm::LivelinessChangedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_requested_deadline_missed_status(
        &self,
        status: &mut rust_dds_api::dcps_psm::RequestedDeadlineMissedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_requested_incompatible_qos_status(
        &self,
        status: &mut rust_dds_api::dcps_psm::RequestedIncompatibleQosStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_sample_lost_status(
        &self,
        status: &mut rust_dds_api::dcps_psm::SampleLostStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_sample_rejected_status(
        &self,
        status: &mut rust_dds_api::dcps_psm::SampleRejectedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_subscription_matched_status(
        &self,
        status: &mut rust_dds_api::dcps_psm::SubscriptionMatchedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_topicdescription(
        &self,
    ) -> &dyn rust_dds_api::topic::topic_description::TopicDescription<T> {
        todo!()
    }

    fn get_subscriber(&self) -> &dyn rust_dds_api::subscription::subscriber::Subscriber {
        todo!()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn wait_for_historical_data(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_matched_publication_data(
        &self,
        publication_data: &mut rust_dds_api::builtin_topics::PublicationBuiltinTopicData,
        publication_handle: rust_dds_api::dcps_psm::InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_match_publication(
        &self,
        publication_handles: &mut [rust_dds_api::dcps_psm::InstanceHandle],
    ) -> DDSResult<()> {
        todo!()
    }
}

impl Entity for DataReaderImpl {
    type Qos = DataReaderQos;

    type Listener = &'static dyn DataReaderListener<DataPIM = ()>;

    fn set_qos(&mut self, qos: Option<Self::Qos>) -> DDSResult<()> {
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        todo!()
    }

    fn set_listener(
        &self,
        a_listener: Option<Self::Listener>,
        mask: rust_dds_api::dcps_psm::StatusMask,
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

    fn get_instance_handle(&self) -> DDSResult<rust_dds_api::dcps_psm::InstanceHandle> {
        todo!()
    }
}

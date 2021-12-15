use rust_dds_api::{
    dcps_psm::{SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus},
    infrastructure::{entity::Entity, qos::DataReaderQos},
    return_type::{DDSError, DDSResult},
    subscription::{
        data_reader::{DataReader, DataReaderBorrowedSamples},
        data_reader_listener::DataReaderListener,
    },
    topic::topic_description::TopicDescription,
};
use rust_rtps_pim::{
    behavior::{reader::reader::RtpsReader, stateless_reader_behavior::StatelessReaderBehavior},
    structure::types::{GuidPrefix, Locator},
};
use rust_rtps_psm::messages::submessages::DataSubmessageRead;

use crate::{
    dds_type::DdsDeserialize,
    rtps_impl::{
        rtps_reader_history_cache_impl::{ReaderHistoryCache, ReaderHistoryCacheGetChange},
        rtps_stateless_reader_impl::RtpsStatelessReaderImpl, rtps_stateful_reader_impl::RtpsStatefulReaderImpl,
    },
    utils::message_receiver::ProcessDataSubmessage,
};

pub type RtpsReaderType<Foo> = RtpsReader<Vec<Locator>, ReaderHistoryCache<Foo>>;

pub struct DataReaderImpl<Foo, R> {
    rtps_reader: R,
    _qos: DataReaderQos,
    _listener: Option<Box<dyn DataReaderListener<DataType = Foo> + Send + Sync>>,
}

impl<Foo, R> AsRef<R> for DataReaderImpl<Foo, R> {
    fn as_ref(&self) -> &R {
        &self.rtps_reader
    }
}

impl<Foo, R> AsMut<R> for DataReaderImpl<Foo, R> {
    fn as_mut(&mut self) -> &mut R {
        &mut self.rtps_reader
    }
}

impl<Foo> ProcessDataSubmessage for DataReaderImpl<Foo, RtpsStatelessReaderImpl<Foo>>
where
    Foo: for<'a> DdsDeserialize<'a>,
{
    fn process_data_submessage(
        &mut self,
        source_guid_prefix: GuidPrefix,
        data: &DataSubmessageRead,
    ) {
        self.rtps_reader.0.receive_data(source_guid_prefix, data)
    }
}

impl<Foo> ProcessDataSubmessage for DataReaderImpl<Foo, RtpsStatefulReaderImpl<Foo>>
where
    Foo: for<'a> DdsDeserialize<'a>,
{
    fn process_data_submessage(
        &mut self,
        _source_guid_prefix: GuidPrefix,
        _data: &DataSubmessageRead,
    ) {
        todo!("Stateful reader process data submessage")
        // self.rtps_reader.0.receive_data(source_guid_prefix, data)
    }
}

impl<Foo, R> DataReaderImpl<Foo, R> {
    pub fn new(qos: DataReaderQos, rtps_reader: R) -> Self {
        Self {
            rtps_reader,
            _qos: qos,
            _listener: None,
        }
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

impl<'a, Foo, R> DataReaderBorrowedSamples<'a> for DataReaderImpl<Foo, R>
where
    Foo: 'static,
    R: ReaderHistoryCacheGetChange<'a, Foo>,
{
    type Samples = Vec<&'a Foo>;

    fn read_borrowed_samples(
        &'a self,
        max_samples: i32,
        sample_states: &[rust_dds_api::dcps_psm::SampleStateKind],
        view_states: &[rust_dds_api::dcps_psm::ViewStateKind],
        instance_states: &[rust_dds_api::dcps_psm::InstanceStateKind],
    ) -> DDSResult<Self::Samples> {
        if let Some(cc) = self
            .rtps_reader
            .get_reader_history_cache_get_change()
            .get_change(&1)
        {
            Ok(vec![cc.data_value])
        } else {
            Err(DDSError::NoData)
        }
    }
}

impl<Foo, R> DataReader<Foo> for DataReaderImpl<Foo, R>
where
    Foo: for<'de> DdsDeserialize<'de> + 'static,
    R: for<'a> ReaderHistoryCacheGetChange<'a, Foo>,
{
    fn take(
        &self,
        _data_values: &mut [Foo],
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
        _data_values: &mut [Foo],
        _sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
        _max_samples: i32,
        _a_condition: rust_dds_api::infrastructure::read_condition::ReadCondition,
    ) -> DDSResult<()> {
        todo!()
    }

    fn take_w_condition(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
        _max_samples: i32,
        _a_condition: rust_dds_api::infrastructure::read_condition::ReadCondition,
    ) -> DDSResult<()> {
        todo!()
    }

    fn read_next_sample(
        &self,
        _data_value: &mut [Foo],
        _sample_info: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
    ) -> DDSResult<()> {
        todo!()
    }

    fn take_next_sample(
        &self,
        _data_value: &mut [Foo],
        _sample_info: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
    ) -> DDSResult<()> {
        todo!()
    }

    fn read_instance(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
        _max_samples: i32,
        _a_handle: rust_dds_api::dcps_psm::InstanceHandle,
        _sample_states: &[rust_dds_api::dcps_psm::SampleStateKind],
        _view_states: &[rust_dds_api::dcps_psm::ViewStateKind],
        _instance_states: &[rust_dds_api::dcps_psm::InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    fn take_instance(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
        _max_samples: i32,
        _a_handle: rust_dds_api::dcps_psm::InstanceHandle,
        _sample_states: &[rust_dds_api::dcps_psm::SampleStateKind],
        _view_states: &[rust_dds_api::dcps_psm::ViewStateKind],
        _instance_states: &[rust_dds_api::dcps_psm::InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    fn read_next_instance(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
        _max_samples: i32,
        _previous_handle: rust_dds_api::dcps_psm::InstanceHandle,
        _sample_states: &[rust_dds_api::dcps_psm::SampleStateKind],
        _view_states: &[rust_dds_api::dcps_psm::ViewStateKind],
        _instance_states: &[rust_dds_api::dcps_psm::InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    fn take_next_instance(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
        _max_samples: i32,
        _previous_handle: rust_dds_api::dcps_psm::InstanceHandle,
        _sample_states: &[rust_dds_api::dcps_psm::SampleStateKind],
        _view_states: &[rust_dds_api::dcps_psm::ViewStateKind],
        _instance_states: &[rust_dds_api::dcps_psm::InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    fn read_next_instance_w_condition(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
        _max_samples: i32,
        _previous_handle: rust_dds_api::dcps_psm::InstanceHandle,
        _a_condition: rust_dds_api::infrastructure::read_condition::ReadCondition,
    ) -> DDSResult<()> {
        todo!()
    }

    fn take_next_instance_w_condition(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
        _max_samples: i32,
        _previous_handle: rust_dds_api::dcps_psm::InstanceHandle,
        _a_condition: rust_dds_api::infrastructure::read_condition::ReadCondition,
    ) -> DDSResult<()> {
        todo!()
    }

    fn return_loan(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [rust_dds_api::infrastructure::sample_info::SampleInfo],
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_key_value(
        &self,
        _key_holder: &mut Foo,
        _handle: rust_dds_api::dcps_psm::InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn lookup_instance(&self, _instance: &Foo) -> rust_dds_api::dcps_psm::InstanceHandle {
        todo!()
    }

    fn create_readcondition(
        &self,
        _sample_states: &[rust_dds_api::dcps_psm::SampleStateKind],
        _view_states: &[rust_dds_api::dcps_psm::ViewStateKind],
        _instance_states: &[rust_dds_api::dcps_psm::InstanceStateKind],
    ) -> rust_dds_api::infrastructure::read_condition::ReadCondition {
        todo!()
    }

    fn create_querycondition(
        &self,
        _sample_states: &[rust_dds_api::dcps_psm::SampleStateKind],
        _view_states: &[rust_dds_api::dcps_psm::ViewStateKind],
        _instance_states: &[rust_dds_api::dcps_psm::InstanceStateKind],
        _query_expression: &'static str,
        _query_parameters: &[&'static str],
    ) -> rust_dds_api::subscription::query_condition::QueryCondition {
        todo!()
    }

    fn delete_readcondition(
        &self,
        _a_condition: rust_dds_api::infrastructure::read_condition::ReadCondition,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_liveliness_changed_status(
        &self,
        _status: &mut rust_dds_api::dcps_psm::LivelinessChangedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_requested_deadline_missed_status(
        &self,
        _status: &mut rust_dds_api::dcps_psm::RequestedDeadlineMissedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_requested_incompatible_qos_status(
        &self,
        _status: &mut rust_dds_api::dcps_psm::RequestedIncompatibleQosStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_sample_lost_status(&self, _status: &mut SampleLostStatus) -> DDSResult<()> {
        todo!()
    }

    fn get_sample_rejected_status(&self, _status: &mut SampleRejectedStatus) -> DDSResult<()> {
        todo!()
    }

    fn get_subscription_matched_status(
        &self,
        _status: &mut SubscriptionMatchedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_topicdescription(&self) -> &dyn TopicDescription<Foo> {
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
        _publication_data: &mut rust_dds_api::builtin_topics::PublicationBuiltinTopicData,
        _publication_handle: rust_dds_api::dcps_psm::InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_match_publication(
        &self,
        _publication_handles: &mut [rust_dds_api::dcps_psm::InstanceHandle],
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<Foo, R> Entity for DataReaderImpl<Foo, R> {
    type Qos = DataReaderQos;

    type Listener = Box<dyn DataReaderListener<DataType = Foo>>;

    fn set_qos(&mut self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
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

    fn get_instance_handle(&self) -> DDSResult<rust_dds_api::dcps_psm::InstanceHandle> {
        todo!()
    }
}

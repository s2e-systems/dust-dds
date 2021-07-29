use rust_dds_api::{
    builtin_topics::PublicationBuiltinTopicData,
    dcps_psm::{
        InstanceHandle, InstanceStateKind, LivelinessChangedStatus, RequestedDeadlineMissedStatus,
        RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus, SampleStateKind,
        StatusMask, SubscriptionMatchedStatus, ViewStateKind,
    },
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::DataReaderQos,
        read_condition::ReadCondition,
        sample_info::SampleInfo,
    },
    return_type::DDSResult,
    subscription::{
        data_reader::AnyDataReader, data_reader_listener::DataReaderListener,
        query_condition::QueryCondition, subscriber::Subscriber,
    },
    topic::topic_description::TopicDescription,
};

use crate::utils::shared_object::RtpsWeak;

use super::data_reader_storage::DataReaderStorage;

pub struct DataReaderImpl<'dr, T: 'static> {
    _subscriber: &'dr dyn Subscriber,
    _topic: &'dr dyn TopicDescription<T>,
    reader: RtpsWeak<DataReaderStorage>,
}

impl<'dr, T> rust_dds_api::subscription::data_reader::DataReader<T> for DataReaderImpl<'dr, T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    type Samples = Vec<(T, SampleInfo)>;

    fn read(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DDSResult<Self::Samples> {
        let shared_reader = self.reader.upgrade()?;
        let mut reader = shared_reader.lock();
        reader.read(max_samples, sample_states, view_states, instance_states)
    }

    fn take(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    fn read_w_condition(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_condition: ReadCondition,
    ) -> DDSResult<()> {
        todo!()
    }

    fn take_w_condition(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_condition: ReadCondition,
    ) -> DDSResult<()> {
        todo!()
    }

    fn read_next_sample(
        &self,
        _data_value: &mut [T],
        _sample_info: &mut [SampleInfo],
    ) -> DDSResult<()> {
        todo!()
    }

    fn take_next_sample(
        &self,
        _data_value: &mut [T],
        _sample_info: &mut [SampleInfo],
    ) -> DDSResult<()> {
        todo!()
    }

    fn read_instance(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_handle: InstanceHandle,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    fn take_instance(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_handle: InstanceHandle,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    fn read_next_instance(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    fn take_next_instance(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    fn read_next_instance_w_condition(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _a_condition: ReadCondition,
    ) -> DDSResult<()> {
        todo!()
    }

    fn take_next_instance_w_condition(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _a_condition: ReadCondition,
    ) -> DDSResult<()> {
        todo!()
    }

    fn return_loan(
        &self,
        _data_values: &mut [T],
        _sample_infos: &mut [SampleInfo],
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_key_value(&self, _key_holder: &mut T, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn lookup_instance(&self, _instance: &T) -> InstanceHandle {
        todo!()
    }

    fn create_readcondition(
        &self,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> ReadCondition {
        todo!()
    }

    fn create_querycondition(
        &self,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
        _query_expression: &'static str,
        _query_parameters: &[&'static str],
    ) -> QueryCondition {
        todo!()
    }

    fn delete_readcondition(&self, _a_condition: ReadCondition) -> DDSResult<()> {
        todo!()
    }

    fn get_liveliness_changed_status(
        &self,
        _status: &mut LivelinessChangedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_requested_deadline_missed_status(
        &self,
        _status: &mut RequestedDeadlineMissedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_requested_incompatible_qos_status(
        &self,
        _status: &mut RequestedIncompatibleQosStatus,
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

    // fn get_topicdescription(&self) -> &dyn TopicDescription {
    //     todo!()
    // }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn wait_for_historical_data(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_matched_publication_data(
        &self,
        _publication_data: &mut PublicationBuiltinTopicData,
        _publication_handle: InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_match_publication(&self, _publication_handles: &mut [InstanceHandle]) -> DDSResult<()> {
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
}

impl<'dr, T> Entity for DataReaderImpl<'dr, T> {
    type Qos = DataReaderQos;
    type Listener = &'static dyn DataReaderListener<DataPIM = T>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DDSResult<()> {
        self.reader
            .upgrade()?
            .lock()
            .set_qos(qos.unwrap_or_default())
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        Ok(self.reader.upgrade()?.lock().qos().clone())
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(&self) -> StatusCondition {
        todo!()
    }

    fn get_status_changes(&self) -> StatusMask {
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
    }
}

impl<'dr, T> AnyDataReader for DataReaderImpl<'dr, T> {}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use rust_dds_api::{
        infrastructure::qos::{SubscriberQos, TopicQos},
        topic::topic_listener::TopicListener,
    };
    use rust_rtps_pim::structure::{types::GUID_UNKNOWN, RTPSHistoryCache, RtpsCacheChange};

    struct MockSubcriber;
    impl rust_dds_api::subscription::subscriber::Subscriber for MockSubcriber {
        fn begin_access(&self) -> rust_dds_api::return_type::DDSResult<()> {
            todo!()
        }

        fn end_access(&self) -> rust_dds_api::return_type::DDSResult<()> {
            todo!()
        }

        fn get_datareaders(
            &self,
            _readers: &mut [&mut dyn rust_dds_api::subscription::data_reader::AnyDataReader],
            _sample_states: &[rust_dds_api::dcps_psm::SampleStateKind],
            _view_states: &[rust_dds_api::dcps_psm::ViewStateKind],
            _instance_states: &[rust_dds_api::dcps_psm::InstanceStateKind],
        ) -> rust_dds_api::return_type::DDSResult<()> {
            todo!()
        }

        fn notify_datareaders(&self) -> rust_dds_api::return_type::DDSResult<()> {
            todo!()
        }

        fn get_participant(
            &self,
        ) -> &dyn rust_dds_api::domain::domain_participant::DomainParticipant {
            todo!()
        }

        fn get_sample_lost_status(
            &self,
            _status: &mut rust_dds_api::dcps_psm::SampleLostStatus,
        ) -> rust_dds_api::return_type::DDSResult<()> {
            todo!()
        }

        fn delete_contained_entities(&self) -> rust_dds_api::return_type::DDSResult<()> {
            todo!()
        }

        fn set_default_datareader_qos(
            &self,
            _qos: Option<rust_dds_api::infrastructure::qos::DataReaderQos>,
        ) -> rust_dds_api::return_type::DDSResult<()> {
            todo!()
        }

        fn get_default_datareader_qos(
            &self,
        ) -> rust_dds_api::return_type::DDSResult<rust_dds_api::infrastructure::qos::DataReaderQos>
        {
            todo!()
        }

        fn copy_from_topic_qos(
            &self,
            _a_datareader_qos: &mut rust_dds_api::infrastructure::qos::DataReaderQos,
            _a_topic_qos: &rust_dds_api::infrastructure::qos::TopicQos,
        ) -> rust_dds_api::return_type::DDSResult<()> {
            todo!()
        }
    }

    impl rust_dds_api::infrastructure::entity::Entity for MockSubcriber {
        type Qos = SubscriberQos;
        type Listener =
            &'static dyn rust_dds_api::subscription::subscriber_listener::SubscriberListener;

        fn set_qos(&self, _qos: Option<Self::Qos>) -> rust_dds_api::return_type::DDSResult<()> {
            todo!()
        }

        fn get_qos(&self) -> rust_dds_api::return_type::DDSResult<Self::Qos> {
            todo!()
        }

        fn set_listener(
            &self,
            _a_listener: Option<Self::Listener>,
            _mask: rust_dds_api::dcps_psm::StatusMask,
        ) -> rust_dds_api::return_type::DDSResult<()> {
            todo!()
        }

        fn get_listener(&self) -> rust_dds_api::return_type::DDSResult<Option<Self::Listener>> {
            todo!()
        }

        fn get_statuscondition(&self) -> rust_dds_api::infrastructure::entity::StatusCondition {
            todo!()
        }

        fn get_status_changes(&self) -> rust_dds_api::dcps_psm::StatusMask {
            todo!()
        }

        fn enable(&self) -> rust_dds_api::return_type::DDSResult<()> {
            todo!()
        }

        fn get_instance_handle(
            &self,
        ) -> rust_dds_api::return_type::DDSResult<rust_dds_api::dcps_psm::InstanceHandle> {
            todo!()
        }
    }

    struct MockTopic<T>(PhantomData<T>);

    impl<T: 'static> rust_dds_api::topic::topic_description::TopicDescription<T> for MockTopic<T> {
        fn get_participant(
            &self,
        ) -> &dyn rust_dds_api::domain::domain_participant::DomainParticipant {
            todo!()
        }

        fn get_type_name(&self) -> rust_dds_api::return_type::DDSResult<&'static str> {
            todo!()
        }

        fn get_name(&self) -> rust_dds_api::return_type::DDSResult<&str> {
            todo!()
        }
    }

    impl<T: 'static> rust_dds_api::infrastructure::entity::Entity for MockTopic<T> {
        type Qos = TopicQos;
        type Listener = &'static dyn TopicListener<DataPIM = T>;

        fn set_qos(&self, _qos: Option<Self::Qos>) -> rust_dds_api::return_type::DDSResult<()> {
            todo!()
        }

        fn get_qos(&self) -> rust_dds_api::return_type::DDSResult<Self::Qos> {
            todo!()
        }

        fn set_listener(
            &self,
            _a_listener: Option<Self::Listener>,
            _mask: rust_dds_api::dcps_psm::StatusMask,
        ) -> rust_dds_api::return_type::DDSResult<()> {
            todo!()
        }

        fn get_listener(&self) -> rust_dds_api::return_type::DDSResult<Option<Self::Listener>> {
            todo!()
        }

        fn get_statuscondition(&self) -> rust_dds_api::infrastructure::entity::StatusCondition {
            todo!()
        }

        fn get_status_changes(&self) -> rust_dds_api::dcps_psm::StatusMask {
            todo!()
        }

        fn enable(&self) -> rust_dds_api::return_type::DDSResult<()> {
            todo!()
        }

        fn get_instance_handle(
            &self,
        ) -> rust_dds_api::return_type::DDSResult<rust_dds_api::dcps_psm::InstanceHandle> {
            todo!()
        }
    }

    struct MockHistoryCache(());

    impl RTPSHistoryCache for MockHistoryCache {
        fn new() -> Self
        where
            Self: Sized,
        {
            todo!()
        }

        fn add_change(&mut self, _change: &RtpsCacheChange) {
            todo!()
        }

        fn remove_change(&mut self, _seq_num: &rust_rtps_pim::structure::types::SequenceNumber) {
            todo!()
        }

        fn get_change(
            &self,
            _seq_num: &rust_rtps_pim::structure::types::SequenceNumber,
        ) -> Option<RtpsCacheChange> {
            Some(RtpsCacheChange::new(
                rust_rtps_pim::structure::types::ChangeKind::Alive,
                GUID_UNKNOWN,
                0,
                1,
                &[1],
                &[],
            ))
        }

        fn get_seq_num_min(&self) -> Option<rust_rtps_pim::structure::types::SequenceNumber> {
            Some(1)
        }

        fn get_seq_num_max(&self) -> Option<rust_rtps_pim::structure::types::SequenceNumber> {
            Some(1)
        }
    }

    struct MockRtpsReader(MockHistoryCache);

    impl rust_rtps_pim::behavior::reader::reader::RTPSReader for MockRtpsReader {
        type HistoryCacheType = MockHistoryCache;

        fn heartbeat_response_delay(&self) -> &rust_rtps_pim::behavior::types::Duration {
            todo!()
        }

        fn heartbeat_supression_duration(&self) -> &rust_rtps_pim::behavior::types::Duration {
            todo!()
        }

        fn reader_cache(&self) -> &Self::HistoryCacheType {
            &self.0
        }

        fn reader_cache_mut(&mut self) -> &mut Self::HistoryCacheType {
            todo!()
        }

        fn expects_inline_qos(&self) -> bool {
            todo!()
        }
    }

    // #[test]
    // fn read() {
    //     let reader = DataReaderStorage {};
    //     let shared_reader = RtpsShared::new(reader);

    //     let data_reader = DataReaderImpl::<u8> {
    //         _subscriber: &MockSubcriber,
    //         _topic: &MockTopic(PhantomData),
    //         reader: shared_reader.downgrade(),
    //     };

    //     let sample = data_reader.read(1, &[], &[], &[]).unwrap();
    //     assert_eq!(sample[0].0, 1);
    // }
}

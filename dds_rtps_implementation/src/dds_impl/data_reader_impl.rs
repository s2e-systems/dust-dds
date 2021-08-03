use rust_dds_api::{
    builtin_topics::PublicationBuiltinTopicData,
    dcps_psm::{
        InstanceHandle, InstanceStateKind, LivelinessChangedStatus, RequestedDeadlineMissedStatus,
        RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus, SampleStateKind,
        StatusMask, SubscriptionMatchedStatus, Time, ViewStateKind,
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
use rust_rtps_pim::behavior::reader::reader::RtpsReader;

use crate::{rtps_impl::rtps_reader_impl::RtpsReaderImpl, utils::shared_object::RtpsWeak};

pub struct DataReaderStorage {
    rtps_reader: RtpsReaderImpl,
    qos: DataReaderQos,
}

impl DataReaderStorage {
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
}

pub struct DataReaderImpl<'dr, T: 'static> {
    subscriber: &'dr dyn Subscriber,
    topic: &'dr dyn TopicDescription<T>,
    reader: RtpsWeak<DataReaderStorage>,
}

impl<'dr, T: 'static> DataReaderImpl<'dr, T> {
    pub fn new(
        subscriber: &'dr dyn Subscriber,
        topic: &'dr dyn TopicDescription<T>,
        reader: RtpsWeak<DataReaderStorage>,
    ) -> Self {
        Self {
            subscriber,
            topic,
            reader,
        }
    }
}

impl<'dr, T> rust_dds_api::subscription::data_reader::DataReader<T> for DataReaderImpl<'dr, T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    type Samples = Vec<(T, SampleInfo)>;

    fn read(
        &self,
        _max_samples: i32,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DDSResult<Self::Samples> {
        let shared_reader = self.reader.upgrade()?;
        let mut reader = shared_reader.lock();
        let reader_cache = reader.rtps_reader_mut().reader_cache_mut();
        Ok(reader_cache
            .changes_mut()
            .iter()
            .map(|cc| {
                let data = cc.data();
                let value = cdr::deserialize(data).unwrap();
                let sample_info = SampleInfo {
                    sample_state: *cc.sample_state_kind(),
                    view_state: *cc.view_state_kind(),
                    instance_state: *cc.instance_state_kind(),
                    disposed_generation_count: 0,
                    no_writers_generation_count: 0,
                    sample_rank: 0,
                    generation_rank: 0,
                    absolute_generation_rank: 0,
                    source_timestamp: Time { sec: 0, nanosec: 0 },
                    instance_handle: 0,
                    publication_handle: 0,
                    valid_data: true,
                };
                (value, sample_info)
            })
            .collect())
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
        self.topic
    }

    fn get_subscriber(&self) -> &dyn rust_dds_api::subscription::subscriber::Subscriber {
        self.subscriber
    }
}

impl<'dr, T> Entity for DataReaderImpl<'dr, T> {
    type Qos = DataReaderQos;
    type Listener = &'static dyn DataReaderListener<DataPIM = T>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DDSResult<()> {
        self.reader.upgrade()?.lock().qos = qos.unwrap_or_default();
        Ok(())
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        Ok(self.reader.upgrade()?.lock().qos.clone())
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

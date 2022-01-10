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
    behavior::reader::reader::RtpsReaderAttributes,
    structure::{
        cache_change::RtpsCacheChangeAttributes, history_cache::RtpsHistoryCacheGetChange,
    },
};

use crate::{
    dds_type::DdsDeserialize,
    rtps_impl::{
        rtps_stateful_reader_impl::RtpsStatefulReaderImpl,
        rtps_stateless_reader_impl::RtpsStatelessReaderImpl,
    },
};

pub struct Samples<Foo> {
    samples: Vec<Foo>,
}

impl<Foo> std::ops::Deref for Samples<Foo> {
    type Target = [Foo];

    fn deref(&self) -> &Self::Target {
        &self.samples
    }
}

pub enum RtpsReader {
    Stateless(RtpsStatelessReaderImpl),
    Stateful(RtpsStatefulReaderImpl),
}

impl RtpsReader {
    pub fn try_as_stateless_reader(&mut self) -> DDSResult<&mut RtpsStatelessReaderImpl> {
        match self {
            RtpsReader::Stateless(x) => Ok(x),
            RtpsReader::Stateful(_) => Err(DDSError::PreconditionNotMet(
                "Not a stateless reader".to_string(),
            )),
        }
    }

    pub fn try_as_stateful_reader(&mut self) -> DDSResult<&mut RtpsStatefulReaderImpl> {
        match self {
            RtpsReader::Stateless(_) => Err(DDSError::PreconditionNotMet(
                "Not a stateful reader".to_string(),
            )),
            RtpsReader::Stateful(x) => Ok(x),
        }
    }
}

pub struct DataReaderImpl<Foo> {
    rtps_reader: RtpsReader,
    _qos: DataReaderQos,
    _listener: Option<Box<dyn DataReaderListener<DataType = Foo> + Send + Sync>>,
}

impl<Foo> AsRef<RtpsReader> for DataReaderImpl<Foo> {
    fn as_ref(&self) -> &RtpsReader {
        &self.rtps_reader
    }
}

impl<Foo> AsMut<RtpsReader> for DataReaderImpl<Foo> {
    fn as_mut(&mut self) -> &mut RtpsReader {
        &mut self.rtps_reader
    }
}

impl<Foo> DataReaderImpl<Foo> {
    pub fn new(qos: DataReaderQos, rtps_reader: RtpsReader) -> Self {
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

impl<'a, Foo> DataReaderBorrowedSamples<'a> for DataReaderImpl<Foo>
where
    Foo: for<'de> DdsDeserialize<'de> + 'static,
{
    type Samples = Samples<Foo>;

    fn read_borrowed_samples(
        &'a mut self,
        _max_samples: i32,
        _sample_states: &[rust_dds_api::dcps_psm::SampleStateKind],
        _view_states: &[rust_dds_api::dcps_psm::ViewStateKind],
        _instance_states: &[rust_dds_api::dcps_psm::InstanceStateKind],
    ) -> DDSResult<Self::Samples> {
        match &self.rtps_reader {
            RtpsReader::Stateless(rtps_reader) => {
                if let Some(cc) = rtps_reader.reader_cache().get_change(&1) {
                    Ok(Samples {
                        samples: vec![DdsDeserialize::deserialize(&mut cc.data_value()).unwrap()],
                    })
                } else {
                    Err(DDSError::NoData)
                }
            }
            RtpsReader::Stateful(rtps_reader) => {
                if let Some(cc) = rtps_reader.reader_cache().get_change(&1) {
                    Ok(Samples {
                        samples: vec![DdsDeserialize::deserialize(&mut cc.data_value()).unwrap()],
                    })
                } else {
                    Err(DDSError::NoData)
                }
            }
        }
    }
}

impl<Foo> DataReader<Foo> for DataReaderImpl<Foo>
where
    Foo: for<'de> DdsDeserialize<'de> + 'static,
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

impl<Foo> Entity for DataReaderImpl<Foo> {
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

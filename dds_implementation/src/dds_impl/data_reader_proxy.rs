use std::{marker::PhantomData, ops::DerefMut};

use crate::{
    dds_type::DdsDeserialize,
    utils::{
        rtps_structure::RtpsStructure,
        shared_object::{DdsRwLock, DdsShared, DdsWeak},
    },
};
use dds_api::{
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
    return_type::{DDSError, DDSResult},
    subscription::{
        data_reader::{DataReader, DataReaderGetSubscriber, DataReaderGetTopicDescription},
        data_reader_listener::DataReaderListener,
        query_condition::QueryCondition,
    },
};
use rtps_pim::{
    behavior::reader::reader::RtpsReaderAttributes,
    structure::{
        cache_change::RtpsCacheChangeAttributes,
        history_cache::{RtpsHistoryCacheAttributes, RtpsHistoryCacheOperations},
    },
};

use super::{
    subscriber_proxy::{SubscriberAttributes, SubscriberProxy},
    topic_proxy::{TopicAttributes, TopicProxy},
};

pub trait AnyDataReaderListener<Rtps>
where
    Rtps: RtpsStructure,
{
    fn trigger_on_data_available(&self, reader: DdsShared<DataReaderAttributes<Rtps>>);
    fn trigger_on_sample_rejected(
        &self,
        reader: DdsShared<DataReaderAttributes<Rtps>>,
        status: SampleRejectedStatus,
    );
    fn trigger_on_liveliness_changed(
        &self,
        reader: DdsShared<DataReaderAttributes<Rtps>>,
        status: LivelinessChangedStatus,
    );
    fn trigger_on_requested_deadline_missed(
        &self,
        reader: DdsShared<DataReaderAttributes<Rtps>>,
        status: RequestedDeadlineMissedStatus,
    );
    fn trigger_on_requested_incompatible_qos(
        &self,
        reader: DdsShared<DataReaderAttributes<Rtps>>,
        status: RequestedIncompatibleQosStatus,
    );
    fn trigger_on_subscription_matched(
        &self,
        reader: DdsShared<DataReaderAttributes<Rtps>>,
        status: SubscriptionMatchedStatus,
    );
    fn trigger_on_sample_lost(
        &self,
        reader: DdsShared<DataReaderAttributes<Rtps>>,
        status: SampleLostStatus,
    );
}

impl<Foo, Rtps> AnyDataReaderListener<Rtps> for Box<dyn DataReaderListener<Foo = Foo> + Send + Sync>
where
    Foo: for<'de> DdsDeserialize<'de> + 'static,
    Rtps: RtpsStructure,
{
    fn trigger_on_data_available(&self, reader: DdsShared<DataReaderAttributes<Rtps>>) {
        let data_reader = DataReaderProxy::new(reader.downgrade());
        self.on_data_available(&data_reader)
    }

    fn trigger_on_sample_rejected(
        &self,
        reader: DdsShared<DataReaderAttributes<Rtps>>,
        status: SampleRejectedStatus,
    ) {
        let data_reader = DataReaderProxy::new(reader.downgrade());
        self.on_sample_rejected(&data_reader, status)
    }

    fn trigger_on_liveliness_changed(
        &self,
        reader: DdsShared<DataReaderAttributes<Rtps>>,
        status: LivelinessChangedStatus,
    ) {
        let data_reader = DataReaderProxy::new(reader.downgrade());
        self.on_liveliness_changed(&data_reader, status)
    }

    fn trigger_on_requested_deadline_missed(
        &self,
        reader: DdsShared<DataReaderAttributes<Rtps>>,
        status: RequestedDeadlineMissedStatus,
    ) {
        let data_reader = DataReaderProxy::new(reader.downgrade());
        self.on_requested_deadline_missed(&data_reader, status)
    }

    fn trigger_on_requested_incompatible_qos(
        &self,
        reader: DdsShared<DataReaderAttributes<Rtps>>,
        status: RequestedIncompatibleQosStatus,
    ) {
        let data_reader = DataReaderProxy::new(reader.downgrade());
        self.on_requested_incompatible_qos(&data_reader, status)
    }

    fn trigger_on_subscription_matched(
        &self,
        reader: DdsShared<DataReaderAttributes<Rtps>>,
        status: SubscriptionMatchedStatus,
    ) {
        let data_reader = DataReaderProxy::new(reader.downgrade());
        self.on_subscription_matched(&data_reader, status)
    }

    fn trigger_on_sample_lost(
        &self,
        reader: DdsShared<DataReaderAttributes<Rtps>>,
        status: SampleLostStatus,
    ) {
        let data_reader = DataReaderProxy::new(reader.downgrade());
        self.on_sample_lost(&data_reader, status)
    }
}

pub enum RtpsReader<Rtps>
where
    Rtps: RtpsStructure,
{
    Stateless(Rtps::StatelessReader),
    Stateful(Rtps::StatefulReader),
}

impl<Rtps> RtpsReader<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn try_as_stateless_reader(&mut self) -> DDSResult<&mut Rtps::StatelessReader> {
        match self {
            RtpsReader::Stateless(x) => Ok(x),
            RtpsReader::Stateful(_) => Err(DDSError::PreconditionNotMet(
                "Not a stateless reader".to_string(),
            )),
        }
    }

    pub fn try_as_stateful_reader(&mut self) -> DDSResult<&mut Rtps::StatefulReader> {
        match self {
            RtpsReader::Stateless(_) => Err(DDSError::PreconditionNotMet(
                "Not a stateful reader".to_string(),
            )),
            RtpsReader::Stateful(x) => Ok(x),
        }
    }
}

pub struct DataReaderAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub rtps_reader: DdsRwLock<RtpsReader<Rtps>>,
    pub _qos: DataReaderQos,
    pub topic: DdsShared<TopicAttributes<Rtps>>,
    pub listener: DdsRwLock<Option<Box<dyn AnyDataReaderListener<Rtps> + Send + Sync>>>,
    pub parent_subscriber: DdsWeak<SubscriberAttributes<Rtps>>,
    pub status: DdsRwLock<SubscriptionMatchedStatus>,
}

impl<Rtps> DataReaderAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(
        qos: DataReaderQos,
        rtps_reader: RtpsReader<Rtps>,
        topic: DdsShared<TopicAttributes<Rtps>>,
        listener: Option<Box<dyn AnyDataReaderListener<Rtps> + Send + Sync>>,
        parent_subscriber: DdsWeak<SubscriberAttributes<Rtps>>,
    ) -> Self {
        Self {
            rtps_reader: DdsRwLock::new(rtps_reader),
            _qos: qos,
            topic,
            listener: DdsRwLock::new(listener),
            parent_subscriber,
            status: DdsRwLock::new(SubscriptionMatchedStatus {
                total_count: 0,
                total_count_change: 0,
                last_publication_handle: 0,
                current_count: 0,
                current_count_change: 0,
            }),
        }
    }
}

pub struct DataReaderProxy<Foo, Rtps>
where
    Rtps: RtpsStructure,
{
    data_reader_impl: DdsWeak<DataReaderAttributes<Rtps>>,
    phantom: PhantomData<Foo>,
}

// Not automatically derived because in that case it is only available if Foo: Clone
impl<Foo, Rtps> Clone for DataReaderProxy<Foo, Rtps>
where
    Rtps: RtpsStructure,
{
    fn clone(&self) -> Self {
        Self {
            data_reader_impl: self.data_reader_impl.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<Foo, Rtps> DataReaderProxy<Foo, Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(data_reader_impl: DdsWeak<DataReaderAttributes<Rtps>>) -> Self {
        Self {
            data_reader_impl,
            phantom: PhantomData,
        }
    }
}

impl<Foo, Rtps> AsRef<DdsWeak<DataReaderAttributes<Rtps>>> for DataReaderProxy<Foo, Rtps>
where
    Rtps: RtpsStructure,
{
    fn as_ref(&self) -> &DdsWeak<DataReaderAttributes<Rtps>> {
        &self.data_reader_impl
    }
}

impl<Foo, Rtps> DataReaderGetSubscriber for DataReaderProxy<Foo, Rtps>
where
    Rtps: RtpsStructure,
{
    type Subscriber = SubscriberProxy<Rtps>;

    fn data_reader_get_subscriber(&self) -> DDSResult<Self::Subscriber> {
        todo!()
    }
}

impl<Foo, Rtps> DataReaderGetTopicDescription for DataReaderProxy<Foo, Rtps>
where
    Rtps: RtpsStructure,
{
    type TopicDescription = TopicProxy<Foo, Rtps>;

    fn data_reader_get_topicdescription(&self) -> DDSResult<Self::TopicDescription> {
        todo!()
    }
}

impl<Foo, Rtps> DataReader<Foo> for DataReaderProxy<Foo, Rtps>
where
    Foo: for<'de> DdsDeserialize<'de> + 'static,
    Rtps: RtpsStructure,
{
    fn read(
        &self,
        max_samples: i32,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DDSResult<Vec<(Foo, SampleInfo)>> {
        let data_reader_shared = self.data_reader_impl.upgrade()?;
        let mut rtps_reader = data_reader_shared.rtps_reader.write_lock();
        match rtps_reader.deref_mut() {
            RtpsReader::Stateless(rtps_reader) => {
                let samples = rtps_reader
                    .reader_cache()
                    .changes()
                    .iter()
                    .take(max_samples as usize)
                    .map(|cache_change| {
                        let mut data_value = cache_change.data_value();

                        let foo = DdsDeserialize::deserialize(&mut data_value).unwrap();
                        let sample_info = SampleInfo {
                            sample_state: SampleStateKind::NotRead,
                            view_state: ViewStateKind::New,
                            instance_state: InstanceStateKind::Alive,
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

                        (foo, sample_info)
                    })
                    .collect::<Vec<_>>();

                if samples.is_empty() {
                    Err(DDSError::NoData)
                } else {
                    Ok(samples)
                }
            }
            RtpsReader::Stateful(rtps_reader) => {
                let samples = rtps_reader
                    .reader_cache()
                    .changes()
                    .iter()
                    .take(max_samples as usize)
                    .map(|cache_change| {
                        let mut data_value = cache_change.data_value();

                        let foo = DdsDeserialize::deserialize(&mut data_value).unwrap();
                        let sample_info = SampleInfo {
                            sample_state: SampleStateKind::NotRead,
                            view_state: ViewStateKind::New,
                            instance_state: InstanceStateKind::Alive,
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

                        (foo, sample_info)
                    })
                    .collect::<Vec<_>>();

                if samples.is_empty() {
                    Err(DDSError::NoData)
                } else {
                    Ok(samples)
                }
            }
        }
    }

    fn take(
        &self,
        _max_samples: i32,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DDSResult<Vec<(Foo, SampleInfo)>> {
        let data_reader_shared = self.data_reader_impl.upgrade()?;
        let mut rtps_reader = data_reader_shared.rtps_reader.write_lock();
        match rtps_reader.deref_mut() {
            RtpsReader::Stateless(rtps_reader) => {
                let seq_num = rtps_reader
                    .reader_cache()
                    .get_seq_num_min()
                    .ok_or(DDSError::NoData)?;

                let samples = rtps_reader
                    .reader_cache()
                    .changes()
                    .iter()
                    .filter(|change| change.sequence_number() == seq_num)
                    .map(|change| {
                        let mut data_value = change.data_value();
                        (
                            DdsDeserialize::deserialize(&mut data_value).unwrap(),
                            SampleInfo {
                                sample_state: SampleStateKind::NotRead,
                                view_state: ViewStateKind::New,
                                instance_state: InstanceStateKind::Alive,
                                disposed_generation_count: 0,
                                no_writers_generation_count: 0,
                                sample_rank: 0,
                                generation_rank: 0,
                                absolute_generation_rank: 0,
                                source_timestamp: Time { sec: 0, nanosec: 0 },
                                instance_handle: 0,
                                publication_handle: 0,
                                valid_data: true,
                            },
                        )
                    })
                    .collect::<Vec<_>>();

                rtps_reader
                    .reader_cache()
                    .remove_change(|cc| cc.sequence_number() == seq_num);

                Ok(samples)
            }
            RtpsReader::Stateful(rtps_reader) => {
                let seq_num = rtps_reader
                    .reader_cache()
                    .get_seq_num_min()
                    .ok_or(DDSError::NoData)?;

                let samples = rtps_reader
                    .reader_cache()
                    .changes()
                    .iter()
                    .filter(|change| change.sequence_number() == seq_num)
                    .map(|change| {
                        let mut data_value = change.data_value();
                        (
                            DdsDeserialize::deserialize(&mut data_value).unwrap(),
                            SampleInfo {
                                sample_state: SampleStateKind::NotRead,
                                view_state: ViewStateKind::New,
                                instance_state: InstanceStateKind::Alive,
                                disposed_generation_count: 0,
                                no_writers_generation_count: 0,
                                sample_rank: 0,
                                generation_rank: 0,
                                absolute_generation_rank: 0,
                                source_timestamp: Time { sec: 0, nanosec: 0 },
                                instance_handle: 0,
                                publication_handle: 0,
                                valid_data: true,
                            },
                        )
                    })
                    .collect::<Vec<_>>();

                rtps_reader
                    .reader_cache()
                    .remove_change(|cc| cc.sequence_number() == seq_num);

                Ok(samples)
            }
        }
    }

    fn read_w_condition(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_condition: ReadCondition,
    ) -> DDSResult<()> {
        todo!()
    }

    fn take_w_condition(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _a_condition: ReadCondition,
    ) -> DDSResult<()> {
        todo!()
    }

    fn read_next_sample(
        &self,
        _data_value: &mut [Foo],
        _sample_info: &mut [SampleInfo],
    ) -> DDSResult<()> {
        todo!()
    }

    fn take_next_sample(
        &self,
        _data_value: &mut [Foo],
        _sample_info: &mut [SampleInfo],
    ) -> DDSResult<()> {
        todo!()
    }

    fn read_instance(
        &self,
        _data_values: &mut [Foo],
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
        _data_values: &mut [Foo],
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
        _data_values: &mut [Foo],
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
        _data_values: &mut [Foo],
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
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _a_condition: ReadCondition,
    ) -> DDSResult<()> {
        todo!()
    }

    fn take_next_instance_w_condition(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
        _max_samples: i32,
        _previous_handle: InstanceHandle,
        _a_condition: ReadCondition,
    ) -> DDSResult<()> {
        todo!()
    }

    fn return_loan(
        &self,
        _data_values: &mut [Foo],
        _sample_infos: &mut [SampleInfo],
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_key_value(&self, _key_holder: &mut Foo, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn lookup_instance(&self, _instance: &Foo) -> DDSResult<InstanceHandle> {
        todo!()
    }

    fn create_readcondition(
        &self,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DDSResult<ReadCondition> {
        todo!()
    }

    fn create_querycondition(
        &self,
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
        _query_expression: &'static str,
        _query_parameters: &[&'static str],
    ) -> DDSResult<QueryCondition> {
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
}

impl<Foo, Rtps> Entity for DataReaderProxy<Foo, Rtps>
where
    Foo: for<'de> DdsDeserialize<'de> + 'static,
    Rtps: RtpsStructure,
{
    type Qos = DataReaderQos;
    type Listener = Box<dyn DataReaderListener<Foo = Foo> + Send + Sync>;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.data_reader_impl)?).set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_reader_impl)?).get_qos()
        todo!()
    }

    fn set_listener(&self, listener: Option<Self::Listener>, _mask: StatusMask) -> DDSResult<()> {
        *self.as_ref().upgrade()?.listener.write_lock() = match listener {
            Some(l) => Some(Box::new(l)),
            None => None,
        };
        Ok(())
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_reader_impl)?).get_listener()
        todo!()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_reader_impl)?).get_statuscondition()
        todo!()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_reader_impl)?).get_status_changes()
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_reader_impl)?).enable()
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_reader_impl)?).get_instance_handle()
        todo!()
    }
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn read() {
    //     let reader = DataReaderStorage {};
    //     let shared_reader = DdsShared::new(reader);

    //     let data_reader = DataReaderImpl::<u8> {
    //         _subscriber: &MockSubcriber,
    //         _topic: &MockTopic(PhantomData),
    //         reader: shared_reader.downgrade(),
    //     };

    //     let sample = data_reader.read(1, &[], &[], &[]).unwrap();
    //     assert_eq!(sample[0].0, 1);
    // }
}

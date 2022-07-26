use dds_api::{
    builtin_topics::PublicationBuiltinTopicData,
    dcps_psm::{
        InstanceHandle, InstanceStateMask, LivelinessChangedStatus, RequestedDeadlineMissedStatus,
        RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus, SampleStateMask,
        StatusMask, SubscriptionMatchedStatus, ViewStateMask,
    },
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::DataReaderQos,
        read_condition::ReadCondition,
        sample_info::SampleInfo,
    },
    return_type::DdsResult,
    subscription::{
        data_reader::{
            DataReader, DataReaderGetSubscriber, DataReaderGetTopicDescription, FooDataReader,
            Sample,
        },
        data_reader_listener::DataReaderListener,
        query_condition::QueryCondition,
    },
};
use dds_implementation::{
    dds_impl::data_reader_impl::{AnyDataReaderListener, DataReaderImpl},
    dds_type::{DdsDeserialize, DdsType},
    utils::{
        shared_object::{DdsShared, DdsWeak},
        timer::ThreadTimer,
    },
};

use std::marker::PhantomData;

use crate::{subscriber_proxy::SubscriberProxy, topic_proxy::TopicProxy};

pub struct DataReaderProxy<Foo> {
    data_reader_attributes: DdsWeak<DataReaderImpl<ThreadTimer>>,
    phantom: PhantomData<Foo>,
}

// Not automatically derived because in that case it is only available if Foo: Clone
impl<Foo> Clone for DataReaderProxy<Foo> {
    fn clone(&self) -> Self {
        Self {
            data_reader_attributes: self.data_reader_attributes.clone(),
            phantom: self.phantom,
        }
    }
}

impl<Foo> DataReaderProxy<Foo> {
    pub fn new(data_reader_attributes: DdsWeak<DataReaderImpl<ThreadTimer>>) -> Self {
        Self {
            data_reader_attributes,
            phantom: PhantomData,
        }
    }
}

impl<Foo> AsRef<DdsWeak<DataReaderImpl<ThreadTimer>>> for DataReaderProxy<Foo> {
    fn as_ref(&self) -> &DdsWeak<DataReaderImpl<ThreadTimer>> {
        &self.data_reader_attributes
    }
}

impl<Foo> DataReaderGetSubscriber for DataReaderProxy<Foo> {
    type Subscriber = SubscriberProxy;

    fn data_reader_get_subscriber(&self) -> DdsResult<Self::Subscriber> {
        self.data_reader_attributes
            .upgrade()?
            .data_reader_get_subscriber()
            .map(|x| SubscriberProxy::new(x.downgrade()))
    }
}

impl<Foo> DataReaderGetTopicDescription for DataReaderProxy<Foo> {
    type TopicDescription = TopicProxy<Foo>;

    fn data_reader_get_topicdescription(&self) -> DdsResult<Self::TopicDescription> {
        self.data_reader_attributes
            .upgrade()?
            .data_reader_get_topicdescription()
            .map(|x| TopicProxy::new(x.downgrade()))
    }
}

impl<Foo> FooDataReader<Foo> for DataReaderProxy<Foo>
where
    Foo: DdsType + for<'de> DdsDeserialize<'de>,
{
    fn read(
        &self,
        max_samples: i32,
        sample_states: SampleStateMask,
        view_states: ViewStateMask,
        instance_states: InstanceStateMask,
    ) -> DdsResult<Vec<Sample<Foo>>> {
        self.data_reader_attributes.upgrade()?.read(
            max_samples,
            sample_states,
            view_states,
            instance_states,
        )
    }

    fn take(
        &self,
        max_samples: i32,
        sample_states: SampleStateMask,
        view_states: ViewStateMask,
        instance_states: InstanceStateMask,
    ) -> DdsResult<Vec<Sample<Foo>>> {
        self.data_reader_attributes.upgrade()?.take(
            max_samples,
            sample_states,
            view_states,
            instance_states,
        )
    }

    fn read_w_condition(
        &self,
        data_values: &mut [Foo],
        sample_infos: &mut [SampleInfo],
        max_samples: i32,
        a_condition: ReadCondition,
    ) -> DdsResult<()> {
        self.data_reader_attributes.upgrade()?.read_w_condition(
            data_values,
            sample_infos,
            max_samples,
            a_condition,
        )
    }

    fn take_w_condition(
        &self,
        data_values: &mut [Foo],
        sample_infos: &mut [SampleInfo],
        max_samples: i32,
        a_condition: ReadCondition,
    ) -> DdsResult<()> {
        self.data_reader_attributes.upgrade()?.take_w_condition(
            data_values,
            sample_infos,
            max_samples,
            a_condition,
        )
    }

    fn read_next_sample(
        &self,
        data_value: &mut [Foo],
        sample_info: &mut [SampleInfo],
    ) -> DdsResult<()> {
        self.data_reader_attributes
            .upgrade()?
            .read_next_sample(data_value, sample_info)
    }

    fn take_next_sample(
        &self,
        data_value: &mut [Foo],
        sample_info: &mut [SampleInfo],
    ) -> DdsResult<()> {
        self.data_reader_attributes
            .upgrade()?
            .take_next_sample(data_value, sample_info)
    }

    fn read_instance(
        &self,
        data_values: &mut [Foo],
        sample_infos: &mut [SampleInfo],
        max_samples: i32,
        a_handle: InstanceHandle,
        sample_states: SampleStateMask,
        view_states: ViewStateMask,
        instance_states: InstanceStateMask,
    ) -> DdsResult<()> {
        self.data_reader_attributes.upgrade()?.read_instance(
            data_values,
            sample_infos,
            max_samples,
            a_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    fn take_instance(
        &self,
        data_values: &mut [Foo],
        sample_infos: &mut [SampleInfo],
        max_samples: i32,
        a_handle: InstanceHandle,
        sample_states: SampleStateMask,
        view_states: ViewStateMask,
        instance_states: InstanceStateMask,
    ) -> DdsResult<()> {
        self.data_reader_attributes.upgrade()?.take_instance(
            data_values,
            sample_infos,
            max_samples,
            a_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    fn read_next_instance(
        &self,
        data_values: &mut [Foo],
        sample_infos: &mut [SampleInfo],
        max_samples: i32,
        previous_handle: InstanceHandle,
        sample_states: SampleStateMask,
        view_states: ViewStateMask,
        instance_states: InstanceStateMask,
    ) -> DdsResult<()> {
        self.data_reader_attributes.upgrade()?.read_next_instance(
            data_values,
            sample_infos,
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    fn take_next_instance(
        &self,
        data_values: &mut [Foo],
        sample_infos: &mut [SampleInfo],
        max_samples: i32,
        previous_handle: InstanceHandle,
        sample_states: SampleStateMask,
        view_states: ViewStateMask,
        instance_states: InstanceStateMask,
    ) -> DdsResult<()> {
        self.data_reader_attributes.upgrade()?.take_next_instance(
            data_values,
            sample_infos,
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    fn read_next_instance_w_condition(
        &self,
        data_values: &mut [Foo],
        sample_infos: &mut [SampleInfo],
        max_samples: i32,
        previous_handle: InstanceHandle,
        a_condition: ReadCondition,
    ) -> DdsResult<()> {
        self.data_reader_attributes
            .upgrade()?
            .read_next_instance_w_condition(
                data_values,
                sample_infos,
                max_samples,
                previous_handle,
                a_condition,
            )
    }

    fn take_next_instance_w_condition(
        &self,
        data_values: &mut [Foo],
        sample_infos: &mut [SampleInfo],
        max_samples: i32,
        previous_handle: InstanceHandle,
        a_condition: ReadCondition,
    ) -> DdsResult<()> {
        self.data_reader_attributes
            .upgrade()?
            .take_next_instance_w_condition(
                data_values,
                sample_infos,
                max_samples,
                previous_handle,
                a_condition,
            )
    }

    fn return_loan(
        &self,
        data_values: &mut [Foo],
        sample_infos: &mut [SampleInfo],
    ) -> DdsResult<()> {
        self.data_reader_attributes
            .upgrade()?
            .return_loan(data_values, sample_infos)
    }

    fn get_key_value(&self, key_holder: &mut Foo, handle: InstanceHandle) -> DdsResult<()> {
        self.data_reader_attributes
            .upgrade()?
            .get_key_value(key_holder, handle)
    }

    fn lookup_instance(&self, instance: &Foo) -> DdsResult<InstanceHandle> {
        self.data_reader_attributes
            .upgrade()?
            .lookup_instance(instance)
    }
}

impl<Foo> DataReader for DataReaderProxy<Foo> {
    fn create_readcondition(
        &self,
        sample_states: SampleStateMask,
        view_states: ViewStateMask,
        instance_states: InstanceStateMask,
    ) -> DdsResult<ReadCondition> {
        self.data_reader_attributes.upgrade()?.create_readcondition(
            sample_states,
            view_states,
            instance_states,
        )
    }

    fn create_querycondition(
        &self,
        sample_states: SampleStateMask,
        view_states: ViewStateMask,
        instance_states: InstanceStateMask,
        query_expression: &'static str,
        query_parameters: &[&'static str],
    ) -> DdsResult<QueryCondition> {
        self.data_reader_attributes
            .upgrade()?
            .create_querycondition(
                sample_states,
                view_states,
                instance_states,
                query_expression,
                query_parameters,
            )
    }

    fn delete_readcondition(&self, a_condition: ReadCondition) -> DdsResult<()> {
        self.data_reader_attributes
            .upgrade()?
            .delete_readcondition(a_condition)
    }

    fn get_liveliness_changed_status(&self) -> DdsResult<LivelinessChangedStatus> {
        self.data_reader_attributes
            .upgrade()?
            .get_liveliness_changed_status()
    }

    fn get_requested_deadline_missed_status(&self) -> DdsResult<RequestedDeadlineMissedStatus> {
        self.data_reader_attributes
            .upgrade()?
            .get_requested_deadline_missed_status()
    }

    fn get_requested_incompatible_qos_status(&self) -> DdsResult<RequestedIncompatibleQosStatus> {
        self.data_reader_attributes
            .upgrade()?
            .get_requested_incompatible_qos_status()
    }

    fn get_sample_lost_status(&self) -> DdsResult<SampleLostStatus> {
        self.data_reader_attributes
            .upgrade()?
            .get_sample_lost_status()
    }

    fn get_sample_rejected_status(&self) -> DdsResult<SampleRejectedStatus> {
        self.data_reader_attributes
            .upgrade()?
            .get_sample_rejected_status()
    }

    fn get_subscription_matched_status(&self) -> DdsResult<SubscriptionMatchedStatus> {
        self.data_reader_attributes
            .upgrade()?
            .get_subscription_matched_status()
    }

    fn delete_contained_entities(&self) -> DdsResult<()> {
        self.data_reader_attributes
            .upgrade()?
            .delete_contained_entities()
    }

    fn wait_for_historical_data(&self) -> DdsResult<()> {
        self.data_reader_attributes
            .upgrade()?
            .wait_for_historical_data()
    }

    fn get_matched_publication_data(
        &self,
        publication_handle: InstanceHandle,
    ) -> DdsResult<PublicationBuiltinTopicData> {
        self.data_reader_attributes
            .upgrade()?
            .get_matched_publication_data(publication_handle)
    }

    fn get_matched_publications(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.data_reader_attributes
            .upgrade()?
            .get_matched_publications()
    }
}

impl<Foo> Entity for DataReaderProxy<Foo>
where
    Foo: DdsType + for<'de> DdsDeserialize<'de> + 'static,
{
    type Qos = DataReaderQos;
    type Listener = Box<dyn DataReaderListener<Foo = Foo> + Send + Sync>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DdsResult<()> {
        self.data_reader_attributes.upgrade()?.set_qos(qos)
    }

    fn get_qos(&self) -> DdsResult<Self::Qos> {
        self.data_reader_attributes.upgrade()?.get_qos()
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DdsResult<()> {
        #[allow(clippy::redundant_closure)]
        self.data_reader_attributes.upgrade()?.set_listener(
            a_listener.map::<Box<
                dyn AnyDataReaderListener<DdsShared<DataReaderImpl<ThreadTimer>>> + Send + Sync,
            >, _>(|l| Box::new(l)),
            mask,
        )
    }

    fn get_listener(&self) -> DdsResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        self.data_reader_attributes.upgrade()?.get_statuscondition()
    }

    fn get_status_changes(&self) -> DdsResult<StatusMask> {
        self.data_reader_attributes.upgrade()?.get_status_changes()
    }

    fn enable(&self) -> DdsResult<()> {
        self.data_reader_attributes.upgrade()?.enable()
    }

    fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.data_reader_attributes.upgrade()?.get_instance_handle()
    }
}

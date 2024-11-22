use std::{future::Future, pin::Pin};

use crate::{
    dds_async::{
        data_reader::DataReaderAsync, data_reader_listener::DataReaderListenerAsync,
        subscriber::SubscriberAsync, topic::TopicAsync,
    },
    implementation::status_condition::status_condition_actor::StatusConditionActor,
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    runtime::{actor::ActorAddress, executor::block_on},
};

pub enum DataReaderListenerOperation {
    DataAvailable,
    SampleRejected(SampleRejectedStatus),
    _LivelinessChanged(LivelinessChangedStatus),
    RequestedDeadlineMissed(RequestedDeadlineMissedStatus),
    RequestedIncompatibleQos(RequestedIncompatibleQosStatus),
    SubscriptionMatched(SubscriptionMatchedStatus),
    SampleLost(SampleLostStatus),
}

pub trait AnyDataReaderListener: Send {
    fn trigger_on_data_available(&mut self, the_reader: DataReaderAsync<()>);
}

impl<'a, Foo> AnyDataReaderListener for Box<dyn DataReaderListenerAsync<'a, Foo = Foo> + Send + 'a>
where
    Foo: 'a,
{
    fn trigger_on_data_available(&mut self, the_reader: DataReaderAsync<()>) {
        block_on(self.on_data_available(the_reader.change_foo_type()));
    }
}

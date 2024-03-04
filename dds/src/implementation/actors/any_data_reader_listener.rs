use crate::{
    dds_async::{data_reader::DataReaderAsync, subscriber::SubscriberAsync, topic::TopicAsync},
    implementation::utils::actor::ActorAddress,
    subscription::{data_reader::DataReader, data_reader_listener::DataReaderListener},
};

use super::{
    data_reader_actor::DataReaderActor, data_reader_listener_actor::DataReaderListenerOperation,
    status_condition_actor::StatusConditionActor,
};

pub trait AnyDataReaderListener {
    fn call_listener_function(
        &mut self,
        listener_operation: DataReaderListenerOperation,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
    );
}

impl<T> AnyDataReaderListener for T
where
    T: DataReaderListener,
{
    fn call_listener_function(
        &mut self,
        listener_operation: DataReaderListenerOperation,
        reader_address: ActorAddress<DataReaderActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        subscriber: SubscriberAsync,
        topic: TopicAsync,
    ) {
        {
            let reader = DataReader::new(DataReaderAsync::new(
                reader_address,
                status_condition_address,
                subscriber,
                topic,
            ));
            match listener_operation {
                DataReaderListenerOperation::OnDataAvailable => self.on_data_available(&reader),
                DataReaderListenerOperation::OnSampleRejected(status) => {
                    self.on_sample_rejected(&reader, status)
                }
                DataReaderListenerOperation::_OnLivelinessChanged(status) => {
                    self.on_liveliness_changed(&reader, status)
                }
                DataReaderListenerOperation::OnRequestedDeadlineMissed(status) => {
                    self.on_requested_deadline_missed(&reader, status)
                }
                DataReaderListenerOperation::OnRequestedIncompatibleQos(status) => {
                    self.on_requested_incompatible_qos(&reader, status)
                }
                DataReaderListenerOperation::OnSubscriptionMatched(status) => {
                    self.on_subscription_matched(&reader, status)
                }
                DataReaderListenerOperation::OnSampleLost(status) => {
                    self.on_sample_lost(&reader, status)
                }
            }
        }
    }
}

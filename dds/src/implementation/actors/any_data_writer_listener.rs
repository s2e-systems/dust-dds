use std::{future::Future, pin::Pin};

use crate::{
    dds_async::{
        data_writer::DataWriterAsync, data_writer_listener::DataWriterListenerAsync,
        publisher::PublisherAsync, topic::TopicAsync,
    },
    implementation::actor::ActorAddress,
    infrastructure::status::{OfferedIncompatibleQosStatus, PublicationMatchedStatus},
};

use super::{data_writer_actor::DataWriterActor, status_condition_actor::StatusConditionActor};

pub enum DataWriterListenerOperation {
    OfferedIncompatibleQos(OfferedIncompatibleQosStatus),
    PublicationMatched(PublicationMatchedStatus),
}

pub trait AnyDataWriterListener {
    fn call_listener_function(
        &mut self,
        listener_operation: DataWriterListenerOperation,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

impl<'a, Foo> AnyDataWriterListener for Box<dyn DataWriterListenerAsync<'a, Foo = Foo> + Send + 'a>
where
    Foo: 'a,
{
    fn call_listener_function(
        &mut self,
        listener_operation: DataWriterListenerOperation,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {
            let the_writer =
                DataWriterAsync::new(writer_address, status_condition_address, publisher, topic);
            match listener_operation {
                DataWriterListenerOperation::OfferedIncompatibleQos(status) => {
                    self.on_offered_incompatible_qos(the_writer, status).await
                }
                DataWriterListenerOperation::PublicationMatched(status) => {
                    self.on_publication_matched(the_writer, status).await
                }
            }
        })
    }
}

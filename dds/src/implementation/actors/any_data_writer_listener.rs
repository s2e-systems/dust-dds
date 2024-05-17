use std::{future::Future, pin::Pin};

use crate::{
    dds_async::{
        data_writer::DataWriterAsync, data_writer_listener::DataWriterListenerAsync,
        publisher::PublisherAsync, topic::TopicAsync,
    },
    implementation::actor::ActorAddress,
};

use super::{
    data_writer_actor::DataWriterActor, data_writer_listener_actor::DataWriterListenerOperation,
    status_condition_actor::StatusConditionActor,
};

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

impl<'a, Foo> AnyDataWriterListener for Box<dyn DataWriterListenerAsync<Foo = Foo> + Send + 'a>
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
                DataWriterListenerOperation::OnOfferedIncompatibleQos(status) => {
                    self.on_offered_incompatible_qos(the_writer, status).await
                }
                DataWriterListenerOperation::OnPublicationMatched(status) => {
                    self.on_publication_matched(the_writer, status).await
                }
            }
        })
    }
}

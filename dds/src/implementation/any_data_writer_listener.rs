use std::{future::Future, pin::Pin};

use crate::{
    dds_async::{
        data_writer_listener::DataWriterListenerAsync, publisher::PublisherAsync, topic::TopicAsync,
    },
    implementation::status_condition::status_condition_actor::StatusConditionActor,
    infrastructure::status::{
        OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
    },
    runtime::actor::ActorAddress,
};

use super::domain_participant_backend::entities::data_writer::DataWriterEntity;

pub enum DataWriterListenerOperation {
    OfferedDeadlineMissed(OfferedDeadlineMissedStatus),
    OfferedIncompatibleQos(OfferedIncompatibleQosStatus),
    PublicationMatched(PublicationMatchedStatus),
}

pub trait AnyDataWriterListener {
    fn call_listener_function(
        &mut self,
        listener_operation: DataWriterListenerOperation,
        writer_address: ActorAddress<DataWriterEntity>,
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
        writer_address: ActorAddress<DataWriterEntity>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        todo!()
        // Box::pin(async {
        //     let the_writer =
        //         DataWriterAsync::new(writer_address, status_condition_address, publisher, topic);
        //     match listener_operation {
        //         DataWriterListenerOperation::OfferedDeadlineMissed(status) => {
        //             self.on_offered_deadline_missed(the_writer, status).await
        //         }
        //         DataWriterListenerOperation::OfferedIncompatibleQos(status) => {
        //             self.on_offered_incompatible_qos(the_writer, status).await
        //         }
        //         DataWriterListenerOperation::PublicationMatched(status) => {
        //             self.on_publication_matched(the_writer, status).await
        //         }
        //     }
        // })
    }
}

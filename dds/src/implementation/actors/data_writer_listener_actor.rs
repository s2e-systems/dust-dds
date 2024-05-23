use crate::{
    dds_async::{publisher::PublisherAsync, topic::TopicAsync},
    implementation::actor::{ActorAddress, ActorHandler, Mail, MailHandler},
    infrastructure::status::{OfferedIncompatibleQosStatus, PublicationMatchedStatus},
};

use super::{
    any_data_writer_listener::AnyDataWriterListener, data_writer_actor::DataWriterActor,
    status_condition_actor::StatusConditionActor,
};

pub struct DataWriterListenerActor {
    listener: Option<Box<dyn AnyDataWriterListener + Send>>,
}

impl DataWriterListenerActor {
    pub fn new(listener: Option<Box<dyn AnyDataWriterListener + Send>>) -> Self {
        Self { listener }
    }
}

pub enum DataWriterListenerOperation {
    OnOfferedIncompatibleQos(OfferedIncompatibleQosStatus),
    OnPublicationMatched(PublicationMatchedStatus),
}

pub struct CallListenerFunction {
    pub listener_operation: DataWriterListenerOperation,
    pub writer_address: ActorAddress<DataWriterActor>,
    pub status_condition_address: ActorAddress<StatusConditionActor>,
    pub publisher: PublisherAsync,
    pub topic: TopicAsync,
}
impl Mail for CallListenerFunction {
    type Result = ();
}
impl MailHandler<CallListenerFunction> for DataWriterListenerActor {
    async fn handle(
        &mut self,
        message: CallListenerFunction,
    ) -> <CallListenerFunction as Mail>::Result {
        if let Some(l) = &mut self.listener {
            l.call_listener_function(
                message.listener_operation,
                message.writer_address,
                message.status_condition_address,
                message.publisher,
                message.topic,
            )
            .await
        }
    }
}

impl ActorHandler for DataWriterListenerActor {
    type Message = ();

    async fn handle_message(&mut self, _: Self::Message) -> () {}
}

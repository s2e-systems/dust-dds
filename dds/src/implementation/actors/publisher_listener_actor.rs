use crate::{
    dds_async::publisher_listener::PublisherListenerAsync,
    implementation::actor::{ActorHandler, Mail, MailHandler},
    infrastructure::status::{OfferedIncompatibleQosStatus, PublicationMatchedStatus},
};

pub struct PublisherListenerActor {
    listener: Option<Box<dyn PublisherListenerAsync + Send>>,
}

impl PublisherListenerActor {
    pub fn new(listener: Option<Box<dyn PublisherListenerAsync + Send>>) -> Self {
        Self { listener }
    }
}

pub enum PublisherListenerOperation {
    OfferedIncompatibleQos(OfferedIncompatibleQosStatus),
    PublicationMatched(PublicationMatchedStatus),
}

pub struct CallListenerFunction {
    pub listener_operation: PublisherListenerOperation,
}
impl Mail for CallListenerFunction {
    type Result = ();
}
impl MailHandler<CallListenerFunction> for PublisherListenerActor {
    async fn handle(
        &mut self,
        message: CallListenerFunction,
    ) -> <CallListenerFunction as Mail>::Result {
        if let Some(l) = &mut self.listener {
            match message.listener_operation {
                PublisherListenerOperation::OfferedIncompatibleQos(status) => {
                    l.on_offered_incompatible_qos(&(), status).await
                }
                PublisherListenerOperation::PublicationMatched(status) => {
                    l.on_publication_matched(&(), status).await
                }
            }
        }
    }
}

impl ActorHandler for PublisherListenerActor {
    type Message = ();

    async fn handle_message(&mut self, _: Self::Message) {}
}

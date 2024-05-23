use crate::{
    dds_async::domain_participant_listener::DomainParticipantListenerAsync,
    implementation::actor::{ActorHandler, Mail, MailHandler},
    infrastructure::status::{
        OfferedIncompatibleQosStatus, PublicationMatchedStatus, RequestedDeadlineMissedStatus,
        RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
    },
};

pub struct DomainParticipantListenerActor {
    listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
}

impl DomainParticipantListenerActor {
    pub fn new(listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>) -> Self {
        Self { listener }
    }
}

pub enum DomainParticipantListenerOperation {
    SampleRejected(SampleRejectedStatus),
    RequestedIncompatibleQos(RequestedIncompatibleQosStatus),
    RequestedDeadlineMissed(RequestedDeadlineMissedStatus),
    SubscriptionMatched(SubscriptionMatchedStatus),
    SampleLost(SampleLostStatus),
    OfferedIncompatibleQos(OfferedIncompatibleQosStatus),
    PublicationMatched(PublicationMatchedStatus),
}

pub struct CallListenerFunction {
    pub listener_operation: DomainParticipantListenerOperation,
}
impl Mail for CallListenerFunction {
    type Result = ();
}
impl MailHandler<CallListenerFunction> for DomainParticipantListenerActor {
    async fn handle(
        &mut self,
        message: CallListenerFunction,
    ) -> <CallListenerFunction as Mail>::Result {
        if let Some(l) = &mut self.listener {
            match message.listener_operation {
                DomainParticipantListenerOperation::SampleRejected(status) => {
                    l.on_sample_rejected(&(), status).await
                }
                DomainParticipantListenerOperation::RequestedIncompatibleQos(status) => {
                    l.on_requested_incompatible_qos(&(), status).await
                }
                DomainParticipantListenerOperation::RequestedDeadlineMissed(status) => {
                    l.on_requested_deadline_missed(&(), status).await
                }
                DomainParticipantListenerOperation::SubscriptionMatched(status) => {
                    l.on_subscription_matched(&(), status).await
                }
                DomainParticipantListenerOperation::SampleLost(status) => {
                    l.on_sample_lost(&(), status).await
                }
                DomainParticipantListenerOperation::OfferedIncompatibleQos(status) => {
                    l.on_offered_incompatible_qos(&(), status).await
                }
                DomainParticipantListenerOperation::PublicationMatched(status) => {
                    l.on_publication_matched(&(), status).await
                }
            }
        }
    }
}

impl ActorHandler for DomainParticipantListenerActor {
    type Message = ();

    async fn handle_message(&mut self, _: Self::Message) {}
}

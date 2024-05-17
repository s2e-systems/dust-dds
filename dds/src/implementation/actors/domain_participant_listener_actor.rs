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
    OnSampleRejected(SampleRejectedStatus),
    OnRequestedIncompatibleQos(RequestedIncompatibleQosStatus),
    OnRequestedDeadlineMissed(RequestedDeadlineMissedStatus),
    OnSubscriptionMatched(SubscriptionMatchedStatus),
    OnSampleLost(SampleLostStatus),
    OnOfferedIncompatibleQos(OfferedIncompatibleQosStatus),
    OnPublicationMatched(PublicationMatchedStatus),
}

pub struct CallListenerFunction {
    pub listener_operation: DomainParticipantListenerOperation,
}
impl Mail for CallListenerFunction {
    type Result = ();
}
impl MailHandler<CallListenerFunction> for DomainParticipantListenerActor {
    fn handle(
        &mut self,
        message: CallListenerFunction,
    ) -> impl std::future::Future<Output = <CallListenerFunction as Mail>::Result> + Send {
        async move {
            if let Some(l) = &mut self.listener {
                match message.listener_operation {
                    DomainParticipantListenerOperation::OnSampleRejected(status) => {
                        l.on_sample_rejected(&(), status).await
                    }
                    DomainParticipantListenerOperation::OnRequestedIncompatibleQos(status) => {
                        l.on_requested_incompatible_qos(&(), status).await
                    }
                    DomainParticipantListenerOperation::OnRequestedDeadlineMissed(status) => {
                        l.on_requested_deadline_missed(&(), status).await
                    }
                    DomainParticipantListenerOperation::OnSubscriptionMatched(status) => {
                        l.on_subscription_matched(&(), status).await
                    }
                    DomainParticipantListenerOperation::OnSampleLost(status) => {
                        l.on_sample_lost(&(), status).await
                    }
                    DomainParticipantListenerOperation::OnOfferedIncompatibleQos(status) => {
                        l.on_offered_incompatible_qos(&(), status).await
                    }
                    DomainParticipantListenerOperation::OnPublicationMatched(status) => {
                        l.on_publication_matched(&(), status).await
                    }
                }
            }
        }
    }
}

impl ActorHandler for DomainParticipantListenerActor {
    type Message = ();

    fn handle_message(&mut self, _: Self::Message) -> impl std::future::Future<Output = ()> + Send {
        async {}
    }
}

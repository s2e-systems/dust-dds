use crate::{
    implementation::{
        actor::ActorAddress,
        actors::{domain_participant_actor, domain_participant_actor::DomainParticipantActor},
    },
    infrastructure::{error::DdsResult, status::StatusKind},
    rtps::types::Guid,
};

/// Async version of [`StatusCondition`](crate::infrastructure::condition::StatusCondition).
#[derive(Clone)]
pub struct StatusConditionAsync {
    participant_address: ActorAddress<DomainParticipantActor>,
    guid: Guid,
}

impl StatusConditionAsync {
    pub(crate) fn new(
        participant_address: ActorAddress<DomainParticipantActor>,
        guid: Guid,
    ) -> Self {
        Self {
            participant_address,
            guid,
        }
    }
}

impl StatusConditionAsync {
    /// Async version of [`get_enabled_statuses`](crate::infrastructure::condition::StatusCondition::get_enabled_statuses).
    #[tracing::instrument(skip(self))]
    pub async fn get_enabled_statuses(&self) -> DdsResult<Vec<StatusKind>> {
        self.participant_address
            .send_actor_mail(
                domain_participant_actor::GetStatusConditionEnabledStatuses { guid: self.guid },
            )?
            .receive_reply()
            .await
    }

    /// Async version of [`set_enabled_statuses`](crate::infrastructure::condition::StatusCondition::set_enabled_statuses).
    #[tracing::instrument(skip(self))]
    pub async fn set_enabled_statuses(&self, mask: &[StatusKind]) -> DdsResult<()> {
        self.participant_address
            .send_actor_mail(
                domain_participant_actor::SetStatusConditionEnabledStatuses {
                    guid: self.guid,
                    status_mask: mask.to_vec(),
                },
            )?
            .receive_reply()
            .await
    }

    /// Async version of [`get_entity`](crate::infrastructure::condition::StatusCondition::get_entity).
    #[tracing::instrument(skip(self))]
    pub async fn get_entity(&self) {
        todo!()
    }
}

impl StatusConditionAsync {
    /// Async version of [`get_trigger_value`](crate::infrastructure::condition::StatusCondition::get_trigger_value).
    #[tracing::instrument(skip(self))]
    pub async fn get_trigger_value(&self) -> DdsResult<bool> {
        self.participant_address
            .send_actor_mail(domain_participant_actor::GetStatusConditionTriggerValue {
                guid: self.guid,
            })?
            .receive_reply()
            .await
    }
}

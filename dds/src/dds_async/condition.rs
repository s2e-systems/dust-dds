use crate::{
    implementation::{actor::ActorAddress, actors::status_condition_actor::StatusConditionActor},
    infrastructure::{error::DdsResult, status::StatusKind},
};

/// Async version of [`StatusCondition`](crate::infrastructure::condition::StatusCondition).
#[derive(Clone)]
pub struct StatusConditionAsync {
    address: ActorAddress<StatusConditionActor>,
    runtime_handle: tokio::runtime::Handle,
}

impl StatusConditionAsync {
    pub(crate) fn new(
        address: ActorAddress<StatusConditionActor>,
        runtime_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            address,
            runtime_handle,
        }
    }

    pub(crate) fn address(&self) -> &ActorAddress<StatusConditionActor> {
        &self.address
    }

    pub(crate) fn runtime_handle(&self) -> &tokio::runtime::Handle {
        &self.runtime_handle
    }
}

impl StatusConditionAsync {
    /// Async version of [`get_enabled_statuses`](crate::infrastructure::condition::StatusCondition::get_enabled_statuses).
    #[tracing::instrument(skip(self))]
    pub async fn get_enabled_statuses(&self) -> DdsResult<Vec<StatusKind>> {
        self.address.get_enabled_statuses().await
    }

    /// Async version of [`set_enabled_statuses`](crate::infrastructure::condition::StatusCondition::set_enabled_statuses).
    #[tracing::instrument(skip(self))]
    pub async fn set_enabled_statuses(&self, mask: &[StatusKind]) -> DdsResult<()> {
        self.address.set_enabled_statuses(mask.to_vec()).await?;
        Ok(())
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
        self.address.get_trigger_value().await
    }
}

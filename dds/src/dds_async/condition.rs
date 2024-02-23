use crate::{
    implementation::{
        actors::status_condition_actor::{self, StatusConditionActor},
        utils::actor::ActorAddress,
    },
    infrastructure::{error::DdsResult, status::StatusKind},
};

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

    pub(crate) fn runtime_handle(&self) -> &tokio::runtime::Handle {
        &self.runtime_handle
    }
}

impl StatusConditionAsync {
    pub async fn get_enabled_statuses(&self) -> DdsResult<Vec<StatusKind>> {
        self.address
            .send_mail_and_await_reply(status_condition_actor::get_enabled_statuses::new())
            .await
    }

    pub async fn set_enabled_statuses(&self, mask: &[StatusKind]) -> DdsResult<()> {
        self.address
            .send_mail_and_await_reply(status_condition_actor::set_enabled_statuses::new(
                mask.to_vec(),
            ))
            .await
    }

    pub async fn get_entity(&self) {
        todo!()
    }
}

impl StatusConditionAsync {
    pub async fn get_trigger_value(&self) -> DdsResult<bool> {
        self.address
            .send_mail_and_await_reply(status_condition_actor::get_trigger_value::new())
            .await
    }
}

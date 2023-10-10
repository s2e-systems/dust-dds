use crate::{
    implementation::{
        actors::status_condition_actor::{self, StatusConditionActor},
        utils::actor::ActorAddress,
    },
    infrastructure::error::DdsResult,
};

use super::status::StatusKind;

/// A [`StatusCondition`] object is a specific Condition that is associated with each Entity.
/// The *trigger_value* of the [`StatusCondition`] depends on the communication status of that entity (e.g., arrival of data, loss of
/// information, etc.), ‘filtered’ by the set of *enabled_statuses* on the [`StatusCondition`].
#[derive(Clone)]
pub struct StatusCondition(ActorAddress<StatusConditionActor>);

impl StatusCondition {
    /// This operation retrieves the list of communication statuses that are taken into account to determine the *trigger_value* of the
    /// [`StatusCondition`]. This operation returns the statuses that were explicitly set on the last call to [`StatusCondition::set_enabled_statuses`] or, if
    /// it was never called, the default list of enabled statuses which includes all the statuses.
    pub fn get_enabled_statuses(&self) -> DdsResult<Vec<StatusKind>> {
        self.0
            .send_mail_and_await_reply_blocking(status_condition_actor::get_enabled_statuses::new())
    }

    /// This operation defines the list of communication statuses that are taken into account to determine the *trigger_value* of the
    /// [`StatusCondition`]. This operation may change the *trigger_value* of the [`StatusCondition`].
    /// [`WaitSet`](crate::infrastructure::wait_set::WaitSet) objects behavior depend on the changes of the *trigger_value* of their
    /// attached conditions. Therefore, any [`WaitSet`](crate::infrastructure::wait_set::WaitSet) to which the [`StatusCondition`] is attached is potentially affected by this operation.
    /// If this function is not invoked, the default list of enabled statuses includes all the statuses.
    pub fn set_enabled_statuses(&self, mask: &[StatusKind]) -> DdsResult<()> {
        self.0.send_mail_and_await_reply_blocking(
            status_condition_actor::set_enabled_statuses::new(mask.to_vec()),
        )
    }

    /// This operation returns the Entity associated with the [`StatusCondition`]. Note that there is exactly one Entity associated with
    /// each [`StatusCondition`].
    pub fn get_entity(&self) {
        todo!()
    }
}

/// This implementation block contains the Condition operations for the [`StatusCondition`].
impl StatusCondition {
    /// This operation retrieves the *trigger_value* of the [`StatusCondition`].
    pub fn get_trigger_value(&self) -> DdsResult<bool> {
        self.0
            .send_mail_and_await_reply_blocking(status_condition_actor::get_trigger_value::new())
    }
}

impl StatusCondition {
    pub(crate) fn new(dds_status_condition: ActorAddress<StatusConditionActor>) -> Self {
        Self(dds_status_condition)
    }
}

use std::sync::{Arc, Condvar};

use crate::{
    implementation::{
        dds_impl::status_condition_impl::StatusConditionImpl,
        utils::shared_object::{DdsRwLock, DdsShared},
    },
    infrastructure::error::DdsResult,
};

use super::status::StatusKind;

#[derive(Clone)]
pub struct StatusCondition(DdsShared<DdsRwLock<StatusConditionImpl>>);

impl StatusCondition {
    pub(crate) fn new(status_condition_impl: DdsShared<DdsRwLock<StatusConditionImpl>>) -> Self {
        Self(status_condition_impl)
    }

    /// This operation retrieves the list of communication statuses that are taken into account to determine the trigger_value of the
    /// [`StatusCondition`]. This operation returns the statuses that were explicitly set on the last call to [`set_enabled_statuses`] or, if
    /// [`set_enabled_statuses`] was never called, the default list of enabled statuses which includes all the statuses.
    pub fn get_enabled_statuses(&self) -> Vec<StatusKind> {
        self.0.read_lock().get_enabled_statuses()
    }

    /// This operation defines the list of communication statuses that are taken into account to determine the trigger_value of the
    /// StatusCondition. This operation may change the trigger_value of the StatusCondition.
    /// WaitSet objects behavior depend on the changes of the trigger_value of their attached conditions. Therefore, any WaitSet to
    /// which the StatusCondition is attached is potentially affected by this operation.
    /// If this function is not invoked, the default list of enabled statuses includes all the statuses.
    pub fn set_enabled_statuses(&self, mask: &[StatusKind]) -> DdsResult<()> {
        self.0.write_lock().set_enabled_statuses(mask)
    }

    /// This operation returns the Entity associated with the StatusCondition. Note that there is exactly one Entity associated with
    /// each StatusCondition.
    pub fn get_entity(&self) {
        todo!()
    }

    /// This operation retrieves the trigger_value of the Condition.
    pub fn get_trigger_value(&self) -> bool {
        self.0.read_lock().get_trigger_value()
    }

    pub(crate) fn push_cvar(&self, cvar: Arc<Condvar>) {
        self.0.write_lock().push_cvar(cvar)
    }
}

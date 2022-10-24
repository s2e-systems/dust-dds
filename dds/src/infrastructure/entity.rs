use std::sync::{Arc, Condvar};

use crate::{
    implementation::utils::shared_object::{DdsRwLock, DdsShared},
    infrastructure::error::DdsResult,
};

use super::{
    instance::InstanceHandle,
    status::{StatusKind, StatusMask},
};

struct StatusConditionImpl {
    enabled_statuses: DdsRwLock<StatusMask>,
    communication_status: DdsRwLock<StatusMask>,
    cvar_list: DdsRwLock<Vec<Arc<Condvar>>>,
}

impl Default for StatusConditionImpl {
    fn default() -> Self {
        Self {
            enabled_statuses: DdsRwLock::new(0xFFFF),
            communication_status: DdsRwLock::new(0),
            cvar_list: DdsRwLock::new(Vec::new()),
        }
    }
}

#[derive(Clone)]
pub struct StatusCondition(DdsShared<StatusConditionImpl>);

impl Default for StatusCondition {
    fn default() -> Self {
        Self(DdsShared::new(Default::default()))
    }
}

impl StatusCondition {
    pub fn get_enabled_statuses(&self) -> StatusMask {
        *self.0.enabled_statuses.read_lock()
    }

    /// This operation defines the list of communication statuses that are taken into account to determine the trigger_value of the
    /// StatusCondition. This operation may change the trigger_value of the StatusCondition.
    /// WaitSet objects behavior depend on the changes of the trigger_value of their attached conditions. Therefore, any WaitSet to
    /// which the StatusCondition is attached is potentially affected by this operation.
    /// If this function is not invoked, the default list of enabled statuses includes all the statuses.
    pub fn set_enabled_statuses(&mut self, mask: StatusMask) -> DdsResult<()> {
        *self.0.enabled_statuses.write_lock() = mask;
        Ok(())
    }

    pub fn get_entity(&self) {
        todo!()
    }

    pub fn get_trigger_value(&self) -> bool {
        *self.0.enabled_statuses.read_lock() & *self.0.communication_status.read_lock() != 0
    }

    pub(crate) fn add_communication_state(&mut self, state: StatusKind) {
        *self.0.communication_status.write_lock() |= state;

        if self.get_trigger_value() {
            for cvar in self.0.cvar_list.read_lock().iter() {
                cvar.notify_all();
            }
        }
    }

    pub(crate) fn push_cvar(&self, cvar: Arc<Condvar>) {
        self.0.cvar_list.write_lock().push(cvar)
    }
}

pub trait DomainEntity {}

pub trait Entity {
    type Qos;
    type Listener;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DdsResult<()>;

    fn get_qos(&self) -> DdsResult<Self::Qos>;

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DdsResult<()>;

    fn get_listener(&self) -> DdsResult<Option<Self::Listener>>;

    fn get_statuscondition(&self) -> DdsResult<StatusCondition>;

    fn get_status_changes(&self) -> DdsResult<StatusMask>;

    fn enable(&self) -> DdsResult<()>;

    fn get_instance_handle(&self) -> DdsResult<InstanceHandle>;
}

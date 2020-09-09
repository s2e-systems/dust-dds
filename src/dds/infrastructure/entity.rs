use crate::dds::types::{ReturnCode, StatusKind, InstanceHandle};
use crate::dds::infrastructure::qos_policy::QosPolicy;
// use crate::dds::infrastructure::Listener;

pub struct StatusCondition;

pub trait Entity {
    type Listener;

    fn set_qos(&self, qos_list: &[&dyn QosPolicy]) -> ReturnCode;

    fn get_qos(&self, qos_list: &mut [&dyn QosPolicy]) -> ReturnCode;

    fn set_listener(&self, a_listener: Self::Listener, mask: &[StatusKind]) -> ReturnCode;

    fn get_listener(&self, ) -> Self::Listener;

    fn get_statuscondition(&self, ) -> StatusCondition;

    fn get_status_changes(&self, ) -> StatusKind;

    fn enable(&self, ) -> ReturnCode;

    fn get_instance_handle(&self, ) -> InstanceHandle;
}
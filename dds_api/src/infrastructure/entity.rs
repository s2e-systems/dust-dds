use rust_dds_types::{InstanceHandle, ReturnCode};

use super::status::StatusMask;

pub struct StatusCondition;

pub trait DomainEntity: Entity {}

pub trait Entity {
    type Qos;
    type Listener;

    fn set_qos(&self, qos: Option<Self::Qos>) -> ReturnCode<()>;

    fn get_qos(&self) -> ReturnCode<Self::Qos>;

    fn set_listener(&self, a_listener: Self::Listener, mask: StatusMask) -> ReturnCode<()>;

    fn get_listener(&self) -> &Self::Listener;

    fn get_statuscondition(&self) -> StatusCondition;

    fn get_status_changes(&self) -> StatusMask;

    fn enable(&self) -> ReturnCode<()>;

    fn get_instance_handle(&self) -> ReturnCode<InstanceHandle>;
}

use core::future::Future;

use super::infrastructure::status::StatusKind;

pub trait StatusCondition {
    fn add_state(&mut self, state: StatusKind) -> impl Future<Output = ()> + Send;

    fn remove_state(&mut self, state: StatusKind) -> impl Future<Output = ()> + Send;
}

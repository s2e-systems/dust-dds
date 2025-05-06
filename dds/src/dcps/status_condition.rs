use super::infrastructure::status::StatusKind;

pub trait StatusCondition {
    fn add_state(&mut self, state: StatusKind);

    fn remove_state(&mut self, state: StatusKind);
}

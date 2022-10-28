use std::sync::{Arc, Condvar, Mutex};

use crate::infrastructure::{
    error::{DdsError, DdsResult},
    time::Duration,
};

use super::condition::StatusCondition;

#[derive(Clone)]
pub struct ConditionSeq(Vec<Condition>);

#[derive(Clone)]
pub enum Condition {
    StatusCondition(StatusCondition),
}
impl Condition {
    pub fn get_trigger_value(&self) -> bool {
        match self {
            Condition::StatusCondition(c) => c.get_trigger_value(),
        }
    }
}

pub struct WaitSet {
    conditions: ConditionSeq,
    enabled: Mutex<()>,
    cvar: Arc<Condvar>,
}

impl Default for WaitSet {
    fn default() -> Self {
        Self {
            conditions: ConditionSeq(vec![]),
            enabled: Mutex::new(()),
            cvar: Arc::new(Condvar::new()),
        }
    }
}

impl WaitSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn wait(&self, timeout: Duration) -> DdsResult<ConditionSeq> {
        let enabled = self.enabled.lock().unwrap();
        let std_duration = std::time::Duration::new(timeout.sec() as u64, timeout.nanosec());

        let result = self.cvar.wait_timeout(enabled, std_duration).unwrap();

        if result.1.timed_out() {
            return Err(DdsError::Timeout);
        }

        Ok(ConditionSeq(
            self.conditions
                .0
                .iter()
                .filter(|x| x.get_trigger_value())
                .cloned()
                .collect(),
        ))
    }

    pub fn attach_condition(&mut self, cond: Condition) -> DdsResult<()> {
        match &cond {
            Condition::StatusCondition(c) => c.push_cvar(self.cvar.clone()),
        }
        self.conditions.0.push(cond);
        Ok(())
    }

    pub fn detach_condition(&self, _cond: Condition) -> DdsResult<()> {
        todo!()
    }

    pub fn get_conditions(&self) -> DdsResult<ConditionSeq> {
        Ok(self.conditions.clone())
    }
}

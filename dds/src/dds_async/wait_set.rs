use core::task::Poll;

use super::condition::StatusConditionAsync;
use crate::{
    infrastructure::{
        error::{DdsError, DdsResult},
        time::Duration,
    },
    runtime::{DdsRuntime, Timer},
};
use alloc::{boxed::Box, string::String, vec, vec::Vec};

/// Async version of [`Condition`](crate::infrastructure::wait_set::Condition).
pub enum ConditionAsync<R: DdsRuntime> {
    /// Status condition variant
    StatusCondition(StatusConditionAsync<R>),
}

impl<R: DdsRuntime> Clone for ConditionAsync<R> {
    fn clone(&self) -> Self {
        match self {
            Self::StatusCondition(arg0) => Self::StatusCondition(arg0.clone()),
        }
    }
}

impl<R: DdsRuntime> ConditionAsync<R> {
    /// Async version of [`get_trigger_value`](crate::infrastructure::wait_set::Condition::get_trigger_value).
    #[tracing::instrument(skip(self))]
    pub async fn get_trigger_value(&self) -> DdsResult<bool> {
        match self {
            ConditionAsync::StatusCondition(c) => c.get_trigger_value().await,
        }
    }
}

/// Async version of [`WaitSet`](crate::infrastructure::wait_set::WaitSet).
pub struct WaitSetAsync<R: DdsRuntime> {
    conditions: Vec<ConditionAsync<R>>,
}

impl<R: DdsRuntime> Default for WaitSetAsync<R> {
    fn default() -> Self {
        Self {
            conditions: Default::default(),
        }
    }
}

impl<R: DdsRuntime> WaitSetAsync<R> {
    /// Create a new [`WaitSetAsync`]
    #[tracing::instrument]
    pub fn new() -> Self {
        Self::default()
    }

    /// Async version of [`wait`](crate::infrastructure::wait_set::WaitSet::wait).
    #[tracing::instrument(skip(self))]
    pub async fn wait(&self, timeout: Duration) -> DdsResult<Vec<ConditionAsync<R>>> {
        if self.conditions.is_empty() {
            return Err(DdsError::PreconditionNotMet(String::from(
                "WaitSet has no attached conditions",
            )));
        };

        let mut timer_handle = match &self.conditions[0] {
            ConditionAsync::StatusCondition(status_condition_async) => {
                status_condition_async.timer_handle().clone()
            }
        };
        let mut timer = Box::pin(timer_handle.delay(timeout.into()));

        let mut trigger_conditions = Vec::new();
        // Check if conditions are already triggered
        for condition in &self.conditions {
            if condition.get_trigger_value().await? {
                trigger_conditions.push(condition.clone());
            }
        }

        if !trigger_conditions.is_empty() {
            return Ok(trigger_conditions);
        }

        // No status condition is yet triggered so now we have to wait for at least one status condition to trigger
        let mut notification_channels = Vec::new();
        for condition in &self.conditions {
            match condition {
                ConditionAsync::StatusCondition(status_condition_async) => notification_channels
                    .push(status_condition_async.register_notification().await?),
            }
        }
        let mut notification_futures: Vec<_> = notification_channels
            .iter_mut()
            .map(|x| Box::pin(x.receive()))
            .collect();

        let condition_index = core::future::poll_fn(move |cx| {
            for (condition_index, notification) in notification_futures.iter_mut().enumerate() {
                if notification.as_mut().poll(cx).is_ready() {
                    return Poll::Ready(Ok(condition_index));
                }
            }
            if timer.as_mut().poll(cx).is_ready() {
                return Poll::Ready(Err(DdsError::Timeout));
            }
            Poll::Pending
        })
        .await?;

        Ok(vec![self.conditions[condition_index].clone()])
    }

    /// Async version of [`attach_condition`](crate::infrastructure::wait_set::WaitSet::attach_condition).
    #[tracing::instrument(skip(self, cond))]
    pub async fn attach_condition(&mut self, cond: ConditionAsync<R>) -> DdsResult<()> {
        self.conditions.push(cond);
        Ok(())
    }

    /// Async version of [`detach_condition`](crate::infrastructure::wait_set::WaitSet::detach_condition).
    #[tracing::instrument(skip(self, _cond))]
    pub async fn detach_condition(&self, _cond: ConditionAsync<R>) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`get_conditions`](crate::infrastructure::wait_set::WaitSet::get_conditions).
    #[tracing::instrument(skip(self))]
    pub async fn get_conditions(&self) -> DdsResult<Vec<ConditionAsync<R>>> {
        Ok(self.conditions.clone())
    }
}

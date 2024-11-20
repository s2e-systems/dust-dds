use super::{entities::domain_participant::DomainParticipantEntity, handle::InstanceHandleCounter};
use crate::{
    rtps::transport::Transport,
    runtime::{executor::Executor, timer::TimerDriver},
};

pub struct DomainParticipantActor {
    pub transport: Box<dyn Transport>,
    pub instance_handle_counter: InstanceHandleCounter,
    pub domain_participant: DomainParticipantEntity,
    pub executor: Executor,
    pub timer_driver: TimerDriver,
}

impl DomainParticipantActor {
    pub fn new(
        domain_participant: DomainParticipantEntity,
        transport: Box<dyn Transport>,
        executor: Executor,
        timer_driver: TimerDriver,
        instance_handle_counter: InstanceHandleCounter,
    ) -> Self {
        Self {
            transport,
            instance_handle_counter,
            domain_participant,
            executor,
            timer_driver,
        }
    }
}

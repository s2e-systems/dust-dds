use super::{entities::domain_participant::DomainParticipantEntity, handle::InstanceHandleCounter};
use crate::{
    dds_async::domain_participant_listener::DomainParticipantListenerAsync,
    domain::domain_participant_factory::DomainId,
    implementation::runtime::{executor::Executor, timer::TimerDriver},
    infrastructure::{qos::DomainParticipantQos, status::StatusKind},
    rtps::transport::Transport,
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
        domain_id: DomainId,
        domain_participant_qos: DomainParticipantQos,
        listener: Option<Box<dyn DomainParticipantListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
        executor: Executor,
        timer_driver: TimerDriver,
        mut transport: Box<dyn Transport>,
    ) -> Self {
        let mut instance_handle_counter = InstanceHandleCounter::default();
        let domain_participant = DomainParticipantEntity::new(
            domain_id,
            domain_participant_qos,
            listener,
            status_kind,
            &mut instance_handle_counter,
            &executor,
            transport.as_mut(),
        );

        Self {
            transport,
            instance_handle_counter,
            domain_participant,
            executor,
            timer_driver,
        }
    }
}

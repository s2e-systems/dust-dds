use crate::{
    implementation::utils::actor::{actor_interface, Actor, ActorAddress},
    infrastructure::{
        error::{DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos},
    },
};

use super::dds_domain_participant::DdsDomainParticipant;
pub struct DdsDomainParticipantFactory {
    domain_participant_list: Vec<Actor<DdsDomainParticipant>>,
    domain_participant_counter: u32,
    qos: DomainParticipantFactoryQos,
    default_participant_qos: DomainParticipantQos,
}

impl Default for DdsDomainParticipantFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl DdsDomainParticipantFactory {
    pub fn new() -> Self {
        Self {
            domain_participant_list: Vec::new(),
            domain_participant_counter: 0,
            qos: DomainParticipantFactoryQos::default(),
            default_participant_qos: DomainParticipantQos::default(),
        }
    }
}

actor_interface! {
impl DdsDomainParticipantFactory {
    pub fn add_participant(&mut self, participant: Actor<DdsDomainParticipant>) {
        self.domain_participant_list.push(participant)
    }

    pub fn get_participant_list(&self) -> Vec<ActorAddress<DdsDomainParticipant>> {
        self.domain_participant_list.iter().map(|dp| dp.address()).collect()
    }

    pub fn get_unique_participant_id(&mut self) -> u32 {
        let counter = self.domain_participant_counter;
        self.domain_participant_counter += 1;
        counter
    }

    pub fn delete_participant(&mut self, handle: InstanceHandle) {
        self.domain_participant_list.retain(|x|
            if let Ok(h) = x.address().get_instance_handle() {
                h != handle
            } else {
                false
            })
    }

    pub fn get_qos(&self) -> DomainParticipantFactoryQos {
        self.qos.clone()
    }

    pub fn set_qos(&mut self, qos: DomainParticipantFactoryQos) {
        self.qos = qos;
    }

    pub fn get_default_participant_qos(&self) -> DomainParticipantQos {
        self.default_participant_qos.clone()
    }

    pub fn set_default_participant_qos(&mut self, qos: DomainParticipantQos) {
        self.default_participant_qos = qos;
    }
}
}

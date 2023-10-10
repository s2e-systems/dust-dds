use std::collections::HashMap;

use dust_dds_derive::actor_interface;

use crate::{
    implementation::utils::actor::{Actor, ActorAddress},
    infrastructure::{
        instance::InstanceHandle,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos},
    },
};

use super::domain_participant_actor::DomainParticipantActor;

pub struct DomainParticipantFactoryActor {
    domain_participant_list: HashMap<InstanceHandle, Actor<DomainParticipantActor>>,
    domain_participant_counter: u32,
    qos: DomainParticipantFactoryQos,
    default_participant_qos: DomainParticipantQos,
}

impl Default for DomainParticipantFactoryActor {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainParticipantFactoryActor {
    pub fn new() -> Self {
        Self {
            domain_participant_list: HashMap::new(),
            domain_participant_counter: 0,
            qos: DomainParticipantFactoryQos::default(),
            default_participant_qos: DomainParticipantQos::default(),
        }
    }
}

#[actor_interface]
impl DomainParticipantFactoryActor {
    async fn add_participant(
        &mut self,
        instance_handle: InstanceHandle,
        participant: Actor<DomainParticipantActor>,
    ) {
        self.domain_participant_list
            .insert(instance_handle, participant);
    }

    async fn get_participant_list(&self) -> Vec<ActorAddress<DomainParticipantActor>> {
        self.domain_participant_list
            .values()
            .map(|dp| dp.address())
            .collect()
    }

    async fn get_unique_participant_id(&mut self) -> u32 {
        let counter = self.domain_participant_counter;
        self.domain_participant_counter += 1;
        counter
    }

    async fn delete_participant(&mut self, handle: InstanceHandle) {
        self.domain_participant_list.remove(&handle);
    }

    async fn get_qos(&self) -> DomainParticipantFactoryQos {
        self.qos.clone()
    }

    async fn set_qos(&mut self, qos: DomainParticipantFactoryQos) {
        self.qos = qos;
    }

    async fn get_default_participant_qos(&self) -> DomainParticipantQos {
        self.default_participant_qos.clone()
    }

    async fn set_default_participant_qos(&mut self, qos: DomainParticipantQos) {
        self.default_participant_qos = qos;
    }
}

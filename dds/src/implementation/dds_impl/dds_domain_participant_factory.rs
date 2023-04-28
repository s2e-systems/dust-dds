use lazy_static::lazy_static;

use crate::{
    implementation::utils::shared_object::DdsRwLock,
    infrastructure::qos::{DomainParticipantFactoryQos, DomainParticipantQos},
};

use super::domain_participant_list::DomainParticipantList;

pub struct DdsDomainParticipantFactory {
    domain_participant_list: DomainParticipantList,
    qos: DdsRwLock<DomainParticipantFactoryQos>,
    default_participant_qos: DdsRwLock<DomainParticipantQos>,
}

impl DdsDomainParticipantFactory {
    pub fn new() -> Self {
        Self {
            domain_participant_list: DomainParticipantList::new(),
            qos: DdsRwLock::new(DomainParticipantFactoryQos::default()),
            default_participant_qos: DdsRwLock::new(DomainParticipantQos::default()),
        }
    }

    pub fn qos(&self) -> DomainParticipantFactoryQos {
        self.qos.read_lock().clone()
    }

    pub fn set_qos(&mut self, qos: DomainParticipantFactoryQos) {
        *self.qos.write_lock() = qos;
    }

    pub fn default_participant_qos(&self) -> DomainParticipantQos {
        self.default_participant_qos.read_lock().clone()
    }

    pub fn set_default_participant_qos(&mut self, default_participant_qos: DomainParticipantQos) {
        *self.default_participant_qos.write_lock() = default_participant_qos;
    }

    pub fn domain_participant_list(&self) -> &DomainParticipantList {
        &self.domain_participant_list
    }
}

lazy_static! {
    pub static ref THE_DDS_DOMAIN_PARTICIPANT_FACTORY: DdsDomainParticipantFactory =
        DdsDomainParticipantFactory::new();
}

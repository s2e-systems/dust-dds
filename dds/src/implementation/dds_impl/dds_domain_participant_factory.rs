use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::{
    implementation::{
        dds_impl::{dcps_service::DcpsService, dds_domain_participant::DdsDomainParticipant},
        rtps::types::GuidPrefix,
        utils::shared_object::DdsRwLock,
    },
    infrastructure::qos::{DomainParticipantFactoryQos, DomainParticipantQos},
};

pub struct DdsDomainParticipantFactory {
    domain_participant_list: DdsRwLock<HashMap<GuidPrefix, (DdsDomainParticipant, DcpsService)>>,
    qos: DdsRwLock<DomainParticipantFactoryQos>,
    default_participant_qos: DdsRwLock<DomainParticipantQos>,
}

impl DdsDomainParticipantFactory {
    pub fn new() -> Self {
        Self {
            domain_participant_list: DdsRwLock::new(HashMap::new()),
            qos: DdsRwLock::new(DomainParticipantFactoryQos::default()),
            default_participant_qos: DdsRwLock::new(DomainParticipantQos::default()),
        }
    }

    pub fn qos(&self) -> DomainParticipantFactoryQos {
        self.qos.read_lock().clone()
    }

    pub fn set_qos(&self, qos: DomainParticipantFactoryQos) {
        *self.qos.write_lock() = qos;
    }

    pub fn default_participant_qos(&self) -> DomainParticipantQos {
        self.default_participant_qos.read_lock().clone()
    }

    pub fn set_default_participant_qos(&self, default_participant_qos: DomainParticipantQos) {
        *self.default_participant_qos.write_lock() = default_participant_qos;
    }

    pub fn add_participant(
        &self,
        guid_prefix: GuidPrefix,
        dds_participant: DdsDomainParticipant,
        dcps_service: DcpsService,
    ) {
        self.domain_participant_list
            .write_lock()
            .insert(guid_prefix, (dds_participant, dcps_service));
    }

    pub fn remove_participant(&self, guid_prefix: &GuidPrefix) {
        self.domain_participant_list
            .write_lock()
            .remove(guid_prefix);
    }

    pub fn get_participant<F, O>(&self, guid_prefix: &GuidPrefix, f: F) -> O
    where
        F: FnOnce(Option<&DdsDomainParticipant>) -> O,
    {
        f(self
            .domain_participant_list
            .read_lock()
            .get(&guid_prefix)
            .map(|o| &o.0))
    }

    pub fn get_participant_mut<F, O>(&self, guid_prefix: &GuidPrefix, f: F) -> O
    where
        F: FnOnce(Option<&mut DdsDomainParticipant>) -> O,
    {
        f(self
            .domain_participant_list
            .write_lock()
            .get_mut(&guid_prefix)
            .map(|o| &mut o.0))
    }

    pub fn get_dcps_service<F, O>(&self, guid_prefix: &GuidPrefix, f: F) -> O
    where
        F: FnOnce(Option<&DcpsService>) -> O,
    {
        f(self
            .domain_participant_list
            .read_lock()
            .get(&guid_prefix)
            .map(|o| &o.1))
    }
}

lazy_static! {
    pub(super) static ref THE_DDS_DOMAIN_PARTICIPANT_FACTORY: DdsDomainParticipantFactory =
        DdsDomainParticipantFactory::new();
}

use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::{
        rtps::types::Guid,
        utils::map::{DdsMap, DdsMapValueIter, DdsMapValueIterMut},
    },
};

use super::{
    dcps_service::DcpsService, dds_domain_participant::DdsDomainParticipant,
    dds_entity_list::DdsEntityList,
};

pub struct DomainParticipantList {
    participant_list: DdsEntityList<
        DdsDomainParticipant,
        Option<Box<dyn DomainParticipantListener + Send + Sync>>,
    >,
    dcps_service_list: DdsMap<Guid, DcpsService>,
}

#[allow(dead_code)]
impl DomainParticipantList {
    pub fn new() -> Self {
        Self {
            participant_list: DdsEntityList::new(),
            dcps_service_list: DdsMap::new(),
        }
    }

    pub fn add_participant(
        &self,
        guid: Guid,
        entity: DdsDomainParticipant,
        listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
        dcps_service: DcpsService,
    ) {
        self.participant_list.add_entity(guid, entity, listener);
        self.dcps_service_list.add(guid, dcps_service);
    }

    pub fn remove_participant(&self, guid: &Guid) {
        self.participant_list.remove_entity(guid);
        self.dcps_service_list.remove(guid);
    }

    pub fn get_participant<F, O>(&self, guid: &Guid, f: F) -> O
    where
        F: FnOnce(Option<&DdsDomainParticipant>) -> O,
    {
        self.participant_list.get_entity(guid, |x| f(x))
    }

    pub fn get_participant_mut<F, O>(&self, guid: &Guid, f: F) -> O
    where
        F: FnOnce(Option<&mut DdsDomainParticipant>) -> O,
    {
        self.participant_list.get_entity_mut(guid, |x| f(x))
    }

    pub fn get_listener<F, O>(&self, guid: &Guid, f: F) -> O
    where
        F: FnOnce(Option<&Option<Box<dyn DomainParticipantListener + Send + Sync>>>) -> O,
    {
        self.participant_list.get_listener(guid, |x| f(x))
    }

    pub fn get_listener_mut<F, O>(&self, guid: &Guid, f: F) -> O
    where
        F: FnOnce(Option<&mut Option<Box<dyn DomainParticipantListener + Send + Sync>>>) -> O,
    {
        self.participant_list.get_listener_mut(guid, |x| f(x))
    }

    pub fn get_dcps_service<F, O>(&self, guid: &Guid, f: F) -> O
    where
        F: FnOnce(Option<&DcpsService>) -> O,
    {
        self.dcps_service_list.get(guid, |x| f(x))
    }

    pub fn iter<F, O>(&self, f: F) -> O
    where
        F: for<'a> FnOnce(&mut DdsMapValueIter<'a, Guid, DdsDomainParticipant>) -> O,
    {
        self.participant_list.iter(f)
    }

    pub fn iter_mut<F, O>(&self, f: F) -> O
    where
        F: for<'a> FnOnce(&mut DdsMapValueIterMut<'a, Guid, DdsDomainParticipant>) -> O,
    {
        self.participant_list.iter_mut(f)
    }
}

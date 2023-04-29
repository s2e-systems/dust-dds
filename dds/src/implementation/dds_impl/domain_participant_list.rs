use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::{
        rtps::types::Guid,
        utils::map::{DdsMapValueIter, DdsMapValueIterMut},
    },
};

use super::{dds_domain_participant::DdsDomainParticipant, dds_entity_list::DdsEntityList};

pub struct DomainParticipantList {
    participant_list: DdsEntityList<
        DdsDomainParticipant,
        Option<Box<dyn DomainParticipantListener + Send + Sync>>,
    >,
}

impl DomainParticipantList {
    pub fn new() -> Self {
        Self {
            participant_list: DdsEntityList::new(),
        }
    }

    pub fn add_participant(
        &self,
        guid: Guid,
        entity: DdsDomainParticipant,
        listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
    ) {
        self.participant_list.add_entity(guid, entity, listener);
    }

    pub fn remove_participant(&self, guid: &Guid) {
        self.participant_list.remove_entity(guid);
    }

    pub fn get_participant<F, O>(&self, guid: &Guid, mut f: F) -> O
    where
        F: FnMut(Option<&DdsDomainParticipant>) -> O,
    {
        self.participant_list.get_entity(guid, |x| f(x))
    }

    pub fn get_participant_mut<F, O>(&self, guid: &Guid, mut f: F) -> O
    where
        F: FnMut(Option<&mut DdsDomainParticipant>) -> O,
    {
        self.participant_list.get_entity_mut(guid, |x| f(x))
    }

    pub fn get_listener<F, O>(&self, guid: &Guid, mut f: F) -> O
    where
        F: FnMut(Option<&Option<Box<dyn DomainParticipantListener + Send + Sync>>>) -> O,
    {
        self.participant_list.get_listener(guid, |x| f(x))
    }

    pub fn get_listener_mut<F, O>(&self, guid: &Guid, mut f: F) -> O
    where
        F: FnMut(Option<&mut Option<Box<dyn DomainParticipantListener + Send + Sync>>>) -> O,
    {
        self.participant_list.get_listener_mut(guid, |x| f(x))
    }

    pub fn iter<F, O>(&self, f: F) -> O
    where
        F: for<'a> FnMut(&mut DdsMapValueIter<'a, Guid, DdsDomainParticipant>) -> O,
    {
        self.participant_list.iter(f)
    }

    pub fn iter_mut<F, O>(&self, f: F) -> O
    where
        F: for<'a> FnMut(&mut DdsMapValueIterMut<'a, Guid, DdsDomainParticipant>) -> O,
    {
        self.participant_list.iter_mut(f)
    }
}

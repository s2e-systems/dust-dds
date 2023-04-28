use crate::implementation::{
    rtps::types::Guid,
    utils::map::{DdsMap, DdsMapValueIter, DdsMapValueIterMut},
};

use super::dds_entity_list::DdsEntityList;

#[allow(dead_code)]
pub struct DdsDomainEntityList<T, L, P> {
    entity_list: DdsEntityList<T, L>,
    parent_list: DdsMap<Guid, P>,
}

#[allow(dead_code)]
impl<T, L, P> DdsDomainEntityList<T, L, P> {
    pub fn new() -> Self {
        Self {
            entity_list: DdsEntityList::new(),
            parent_list: DdsMap::new(),
        }
    }

    pub fn add_domain_entity(&self, guid: Guid, entity: T, listener: L, parent: P) {
        self.entity_list.add_entity(guid, entity, listener);
        self.parent_list.add(guid, parent);
    }

    pub fn remove_publisher(&self, guid: &Guid) {
        self.entity_list.remove_entity(guid);
        self.parent_list.remove(guid)
    }

    pub fn get_entity<F, O>(&self, guid: &Guid, mut f: F) -> O
    where
        F: FnMut(Option<&T>) -> O,
    {
        self.entity_list.get_entity(guid, |x| f(x))
    }

    pub fn get_entity_mut<F, O>(&self, guid: &Guid, mut f: F) -> O
    where
        F: FnMut(Option<&mut T>) -> O,
    {
        self.entity_list.get_entity_mut(guid, |x| f(x))
    }

    pub fn get_listener<F, O>(&self, guid: &Guid, mut f: F) -> O
    where
        F: FnMut(Option<&L>) -> O,
    {
        self.entity_list.get_listener(guid, |x| f(x))
    }

    pub fn get_listener_mut<F, O>(&self, guid: &Guid, mut f: F) -> O
    where
        F: FnMut(Option<&mut L>) -> O,
    {
        self.entity_list.get_listener_mut(guid, |x| f(x))
    }

    pub fn iter<F, O>(&self, f: F) -> O
    where
        F: for<'a> FnMut(&mut DdsMapValueIter<'a, Guid, T>) -> O,
    {
        self.entity_list.iter(f)
    }

    pub fn iter_mut<F, O>(&self, f: F) -> O
    where
        F: for<'a> FnMut(&mut DdsMapValueIterMut<'a, Guid, T>) -> O,
    {
        self.entity_list.iter_mut(f)
    }
}

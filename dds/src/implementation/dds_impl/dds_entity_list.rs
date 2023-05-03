use crate::implementation::{
    rtps::types::Guid,
    utils::map::{DdsMap, DdsMapValueIter, DdsMapValueIterMut},
};

#[allow(dead_code)]
pub struct DdsEntityList<T, L> {
    entity_list: DdsMap<Guid, T>,
    entity_listener_list: DdsMap<Guid, L>,
}

#[allow(dead_code)]
impl<T, L> DdsEntityList<T, L> {
    pub fn new() -> Self {
        Self {
            entity_list: DdsMap::new(),
            entity_listener_list: DdsMap::new(),
        }
    }

    pub fn add_entity(&self, guid: Guid, entity: T, listener: L) {
        self.entity_list.add(guid, entity);
        self.entity_listener_list.add(guid, listener);
    }

    pub fn remove_entity(&self, guid: &Guid) {
        // Remove the listener first to avoid the entity from disappearing if the listener is being used
        self.entity_listener_list.remove(guid);
        self.entity_list.remove(guid);
    }

    pub fn get_entity<F, O>(&self, guid: &Guid, f: F) -> O
    where
        F: FnOnce(Option<&T>) -> O,
    {
        self.entity_list.get(guid, |x| f(x))
    }

    pub fn get_entity_mut<F, O>(&self, guid: &Guid, f: F) -> O
    where
        F: FnOnce(Option<&mut T>) -> O,
    {
        self.entity_list.get_mut(guid, |x| f(x))
    }

    pub fn get_listener<F, O>(&self, guid: &Guid, f: F) -> O
    where
        F: FnOnce(Option<&L>) -> O,
    {
        self.entity_listener_list.get(guid, |x| f(x))
    }

    pub fn get_listener_mut<F, O>(&self, guid: &Guid, f: F) -> O
    where
        F: FnOnce(Option<&mut L>) -> O,
    {
        self.entity_listener_list.get_mut(guid, |x| f(x))
    }

    pub fn iter<F, O>(&self, f: F) -> O
    where
        F: for<'a> FnOnce(&mut DdsMapValueIter<'a, Guid, T>) -> O,
    {
        self.entity_list.values(f)
    }

    pub fn iter_mut<F, O>(&self, f: F) -> O
    where
        F: for<'a> FnOnce(&mut DdsMapValueIterMut<'a, Guid, T>) -> O,
    {
        self.entity_list.values_mut(f)
    }
}

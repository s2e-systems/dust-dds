use crate::types::GUID;

use super::Entity;


pub struct Group {
    pub entity: Entity,
}

impl Group {
    pub fn new(guid: GUID) -> Self {
        let entity = Entity::new(guid);
        Self {
            entity
        }
    }
}
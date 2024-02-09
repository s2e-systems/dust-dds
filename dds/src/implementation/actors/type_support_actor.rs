use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use dust_dds_derive::actor_interface;

use crate::{
    infrastructure::error::{DdsError, DdsResult},
    topic_definition::type_support::DynamicTypeInterface,
};

pub struct TypeSupportActor {
    type_support_list: HashMap<String, Arc<dyn DynamicTypeInterface + Send + Sync>>,
}

impl TypeSupportActor {
    pub fn new(
        type_support_list: HashMap<String, Arc<dyn DynamicTypeInterface + Send + Sync>>,
    ) -> Self {
        Self { type_support_list }
    }
}

#[actor_interface]
impl TypeSupportActor {
    async fn register_type(
        &mut self,
        type_name: String,
        type_support: Arc<dyn DynamicTypeInterface + Send + Sync>,
    ) -> DdsResult<()> {
        match self.type_support_list.entry(type_name.clone()) {
            Entry::Occupied(_) => Err(DdsError::PreconditionNotMet(format!(
                "Type with name {} is already registered",
                &type_name
            ))),
            Entry::Vacant(e) => {
                e.insert(type_support);
                Ok(())
            }
        }
    }

    async fn get_type_support(
        &self,
        type_name: String,
    ) -> Option<Arc<dyn DynamicTypeInterface + Send + Sync>> {
        self.type_support_list.get(&type_name).cloned()
    }
}

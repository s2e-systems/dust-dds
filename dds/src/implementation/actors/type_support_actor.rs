use std::{collections::HashMap, sync::Arc};

use dust_dds_derive::actor_interface;

use crate::topic_definition::type_support::DynamicTypeInterface;

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
    #[allow(clippy::unused_unit)]
    async fn register_type(
        &mut self,
        type_name: String,
        type_support: Arc<dyn DynamicTypeInterface + Send + Sync>,
    ) -> () {
        self.type_support_list.insert(type_name, type_support);
    }

    async fn get_type_support(
        &self,
        type_name: String,
    ) -> Option<Arc<dyn DynamicTypeInterface + Send + Sync>> {
        self.type_support_list.get(&type_name).cloned()
    }
}

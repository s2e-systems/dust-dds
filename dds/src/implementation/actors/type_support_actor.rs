use std::collections::HashMap;

use dust_dds_derive::actor_interface;

use crate::topic_definition::type_support::TypeSupport;

pub struct TypeSupportActor {
    type_support_list: HashMap<String, TypeSupport>,
}

impl TypeSupportActor {
    pub fn new(type_support_list: HashMap<String, TypeSupport>) -> Self {
        Self { type_support_list }
    }
}

#[actor_interface]
impl TypeSupportActor {
    async fn register_type(&mut self, type_name: String, type_support: TypeSupport) {
        self.type_support_list.insert(type_name, type_support);
    }

    async fn get_type_support(&self, type_name: String) -> Option<TypeSupport> {
        self.type_support_list.get(&type_name).cloned()
    }
}

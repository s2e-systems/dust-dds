use crate::xtypes::{
    dynamic_type::DynamicType,
    type_object::{
        CompleteTypeObject, MinimalTypeObject, TypeIdentifier, TypeIdentifierWithSize,
        TypeInformation, TypeObject, get_type_dependencies_with_size,
    },
};
use alloc::{sync::Arc, vec::Vec};

#[derive(Debug, Clone, PartialEq)]
pub enum RegisteredType {
    Local {
        type_name: Arc<str>,
        dynamic_type: DynamicType<'static>,
        type_information: TypeInformation,
    },
    Discovered {
        type_identifier: TypeIdentifier,
        type_object: Option<TypeObject>,
        dependencies: Option<Vec<TypeIdentifierWithSize>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeLookupPendingState {
    PendingDependencies(TypeIdentifier),
    PendingTypes(Vec<TypeIdentifier>),
}

#[derive(Default, Debug)]
pub struct TypeRegister {
    types: Vec<RegisteredType>,
    pending_lookups: Vec<TypeLookupPendingState>,
}

impl TypeRegister {
    pub const fn new() -> Self {
        Self {
            types: Vec::new(),
            pending_lookups: Vec::new(),
        }
    }

    /// Registers a locally defined type and recursively all its constructed dependencies.
    /// Returns the `TypeInformation` for the root type.
    pub fn register_local_type(
        &mut self,
        type_name: Arc<str>,
        dynamic_type: DynamicType<'static>,
    ) -> TypeInformation {
        let type_information = TypeInformation::from(dynamic_type);

        // Register the root type if not already present
        if !self.contains_type_id(&type_information.complete.typeid_with_size.type_id) {
            self.types.push(RegisteredType::Local {
                type_name: type_name.clone(),
                dynamic_type,
                type_information: type_information.clone(),
            });
        }

        // Recursively register all dependent types
        for dep in dynamic_type.get_dependencies() {
            let dep_type_info = TypeInformation::from(dep);
            if !self.contains_type_id(&dep_type_info.complete.typeid_with_size.type_id) {
                self.types.push(RegisteredType::Local {
                    type_name: Arc::from(dep.descriptor.name),
                    dynamic_type: dep,
                    type_information: dep_type_info,
                });
            }
        }

        type_information
    }

    /// Checks if a type ID is present in the register (either Local or Discovered).
    pub fn contains_type_id(&self, type_id: &TypeIdentifier) -> bool {
        self.types.iter().any(|t| match t {
            RegisteredType::Local {
                type_information, ..
            } => {
                &type_information.complete.typeid_with_size.type_id == type_id
                    || &type_information.minimal.typeid_with_size.type_id == type_id
            }
            RegisteredType::Discovered {
                type_identifier, ..
            } => type_identifier == type_id,
        })
    }

    /// Gets the `TypeObject` for a given `TypeIdentifier` if available.
    pub fn get_type_object(&self, type_id: &TypeIdentifier) -> Option<TypeObject> {
        for t in &self.types {
            match t {
                RegisteredType::Local {
                    dynamic_type,
                    type_information,
                    ..
                } => {
                    if &type_information.complete.typeid_with_size.type_id == type_id {
                        return Some(TypeObject::EkComplete {
                            complete: CompleteTypeObject::from(*dynamic_type),
                        });
                    } else if &type_information.minimal.typeid_with_size.type_id == type_id {
                        return Some(TypeObject::EkMinimal {
                            minimal: MinimalTypeObject::from(*dynamic_type),
                        });
                    }
                }
                RegisteredType::Discovered {
                    type_identifier,
                    type_object,
                    ..
                } => {
                    if type_identifier == type_id {
                        return type_object.clone();
                    }
                }
            }
        }
        None
    }

    /// Gets the list of `TypeIdentifierWithSize` dependencies for a given `TypeIdentifier`.
    pub fn get_type_dependencies_with_size(
        &self,
        type_id: &TypeIdentifier,
    ) -> Option<Vec<TypeIdentifierWithSize>> {
        for t in &self.types {
            match t {
                RegisteredType::Local {
                    dynamic_type,
                    type_information,
                    ..
                } => {
                    if &type_information.complete.typeid_with_size.type_id == type_id {
                        return Some(get_type_dependencies_with_size(*dynamic_type));
                    }
                }
                RegisteredType::Discovered {
                    type_identifier,
                    dependencies,
                    ..
                } => {
                    if type_identifier == type_id {
                        return dependencies.clone();
                    }
                }
            }
        }
        None
    }

    /// Gets the `DynamicType` for a locally registered type if available.
    #[allow(dead_code)]
    pub fn get_dynamic_type(&self, type_id: &TypeIdentifier) -> Option<DynamicType<'static>> {
        for t in &self.types {
            if let RegisteredType::Local {
                dynamic_type,
                type_information,
                ..
            } = t
            {
                if &type_information.complete.typeid_with_size.type_id == type_id
                    || &type_information.minimal.typeid_with_size.type_id == type_id
                {
                    return Some(*dynamic_type);
                }
            }
        }
        None
    }

    /// Registers the received dependencies for a discovered type.
    pub fn register_type_dependencies(
        &mut self,
        type_id: &TypeIdentifier,
        deps: Vec<TypeIdentifierWithSize>,
    ) {
        let mut found = false;
        for t in &mut self.types {
            if let RegisteredType::Discovered {
                type_identifier,
                dependencies,
                ..
            } = t
            {
                if type_identifier == type_id {
                    *dependencies = Some(deps.clone());
                    found = true;
                    break;
                }
            }
        }
        if !found {
            self.types.push(RegisteredType::Discovered {
                type_identifier: type_id.clone(),
                type_object: None,
                dependencies: Some(deps.clone()),
            });
        }

        // Also add discovered entries for each dependent type if not present
        for dep in deps {
            if !self.contains_type_id(&dep.type_id) {
                self.types.push(RegisteredType::Discovered {
                    type_identifier: dep.type_id,
                    type_object: None,
                    dependencies: None,
                });
            }
        }
    }

    /// Registers a discovered `TypeObject` for a given `TypeIdentifier`.
    pub fn register_discovered_type_object(
        &mut self,
        type_id: TypeIdentifier,
        type_object: TypeObject,
    ) {
        for t in &mut self.types {
            if let RegisteredType::Discovered {
                type_identifier,
                type_object: existing_obj,
                ..
            } = t
            {
                if type_identifier == &type_id {
                    *existing_obj = Some(type_object);
                    return;
                }
            }
        }
        self.types.push(RegisteredType::Discovered {
            type_identifier: type_id,
            type_object: Some(type_object),
            dependencies: None,
        });
    }

    /// Returns true if the type identified by `type_id` and all its recursive dependencies have their `TypeObject` available.
    pub fn is_type_resolved(&self, type_id: &TypeIdentifier) -> bool {
        if self.get_type_object(type_id).is_none() {
            return false;
        }

        if let Some(deps) = self.get_type_dependencies_with_size(type_id) {
            for dep in deps {
                if !self.is_type_resolved(&dep.type_id) {
                    return false;
                }
            }
        }

        true
    }

    /// Returns all unresolved `TypeIdentifier`s for `type_id` and its dependencies.
    pub fn get_unresolved_type_ids(&self, type_id: &TypeIdentifier) -> Vec<TypeIdentifier> {
        let mut unresolved = Vec::new();
        self.collect_unresolved(type_id, &mut unresolved);
        unresolved
    }

    fn collect_unresolved(&self, type_id: &TypeIdentifier, out: &mut Vec<TypeIdentifier>) {
        if self.get_type_object(type_id).is_none() && !out.contains(type_id) {
            out.push(type_id.clone());
        }
        if let Some(deps) = self.get_type_dependencies_with_size(type_id) {
            for dep in deps {
                self.collect_unresolved(&dep.type_id, out);
            }
        }
    }

    /// Pending lookups management
    pub fn is_dependencies_lookup_pending(&self, type_id: &TypeIdentifier) -> bool {
        self.pending_lookups.iter().any(|p| match p {
            TypeLookupPendingState::PendingDependencies(id) => id == type_id,
            _ => false,
        })
    }

    pub fn is_types_lookup_pending(&self, type_ids: &[TypeIdentifier]) -> bool {
        self.pending_lookups.iter().any(|p| match p {
            TypeLookupPendingState::PendingTypes(ids) => {
                type_ids.iter().any(|req_id| ids.contains(req_id))
            }
            _ => false,
        })
    }

    pub fn add_pending_dependencies_lookup(&mut self, type_id: TypeIdentifier) {
        if !self.is_dependencies_lookup_pending(&type_id) {
            self.pending_lookups
                .push(TypeLookupPendingState::PendingDependencies(type_id));
        }
    }

    pub fn add_pending_types_lookup(&mut self, type_ids: Vec<TypeIdentifier>) {
        self.pending_lookups
            .push(TypeLookupPendingState::PendingTypes(type_ids));
    }

    pub fn remove_pending_dependencies_lookup(&mut self, type_id: &TypeIdentifier) {
        self.pending_lookups.retain(|p| match p {
            TypeLookupPendingState::PendingDependencies(id) => id != type_id,
            _ => true,
        });
    }

    pub fn remove_pending_types_lookup(&mut self, type_ids: &[TypeIdentifier]) {
        self.pending_lookups.retain(|p| match p {
            TypeLookupPendingState::PendingTypes(ids) => {
                !ids.iter().any(|id| type_ids.contains(id))
            }
            _ => true,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xtypes::type_support::TypeSupport;

    #[test]
    fn register_local_type_with_dependencies() {
        #[derive(Debug, PartialEq, TypeSupport)]
        enum MyEnum {
            A,
            B,
        }

        #[derive(Debug, PartialEq, TypeSupport)]
        struct MyStruct {
            id: u32,
            kind: MyEnum,
        }

        let mut register = TypeRegister::new();
        let type_info = register.register_local_type(Arc::from("MyStruct"), MyStruct::get_type());

        let struct_id = &type_info.complete.typeid_with_size.type_id;
        assert!(register.contains_type_id(struct_id));
        assert!(register.get_type_object(struct_id).is_some());

        let deps = register.get_type_dependencies_with_size(struct_id).unwrap();
        assert_eq!(deps.len(), 1);

        let enum_id = &deps[0].type_id;
        assert!(register.contains_type_id(enum_id));
        assert!(register.get_type_object(enum_id).is_some());
        assert!(register.is_type_resolved(struct_id));
    }

    #[test]
    fn register_discovered_type_resolution() {
        let mut register = TypeRegister::new();
        let main_id = TypeIdentifier::EkComplete {
            equivalence_hash: [1; 14],
        };
        let dep_id = TypeIdentifier::EkComplete {
            equivalence_hash: [2; 14],
        };

        // Main type is registered with dependencies, but neither object is present yet
        register.register_type_dependencies(
            &main_id,
            vec![TypeIdentifierWithSize {
                type_id: dep_id.clone(),
                typeobject_serialized_size: 50,
            }],
        );

        assert!(!register.is_type_resolved(&main_id));
        assert_eq!(
            register.get_unresolved_type_ids(&main_id),
            vec![main_id.clone(), dep_id.clone()]
        );

        // Register dep type object
        register.register_discovered_type_object(
            dep_id.clone(),
            TypeObject::EkComplete {
                complete: CompleteTypeObject::from(MyDummyEnum::get_type()),
            },
        );
        assert!(!register.is_type_resolved(&main_id));
        assert_eq!(
            register.get_unresolved_type_ids(&main_id),
            vec![main_id.clone()]
        );

        // Register main type object
        register.register_discovered_type_object(
            main_id.clone(),
            TypeObject::EkComplete {
                complete: CompleteTypeObject::from(MyDummyEnum::get_type()),
            },
        );
        assert!(register.is_type_resolved(&main_id));
        assert!(register.get_unresolved_type_ids(&main_id).is_empty());
    }

    #[derive(Debug, PartialEq, TypeSupport)]
    enum MyDummyEnum {
        X,
        Y,
    }
}

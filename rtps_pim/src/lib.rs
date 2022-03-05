// Enable std crate for testing only
#![cfg_attr(not(test), no_std)]

/// This crate implements the chapter
/// "8 Platform Independent Model (PIM)" from
/// The Real-time Publish-Subscribe Protocol DDS Interoperability Wire Protocol (DDSI-RTPS) Specification
///
/// To enable implementations of it on constraint platforms (i.e without heap allocation)
/// this crate shall not use the std library.
///
/// The standard is mapped in these ways:
/// 1. Types that have their own chapter, i.e. types that have the RTPS prefix
/// are realized with traits. E.g. RTPS HistoryCache
/// 2. Types that have an _t are mapped as concrete types
/// 3. The behavior is realized by generic methods on unit structs with the name <BehaviorName>Behavior
///     e.g.: BestEffortStatefulReaderBehavior
///
/// Detailed mapping description of RTPS types:
/// - the attribute table of a type is mapped to an Rtps<TypeName>Attributes trait
/// - the Operations table (excluding the new operation) is mapped to a Rtps<TypeName>Operations trait
/// - the new operation is mapped to a  Rtps<TypeName>Constructor trait
///
/// Mapping rules:
/// - If types are used in those traits that are not _t types an associated type shall be used.
///     The name shall be <TypeName>Type
/// - The Rtps<TypeName>Attributes trait shall return the attributes in the individual methods as references.
///      - That guarantees that the attributes have to be implements as attributes, i.e. as struct fields
///      - Exception (very common): _t types shall be returned as copy
/// - The Rtps<TypeName>Constructor shall use move semantics for its parameters
///     - Lists are an exception: a slice reference shall be used.
///     - That is because the arguments are to be owned by the struct implementing the trait
/// - Rtps<TypeName>Operations shall additionally define an associated type is a list needs to be returned
///     - for _t types and other types
///     - the name of the associated type shall be <TypeName>ListType
///
pub mod behavior;
pub mod messages;
pub mod structure;
pub mod discovery;

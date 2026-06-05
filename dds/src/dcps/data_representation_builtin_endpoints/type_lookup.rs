use dust_dds_derive::DdsType;

use crate::{
    transport::types::{Guid, SequenceNumber},
    xtypes::type_object::{
        TypeIdentifier, TypeIdentifierPair, TypeIdentifierTypeObjectPair, TypeIdentifierWithSize,
    },
};

// // computed from @hashid("getTypes")
pub const TYPE_LOOKUP_GET_TYPES_HASH_ID: i32 = 0x018252d3;
// computed from @hashid("getDependencies");
pub const TYPE_LOOKUP_GET_DEPENDENCIES_HASH_ID: i32 = 0x05aafb31;

// Query the TypeObjects associated with one or more TypeIdentifiers
#[derive(DdsType)]
#[dust_dds(extensibility = "mutable")]
pub struct TypeLookupGetTypesIn {
    #[dust_dds(hashid)]
    pub type_ids: Vec<TypeIdentifier>,
}

#[derive(DdsType)]
#[dust_dds(extensibility = "mutable")]
pub struct TypeLookupGetTypesOut {
    #[dust_dds(hashid)]
    pub types: Vec<TypeIdentifierTypeObjectPair>,
    #[dust_dds(hashid)]
    pub complete_to_minimal: Vec<TypeIdentifierPair>,
}

// union TypeLookup_getTypes_Result switch(long) {
pub enum TypeLookupGetTypesResult {
    Ok { result: TypeLookupGetTypesOut },
}

// Query TypeIdentifiers that the specified types depend on
#[derive(DdsType)]
#[dust_dds(extensibility = "mutable")]
pub struct TypeLookupGetTypeDependenciesIn {
    #[dust_dds(hashid)]
    pub type_ids: Vec<TypeIdentifier>, // @hashid
    #[dust_dds(hashid)]
    pub continuation_point: Vec<u8>, // @hashid sequence<octet, 32> ;
}

// @extensibility(MUTABLE)
pub struct TypeLookupGetTypeDependenciesOut {
    pub dependent_typeids: Vec<TypeIdentifierWithSize>, //@hashid
    pub continuation_point: Vec<u8>,                    //@hashid sequence<octet, 32>
}

// union TypeLookup_getTypeDependencies_Result switch(long){
pub enum TypeLookupGetTypeDependenciesResult {
    Ok {
        result: TypeLookupGetTypeDependenciesOut,
    },
}

// Service Request
// union TypeLookup_Call switch(long) {
#[derive(DdsType)]
#[dust_dds(switch(i32))]
pub enum TypeLookupCall {
    #[dust_dds(case=TYPE_LOOKUP_GET_TYPES_HASH_ID)]
    TypeLookupGetTypesHashId { get_types: TypeLookupGetTypesIn },
    #[dust_dds(case=TYPE_LOOKUP_GET_DEPENDENCIES_HASH_ID)]
    TypeLookupGetDependenciesHash {
        get_type_dependencies: TypeLookupGetTypeDependenciesIn,
    },
}

#[derive(DdsType)]
pub struct TypeLookupRequest {
    pub header: RequestHeader,
    pub call: TypeLookupCall,
}

// Service Reply
// union TypeLookup_Return switch(long) {
pub enum TypeLookupReturn {
    TypeLookupGetTypesHash {
        get_type: TypeLookupGetTypesResult,
    },
    TypeLookupGetDependenciesHash {
        get_type_dependencies: TypeLookupGetTypeDependenciesResult,
    },
}

pub struct TypeLookupReply {
    pub header: RequestHeader,
    pub r#return: TypeLookupReturn,
}

// DDS RPC Types
#[derive(DdsType)]
pub struct RequestHeader {
    request_id: SampleIdentity,
    instance_name: InstanceName,
}

pub struct ReplyHeader {
    related_request_id: SampleIdentity,
}

#[derive(DdsType)]
pub struct SampleIdentity {
    writer_guid: Guid,
    sequence_number: SequenceNumber,
}

pub type InstanceName = String; //typedef string<255> InstanceName;

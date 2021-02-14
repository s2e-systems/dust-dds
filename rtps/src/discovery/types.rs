// This file implements the types that appear in the built-in topic messages
// using the mapping described in 9.3.2 Mapping of the Types that Appear Within Submessages or Built-in Topic Data

pub type DomainId = u32;

pub struct Time {
    pub seconds: u32,
    pub fraction: u32,
}

pub struct BuiltinEndpointSet(u32);

pub struct BuiltinEndpointQos(u32);

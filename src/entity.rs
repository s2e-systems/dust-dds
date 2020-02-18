use crate::types::GUID;

/// RTPS Entity is the base class for all RTPS entities and maps to a DDS Entity.
pub struct Entity {
    /// Globally and uniquely identifies the RTPS Entity within the DDS domain.
    /// Maps to the value of the DDS BuiltinTopicKey_t used to describe the corresponding DDS Entity.
    pub guid: GUID,
}

#[cfg(test)]
mod tests {
    use super::*;
}

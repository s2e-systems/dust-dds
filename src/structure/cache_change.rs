use std::cmp::Ordering;

use crate::types::{ChangeKind, InstanceHandle, SequenceNumber, GUID, };
use crate::serialized_payload::ParameterList;

#[derive(Debug)]
pub struct CacheChange {
    kind: ChangeKind,
    writer_guid: GUID,
    instance_handle: InstanceHandle,
    sequence_number: SequenceNumber,
    data_value: Vec<u8>,
    inline_qos: ParameterList,
}

impl CacheChange {
    pub fn new(
        kind: ChangeKind,
        writer_guid: GUID,
        instance_handle: InstanceHandle,
        sequence_number: SequenceNumber,
        data_value: Option<Vec<u8>>,
        inline_qos: Option<ParameterList>,
    ) -> CacheChange {
        let data_value = data_value.unwrap_or(Vec::new());
        let inline_qos = inline_qos.unwrap_or(ParameterList::new());
        CacheChange {
            kind,
            writer_guid,
            instance_handle,
            sequence_number,
            inline_qos,
            data_value,
        }
    }

    pub fn change_kind(&self) -> &ChangeKind {
        &self.kind
    }

    pub fn writer_guid(&self) -> &GUID {
        &self.writer_guid
    }

    pub fn instance_handle(&self) -> &InstanceHandle {
        &self.instance_handle
    }

    pub fn sequence_number(&self) -> SequenceNumber {
        self.sequence_number
    }

    pub fn inline_qos(&self) -> &ParameterList {
        &self.inline_qos
    }

    pub fn data_value(&self) -> &Vec<u8> {
        &self.data_value
    }

    pub fn clone_without_data(&self) -> Self {
        Self {
            data_value: Vec::new(),
            inline_qos: ParameterList::new(),
            .. *self
        }
    }
}

// Two cache changes with the same writer GUID, instance handle and sequence number
// are considered equal
impl PartialEq for CacheChange {
    fn eq(&self, other: &Self) -> bool {
        self.writer_guid == other.writer_guid
            && self.instance_handle == other.instance_handle
            && self.sequence_number == other.sequence_number
    }
}

impl Eq for CacheChange {}

// Cache changes are explcitly ordered by their sequence number
impl Ord for CacheChange {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sequence_number.cmp(&other.sequence_number)
    }
}

impl PartialOrd for CacheChange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.sequence_number.cmp(&other.sequence_number))
    }
}

impl ::core::hash::Hash for CacheChange {
    fn hash<__H: ::core::hash::Hasher>(&self, state: &mut __H) -> () {
        match *self {
            CacheChange {
                kind: ref __self_0_0,
                writer_guid: ref __self_0_1,
                instance_handle: ref __self_0_2,
                sequence_number: ref __self_0_3,
                data_value: ref __self_0_5,
                inline_qos: ref __self_0_4,
            } => {
                // The Hash should be such that two equal values return the same
                // hash value. Therefore some field must be ignored.

                // Explicitly ignore the change kind field
                // ::core::hash::Hash::hash(&(*__self_0_0), state);
                ::core::hash::Hash::hash(&(*__self_0_1), state);
                ::core::hash::Hash::hash(&(*__self_0_2), state);
                ::core::hash::Hash::hash(&(*__self_0_3), state);
                // Explicitly ignore the data_value and inline_qos fields
                // ::core::hash::Hash::hash(&(*__self_0_4), state)
                // ::core::hash::Hash::hash(&(*__self_0_5), state)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EntityId, EntityKind};

    #[test]
    fn cache_change_equality_and_ordering() {
        let cc1 = CacheChange::new(
            ChangeKind::Alive,
            GUID::new([1;12], EntityId::new([1;3], EntityKind::UserDefinedUnknown) ),
            [1;16],
            1,
            Some(vec![1,2,3]),
            None,
        );

        let cc2 = CacheChange::new(
            ChangeKind::NotAliveDisposed,
            GUID::new([1;12], EntityId::new([1;3], EntityKind::UserDefinedUnknown) ),
            [1;16],
            1,
            Some(vec![5]),
            None,
        );

        let cc3 = CacheChange::new(
            ChangeKind::NotAliveDisposed,
            GUID::new([1;12], EntityId::new([1;3], EntityKind::UserDefinedUnknown) ),
            [1;16],
            2,
            Some(vec![5]),
            None,
        );

        // Cache changes with the same writer_guid, instance_handle and sequence_number are equal
        assert_eq!(cc1,cc2);
        assert_ne!(cc1,cc3);

        // Cache changes are ordered by the sequence number
        assert!(cc3>cc1);
    }
    
}

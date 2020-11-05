use std::cmp::{Ord, Ordering};
use crate::types::{ChangeKind, InstanceHandle, SequenceNumber, Data, ParameterList};

pub struct CacheChange {
    kind: ChangeKind,
    writer_guid: InstanceHandle /*GUID*/,
    instance_handle: InstanceHandle,
    sequence_number: SequenceNumber,
    data_value: Option<Data>,
    inline_qos: Option<ParameterList>,
}

impl CacheChange {
    pub fn new(
        kind: ChangeKind,
        writer_guid: InstanceHandle /*GUID*/,
        instance_handle: InstanceHandle,
        sequence_number: SequenceNumber,
        data_value: Option<Data>,
        inline_qos: Option<ParameterList>,
    ) -> CacheChange {
        CacheChange {
            kind,
            writer_guid,
            instance_handle,
            sequence_number,
            inline_qos,
            data_value,
        }
    }

    pub fn change_kind(&self) -> ChangeKind {
        self.kind
    }

    pub fn writer_guid(&self) -> InstanceHandle /*GUID*/ {
        self.writer_guid
    }

    pub fn instance_handle(&self) -> InstanceHandle {
        self.instance_handle
    }

    pub fn sequence_number(&self) -> SequenceNumber {
        self.sequence_number
    }

    pub fn inline_qos(&self) -> &Option<ParameterList> {
        &self.inline_qos
    }

    pub fn data_value(&self) -> &Option<Data> {
        &self.data_value
    }
}


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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::types::{EntityId, EntityKind};

//     #[test]
//     fn cache_change_equality_and_ordering() {
//         let cc1 = CacheChange::new(
//             ChangeKind::Alive,
//             GUID::new([1;12], EntityId::new([1;3], EntityKind::UserDefinedUnknown) ),
//             [1;16],
//             1,
//             Some(vec![1,2,3]),
//             None,
//         );

//         let cc2 = CacheChange::new(
//             ChangeKind::NotAliveDisposed,
//             GUID::new([1;12], EntityId::new([1;3], EntityKind::UserDefinedUnknown) ),
//             [1;16],
//             1,
//             Some(vec![5]),
//             None,
//         );

//         let cc3 = CacheChange::new(
//             ChangeKind::NotAliveDisposed,
//             GUID::new([1;12], EntityId::new([1;3], EntityKind::UserDefinedUnknown) ),
//             [1;16],
//             2,
//             Some(vec![5]),
//             None,
//         );

//         // Cache changes with the same writer_guid, instance_handle and sequence_number are equal
//         assert_eq!(cc1,cc2);
//         assert_ne!(cc1,cc3);

//         // Cache changes are ordered by the sequence number
//         assert!(cc3>cc1);
//     }
    
// }
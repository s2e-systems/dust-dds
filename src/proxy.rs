
use crate::types::{LocatorList, GUID, EntityId, SequenceNumber, SequenceNumberSet};
use crate::cache::{CacheChange};

enum ChangeFromWriterStatusKind
{
    Lost,
    Missing,
    Received,
    Unknown,
}

pub struct ChangeFromWriter
{
    status : ChangeFromWriterStatusKind,
    is_relevant : bool,
}

pub struct WriterProxy
{
    remote_writer_guid : GUID,
    unicast_locator_list : LocatorList,
    multicast_locator_list : LocatorList,
    data_max_size_serialized : Option<i32>,
    changes_from_writer : Vec<CacheChange>,
    remote_group_entity_id : EntityId,
}

impl WriterProxy
{
    pub fn new(/*attribute_values : Set<Attribute>*/) -> WriterProxy
    {
        unimplemented!();
    }

    pub fn available_changes_max() -> SequenceNumber
    {
        unimplemented!();
    }

    pub fn irrelevant_change_set(a_seq_num : SequenceNumber) 
    {
        unimplemented!();
    }

    pub fn lost_changes_update(first_available_seq_num : SequenceNumber)
    {
        unimplemented!();
    }

    pub fn missing_changes() -> SequenceNumberSet
    {
        unimplemented!();
    }

    pub fn missing_changes_update(last_available_seq_num : SequenceNumber)
    {
        unimplemented!();
    }

    pub fn received_change_set(a_seq_num : SequenceNumber) 
    {
        unimplemented!();
    }
}
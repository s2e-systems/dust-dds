
use crate::types::{LocatorList, GUID, EntityId, SequenceNumber, SequenceNumberSet};
use crate::cache::{ReaderCacheChange, ReaderHistoryCache, ChangeFromWriterStatusKind};


pub struct WriterProxy<'a>
{
    remote_writer_guid : GUID,
    unicast_locator_list : LocatorList,
    multicast_locator_list : LocatorList,
    data_max_size_serialized : Option<i32>,
    changes_from_writer : &'a ReaderHistoryCache,
    remote_group_entity_id : EntityId,
}



impl WriterProxy<'_>
{
    pub fn new( remote_writer_guid : GUID,
        unicast_locator_list : LocatorList,
        multicast_locator_list : LocatorList,
        data_max_size_serialized : Option<i32>,
        changes_from_writer : &ReaderHistoryCache,
        remote_group_entity_id : EntityId) -> WriterProxy
    {
        WriterProxy{
            remote_writer_guid, unicast_locator_list, multicast_locator_list, data_max_size_serialized, changes_from_writer, remote_group_entity_id
        }
    }

    pub fn remote_writer_guid(&self) -> GUID
    {
        self.remote_writer_guid
    }

    pub fn available_changes_max(&self) -> Option<SequenceNumber>
    {
        let cache_changes_lock = self.changes_from_writer.changes.lock().unwrap();

        let cache_change = cache_changes_lock.iter()
            .filter(|&rcc|rcc.cache_change.writer_guid == self.remote_writer_guid)
            .filter(|&rcc|rcc.cache_change.is_status(ChangeFromWriterStatusKind::Lost) || rcc.cache_change.is_status(ChangeFromWriterStatusKind::Missing))
            .max();
        match cache_change 
        {
            None => None,
            Some(a) => Some(a.cache_change.sequence_number),
        }
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

    pub fn received_change_set(&self, a_seq_num : SequenceNumber) 
    {
        let mut cache_changes_lock = self.changes_from_writer.changes.lock().unwrap();
        let mut reader_cache_change = cache_changes_lock.iter_mut()
        .filter(|&rcc|rcc.cache_change.writer_guid == self.remote_writer_guid)
        .find(|&rcc|rcc.cache_change.sequence_number == a_seq_num).unwrap();
        reader_cache_change.change_from_writer.status = ChangeFromWriterStatusKind::Received;
    }
}

#[cfg(test)]
mod tests{

    use super::*;
    use crate::types::{EntityId, ChangeKind, InstanceHandle};
    
    #[test]
    fn test_writer_proxy_max(){
        let hc = ReaderHistoryCache::new();

        let writer_guid = GUID::new([0,1,2,3,4,5,6,7,8,9,10,11], EntityId::new([0,1,0], 1));
        let sequence_number = 1;
        let instance_handle = [1; 16];
        let cc = ReaderCacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, sequence_number, None, None);
        let other_writer_guid = GUID::new([12,1,2,3,4,5,6,7,8,9,10,11], EntityId::new([0,1,0], 1));
        let other_cc = ReaderCacheChange::new(ChangeKind::Alive, other_writer_guid, instance_handle, sequence_number, None, None);
        let yet_other_cc = ReaderCacheChange::new(ChangeKind::Alive, other_writer_guid, instance_handle, sequence_number+2, None, None);

        hc.add_change(cc);
        hc.add_change(other_cc);
        hc.add_change(yet_other_cc);

        let remote_group_entity_id = EntityId::new([0,1,0], 2);
        let writer_proxy = WriterProxy::new(writer_guid, Vec::new(), Vec::new(), None, &hc, remote_group_entity_id);

        let result = writer_proxy.available_changes_max();
        assert_eq!(result, Some(sequence_number));

        let cc = ReaderCacheChange::new(ChangeKind::Alive, writer_guid, instance_handle, sequence_number+1, None, None);
        hc.add_change(cc);

        let result = writer_proxy.available_changes_max();
        assert_eq!(result, Some(sequence_number+1));




    }
}

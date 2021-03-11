use crate::types::SequenceNumber;

use super::RTPSCacheChange;
pub trait RTPSHistoryCache {
    type CacheChangeType: RTPSCacheChange;

    /// This operation creates a new RTPS HistoryCache. The newly-created history cache is initialized with an empty list of changes.
    fn new() -> Self;

    /// This operation inserts the CacheChange a_change into the HistoryCache.
    /// This operation will only fail if there are not enough resources to add the change to the HistoryCache. It is the responsibility
    /// of the DDS service implementation to configure the HistoryCache in a manner consistent with the DDS Entity RESOURCE_LIMITS QoS
    /// and to propagate any errors to the DDS-user in the manner specified by the DDS specification.
    fn add_change(&mut self, change: Self::CacheChangeType);

    /// This operation indicates that a previously-added CacheChange has become irrelevant and the details regarding the CacheChange need
    /// not be maintained in the HistoryCache. The determination of irrelevance is made based on the QoS associated with the related DDS
    /// entity and on the acknowledgment status of the CacheChange. This is described in 8.4.1.
    fn remove_change(&mut self, seq_num: SequenceNumber);

    fn get_change(&self, seq_num: SequenceNumber) -> Option<&Self::CacheChangeType>;

    /// This operation retrieves the smallest value of the CacheChange::sequenceNumber attribute among the CacheChange stored in the HistoryCache.
    fn get_seq_num_min(&self) -> Option<SequenceNumber>;

    /// This operation retrieves the largest value of the CacheChange::sequenceNumber attribute among the CacheChange stored in the HistoryCache.
    fn get_seq_num_max(&self) -> Option<SequenceNumber>;
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::types::{EntityId, EntityKind, GUID, ChangeKind};

//     #[test]
//     fn cache_change_list() {
//         let mut history_cache = HistoryCache::new(HistoryCacheResourceLimits::default() );
//         let guid_prefix = [8; 12];
//         let entity_id = EntityId::new([1, 2, 3], EntityKind::BuiltInReaderWithKey);
//         let guid = GUID::new(guid_prefix, entity_id);
//         let instance_handle = [9; 16];
//         let sequence_number = 1;
//         let data_value = Some(vec![4, 5, 6]);
//         let cc = CacheChange::new(
//             ChangeKind::Alive,
//             guid,
//             instance_handle,
//             sequence_number,
//             data_value,
//             None,
//         );

//         let cc_copy_no_data =  CacheChange::new(
//             ChangeKind::Alive,
//             guid,
//             instance_handle,
//             sequence_number,
//             None,
//             None,
//         );

//         assert_eq!(history_cache.changes().len(), 0);
//         history_cache.add_change(cc).unwrap();
//         assert_eq!(history_cache.changes().len(), 1);
//         history_cache.remove_change(&cc_copy_no_data);
//         assert_eq!(history_cache.changes().len(), 0);
//     }

//     #[test]
//     fn cache_change_sequence_number() {
//         let mut history_cache = HistoryCache::new(HistoryCacheResourceLimits::default());

//         let guid_prefix = [8; 12];
//         let entity_id = EntityId::new([1, 2, 3], EntityKind::BuiltInReaderWithKey);
//         let guid = GUID::new(guid_prefix, entity_id);
//         let instance_handle = [9; 16];
//         let data_value = Some(vec![4, 5, 6]);
//         let sequence_number_min = 1;
//         let sequence_number_max = 2;
//         let cc1 = CacheChange::new(
//             ChangeKind::Alive,
//             guid.clone(),
//             instance_handle,
//             sequence_number_min,
//             data_value.clone(),
//             None,
//         );
//         let cc2 = CacheChange::new(
//             ChangeKind::Alive,
//             guid.clone(),
//             instance_handle,
//             sequence_number_max,
//             data_value.clone(),
//             None,
//         );

//         assert_eq!(history_cache.get_seq_num_max(), None);
//         history_cache.add_change(cc1).unwrap();
//         assert_eq!(
//             history_cache.get_seq_num_min(),
//             history_cache.get_seq_num_max()
//         );
//         history_cache.add_change(cc2).unwrap();
//         assert_eq!(history_cache.get_seq_num_min(), Some(sequence_number_min));
//         assert_eq!(history_cache.get_seq_num_max(), Some(sequence_number_max));
//     }

//     #[test]
//     fn max_samples_reached() {
//         let history_cache_resource_limits = HistoryCacheResourceLimits{
//             max_samples: 2,
//             max_instances: LENGTH_UNLIMITED,
//             max_samples_per_instance: 2};
//         let mut history_cache = HistoryCache::new(history_cache_resource_limits);

//         let entity_id = EntityId::new([1, 2, 3], EntityKind::BuiltInReaderWithKey);
//         let instance_handle = [9; 16];

//         let cc1 = CacheChange::new(
//             ChangeKind::Alive,
//             GUID::new([1;12], entity_id),
//             instance_handle,
//             1,
//             None,
//             None,
//         );
//         let cc2 = CacheChange::new(
//             ChangeKind::Alive,
//             GUID::new([2;12], entity_id),
//             instance_handle,
//             2,
//             None,
//             None,
//         );
//         let cc3 = CacheChange::new(
//             ChangeKind::Alive,
//             GUID::new([3;12], entity_id),
//             instance_handle,
//             2,
//             None,
//             None,
//         );

//         history_cache.add_change(cc1).unwrap();
//         history_cache.add_change(cc2).unwrap();

//         assert_eq!(history_cache.add_change(cc3), Err(ReturnCodes::OutOfResources));
//     }

//     #[test]
//     fn max_instances_reached() {
//         let history_cache_resource_limits = HistoryCacheResourceLimits{
//             max_samples: LENGTH_UNLIMITED,
//             max_instances: 2,
//             max_samples_per_instance: LENGTH_UNLIMITED};
//         let mut history_cache = HistoryCache::new(history_cache_resource_limits);

//         let entity_id = EntityId::new([1, 2, 3], EntityKind::BuiltInReaderWithKey);

//         let cc1 = CacheChange::new(
//             ChangeKind::Alive,
//             GUID::new([1;12], entity_id),
//             [9; 16],
//             1,
//             None,
//             None,
//         );
//         let cc2 = CacheChange::new(
//             ChangeKind::Alive,
//             GUID::new([1;12], entity_id),
//             [8; 16],
//             2,
//             None,
//             None,
//         );
//         let cc3 = CacheChange::new(
//             ChangeKind::Alive,
//             GUID::new([1;12], entity_id),
//             [7; 16],
//             2,
//             None,
//             None,
//         );

//         history_cache.add_change(cc1).unwrap();
//         history_cache.add_change(cc2).unwrap();

//         assert_eq!(history_cache.add_change(cc3), Err(ReturnCodes::OutOfResources));
//     }

//     #[test]
//     fn max_samples_per_instance_reached() {
//         let history_cache_resource_limits = HistoryCacheResourceLimits{
//             max_samples: LENGTH_UNLIMITED,
//             max_instances: LENGTH_UNLIMITED,
//             max_samples_per_instance: 2};
//         let mut history_cache = HistoryCache::new(history_cache_resource_limits);

//         let guid_prefix = [8; 12];
//         let entity_id = EntityId::new([1, 2, 3], EntityKind::BuiltInReaderWithKey);
//         let guid = GUID::new(guid_prefix, entity_id);
//         let instance_handle = [9; 16];

//         let cc1 = CacheChange::new(
//             ChangeKind::Alive,
//             guid.clone(),
//             instance_handle,
//             1,
//             None,
//             None,
//         );
//         let cc2 = CacheChange::new(
//             ChangeKind::Alive,
//             guid.clone(),
//             instance_handle,
//             2,
//             None,
//             None,
//         );
//         let cc3 = CacheChange::new(
//             ChangeKind::Alive,
//             guid.clone(),
//             instance_handle,
//             3,
//             None,
//             None,
//         );

//         history_cache.add_change(cc1).unwrap();
//         history_cache.add_change(cc2).unwrap();

//         assert_eq!(history_cache.add_change(cc3), Err(ReturnCodes::OutOfResources));
//     }
// }

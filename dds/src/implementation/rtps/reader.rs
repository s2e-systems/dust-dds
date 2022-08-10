use std::collections::HashSet;

use dds_transport::types::Locator;

use crate::{
    dcps_psm::{Duration, LENGTH_UNLIMITED},
    infrastructure::{qos::DataReaderQos, qos_policy::HistoryQosPolicyKind},
    return_type::{DdsError, DdsResult},
};

use super::{
    endpoint::RtpsEndpoint,
    reader_cache_change::RtpsReaderCacheChange,
    types::{Guid, SequenceNumber, TopicKind},
};

struct ReaderHistoryCache {
    changes: Vec<RtpsReaderCacheChange>,
}

impl ReaderHistoryCache {
    fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }
}

pub struct RtpsReader {
    endpoint: RtpsEndpoint,
    heartbeat_response_delay: Duration,
    heartbeat_suppression_duration: Duration,
    reader_cache: ReaderHistoryCache,
    expects_inline_qos: bool,
    qos: DataReaderQos,
}

impl RtpsReader {
    pub fn new(
        endpoint: RtpsEndpoint,
        heartbeat_response_delay: Duration,
        heartbeat_suppression_duration: Duration,
        expects_inline_qos: bool,
        qos: DataReaderQos,
    ) -> Self {
        Self {
            endpoint,
            heartbeat_response_delay,
            heartbeat_suppression_duration,
            reader_cache: ReaderHistoryCache::new(),
            expects_inline_qos,
            qos,
        }
    }
}

impl RtpsReader {
    pub fn guid(&self) -> Guid {
        self.endpoint.guid()
    }
}

impl RtpsReader {
    pub fn topic_kind(&self) -> TopicKind {
        self.endpoint.topic_kind()
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        self.endpoint.unicast_locator_list()
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        self.endpoint.multicast_locator_list()
    }
}

impl RtpsReader {
    pub fn heartbeat_response_delay(&self) -> Duration {
        self.heartbeat_response_delay
    }

    pub fn heartbeat_suppression_duration(&self) -> Duration {
        self.heartbeat_suppression_duration
    }

    pub fn expects_inline_qos(&self) -> bool {
        self.expects_inline_qos
    }
}

impl RtpsReader {
    pub fn changes(&self) -> &[RtpsReaderCacheChange] {
        self.reader_cache.changes.as_ref()
    }

    pub fn add_change(&mut self, change: RtpsReaderCacheChange) -> DdsResult<()> {
        if self.qos.history.kind == HistoryQosPolicyKind::KeepLastHistoryQoS {
            let num_instance_samples = self
                .reader_cache
                .changes
                .iter()
                .filter(|cc| cc.instance_handle() == change.instance_handle())
                .count() as i32;

            if num_instance_samples >= self.qos.history.depth {
                // Remove the lowest sequence number for the instance handle of the cache change
                // Only one sample is to be removed since cache changes come one by one
                let min_seq_num = self
                    .reader_cache
                    .changes
                    .iter()
                    .filter(|cc| cc.instance_handle() == change.instance_handle())
                    .map(|cc| cc.sequence_number())
                    .min()
                    .expect("If there are samples there must be a min sequence number");

                self.remove_change(|c| c.sequence_number() == min_seq_num);
            }
        }

        let instance_handle_list: HashSet<_> = self
            .reader_cache
            .changes
            .iter()
            .map(|cc| cc.instance_handle())
            .collect();

        let max_samples_limit_not_reached = self.qos.resource_limits.max_samples
            == LENGTH_UNLIMITED
            || (self.reader_cache.changes.len() as i32) < self.qos.resource_limits.max_samples;

        let max_instances_limit_not_reached = instance_handle_list
            .contains(&change.instance_handle())
            || self.qos.resource_limits.max_instances == LENGTH_UNLIMITED
            || (instance_handle_list.len() as i32) < self.qos.resource_limits.max_instances;

        let max_samples_per_instance_limit_not_reached =
            self.qos.resource_limits.max_samples_per_instance == LENGTH_UNLIMITED
                || (self
                    .changes()
                    .iter()
                    .filter(|cc| cc.instance_handle() == change.instance_handle())
                    .count() as i32)
                    < self.qos.resource_limits.max_samples_per_instance;

        if max_samples_limit_not_reached
            && max_instances_limit_not_reached
            && max_samples_per_instance_limit_not_reached
        {
            self.reader_cache.changes.push(change);
            Ok(())
        } else {
            Err(DdsError::OutOfResources)
        }
    }

    pub fn remove_change<F>(&mut self, mut f: F)
    where
        F: FnMut(&RtpsReaderCacheChange) -> bool,
    {
        self.reader_cache.changes.retain(|cc| !f(cc));
    }

    pub fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.reader_cache
            .changes
            .iter()
            .map(|cc| cc.sequence_number())
            .min()
    }

    pub fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.reader_cache
            .changes
            .iter()
            .map(|cc| cc.sequence_number())
            .max()
    }
}

impl RtpsReader {
    pub fn get_qos(&self) -> &DataReaderQos {
        &self.qos
    }

    pub fn set_qos(&mut self, qos: DataReaderQos) -> DdsResult<()> {
        qos.is_consistent()?;
        self.qos = qos;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        dcps_psm::DURATION_ZERO,
        implementation::rtps::types::{ChangeKind, GUID_UNKNOWN},
        infrastructure::qos_policy::{HistoryQosPolicy, ResourceLimitsQosPolicy},
    };

    use super::*;

    #[test]
    fn reader_no_key_add_change_keep_last_1() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLastHistoryQoS,
                depth: 1,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]);
        let mut reader = RtpsReader::new(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        let change1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16],
            1,
            vec![1],
            vec![],
            None,
        );
        let change2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16],
            2,
            vec![1],
            vec![],
            None,
        );
        reader.add_change(change1).unwrap();
        reader.add_change(change2.clone()).unwrap();

        assert_eq!(reader.changes().len(), 1);
        assert_eq!(reader.changes()[0], change2);
    }

    #[test]
    fn reader_with_key_add_change_keep_last_1() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLastHistoryQoS,
                depth: 1,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]);
        let mut reader = RtpsReader::new(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        let change1_instance1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [1; 16],
            1,
            vec![1],
            vec![],
            None,
        );
        let change2_instance1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [1; 16],
            2,
            vec![1],
            vec![],
            None,
        );
        reader.add_change(change1_instance1).unwrap();
        reader.add_change(change2_instance1.clone()).unwrap();

        let change1_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [2; 16],
            1,
            vec![1],
            vec![],
            None,
        );
        let change2_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [2; 16],
            2,
            vec![1],
            vec![],
            None,
        );
        reader.add_change(change1_instance2).unwrap();
        reader.add_change(change2_instance2.clone()).unwrap();

        assert_eq!(reader.changes().len(), 2);
        assert!(reader.changes().contains(&change2_instance1));
        assert!(reader.changes().contains(&change2_instance2));
    }

    #[test]
    fn reader_no_key_add_change_keep_last_3() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLastHistoryQoS,
                depth: 3,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]);
        let mut reader = RtpsReader::new(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        let change1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16],
            1,
            vec![1],
            vec![],
            None,
        );
        let change2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16],
            2,
            vec![2],
            vec![],
            None,
        );
        let change3 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16],
            3,
            vec![3],
            vec![],
            None,
        );
        let change4 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16],
            4,
            vec![4],
            vec![],
            None,
        );
        reader.add_change(change1).unwrap();
        reader.add_change(change2.clone()).unwrap();
        reader.add_change(change3.clone()).unwrap();
        reader.add_change(change4.clone()).unwrap();

        assert_eq!(reader.changes().len(), 3);
        assert!(reader.changes().contains(&change2));
        assert!(reader.changes().contains(&change3));
        assert!(reader.changes().contains(&change4));
    }

    #[test]
    fn reader_with_key_add_change_keep_last_3() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepLastHistoryQoS,
                depth: 3,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::WithKey, &[], &[]);
        let mut reader = RtpsReader::new(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        let change1_instance1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [1; 16],
            1,
            vec![1],
            vec![],
            None,
        );
        let change2_instance1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [1; 16],
            2,
            vec![1],
            vec![],
            None,
        );
        let change3_instance1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [1; 16],
            3,
            vec![1],
            vec![],
            None,
        );
        let change4_instance1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [1; 16],
            4,
            vec![1],
            vec![],
            None,
        );
        reader.add_change(change1_instance1).unwrap();
        reader.add_change(change2_instance1.clone()).unwrap();
        reader.add_change(change3_instance1.clone()).unwrap();
        reader.add_change(change4_instance1.clone()).unwrap();

        let change1_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [2; 16],
            1,
            vec![1],
            vec![],
            None,
        );
        let change2_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [2; 16],
            2,
            vec![1],
            vec![],
            None,
        );
        let change3_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [2; 16],
            3,
            vec![1],
            vec![],
            None,
        );
        let change4_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [2; 16],
            4,
            vec![1],
            vec![],
            None,
        );
        reader.add_change(change1_instance2).unwrap();
        reader.add_change(change2_instance2.clone()).unwrap();
        reader.add_change(change3_instance2.clone()).unwrap();
        reader.add_change(change4_instance2.clone()).unwrap();

        assert_eq!(reader.changes().len(), 6);
        assert!(reader.changes().contains(&change2_instance1));
        assert!(reader.changes().contains(&change3_instance1));
        assert!(reader.changes().contains(&change4_instance1));
        assert!(reader.changes().contains(&change2_instance2));
        assert!(reader.changes().contains(&change3_instance2));
        assert!(reader.changes().contains(&change4_instance2));
    }

    #[test]
    fn reader_max_samples() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAllHistoryQos,
                depth: LENGTH_UNLIMITED,
            },
            resource_limits: ResourceLimitsQosPolicy {
                max_samples: 1,
                max_instances: LENGTH_UNLIMITED,
                max_samples_per_instance: LENGTH_UNLIMITED,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]);
        let mut reader = RtpsReader::new(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        let change1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16],
            1,
            vec![1],
            vec![],
            None,
        );
        let change2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16],
            2,
            vec![1],
            vec![],
            None,
        );
        reader.add_change(change1).unwrap();

        assert_eq!(reader.add_change(change2), Err(DdsError::OutOfResources));
    }

    #[test]
    fn reader_max_instances() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAllHistoryQos,
                depth: LENGTH_UNLIMITED,
            },
            resource_limits: ResourceLimitsQosPolicy {
                max_samples: LENGTH_UNLIMITED,
                max_instances: 1,
                max_samples_per_instance: LENGTH_UNLIMITED,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]);
        let mut reader = RtpsReader::new(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        let change1_instance1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [0; 16],
            1,
            vec![1],
            vec![],
            None,
        );
        let change1_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [1; 16],
            2,
            vec![1],
            vec![],
            None,
        );
        reader.add_change(change1_instance1).unwrap();

        assert_eq!(
            reader.add_change(change1_instance2),
            Err(DdsError::OutOfResources)
        );
    }

    #[test]
    fn reader_max_samples_per_instance() {
        let qos = DataReaderQos {
            history: HistoryQosPolicy {
                kind: HistoryQosPolicyKind::KeepAllHistoryQos,
                depth: LENGTH_UNLIMITED,
            },
            resource_limits: ResourceLimitsQosPolicy {
                max_samples: LENGTH_UNLIMITED,
                max_instances: LENGTH_UNLIMITED,
                max_samples_per_instance: 1,
            },
            ..Default::default()
        };
        let endpoint = RtpsEndpoint::new(GUID_UNKNOWN, TopicKind::NoKey, &[], &[]);
        let mut reader = RtpsReader::new(endpoint, DURATION_ZERO, DURATION_ZERO, false, qos);

        let change1_instance1 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [1; 16],
            1,
            vec![1],
            vec![],
            None,
        );
        let change1_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [2; 16],
            2,
            vec![1],
            vec![],
            None,
        );
        let change2_instance2 = RtpsReaderCacheChange::new(
            ChangeKind::Alive,
            GUID_UNKNOWN,
            [2; 16],
            3,
            vec![1],
            vec![],
            None,
        );
        reader.add_change(change1_instance1).unwrap();
        reader.add_change(change1_instance2).unwrap();

        assert_eq!(
            reader.add_change(change2_instance2),
            Err(DdsError::OutOfResources)
        );
    }
}

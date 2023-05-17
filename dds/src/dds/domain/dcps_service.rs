use std::{
    net::UdpSocket,
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
    thread::JoinHandle,
};

use fnmatch_regex::glob_to_regex;

use crate::{
    domain::domain_participant_factory::THE_DDS_DOMAIN_PARTICIPANT_FACTORY,
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, DCPS_SUBSCRIPTION},
            discovered_topic_data::{DiscoveredTopicData, DCPS_TOPIC},
            discovered_writer_data::{DiscoveredWriterData, DCPS_PUBLICATION},
            spdp_discovered_participant_data::{SpdpDiscoveredParticipantData, DCPS_PARTICIPANT},
        },
        dds::{
            dds_data_reader::DdsDataReader,
            dds_data_writer::DdsDataWriter,
            dds_domain_participant::{
                DdsDomainParticipant, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
            },
            dds_subscriber::DdsSubscriber,
            participant_discovery::ParticipantDiscovery,
            status_listener::ListenerTriggerKind,
        },
        rtps::{
            discovery_types::BuiltinEndpointSet,
            history_cache::RtpsWriterCacheChange,
            messages::{
                overall_structure::RtpsMessageHeader,
                submessage_elements::SequenceNumberSet,
                submessages::{GapSubmessage, InfoDestinationSubmessage, InfoTimestampSubmessage},
                types::{FragmentNumber, ProtocolId},
                RtpsMessage, RtpsSubmessageKind,
            },
            reader_locator::WriterAssociatedReaderLocator,
            reader_proxy::{RtpsReaderProxy, WriterAssociatedReaderProxy},
            stateful_reader::RtpsStatefulReader,
            stateful_writer::RtpsStatefulWriter,
            stateless_writer::RtpsStatelessWriter,
            transport::TransportWrite,
            types::{
                DurabilityKind, EntityId, Guid, GuidPrefix, Locator, ReliabilityKind,
                SequenceNumber, ENTITYID_PARTICIPANT, ENTITYID_UNKNOWN,
            },
            writer_proxy::RtpsWriterProxy,
        },
        rtps_udp_psm::udp_transport::UdpTransportWrite,
        utils::{condvar::DdsCondvar, shared_object::DdsRwLock},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        time::{Duration, DurationKind, Time},
    },
    subscription::sample_info::{
        InstanceStateKind, SampleStateKind, ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE,
    },
    topic_definition::type_support::DdsType,
};

pub struct DcpsService {
    quit: Arc<AtomicBool>,
    threads: DdsRwLock<Vec<JoinHandle<()>>>,
    sedp_condvar: DdsCondvar,
    user_defined_data_send_condvar: DdsCondvar,
}

impl Drop for DcpsService {
    fn drop(&mut self) {
        self.quit.store(true, atomic::Ordering::SeqCst);

        self.sedp_condvar.notify_all();
        self.user_defined_data_send_condvar.notify_all();

        while let Some(thread) = self.threads.write_lock().pop() {
            thread.join().unwrap();
        }
    }
}

impl DcpsService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        participant_guid_prefix: GuidPrefix,
        sedp_condvar: &DdsCondvar,
        user_defined_data_send_condvar: &DdsCondvar,
    ) -> DdsResult<Self> {
        let quit = Arc::new(AtomicBool::new(false));
        let mut threads = Vec::new();

        let sedp_condvar = sedp_condvar.clone();
        let user_defined_data_send_condvar = user_defined_data_send_condvar.clone();
        Ok(DcpsService {
            quit,
            threads: DdsRwLock::new(threads),
            sedp_condvar,
            user_defined_data_send_condvar,
        })
    }

    pub fn _sedp_condvar(&self) -> &DdsCondvar {
        &self.sedp_condvar
    }

    pub fn user_defined_data_send_condvar(&self) -> &DdsCondvar {
        &self.user_defined_data_send_condvar
    }
}

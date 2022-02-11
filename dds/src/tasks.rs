use std::{
    ops::Deref,
    sync::{
        atomic::{self, AtomicBool},
        mpsc::{Receiver, SyncSender},
        Arc,
    },
};

use async_std::prelude::StreamExt;
use rust_dds_api::subscription::data_reader::DataReader;
use rust_dds_rtps_implementation::{
    dds_impl::{
        data_reader_proxy::{RtpsReader, Samples},
        subscriber_proxy::SubscriberAttributes,
    },
    rtps_impl::rtps_writer_proxy_impl::RtpsWriterProxyImpl,
    utils::shared_object::RtpsShared,
};
use rust_rtps_pim::{
    behavior::{
        reader::{
            stateful_reader::RtpsStatefulReaderOperations, writer_proxy::RtpsWriterProxyConstructor,
        },
        writer::{
            reader_proxy::RtpsReaderProxyConstructor, stateful_writer::RtpsStatefulWriterOperations,
        },
    },
    discovery::participant_discovery::ParticipantDiscovery,
};

use crate::{
    data_representation_builtin_endpoints::{
        sedp_discovered_writer_data::SedpDiscoveredWriterData,
        spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
    },
    domain_participant_factory::RtpsStructureImpl,
};

pub struct Executor {
    pub receiver: Receiver<EnabledPeriodicTask>,
}

impl Executor {
    pub fn run(&self) {
        while let Ok(mut enabled_periodic_task) = self.receiver.try_recv() {
            async_std::task::spawn(async move {
                let mut interval = async_std::stream::interval(enabled_periodic_task.period);
                loop {
                    if enabled_periodic_task.enabled.load(atomic::Ordering::SeqCst) {
                        (enabled_periodic_task.task)();
                    } else {
                        println!("Not enabled");
                    }
                    interval.next().await;
                }
            });
        }
    }
}

#[derive(Clone)]
pub struct Spawner {
    pub task_sender: SyncSender<EnabledPeriodicTask>,
    pub enabled: Arc<AtomicBool>,
}

impl Spawner {
    pub fn new(task_sender: SyncSender<EnabledPeriodicTask>) -> Self {
        Self {
            task_sender,
            enabled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn spawn_enabled_periodic_task(
        &self,
        name: &'static str,
        task: impl FnMut() -> () + Send + Sync + 'static,
        period: std::time::Duration,
    ) {
        self.task_sender
            .send(EnabledPeriodicTask {
                name,
                task: Box::new(task),
                period,
                enabled: self.enabled.clone(),
            })
            .unwrap();
    }

    // pub fn enable_tasks(&self) {
    //     self.enabled.store(true, atomic::Ordering::SeqCst);
    // }

    // pub fn disable_tasks(&self) {
    //     self.enabled.store(false, atomic::Ordering::SeqCst);
    // }
}

pub struct EnabledPeriodicTask {
    pub name: &'static str,
    pub task: Box<dyn FnMut() -> () + Send + Sync>,
    pub period: std::time::Duration,
    pub enabled: Arc<AtomicBool>,
}

pub fn _spdp_task_discovery<T>(
    spdp_builtin_participant_data_reader_arc: &RtpsShared<
        impl DataReader<SpdpDiscoveredParticipantData, Samples = T>,
    >,
    domain_id: u32,
    domain_tag: &str,
    sedp_builtin_publications_writer: &mut impl RtpsStatefulWriterOperations<
        ReaderProxyType = impl RtpsReaderProxyConstructor,
    >,
    sedp_builtin_publication_reader: &mut impl RtpsStatefulReaderOperations<
        WriterProxyType = impl RtpsWriterProxyConstructor,
    >,
    sedp_builtin_subscriptions_writer: &mut impl RtpsStatefulWriterOperations<
        ReaderProxyType = impl RtpsReaderProxyConstructor,
    >,
    sedp_builtin_subscriptions_reader: &mut impl RtpsStatefulReaderOperations<
        WriterProxyType = impl RtpsWriterProxyConstructor,
    >,
    sedp_builtin_topics_writer: &mut impl RtpsStatefulWriterOperations<
        ReaderProxyType = impl RtpsReaderProxyConstructor,
    >,
    sedp_builtin_topics_reader: &mut impl RtpsStatefulReaderOperations<
        WriterProxyType = impl RtpsWriterProxyConstructor,
    >,
) where
    T: Deref<Target = [SpdpDiscoveredParticipantData]>,
{
    let mut spdp_builtin_participant_data_reader_lock =
        spdp_builtin_participant_data_reader_arc.write_lock();
    if let Ok(samples) = spdp_builtin_participant_data_reader_lock.read(1, &[], &[], &[]) {
        for discovered_participant in samples.into_iter() {
            if let Ok(participant_discovery) = ParticipantDiscovery::new(
                &discovered_participant.participant_proxy,
                &(domain_id as u32),
                domain_tag,
            ) {
                participant_discovery.discovered_participant_add_publications_writer(
                    sedp_builtin_publications_writer,
                );

                participant_discovery.discovered_participant_add_publications_reader(
                    sedp_builtin_publication_reader,
                );

                participant_discovery.discovered_participant_add_subscriptions_writer(
                    sedp_builtin_subscriptions_writer,
                );

                participant_discovery.discovered_participant_add_subscriptions_reader(
                    sedp_builtin_subscriptions_reader,
                );

                participant_discovery
                    .discovered_participant_add_topics_writer(sedp_builtin_topics_writer);

                participant_discovery
                    .discovered_participant_add_topics_reader(sedp_builtin_topics_reader);
            }
        }
    }
}

pub fn _task_sedp_discovery(
    sedp_builtin_publications_data_reader: &RtpsShared<
        impl DataReader<SedpDiscoveredWriterData, Samples = Samples<SedpDiscoveredWriterData>>,
    >,
    subscriber_list: &RtpsShared<Vec<RtpsShared<SubscriberAttributes<RtpsStructureImpl>>>>,
) {
    let mut sedp_builtin_publications_data_reader_lock =
        sedp_builtin_publications_data_reader.write_lock();
    if let Ok(samples) = sedp_builtin_publications_data_reader_lock.read(1, &[], &[], &[]) {
        if let Some(sample) = samples.into_iter().next() {
            let topic_name = &sample.publication_builtin_topic_data.topic_name;
            let type_name = &sample.publication_builtin_topic_data.type_name;
            let subscriber_list_lock = subscriber_list.read_lock();
            for subscriber in subscriber_list_lock.iter() {
                let subscriber_lock = subscriber.read_lock();
                for data_reader in subscriber_lock.data_reader_list.iter() {
                    let mut data_reader_lock = data_reader.write_lock();
                    let reader_topic_name = &data_reader_lock.topic.read_lock().topic_name.clone();
                    let reader_type_name = data_reader_lock.topic.read_lock().type_name;
                    if topic_name == reader_topic_name && type_name == reader_type_name {
                        let writer_proxy = RtpsWriterProxyImpl::new(
                            sample.writer_proxy.remote_writer_guid,
                            sample.writer_proxy.unicast_locator_list.as_ref(),
                            sample.writer_proxy.multicast_locator_list.as_ref(),
                            sample.writer_proxy.data_max_size_serialized,
                            sample.writer_proxy.remote_group_entity_id,
                        );
                        match &mut data_reader_lock.rtps_reader {
                            RtpsReader::Stateless(_) => (),
                            RtpsReader::Stateful(rtps_stateful_reader) => {
                                rtps_stateful_reader.matched_writer_add(writer_proxy)
                            }
                        };
                    }
                }
            }
        }
    }
}

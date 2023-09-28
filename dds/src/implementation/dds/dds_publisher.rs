use std::collections::HashMap;

use dust_dds_derive::actor_interface;

use crate::{
    implementation::{
        dds::dds_data_writer_listener::DdsDataWriterListener,
        rtps::{
            endpoint::RtpsEndpoint,
            group::RtpsGroup,
            messages::overall_structure::{RtpsMessageHeader, RtpsMessageRead},
            types::{
                EntityId, Guid, Locator, TopicKind, USER_DEFINED_WRITER_NO_KEY,
                USER_DEFINED_WRITER_WITH_KEY,
            },
            writer::RtpsWriter,
        },
        rtps_udp_psm::udp_transport::UdpTransportWrite,
        utils::actor::{spawn_actor, Actor, ActorAddress},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        status::StatusKind,
        time::{Duration, Time, DURATION_ZERO},
    },
};

use super::{
    dds_data_writer::{self, DdsDataWriter},
    dds_publisher_listener::DdsPublisherListener,
};

pub struct DdsPublisher {
    qos: PublisherQos,
    rtps_group: RtpsGroup,
    data_writer_list: HashMap<InstanceHandle, Actor<DdsDataWriter>>,
    enabled: bool,
    user_defined_data_writer_counter: u8,
    default_datawriter_qos: DataWriterQos,
    listener: Option<Actor<DdsPublisherListener>>,
    status_kind: Vec<StatusKind>,
}

impl DdsPublisher {
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroup,
        listener: Option<Actor<DdsPublisherListener>>,
        status_kind: Vec<StatusKind>,
    ) -> Self {
        Self {
            qos,
            rtps_group,
            data_writer_list: HashMap::new(),
            enabled: false,
            user_defined_data_writer_counter: 0,
            default_datawriter_qos: DataWriterQos::default(),
            listener,
            status_kind,
        }
    }
}

#[actor_interface]
impl DdsPublisher {
    #[allow(clippy::too_many_arguments)]
    async fn create_datawriter(
        &mut self,
        type_name: String,
        topic_name: String,
        has_key: bool,
        data_max_size_serialized: usize,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Actor<DdsDataWriterListener>>,
        mask: Vec<StatusKind>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
    ) -> DdsResult<ActorAddress<DdsDataWriter>> {
        let qos = match qos {
            QosKind::Default => self.default_datawriter_qos.clone(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        let guid_prefix = self.rtps_group.guid().prefix();
        let (entity_kind, topic_kind) = match has_key {
            true => (USER_DEFINED_WRITER_WITH_KEY, TopicKind::WithKey),
            false => (USER_DEFINED_WRITER_NO_KEY, TopicKind::NoKey),
        };
        let entity_key = [
            self.rtps_group.guid().entity_id().entity_key()[0],
            self.get_unique_writer_id().await,
            0,
        ];
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = Guid::new(guid_prefix, entity_id);

        let rtps_writer_impl = RtpsWriter::new(
            RtpsEndpoint::new(
                guid,
                topic_kind,
                &default_unicast_locator_list,
                &default_multicast_locator_list,
            ),
            true,
            Duration::new(0, 200_000_000),
            DURATION_ZERO,
            DURATION_ZERO,
            data_max_size_serialized,
        );

        let data_writer = DdsDataWriter::new(
            rtps_writer_impl,
            type_name,
            topic_name,
            a_listener,
            mask,
            qos,
        );
        let data_writer_actor = spawn_actor(data_writer);
        let data_writer_address = data_writer_actor.address().clone();
        self.data_writer_list.insert(guid.into(), data_writer_actor);

        Ok(data_writer_address)
    }

    async fn enable(&mut self) {
        self.enabled = true;
    }

    async fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn is_empty(&self) -> bool {
        self.data_writer_list.is_empty()
    }

    async fn get_unique_writer_id(&mut self) -> u8 {
        let counter = self.user_defined_data_writer_counter;
        self.user_defined_data_writer_counter += 1;
        counter
    }

    async fn delete_contained_entities(&mut self) {
        self.data_writer_list.clear()
    }

    async fn datawriter_add(
        &mut self,
        instance_handle: InstanceHandle,
        data_writer: Actor<DdsDataWriter>,
    ) {
        self.data_writer_list.insert(instance_handle, data_writer);
    }

    async fn datawriter_delete(&mut self, handle: InstanceHandle) {
        self.data_writer_list.remove(&handle);
    }

    async fn set_default_datawriter_qos(&mut self, qos: DataWriterQos) {
        self.default_datawriter_qos = qos;
    }

    async fn get_default_datawriter_qos(&self) -> DataWriterQos {
        self.default_datawriter_qos.clone()
    }

    async fn set_qos(&mut self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => Default::default(),
            QosKind::Specific(q) => q,
        };

        if self.enabled {
            self.qos.check_immutability(&qos)?;
        }

        self.qos = qos;

        Ok(())
    }

    async fn guid(&self) -> Guid {
        self.rtps_group.guid()
    }

    async fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_group.guid().into()
    }

    async fn get_status_kind(&self) -> Vec<StatusKind> {
        self.status_kind.clone()
    }

    async fn get_listener(&self) -> Option<ActorAddress<DdsPublisherListener>> {
        self.listener.as_ref().map(|l| l.address().clone())
    }

    async fn get_qos(&self) -> PublisherQos {
        self.qos.clone()
    }

    async fn data_writer_list(&self) -> Vec<ActorAddress<DdsDataWriter>> {
        self.data_writer_list
            .values()
            .map(|x| x.address().clone())
            .collect()
    }

    async fn process_rtps_message(&self, message: RtpsMessageRead) {
        for data_writer_address in self.data_writer_list.values().map(|a| a.address()) {
            data_writer_address
                .send_mail(dds_data_writer::process_rtps_message::new(message.clone()))
                .await
                .expect("Should not fail to send command");
        }
    }

    async fn send_message(
        &self,
        header: RtpsMessageHeader,
        udp_transport_write: ActorAddress<UdpTransportWrite>,
        now: Time,
    ) {
        for data_writer_address in self.data_writer_list.values().map(|a| a.address()) {
            data_writer_address
                .send_mail(dds_data_writer::send_message::new(
                    header,
                    udp_transport_write.clone(),
                    now,
                ))
                .await
                .expect("Should not fail to send command");
        }
    }
}

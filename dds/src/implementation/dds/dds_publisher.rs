use crate::{
    implementation::{
        rtps::{group::RtpsGroup, types::Guid},
        utils::actor::{actor_interface, Actor, ActorAddress},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        status::StatusKind,
    },
};

use super::{dds_data_writer::DdsDataWriter, dds_publisher_listener::DdsPublisherListener};

pub struct DdsPublisher {
    qos: PublisherQos,
    rtps_group: RtpsGroup,
    data_writer_list: Vec<Actor<DdsDataWriter>>,
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
            data_writer_list: Vec::new(),
            enabled: false,
            user_defined_data_writer_counter: 0,
            default_datawriter_qos: DataWriterQos::default(),
            listener,
            status_kind,
        }
    }
}

actor_interface! {
impl DdsPublisher {
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_empty(&self) -> bool {
        self.data_writer_list.is_empty()
    }

    pub fn get_unique_writer_id(&mut self) -> u8 {
        let counter = self.user_defined_data_writer_counter;
        self.user_defined_data_writer_counter += 1;
        counter
    }

    pub fn delete_contained_entities(&mut self) {
        self.data_writer_list.clear()
    }

    pub fn datawriter_add(
        &mut self,
        data_writer: Actor<DdsDataWriter>,
    ) {
        self.data_writer_list.push(data_writer)
    }

    pub fn datawriter_delete(&mut self, handle: InstanceHandle) {
        self.data_writer_list.retain(|dw| {
            if let Ok(h) = dw.address().get_instance_handle() {
                h != handle
            } else {
                false
            }
        });
    }

    pub fn data_writer_list(
        &self,
    ) -> Vec<ActorAddress<DdsDataWriter>> {
        self.data_writer_list
            .iter()
            .map(|x| x.address().clone())
            .collect()
    }

    pub fn set_default_datawriter_qos(&mut self, qos: DataWriterQos) {
        self.default_datawriter_qos = qos;
    }

    pub fn get_default_datawriter_qos(&self) -> DataWriterQos {
        self.default_datawriter_qos.clone()
    }

    pub fn set_qos(&mut self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
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

    pub fn get_qos(&self) -> PublisherQos {
        self.qos.clone()
    }

    pub fn guid(&self) -> Guid {
        self.rtps_group.guid()
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_group.guid().into()
    }

    pub fn get_listener(&self) -> Option<ActorAddress<DdsPublisherListener>> {
        self.listener.as_ref().map(|l| l.address().clone())
    }

    pub fn status_kind(&self) -> Vec<StatusKind> {
        self.status_kind.clone()
    }
}
}

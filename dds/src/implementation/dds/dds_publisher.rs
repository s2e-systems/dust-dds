use crate::{
    implementation::{
        rtps::{
            group::RtpsGroup, stateful_writer::RtpsStatefulWriter,
            stateless_writer::RtpsStatelessWriter, types::Guid,
        },
        utils::actor::{actor_interface, Actor, ActorAddress},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
    },
};

use super::dds_data_writer::DdsDataWriter;

pub struct DdsPublisher {
    qos: PublisherQos,
    rtps_group: RtpsGroup,
    stateless_data_writer_list: Vec<Actor<DdsDataWriter<RtpsStatelessWriter>>>,
    stateful_data_writer_list: Vec<Actor<DdsDataWriter<RtpsStatefulWriter>>>,
    enabled: bool,
    user_defined_data_writer_counter: u8,
    default_datawriter_qos: DataWriterQos,
}

impl DdsPublisher {
    pub fn new(qos: PublisherQos, rtps_group: RtpsGroup) -> Self {
        Self {
            qos,
            rtps_group,
            stateless_data_writer_list: Vec::new(),
            stateful_data_writer_list: Vec::new(),
            enabled: false,
            user_defined_data_writer_counter: 0,
            default_datawriter_qos: DataWriterQos::default(),
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
        self.stateful_data_writer_list.is_empty() && self.stateless_data_writer_list.is_empty()
    }

    pub fn get_unique_writer_id(&mut self) -> u8 {
        let counter = self.user_defined_data_writer_counter;
        self.user_defined_data_writer_counter += 1;
        counter
    }

    pub fn delete_contained_entities(&mut self) {
        self.stateful_data_writer_list.clear()
    }

    pub fn stateful_datawriter_add(
        &mut self,
        data_writer: Actor<DdsDataWriter<RtpsStatefulWriter>>,
    ) {
        self.stateful_data_writer_list.push(data_writer)
    }

    pub fn stateful_datawriter_delete(&mut self, handle: InstanceHandle) {
        self.stateful_data_writer_list.retain(|dw| {
            if let Ok(h) = dw.address().get_instance_handle() {
                h != handle
            } else {
                false
            }
        });
    }

    pub fn stateful_data_writer_list(
        &self,
    ) -> Vec<ActorAddress<DdsDataWriter<RtpsStatefulWriter>>> {
        self.stateful_data_writer_list
            .iter()
            .map(|x| x.address().clone())
            .collect()
    }

    pub fn stateless_datawriter_add(
        &mut self,
        data_writer: Actor<DdsDataWriter<RtpsStatelessWriter>>,
    ) {
        self.stateless_data_writer_list.push(data_writer)
    }

    pub fn stateless_datawriter_list(&self) -> Vec<ActorAddress<DdsDataWriter<RtpsStatelessWriter>>> {
        self.stateless_data_writer_list
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
}
}

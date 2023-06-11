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

    pub fn delete_datawriter(&mut self, data_writer_guid: Guid) -> DdsResult<()> {
        todo!()

    }

    pub fn lookup_datawriter(
        &mut self,
        type_name: &'static str,
        topic_name: String,
    ) -> DdsResult<Option<ActorAddress<DdsDataWriter<RtpsStatefulWriter>>>> {
        todo!()
        // Ok(domain_participant
        //     .get_publisher(publisher_guid)
        //     .ok_or(DdsError::AlreadyDeleted)?
        //     .stateful_data_writer_list()
        //     .iter()
        //     .find(|data_reader| {
        //         data_reader.get_topic_name() == topic_name
        //             && data_reader.get_type_name() == type_name
        //     })
        //     .map(|x| DataWriterNode::new(x.guid(), publisher_guid, domain_participant.guid())))
    }

    pub fn get_unique_writer_id(&mut self) -> u8 {
        let counter = self.user_defined_data_writer_counter;
        self.user_defined_data_writer_counter += 1;
        counter
    }

    pub fn delete_contained_entities(&mut self) {
        todo!()
        // for data_writer in self.stateful_data_writer_list.drain(..) {
        //     data_writer.0.
        // }
    }

    pub fn stateful_datawriter_add(
        &mut self,
        data_writer: Actor<DdsDataWriter<RtpsStatefulWriter>>,
    ) {
        self.stateful_data_writer_list.push(data_writer)
    }

    pub fn stateful_datawriter_delete(&mut self, data_writer_handle: InstanceHandle) {
        todo!()
        // self.stateful_data_writer_list
        //     .retain(|x| InstanceHandle::from(x.guid()) != data_writer_handle);
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

    pub fn set_default_datawriter_qos(&mut self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => self.default_datawriter_qos = DataWriterQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                self.default_datawriter_qos = q;
            }
        }
        Ok(())
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

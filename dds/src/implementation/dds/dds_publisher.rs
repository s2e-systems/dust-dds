use crate::{
    implementation::{
        rtps::{
            group::RtpsGroup, stateful_writer::RtpsStatefulWriter,
            stateless_writer::RtpsStatelessWriter, types::Guid,
        },
        utils::actor::{self, ActorAddress, ActorJoinHandle, ActorJoinSet},
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
    stateless_data_writer_list: Vec<DdsDataWriter<RtpsStatelessWriter>>,
    stateful_data_writer_list: Vec<(
        ActorAddress<DdsDataWriter<RtpsStatefulWriter>>,
        ActorJoinHandle,
    )>,
    enabled: bool,
    user_defined_data_writer_counter: u8,
    default_datawriter_qos: DataWriterQos,
}

pub struct Enable;

impl actor::Message for Enable {
    type Result = ();
}

impl actor::Handler<Enable> for DdsPublisher {
    fn handle(
        &mut self,
        _message: Enable,
        _actor_address: &mut ActorAddress<Self>,
        _actor_task: &mut ActorJoinSet,
    ) -> <Enable as actor::Message>::Result {
        self.enable()
    }
}

pub struct IsEnabled;

impl actor::Message for IsEnabled {
    type Result = bool;
}

impl actor::Handler<IsEnabled> for DdsPublisher {
    fn handle(
        &mut self,
        _message: IsEnabled,
        _actor_address: &mut ActorAddress<Self>,
        _actor_task: &mut ActorJoinSet,
    ) -> <IsEnabled as actor::Message>::Result {
        self.is_enabled()
    }
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

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_unique_writer_id(&mut self) -> u8 {
        let counter = self.user_defined_data_writer_counter;
        self.user_defined_data_writer_counter += 1;
        counter
    }

    pub fn stateful_datawriter_add(
        &mut self,
        data_writer: (
            ActorAddress<DdsDataWriter<RtpsStatefulWriter>>,
            ActorJoinHandle,
        ),
    ) {
        self.stateful_data_writer_list.push(data_writer)
    }

    pub fn stateful_datawriter_drain(
        &mut self,
    ) -> std::vec::Drain<(
        ActorAddress<DdsDataWriter<RtpsStatefulWriter>>,
        ActorJoinHandle,
    )> {
        self.stateful_data_writer_list.drain(..)
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
            .map(|x| x.0.clone())
            .collect()
    }

    pub fn stateless_datawriter_add(&mut self, data_writer: DdsDataWriter<RtpsStatelessWriter>) {
        self.stateless_data_writer_list.push(data_writer)
    }

    pub fn _stateless_datawriter_drain(
        &mut self,
    ) -> std::vec::Drain<DdsDataWriter<RtpsStatelessWriter>> {
        self.stateless_data_writer_list.drain(..)
    }

    pub fn _stateless_datawriter_delete(&mut self, data_writer_handle: InstanceHandle) {
        self.stateless_data_writer_list
            .retain(|x| InstanceHandle::from(x.guid()) != data_writer_handle);
    }

    pub fn stateless_data_writer_list(&self) -> &[DdsDataWriter<RtpsStatelessWriter>] {
        &self.stateless_data_writer_list
    }

    pub fn stateless_data_writer_list_mut(&mut self) -> &mut [DdsDataWriter<RtpsStatelessWriter>] {
        &mut self.stateless_data_writer_list
    }

    pub fn get_data_writer(
        &self,
        data_writer_guid: Guid,
    ) -> Option<ActorAddress<DdsDataWriter<RtpsStatefulWriter>>> {
        todo!()
        // self.stateful_data_writer_list()
        //     .iter()
        //     .find(|dw| dw.guid() == data_writer_guid)
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
}

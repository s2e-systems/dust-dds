use crate::{
    implementation::{
        rtps::{
            group::RtpsGroup, stateful_writer::RtpsStatefulWriter,
            stateless_writer::RtpsStatelessWriter, types::Guid,
        },
        utils::{
            iterator::{DdsDrainIntoIterator, DdsListIntoIterator},
            shared_object::{DdsRwLock, DdsShared},
        },
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
    },
};

use super::dds_data_writer::DdsDataWriter;

pub struct DdsPublisher {
    qos: DdsRwLock<PublisherQos>,
    rtps_group: RtpsGroup,
    stateless_data_writer_list: DdsRwLock<Vec<DdsShared<DdsDataWriter<RtpsStatelessWriter>>>>,
    stateful_data_writer_list: Vec<DdsShared<DdsDataWriter<RtpsStatefulWriter>>>,
    enabled: DdsRwLock<bool>,
    user_defined_data_writer_counter: DdsRwLock<u8>,
    default_datawriter_qos: DdsRwLock<DataWriterQos>,
}

impl DdsPublisher {
    pub fn new(qos: PublisherQos, rtps_group: RtpsGroup) -> Self {
        Self {
            qos: DdsRwLock::new(qos),
            rtps_group,
            stateless_data_writer_list: DdsRwLock::new(Vec::new()),
            stateful_data_writer_list: Vec::new(),
            enabled: DdsRwLock::new(false),
            user_defined_data_writer_counter: DdsRwLock::new(0),
            default_datawriter_qos: DdsRwLock::new(DataWriterQos::default()),
        }
    }

    pub fn enable(&self) {
        *self.enabled.write_lock() = true;
    }

    pub fn is_enabled(&self) -> bool {
        *self.enabled.read_lock()
    }

    pub fn get_unique_writer_id(&self) -> u8 {
        let mut counter_lock = self.user_defined_data_writer_counter.write_lock();
        let counter = *counter_lock;
        *counter_lock += 1;
        counter
    }

    pub fn stateful_datawriter_add(
        &mut self,
        data_writer: DdsShared<DdsDataWriter<RtpsStatefulWriter>>,
    ) {
        self.stateful_data_writer_list.push(data_writer)
    }

    pub fn stateful_datawriter_drain(
        &mut self,
    ) -> std::vec::Drain<DdsShared<DdsDataWriter<RtpsStatefulWriter>>> {
        self.stateful_data_writer_list.drain(..)
    }

    pub fn stateful_datawriter_delete(&mut self, data_writer_handle: InstanceHandle) {
        self.stateful_data_writer_list
            .retain(|x| InstanceHandle::from(x.guid()) != data_writer_handle);
    }

    pub fn stateful_data_writer_list(&self) -> &[DdsShared<DdsDataWriter<RtpsStatefulWriter>>] {
        &self.stateful_data_writer_list
    }

    pub fn stateless_datawriter_add(
        &self,
        data_writer: DdsShared<DdsDataWriter<RtpsStatelessWriter>>,
    ) {
        self.stateless_data_writer_list
            .write_lock()
            .push(data_writer)
    }

    pub fn _stateless_datawriter_drain(
        &self,
    ) -> DdsDrainIntoIterator<DdsShared<DdsDataWriter<RtpsStatelessWriter>>> {
        DdsDrainIntoIterator::new(self.stateless_data_writer_list.write_lock())
    }

    pub fn _stateless_datawriter_delete(&self, data_writer_handle: InstanceHandle) {
        self.stateless_data_writer_list
            .write_lock()
            .retain(|x| InstanceHandle::from(x.guid()) != data_writer_handle);
    }

    pub fn stateless_data_writer_list(
        &self,
    ) -> DdsListIntoIterator<DdsShared<DdsDataWriter<RtpsStatelessWriter>>> {
        DdsListIntoIterator::new(self.stateless_data_writer_list.read_lock())
    }

    pub fn get_data_writer(
        &self,
        data_writer_guid: Guid,
    ) -> Option<&DdsShared<DdsDataWriter<RtpsStatefulWriter>>> {
        self.stateful_data_writer_list()
            .iter()
            .find(|dw| dw.guid() == data_writer_guid)
    }

    pub fn set_default_datawriter_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => {
                *self.default_datawriter_qos.write_lock() = DataWriterQos::default()
            }
            QosKind::Specific(q) => {
                q.is_consistent()?;
                *self.default_datawriter_qos.write_lock() = q;
            }
        }
        Ok(())
    }

    pub fn get_default_datawriter_qos(&self) -> DataWriterQos {
        self.default_datawriter_qos.read_lock().clone()
    }

    pub fn set_qos(&self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => Default::default(),
            QosKind::Specific(q) => q,
        };

        if *self.enabled.read_lock() {
            self.qos.read_lock().check_immutability(&qos)?;
        }

        *self.qos.write_lock() = qos;

        Ok(())
    }

    pub fn get_qos(&self) -> PublisherQos {
        self.qos.read_lock().clone()
    }

    pub fn guid(&self) -> Guid {
        self.rtps_group.guid()
    }
}

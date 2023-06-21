use super::{
    dds_data_reader::DdsDataReader, dds_subscriber_listener::DdsSubscriberListener,
    status_condition_impl::StatusConditionImpl,
};
use crate::{
    implementation::{
        rtps::{
            group::RtpsGroup, stateful_reader::RtpsStatefulReader,
            stateless_reader::RtpsStatelessReader, types::Guid,
        },
        utils::{
            actor::{actor_interface, Actor, ActorAddress},
            shared_object::{DdsRwLock, DdsShared},
        },
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos},
        status::StatusKind,
    },
};

pub struct DdsSubscriber {
    qos: SubscriberQos,
    rtps_group: RtpsGroup,
    stateless_data_reader_list: Vec<Actor<DdsDataReader<RtpsStatelessReader>>>,
    stateful_data_reader_list: Vec<Actor<DdsDataReader<RtpsStatefulReader>>>,
    enabled: bool,
    user_defined_data_reader_counter: u8,
    default_data_reader_qos: DataReaderQos,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
    listener: Option<Actor<DdsSubscriberListener>>,
    status_kind: Vec<StatusKind>,
}

impl DdsSubscriber {
    pub fn new(
        qos: SubscriberQos,
        rtps_group: RtpsGroup,
        listener: Option<Actor<DdsSubscriberListener>>,
        status_kind: Vec<StatusKind>,
    ) -> Self {
        DdsSubscriber {
            qos,
            rtps_group,
            stateless_data_reader_list: Vec::new(),
            stateful_data_reader_list: Vec::new(),
            enabled: false,
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: Default::default(),
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
            listener,
            status_kind,
        }
    }
}

actor_interface! {
impl DdsSubscriber {
    pub fn delete_contained_entities(&mut self) {

    }

    pub fn guid(&self) -> Guid {
        self.rtps_group.guid()
    }

    pub fn is_empty(&self) -> bool {
        self.stateless_data_reader_list.is_empty() && self.stateful_data_reader_list.is_empty()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_qos(&self) -> SubscriberQos {
        self.qos.clone()
    }

    pub fn get_unique_reader_id(&mut self) -> u8 {
        let counter = self.user_defined_data_reader_counter;
        self.user_defined_data_reader_counter += 1;
        counter
    }

    pub fn stateless_data_reader_add(
        &mut self,
        data_reader: Actor<DdsDataReader<RtpsStatelessReader>>,
    ) {
        self.stateless_data_reader_list.push(data_reader)
    }

    pub fn stateless_data_reader_list(&self) -> Vec<ActorAddress<DdsDataReader<RtpsStatelessReader>>> {
        self.stateless_data_reader_list.iter().map(|dr| dr.address()).collect()
    }

    pub fn stateful_data_reader_add(
        &mut self,
        data_reader: Actor<DdsDataReader<RtpsStatefulReader>>,
    ) {
        self.stateful_data_reader_list.push(data_reader)
    }

    pub fn stateful_data_reader_delete(&mut self, handle: InstanceHandle) {
        self.stateful_data_reader_list
            .retain(|dr|
                if let Ok(h) = dr.address()
                    .get_instance_handle() {
                        h != handle
                    } else {
                        false
                    });
    }

    pub fn stateful_data_reader_list(&self) -> Vec<ActorAddress<DdsDataReader<RtpsStatefulReader>>> {
        self.stateful_data_reader_list.iter().map(|dr| dr.address()).collect()
    }

    pub fn set_default_datareader_qos(&mut self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => self.default_data_reader_qos = DataReaderQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                self.default_data_reader_qos = q;
            }
        }
        Ok(())
    }

    pub fn get_default_datareader_qos(&self) -> DataReaderQos {
        self.default_data_reader_qos.clone()
    }

    pub fn set_qos(&mut self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
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

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_group.guid().into()
    }

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn get_listener(&self) -> Option<ActorAddress<DdsSubscriberListener>> {
        self.listener.as_ref().map(|l| l.address())
    }

    pub fn status_kind(&self) -> Vec<StatusKind> {
        self.status_kind.clone()
    }
}}

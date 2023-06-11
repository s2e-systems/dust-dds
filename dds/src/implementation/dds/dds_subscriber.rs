use super::dds_data_reader::DdsDataReader;
use crate::{
    implementation::{
        rtps::{
            group::RtpsGroup, stateful_reader::RtpsStatefulReader,
            stateless_reader::RtpsStatelessReader, types::Guid,
        },
        utils::actor::{actor_interface, Actor, ActorAddress},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos},
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
}

impl DdsSubscriber {
    pub fn new(qos: SubscriberQos, rtps_group: RtpsGroup) -> Self {
        DdsSubscriber {
            qos,
            rtps_group,
            stateless_data_reader_list: Vec::new(),
            stateful_data_reader_list: Vec::new(),
            enabled: false,
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: Default::default(),
        }
    }
}

actor_interface! {
impl DdsSubscriber {
    pub fn delete_datareader(&mut self, datareader_guid: Guid) -> DdsResult<()> {
        todo!()
        // let data_reader = domain_participant
        //     .get_subscriber(subscriber_guid)
        //     .ok_or(DdsError::AlreadyDeleted)?
        //     .stateful_data_reader_list()
        //     .iter()
        //     .find(|x| x.guid() == datareader_guid)
        //     .ok_or_else(|| {
        //         DdsError::PreconditionNotMet(
        //             "Data reader can only be deleted from its parent subscriber".to_string(),
        //         )
        //     })?;

        // if data_reader.is_enabled() {
        //     domain_participant
        //         .announce_sender()
        //         .try_send(AnnounceKind::DeletedDataReader(
        //             data_reader.get_instance_handle(),
        //         ))
        //         .ok();
        // }

        // domain_participant
        //     .get_subscriber_mut(subscriber_guid)
        //     .ok_or(DdsError::AlreadyDeleted)?
        //     .stateful_data_reader_delete(datareader_guid);

        // Ok(())
    }

    pub fn delete_contained_entities(&mut self) {

    }

    pub fn lookup_datareader(
        &self,
        type_name: &'static str,
        topic_name: String,
    ) -> DdsResult<Option<ActorAddress<DdsDataReader<RtpsStatefulReader>>>> {
        todo!()
        // Ok(self
        //     .stateful_data_reader_list
        //     .iter()
        //     .find(|data_reader| {
        //         data_reader.get_topic_name() == topic_name
        //             && data_reader.get_type_name() == type_name
        //     })
        //     .map(|x| DataReaderNode::new(x.guid(), subscriber_guid, domain_participant.guid())))
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

    pub fn stateful_data_reader_add(
        &mut self,
        data_reader: Actor<DdsDataReader<RtpsStatefulReader>>,
    ) {
        self.stateful_data_reader_list.push(data_reader)
    }

    pub fn stateful_data_reader_delete(&mut self, datareader_guid: Guid) {
        todo!()
        // self.stateful_data_reader_list
        //     .retain(|x| x.guid() != datareader_guid)
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
}}

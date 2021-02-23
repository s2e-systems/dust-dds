use std::sync::Weak;

use rust_dds_api::{
    dcps_psm::{Duration, InstanceHandle, StatusMask},
    dds_type::DDSType,
    domain::domain_participant::{DomainParticipantChild, TopicGAT},
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataWriterQos, PublisherQos, TopicQos},
    },
    publication::{
        data_writer_listener::DataWriterListener,
        publisher::{DataWriterGAT, Publisher},
        publisher_listener::PublisherListener,
    },
    return_type::{DDSError, DDSResult},
};

use crate::{
    impls::rtps_publisher_impl::RtpsPublisherImpl, rtps_domain_participant::RtpsDomainParticipant,
    utils::node::Node,
};

use super::{rtps_datawriter::RtpsDataWriter, rtps_topic::RtpsTopic};

pub type RtpsPublisher<'a> = Node<&'a RtpsDomainParticipant, Weak<RtpsPublisherImpl>>;

impl<'a, T: DDSType> TopicGAT<'a, T> for RtpsPublisher<'a> {
    type TopicType = RtpsTopic<'a, T>;
}

impl<'a, T: DDSType> DataWriterGAT<'a, T> for RtpsPublisher<'a> {
    type DataWriterType = RtpsDataWriter<'a, T>;
}

impl<'a> DomainParticipantChild<'a> for RtpsPublisher<'a> {
    type DomainParticipantType = RtpsDomainParticipant;
}

impl<'a> Publisher<'a> for RtpsPublisher<'a> {
    fn create_datawriter<T: DDSType>(
        &'a self,
        a_topic: &'a <Self as TopicGAT<'a, T>>::TopicType,
        qos: Option<DataWriterQos>,
        a_listener: Option<Box<dyn DataWriterListener<DataType = T>>>,
        mask: StatusMask,
    ) -> Option<<Self as DataWriterGAT<'a, T>>::DataWriterType> {
        let data_writer_ref = self
            .impl_ref
            .upgrade()?
            .create_datawriter(qos, a_listener, mask)?;

        Some(Node {
            parent: (self, a_topic),
            impl_ref: data_writer_ref,
        })
    }

    fn delete_datawriter<T: DDSType>(
        &'a self,
        a_datawriter: <Self as DataWriterGAT<'a, T>>::DataWriterType,
    ) -> DDSResult<()> {
        if std::ptr::eq(a_datawriter.parent.0, self) {
            self.impl_ref
                .upgrade()
                .ok_or(DDSError::AlreadyDeleted)?
                .delete_datawriter(&a_datawriter.impl_ref)
        } else {
            Err(DDSError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant",
            ))
        }
    }

    fn lookup_datawriter<T: DDSType>(
        &self,
        _topic: &<Self as TopicGAT<'a, T>>::TopicType,
    ) -> Option<<Self as DataWriterGAT<'a, T>>::DataWriterType> {
        todo!()
    }

    fn suspend_publications(&self) -> DDSResult<()> {
        todo!()
    }

    fn resume_publications(&self) -> DDSResult<()> {
        todo!()
    }

    fn begin_coherent_changes(&self) -> DDSResult<()> {
        todo!()
    }

    fn end_coherent_changes(&self) -> DDSResult<()> {
        todo!()
    }

    fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DDSResult<()> {
        todo!()
    }

    fn get_participant(&self) -> &<Self as DomainParticipantChild<'a>>::DomainParticipantType {
        self.parent
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_datawriter_qos(&self, _qos: Option<DataWriterQos>) -> DDSResult<()> {
        // self.publisher_ref.set_default_datawriter_qos(qos)
        todo!()
    }

    fn get_default_datawriter_qos(&self) -> DDSResult<DataWriterQos> {
        // self.publisher_ref.get_default_datawriter_qos()
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<'a> Entity for RtpsPublisher<'a> {
    type Qos = PublisherQos;
    type Listener = Box<dyn PublisherListener + 'a>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DDSResult<()> {
        Ok(self
            .impl_ref
            .upgrade()
            .ok_or(DDSError::AlreadyDeleted)?
            .set_qos(qos))
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        Ok(self
            .impl_ref
            .upgrade()
            .ok_or(DDSError::AlreadyDeleted)?
            .get_qos()
            .clone())
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(&self) -> StatusCondition {
        todo!()
    }

    fn get_status_changes(&self) -> StatusMask {
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        // self.publisher_ref.get_instance_handle()
        todo!()
    }
}

// impl<'a> RtpsPublisherNode<'a> {
// pub fn get(&self) -> DDSResult<&RtpsPublisher> {
//     Ok(MaybeValid::get(&self.maybe_valid_ref)
//         .ok_or(DDSError::AlreadyDeleted)?
//         .as_ref())
// }

// pub fn create_datawriter<T: DDSType>(
//     &self,
//     a_topic: &RtpsAnyTopicRef,
//     qos: Option<DataWriterQos>,
//     // _a_listener: impl DataWriterListener<T>,
//     // _mask: StatusMask
// ) -> Option<RtpsAnyDataWriterRef> {
//     let this = self.get().ok()?;
//     let qos = qos.unwrap_or(self.get_default_datawriter_qos().ok()?);
//     let guid_prefix = this.group.entity.guid.prefix();
//     let entity_key = [
//         0,
//         this.writer_count.fetch_add(1, atomic::Ordering::Relaxed),
//         0,
//     ];
//     this.create_stateful_datawriter::<T>(guid_prefix, entity_key, a_topic, qos)
// }

// pub fn lookup_datawriter<T: DDSType>(&self, topic_name: &str) -> Option<RtpsAnyDataWriterRef> {
//     self.get().ok()?.writer_list.into_iter().find(|writer| {
//         if let Some(any_writer) = writer.get_as::<T>().ok() {
//             let topic_mutex_guard = any_writer.topic.lock().unwrap();
//             match &*topic_mutex_guard {
//                 Some(any_topic) => any_topic.topic_name() == topic_name,
//                 _ => false,
//             }
//         } else {
//             false
//         }
//     })
// }

// pub fn get_default_datawriter_qos(&self) -> DDSResult<DataWriterQos> {
//     Ok(self.get()?.default_datawriter_qos.lock().unwrap().clone())
// }

// pub fn set_default_datawriter_qos(&self, qos: Option<DataWriterQos>) -> DDSResult<()> {
//     let datawriter_qos = qos.unwrap_or_default();
//     datawriter_qos.is_consistent()?;
//     *self.get()?.default_datawriter_qos.lock().unwrap() = datawriter_qos;
//     Ok(())
// }

// pub fn delete(&self) {
//     MaybeValid::delete(&self.maybe_valid_ref)
// }

// pub fn create_stateful_datawriter<T: DDSType>(
//     &self,
//     a_topic: &RtpsAnyTopicRef,
//     qos: Option<DataWriterQos>,
//     // _a_listener: impl DataWriterListener<T>,
//     // _mask: StatusMask
// ) -> Option<RtpsAnyDataWriterRef> {
//     self.create_datawriter::<T>(a_topic, qos, Statefulness::Stateful)
// }

// pub fn create_stateless_datawriter<T: DDSType>(
//     &self,
//     a_topic: &RtpsAnyTopicRef,
//     qos: Option<DataWriterQos>,
//     // _a_listener: impl DataWriterListener<T>,
//     // _mask: StatusMask
// ) -> Option<RtpsAnyDataWriterRef> {
//     self.create_datawriter::<T>(a_topic, qos, Statefulness::Stateless)
// }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use rust_dds_api::{
        domain::domain_participant::DomainParticipant, infrastructure::qos::DomainParticipantQos,
    };
    use rust_rtps::{transport::Transport, types::Locator};

    #[derive(Default)]
    struct MockTransport {
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
    }

    impl Transport for MockTransport {
        fn write(
            &self,
            _message: rust_rtps::messages::RtpsMessage,
            _destination_locator: &rust_rtps::types::Locator,
        ) {
            todo!()
        }

        fn read(
            &self,
        ) -> rust_rtps::transport::TransportResult<
            Option<(rust_rtps::messages::RtpsMessage, rust_rtps::types::Locator)>,
        > {
            todo!()
        }

        fn unicast_locator_list(&self) -> &Vec<rust_rtps::types::Locator> {
            &self.unicast_locator_list
        }

        fn multicast_locator_list(&self) -> &Vec<rust_rtps::types::Locator> {
            &self.multicast_locator_list
        }
    }

    struct TestType;

    impl DDSType for TestType {
        fn type_name() -> &'static str {
            "TestType"
        }

        fn has_key() -> bool {
            true
        }

        fn key(&self) -> Vec<u8> {
            todo!()
        }

        fn serialize(&self) -> Vec<u8> {
            todo!()
        }

        fn deserialize(_data: Vec<u8>) -> Self {
            todo!()
        }
    }

    #[test]
    fn create_datawriter_default_qos() {
        let domain_participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );
        let publisher = domain_participant.create_publisher(None, None, 0).unwrap();
        let topic = domain_participant
            .create_topic("Test", None, None, 0)
            .unwrap();

        let qos = None;
        let a_listener = None;
        let mask = 0;
        let _datawriter = publisher
            .create_datawriter::<TestType>(&topic, qos, a_listener, mask)
            .expect("Error creating data writer");
    }

    #[test]
    fn set_and_get_qos() {
        let domain_participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );
        let publisher = domain_participant.create_publisher(None, None, 0).unwrap();

        let mut publisher_qos = PublisherQos::default();
        publisher_qos.group_data.value = vec![1, 2, 3, 4];
        publisher
            .set_qos(Some(publisher_qos.clone()))
            .expect("Error setting publisher qos");
        assert_eq!(
            publisher.get_qos().expect("Error getting publisher qos"),
            publisher_qos
        );
    }
}

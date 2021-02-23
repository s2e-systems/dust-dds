use crate::{
    impls::rtps_subscriber_impl::RtpsSubscriberImpl,
    rtps_domain_participant::RtpsDomainParticipant, utils::node::Node,
};
use rust_dds_api::{
    dcps_psm::{
        InstanceHandle, InstanceStateKind, SampleLostStatus, SampleStateKind, StatusMask,
        ViewStateKind,
    },
    dds_type::DDSType,
    domain::domain_participant::{DomainParticipantChild, TopicGAT},
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataReaderQos, SubscriberQos, TopicQos},
    },
    return_type::{DDSError, DDSResult},
    subscription::{
        data_reader::AnyDataReader,
        data_reader_listener::DataReaderListener,
        subscriber::{DataReaderGAT, Subscriber},
        subscriber_listener::SubscriberListener,
    },
};

use super::{rtps_datareader::RtpsDataReader, rtps_topic::RtpsTopic};

pub type RtpsSubscriber<'a> = Node<&'a RtpsDomainParticipant, RtpsSubscriberImpl>;

impl<'a, T: DDSType> TopicGAT<'a, T> for RtpsSubscriber<'a> {
    type TopicType = RtpsTopic<'a, T>;
}

impl<'a, T: DDSType> DataReaderGAT<'a, T> for RtpsSubscriber<'a> {
    type DataReaderType = RtpsDataReader<'a, T>;
}

impl<'a> DomainParticipantChild<'a> for RtpsSubscriber<'a> {
    type DomainParticipantType = RtpsDomainParticipant;
}

impl<'a> Subscriber<'a> for RtpsSubscriber<'a> {
    fn create_datareader<T: DDSType>(
        &'a self,
        a_topic: &'a <Self as TopicGAT<'a, T>>::TopicType,
        qos: Option<DataReaderQos>,
        a_listener: Option<Box<dyn DataReaderListener<DataType = T>>>,
        mask: StatusMask,
    ) -> Option<<Self as DataReaderGAT<'a, T>>::DataReaderType> {
        let data_reader_ref = self
            .impl_ref
            .upgrade()?
            .create_datareader(qos, a_listener, mask)?;

        Some(Node {
            parent: (self, a_topic),
            impl_ref: data_reader_ref,
        })
    }

    fn delete_datareader<T: DDSType>(
        &'a self,
        a_datareader: <Self as DataReaderGAT<'a, T>>::DataReaderType,
    ) -> DDSResult<()> {
        if std::ptr::eq(a_datareader.parent.0, self) {
            self.impl_ref
                .upgrade()
                .ok_or(DDSError::AlreadyDeleted)?
                .delete_datareader(&a_datareader.impl_ref)
        } else {
            Err(DDSError::PreconditionNotMet(
                "Publisher can only be deleted from its parent participant",
            ))
        }
    }

    fn lookup_datareader<T: DDSType>(
        &self,
        _topic: &<Self as TopicGAT<'a, T>>::TopicType,
    ) -> Option<<Self as DataReaderGAT<'a, T>>::DataReaderType> {
        todo!()
    }

    fn begin_access(&self) -> DDSResult<()> {
        todo!()
    }

    fn end_access(&self) -> DDSResult<()> {
        todo!()
    }

    fn notify_datareaders(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_participant(&self) -> &<Self as DomainParticipantChild<'a>>::DomainParticipantType {
        self.parent
    }

    fn get_sample_lost_status(&self, _status: &mut SampleLostStatus) -> DDSResult<()> {
        todo!()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_datareader_qos(&self, _qos: Option<DataReaderQos>) -> DDSResult<()> {
        todo!()
    }

    fn get_default_datareader_qos(&self) -> DDSResult<DataReaderQos> {
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_datareaders(
        &self,
        _readers: &mut [&mut dyn AnyDataReader],
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<'a> Entity for RtpsSubscriber<'a> {
    type Qos = SubscriberQos;
    type Listener = Box<dyn SubscriberListener + 'a>;

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
        todo!()
    }
}

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
    fn set_and_get_subscriber_qos() {
        let domain_participant = RtpsDomainParticipant::new(
            0,
            DomainParticipantQos::default(),
            MockTransport::default(),
            MockTransport::default(),
            None,
            0,
        );
        let subscriber = domain_participant.create_subscriber(None, None, 0).unwrap();

        let mut subscriber_qos = SubscriberQos::default();
        subscriber_qos.group_data.value = vec![1, 2, 3, 4];
        subscriber
            .set_qos(Some(subscriber_qos.clone()))
            .expect("Error setting publisher qos");
        assert_eq!(
            subscriber.get_qos().expect("Error getting publisher qos"),
            subscriber_qos
        );
    }
}

use std::marker::PhantomData;

use crate::{
    implementation::{
        dds::{dds_data_writer::DdsDataWriter, dds_publisher::DdsPublisher},
        rtps::{stateful_writer::RtpsStatefulWriter, types::Locator},
        utils::actor::{ActorAddress, Handler, Message},
    },
    infrastructure::{
        error::DdsResult,
        qos::{DataWriterQos, QosKind},
    },
    DdsType,
};

pub struct CreateDataWriter<Foo> {
    phantom: PhantomData<Foo>,
    topic_name: String,
    qos: QosKind<DataWriterQos>,
    default_unicast_locator_list: Vec<Locator>,
    default_multicast_locator_list: Vec<Locator>,
    data_max_size_serialized: usize,
}

impl<Foo> CreateDataWriter<Foo> {
    pub fn new(
        topic_name: String,
        qos: QosKind<DataWriterQos>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        data_max_size_serialized: usize,
    ) -> Self {
        Self {
            phantom: PhantomData,
            topic_name,
            qos,
            default_unicast_locator_list,
            default_multicast_locator_list,
            data_max_size_serialized,
        }
    }
}

impl<Foo> Message for CreateDataWriter<Foo> {
    type Result = DdsResult<ActorAddress<DdsDataWriter<RtpsStatefulWriter>>>;
}

impl<Foo> Handler<CreateDataWriter<Foo>> for DdsPublisher
where
    Foo: DdsType,
{
    fn handle(
        &mut self,
        mail: CreateDataWriter<Foo>,
    ) -> <CreateDataWriter<Foo> as Message>::Result {
        self.create_datawriter::<Foo>(
            mail.topic_name,
            mail.qos,
            mail.default_unicast_locator_list,
            mail.default_multicast_locator_list,
            mail.data_max_size_serialized,
        )
    }
}

pub struct Enable;

impl Message for Enable {
    type Result = ();
}

impl Handler<Enable> for DdsPublisher {
    fn handle(&mut self, _mail: Enable) -> <Enable as Message>::Result {
        self.enable()
    }
}

pub struct IsEnabled;

impl Message for IsEnabled {
    type Result = bool;
}

impl Handler<IsEnabled> for DdsPublisher {
    fn handle(&mut self, _message: IsEnabled) -> <IsEnabled as Message>::Result {
        self.is_enabled()
    }
}

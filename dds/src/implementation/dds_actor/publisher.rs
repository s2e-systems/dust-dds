use std::marker::PhantomData;

use crate::{
    implementation::{
        dds::{dds_data_writer::DdsDataWriter, dds_publisher::DdsPublisher},
        rtps::{
            messages::overall_structure::RtpsMessageWrite, stateful_writer::RtpsStatefulWriter,
            types::Locator,
        },
        utils::actor::{ActorAddress, MailHandler, Mail},
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
    user_defined_rtps_message_channel_sender:
        tokio::sync::mpsc::Sender<(RtpsMessageWrite, Vec<Locator>)>,
}

impl<Foo> CreateDataWriter<Foo> {
    pub fn new(
        topic_name: String,
        qos: QosKind<DataWriterQos>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        data_max_size_serialized: usize,
        user_defined_rtps_message_channel_sender: tokio::sync::mpsc::Sender<(
            RtpsMessageWrite,
            Vec<Locator>,
        )>,
    ) -> Self {
        Self {
            phantom: PhantomData,
            topic_name,
            qos,
            default_unicast_locator_list,
            default_multicast_locator_list,
            data_max_size_serialized,
            user_defined_rtps_message_channel_sender,
        }
    }
}

impl<Foo> Mail for CreateDataWriter<Foo> {
    type Result = DdsResult<ActorAddress<DdsDataWriter<RtpsStatefulWriter>>>;
}

impl<Foo> MailHandler<CreateDataWriter<Foo>> for DdsPublisher
where
    Foo: DdsType,
{
    fn handle(
        &mut self,
        mail: CreateDataWriter<Foo>,
    ) -> <CreateDataWriter<Foo> as Mail>::Result {
        self.create_datawriter::<Foo>(
            mail.topic_name,
            mail.qos,
            mail.default_unicast_locator_list,
            mail.default_multicast_locator_list,
            mail.data_max_size_serialized,
            mail.user_defined_rtps_message_channel_sender,
        )
    }
}

pub struct Enable;

impl Mail for Enable {
    type Result = ();
}

impl MailHandler<Enable> for DdsPublisher {
    fn handle(&mut self, _mail: Enable) -> <Enable as Mail>::Result {
        self.enable()
    }
}

pub struct IsEnabled;

impl Mail for IsEnabled {
    type Result = bool;
}

impl MailHandler<IsEnabled> for DdsPublisher {
    fn handle(&mut self, _message: IsEnabled) -> <IsEnabled as Mail>::Result {
        self.is_enabled()
    }
}

pub struct DeleteContainedEntities;

impl Mail for DeleteContainedEntities {
    type Result = ();
}

impl MailHandler<DeleteContainedEntities> for DdsPublisher {
    fn handle(
        &mut self,
        _mail: DeleteContainedEntities,
    ) -> <DeleteContainedEntities as Mail>::Result {
        // todo!()
        // for data_writer in user_defined_publisher.stateful_datawriter_drain() {
        // if data_writer.is_enabled() {
        //     self.announce_sender
        //         .try_send(AnnounceKind::DeletedDataWriter(data_writer.guid().into()))
        //         .ok();
        // }
        // }
        // self.delete_contained_entities().ok();
    }
}

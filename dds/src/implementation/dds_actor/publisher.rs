use std::marker::PhantomData;

use crate::{
    implementation::{
        dds::{dds_data_writer::DdsDataWriter, dds_publisher::DdsPublisher},
        rtps::{
            messages::overall_structure::RtpsMessageWrite, stateful_writer::RtpsStatefulWriter,
            types::Locator,
        },
        utils::actor::{ActorAddress, Mail, MailHandler},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
    },
    DdsType,
};

impl ActorAddress<DdsPublisher> {
    pub fn create_datawriter<Foo>(
        &self,
        topic_name: String,
        qos: QosKind<DataWriterQos>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        data_max_size_serialized: usize,
        user_defined_rtps_message_channel_sender: tokio::sync::mpsc::Sender<(
            RtpsMessageWrite,
            Vec<Locator>,
        )>,
    ) -> DdsResult<ActorAddress<DdsDataWriter<RtpsStatefulWriter>>>
    where
        Foo: DdsType + Send + 'static,
    {
        struct CreateDataWriter<Foo> {
            phantom: PhantomData<Foo>,
            topic_name: String,
            qos: QosKind<DataWriterQos>,
            default_unicast_locator_list: Vec<Locator>,
            default_multicast_locator_list: Vec<Locator>,
            data_max_size_serialized: usize,
            user_defined_rtps_message_channel_sender:
                tokio::sync::mpsc::Sender<(RtpsMessageWrite, Vec<Locator>)>,
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
        self.send_blocking(CreateDataWriter {
            phantom: PhantomData::<Foo>,
            topic_name,
            qos,
            default_unicast_locator_list,
            default_multicast_locator_list,
            data_max_size_serialized,
            user_defined_rtps_message_channel_sender,
        })?
    }

    pub fn enable(&self) -> DdsResult<()> {
        struct Enable;

        impl Mail for Enable {
            type Result = ();
        }

        impl MailHandler<Enable> for DdsPublisher {
            fn handle(&mut self, _mail: Enable) -> <Enable as Mail>::Result {
                self.enable()
            }
        }

        self.send_blocking(Enable)
    }

    pub fn is_enabled(&self) -> DdsResult<bool> {
        struct IsEnabled;

        impl Mail for IsEnabled {
            type Result = bool;
        }

        impl MailHandler<IsEnabled> for DdsPublisher {
            fn handle(&mut self, _mail: IsEnabled) -> <IsEnabled as Mail>::Result {
                self.is_enabled()
            }
        }
        self.send_blocking(IsEnabled)
    }

    pub fn is_empty(&self) -> DdsResult<bool> {
        struct IsEmpty;

        impl Mail for IsEmpty {
            type Result = bool;
        }

        impl MailHandler<IsEmpty> for DdsPublisher {
            fn handle(&mut self, _mail: IsEmpty) -> <IsEmpty as Mail>::Result {
                self.is_empty()
            }
        }

        self.send_blocking(IsEmpty)
    }

    pub fn delete_contained_entities(&self) -> DdsResult<()> {
        struct DeleteContainedEntities;

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

        self.send_blocking(DeleteContainedEntities)
    }

    pub fn get_qos(&self) -> DdsResult<PublisherQos> {
        struct GetQos;

        impl Mail for GetQos {
            type Result = PublisherQos;
        }

        impl MailHandler<GetQos> for DdsPublisher {
            fn handle(&mut self, _mail: GetQos) -> <GetQos as Mail>::Result {
                self.get_qos()
            }
        }

        self.send_blocking(GetQos)
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        struct GetInstanceHandle;

        impl Mail for GetInstanceHandle {
            type Result = InstanceHandle;
        }

        impl MailHandler<GetInstanceHandle> for DdsPublisher {
            fn handle(&mut self, _mail: GetInstanceHandle) -> <GetInstanceHandle as Mail>::Result {
                self.get_instance_handle()
            }
        }

        self.send_blocking(GetInstanceHandle)
    }
}

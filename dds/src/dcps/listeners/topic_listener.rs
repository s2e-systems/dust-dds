use core::pin::Pin;

use crate::{
    dcps::channels::mpsc::{MpscSender, mpsc_channel},
    runtime::{DdsRuntime, Spawner},
    topic_definition::topic_listener::TopicListener,
};

use super::domain_participant_listener::ListenerMail;

pub struct DcpsTopicListener<R: DdsRuntime> {
    sender: MpscSender<ListenerMail<R>>,
    task: Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl<R: DdsRuntime> DcpsTopicListener<R> {
    pub fn new(_listener: impl TopicListener<R> + Send + 'static) -> Self {
        let (sender, listener_receiver) = mpsc_channel();
        let task =
            Box::pin(async move { while let Some(_m) = listener_receiver.receive().await {} });
        Self { sender, task }
    }

    pub fn spawn(self, spawner: &R::SpawnerHandle) -> MpscSender<ListenerMail<R>> {
        spawner.spawn(self.task);
        self.sender
    }
}

use crate::{
    dcps::runtime::DdsRuntime,
    runtime::{
        executor::ExecutorHandle,
        mpsc::{mpsc_channel, MpscSender},
    },
    topic_definition::topic_listener::TopicListener,
};

use super::domain_participant_listener::ListenerMail;

pub struct TopicListenerActor;

impl TopicListenerActor {
    pub fn spawn<R: DdsRuntime>(
        _listener: impl TopicListener<R> + Send + 'static,
        executor_handle: &ExecutorHandle,
    ) -> MpscSender<ListenerMail<R>> {
        let (listener_sender, listener_receiver) = mpsc_channel();
        executor_handle
            .spawn(async move { while let Some(_m) = listener_receiver.recv().await {} });
        listener_sender
    }
}

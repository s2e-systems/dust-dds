use crate::{
    dds_async::topic_listener::TopicListenerAsync,
    runtime::{
        executor::ExecutorHandle,
        mpsc::{mpsc_channel, MpscSender},
    },
};

pub struct TopicListenerActor {
    _listener: Box<dyn TopicListenerAsync + Send>,
}

impl TopicListenerActor {
    pub fn spawn<'a>(
        _listener: Box<dyn TopicListenerAsync + Send>,
        executor_handle: &ExecutorHandle,
    ) -> MpscSender<TopicListenerActorMail> {
        let (listener_sender, listener_receiver) = mpsc_channel();
        executor_handle
            .spawn(async move { while let Some(_m) = listener_receiver.recv().await {} });
        listener_sender
    }
}

pub enum TopicListenerActorMail {}

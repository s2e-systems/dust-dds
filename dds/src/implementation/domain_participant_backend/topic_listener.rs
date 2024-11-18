use crate::{
    dds_async::topic_listener::TopicListenerAsync,
    implementation::runtime::{
        executor::block_on,
        mpsc::{mpsc_channel, MpscSender},
    },
    infrastructure::error::DdsResult,
};
use std::thread::JoinHandle;

pub enum TopicListenerOperation {}

pub struct TopicListenerMessage {
    pub _listener_operation: TopicListenerOperation,
}

pub struct TopicListenerThread {
    _thread: JoinHandle<()>,
    _sender: MpscSender<TopicListenerMessage>,
}

impl TopicListenerThread {
    fn new(_listener: Box<dyn TopicListenerAsync + Send>) -> Self {
        let (sender, _receiver) = mpsc_channel::<TopicListenerMessage>();
        let thread = std::thread::Builder::new()
            .name("Topic listener".to_string())
            .spawn(move || {
                block_on(async {
                    // TODO
                    // while let Some(m) = receiver.recv().await {
                    //     match m.listener_operation {}
                    // }
                });
            })
            .expect("failed to spawn thread");
        Self {
            _thread: thread,
            _sender: sender,
        }
    }

    fn _sender(&self) -> &MpscSender<TopicListenerMessage> {
        &self._sender
    }

    fn _join(self) -> DdsResult<()> {
        self._sender.close();
        self._thread.join()?;
        Ok(())
    }
}

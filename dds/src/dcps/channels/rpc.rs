use alloc::sync::{Arc, Weak};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

use crate::{
    dcps::{
        channels::oneshot::{OneshotSender, oneshot},
        dcps_mail::DcpsReply,
    },
    infrastructure::error::{DdsError, DdsResult},
};

type InnerChannel<Req> = Channel<CriticalSectionRawMutex, (Req, OneshotSender<DcpsReply>), 1>;

/// Creates a new RPC channel pair.
pub fn rpc_channel<Req>() -> (RpcClient<Req>, RpcMailbox<Req>) {
    let inner = Arc::new(InnerChannel::new());
    (
        RpcClient {
            channel: Arc::downgrade(&inner),
        },
        RpcMailbox { channel: inner },
    )
}

/// Receiving endpoint owned directly by the actor task.
pub struct RpcMailbox<Req> {
    channel: Arc<InnerChannel<Req>>,
}

impl<Req> RpcMailbox<Req> {
    /// Receives a request from a caller.
    pub async fn receive(&self) -> (Req, OneshotSender<DcpsReply>) {
        self.channel.receive().await
    }
}

/// Client handle held by async entities to perform request-response RPC calls with the backend.
pub struct RpcClient<Req> {
    channel: Weak<InnerChannel<Req>>,
}

impl<Req> Clone for RpcClient<Req> {
    fn clone(&self) -> Self {
        Self {
            channel: self.channel.clone(),
        }
    }
}

impl<Req> RpcClient<Req> {
    /// Sends an RPC request to the backend actor and awaits its response.
    pub async fn call(&self, mail: Req) -> DdsResult<DcpsReply> {
        let channel = self.channel.upgrade().ok_or(DdsError::AlreadyDeleted)?;
        let (reply_sender, reply_receiver) = oneshot::<DcpsReply>();

        channel.send((mail, reply_sender)).await;
        reply_receiver.await
    }
}

impl<Req> core::fmt::Debug for RpcClient<Req> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RpcClient").finish()
    }
}

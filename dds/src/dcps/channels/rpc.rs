use alloc::sync::Arc;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

use crate::{
    dcps::{
        channels::oneshot::{OneshotSender, oneshot},
        dcps_mail::DcpsReply,
    },
    infrastructure::error::DdsResult,
};

/// Mailbox owned by an actor loop to receive requests and send responses.
pub struct RpcMailbox<Req> {
    channel: Channel<CriticalSectionRawMutex, (Req, OneshotSender<DcpsReply>), 1>,
}

impl<Req> RpcMailbox<Req> {
    /// Creates a new RPC mailbox.
    pub fn new() -> Self {
        Self {
            channel: Channel::new(),
        }
    }

    /// Receives a request from a caller.
    pub async fn receive(self: &Arc<Self>) -> (Req, OneshotSender<DcpsReply>) {
        self.channel.receive().await
    }
}

impl<Req> Default for RpcMailbox<Req> {
    fn default() -> Self {
        Self::new()
    }
}

/// Client handle held by async entities to perform request-response RPC calls with the backend.
pub struct RpcClient<Req> {
    mailbox: Arc<RpcMailbox<Req>>,
}

impl<Req> Clone for RpcClient<Req> {
    fn clone(&self) -> Self {
        Self {
            mailbox: self.mailbox.clone(),
        }
    }
}

impl<Req> RpcClient<Req> {
    /// Creates a new RPC client wrapping a shared mailbox.
    pub fn new(mailbox: Arc<RpcMailbox<Req>>) -> Self {
        Self { mailbox }
    }

    /// Sends an RPC request to the backend actor and awaits its response.
    pub async fn call(&self, mail: Req) -> DdsResult<DcpsReply> {
        let (reply_sender, reply_receiver) = oneshot::<DcpsReply>();

        self.mailbox.channel.send((mail, reply_sender)).await;
        match reply_receiver.await {
            Ok(reply) => Ok(reply),
            Err(e) => Err(e),
        }
    }
}

impl<Req> core::fmt::Debug for RpcClient<Req> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RpcClient").finish()
    }
}

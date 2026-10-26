use alloc::sync::Arc;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, mutex::Mutex};

use crate::{dcps::dcps_mail::DcpsReply, infrastructure::error::DdsResult};

/// Mailbox owned by an actor loop to receive requests and send responses.
pub struct RpcMailbox<Req> {
    rpc_lock: Mutex<CriticalSectionRawMutex, ()>,
    request_channel: Channel<CriticalSectionRawMutex, Req, 1>,
    response_channel: Channel<CriticalSectionRawMutex, DcpsReply, 1>,
}

impl<Req> RpcMailbox<Req> {
    /// Creates a new RPC mailbox.
    pub fn new() -> Self {
        Self {
            rpc_lock: Mutex::new(()),
            request_channel: Channel::new(),
            response_channel: Channel::new(),
        }
    }

    /// Receives a request from a caller.
    pub async fn receive_request(&self) -> Req {
        self.request_channel.receive().await
    }

    /// Sends a response back to the waiting caller.
    pub async fn send_reply(&self, reply: DcpsReply) {
        self.response_channel.send(reply).await;
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
        let _guard = self.mailbox.rpc_lock.lock().await;
        // Drain any unconsumed response left over from a previous cancelled caller
        let _ = self.mailbox.response_channel.try_receive();
        self.mailbox.request_channel.send(mail).await;
        Ok(self.mailbox.response_channel.receive().await)
    }
}

impl<Req> core::fmt::Debug for RpcClient<Req> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RpcClient").finish()
    }
}

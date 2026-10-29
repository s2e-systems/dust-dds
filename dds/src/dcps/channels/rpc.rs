use alloc::sync::{Arc, Weak};
use core::cell::RefCell;
use critical_section::Mutex as CriticalSectionMutex;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, mutex::Mutex as AsyncMutex,
};

use crate::{
    dcps::dcps_mail::DcpsReply,
    infrastructure::error::{DdsError, DdsResult},
};

struct RpcSlot<Req, Reply> {
    request_id: u64,
    request: Option<Req>,
    reply: Option<Reply>,
}

struct RpcState<Req, Reply> {
    slot: CriticalSectionMutex<RefCell<RpcSlot<Req, Reply>>>,
    request_signal: Channel<CriticalSectionRawMutex, (), 1>,
    reply_signal: Channel<CriticalSectionRawMutex, (), 1>,
    call_lock: AsyncMutex<CriticalSectionRawMutex, ()>,
}

/// Creates a new RPC channel pair with pre-allocated in-place memory.
pub fn rpc_channel<Req>() -> (RpcClient<Req, DcpsReply>, RpcMailbox<Req, DcpsReply>) {
    rpc_channel_typed::<Req, DcpsReply>()
}

/// Creates a new RPC channel pair for a specific Request and Reply type.
pub fn rpc_channel_typed<Req, Reply>() -> (RpcClient<Req, Reply>, RpcMailbox<Req, Reply>) {
    let state = Arc::new(RpcState {
        slot: CriticalSectionMutex::new(RefCell::new(RpcSlot {
            request_id: 0,
            request: None,
            reply: None,
        })),
        request_signal: Channel::new(),
        reply_signal: Channel::new(),
        call_lock: AsyncMutex::new(()),
    });
    (
        RpcClient {
            state: Arc::downgrade(&state),
        },
        RpcMailbox { state },
    )
}

/// Sending endpoint passed with a request to reply to a specific RPC caller.
pub struct RpcSender<Req, Reply> {
    state: Arc<RpcState<Req, Reply>>,
    request_id: u64,
    sent: bool,
}

impl<Req, Reply> RpcSender<Req, Reply> {
    /// Sends a reply back to the waiting caller.
    pub fn send(mut self, reply: Reply) {
        self.sent = true;
        critical_section::with(|cs| {
            let mut slot = self.state.slot.borrow(cs).borrow_mut();
            if slot.request_id == self.request_id {
                slot.reply = Some(reply);
                self.state.reply_signal.try_send(()).ok();
            }
        });
    }
}

impl<Req, Reply> Drop for RpcSender<Req, Reply> {
    fn drop(&mut self) {
        if !self.sent {
            critical_section::with(|cs| {
                let slot = self.state.slot.borrow(cs).borrow_mut();
                if slot.request_id == self.request_id {
                    self.state.reply_signal.try_send(()).ok();
                }
            });
        }
    }
}

/// Receiving endpoint owned directly by the actor task.
pub struct RpcMailbox<Req, Reply = DcpsReply> {
    state: Arc<RpcState<Req, Reply>>,
}

impl<Req, Reply> RpcMailbox<Req, Reply> {
    /// Receives a request from a caller.
    pub async fn receive(&self) -> (Req, RpcSender<Req, Reply>) {
        self.state.request_signal.receive().await;

        let (mail, request_id) = critical_section::with(|cs| {
            let mut slot = self.state.slot.borrow(cs).borrow_mut();
            let mail = slot
                .request
                .take()
                .expect("request must be present in slot");
            (mail, slot.request_id)
        });

        let sender = RpcSender {
            state: self.state.clone(),
            request_id,
            sent: false,
        };

        (mail, sender)
    }
}

/// Client handle held by async entities to perform request-response RPC calls with the backend.
pub struct RpcClient<Req, Reply = DcpsReply> {
    state: Weak<RpcState<Req, Reply>>,
}

impl<Req, Reply> Clone for RpcClient<Req, Reply> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl<Req, Reply> RpcClient<Req, Reply> {
    /// Sends an RPC request to the backend actor and awaits its response.
    pub async fn call(&self, mail: Req) -> DdsResult<Reply> {
        let state = self.state.upgrade().ok_or(DdsError::AlreadyDeleted)?;
        let _guard = state.call_lock.lock().await;

        // Drain any stale reply notification (e.g., if a previous caller was cancelled)
        let _ = state.reply_signal.try_receive();

        // Write request in-place into the pre-allocated slot
        critical_section::with(|cs| {
            let mut slot = state.slot.borrow(cs).borrow_mut();
            slot.request_id = slot.request_id.wrapping_add(1);
            slot.request = Some(mail);
            slot.reply = None;
        });

        // Notify actor that a request is ready
        state.request_signal.send(()).await;

        // Await reply notification from actor
        state.reply_signal.receive().await;

        // Extract the reply from the pre-allocated slot
        let reply = critical_section::with(|cs| {
            let mut slot = state.slot.borrow(cs).borrow_mut();
            slot.reply.take()
        });

        reply.ok_or(DdsError::AlreadyDeleted)
    }
}

impl<Req, Reply> core::fmt::Debug for RpcClient<Req, Reply> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RpcClient").finish()
    }
}

/// Trait associating a strongly-typed request with its specific reply.
pub trait Operation: Send + 'static {
    /// The strongly-typed output returned by this operation.
    type Output: Send + 'static;

    /// Converts this operation into the backend participant mail.
    fn into_mail(self) -> crate::dcps::dcps_mail::DomainParticipantMail;

    /// Extracts the strongly-typed reply from the actor's reply envelope.
    fn extract_reply(reply: DcpsReply) -> DdsResult<Self::Output>;
}

impl RpcClient<crate::dcps::dcps_mail::DomainParticipantMail, DcpsReply> {
    /// Executes a strongly-typed operation, returning its specific reply without runtime matching.
    pub async fn execute<O: Operation>(&self, op: O) -> DdsResult<O::Output> {
        let reply = self.call(op.into_mail()).await?;
        O::extract_reply(reply)
    }
}

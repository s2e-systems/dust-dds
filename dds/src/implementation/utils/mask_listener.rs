use crate::api::dcps_psm::StatusMask;

pub struct MaskListener<T> {
    listener: Option<T>,
    status_mask: StatusMask,
}

impl<T> MaskListener<T> {
    pub fn new(listener: Option<T>, status_mask: StatusMask) -> Self {
        Self {
            listener,
            status_mask,
        }
    }

    pub fn set(&mut self, listener: Option<T>, status_mask: StatusMask) {
        self.status_mask = status_mask;
        self.listener = listener;
    }

    pub fn take(&mut self) -> Option<T> {
        self.status_mask = 0;
        self.listener.take()
    }
}

use crate::infrastructure::status::StatusKind;

pub struct StatusListener<T: ?Sized> {
    listener: Option<Box<T>>,
    status_kind: Vec<StatusKind>,
}

impl<T: ?Sized> StatusListener<T> {
    pub fn new(listener: Option<Box<T>>, status_kind: &[StatusKind]) -> Self {
        Self {
            listener,
            status_kind: status_kind.to_vec(),
        }
    }

    pub fn is_enabled(&self, status_kind: &StatusKind) -> bool {
        self.listener.is_some() && self.status_kind.contains(status_kind)
    }

    pub fn listener_mut(&mut self) -> &mut Option<Box<T>> {
        &mut self.listener
    }
}

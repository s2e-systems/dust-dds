use std::collections::VecDeque;
use std::sync::Mutex;

use crate::types::Locator;
use crate::messages::RtpsMessage;
use crate::transport::Transport;

pub struct StubTransport {
    read: Mutex<VecDeque<(RtpsMessage, Locator)>>,
    write: Mutex<VecDeque<(RtpsMessage, Locator)>>,
}

impl StubTransport {
    pub fn new() -> Self {
        Self {
            read: Mutex::new(VecDeque::new()),
            write: Mutex::new(VecDeque::new()),
        }
    }

    pub fn push_read(&self, message: RtpsMessage, locator: Locator) {
        self.read.lock().unwrap().push_back((message, locator));
    }

    pub fn pop_write(&self) -> Option<(RtpsMessage, Locator)> {
        self.write.lock().unwrap().pop_front()
    }
}

impl Transport for StubTransport {
    fn read(&self) -> crate::transport::Result<Option<(RtpsMessage, Locator)>> {
        match self.read.lock().unwrap().pop_front() {
            Some((message, locator)) => Ok(Some((message, locator))),
            None => Ok(None),
        }
    }

    fn write(&self, message: RtpsMessage, locator: Locator) {
        self.write.lock().unwrap().push_back((message, locator));
    }
}
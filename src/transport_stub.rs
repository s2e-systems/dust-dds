use std::collections::VecDeque;
use std::sync::Mutex;

use crate::types::Locator;
use crate::messages::RtpsMessage;
use crate::transport::Transport;

pub struct StubTransport {
    read: Mutex<VecDeque<(RtpsMessage, Locator)>>,
    write: Mutex<VecDeque<(RtpsMessage, Vec<Locator>)>>,
}

impl StubTransport {
    pub fn push_read(&self, message: RtpsMessage, locator: Locator) {
        self.read.lock().unwrap().push_back((message, locator));
    }

    pub fn pop_write(&self) -> Option<(RtpsMessage, Vec<Locator>)> {
        self.write.lock().unwrap().pop_front()
    }
}

impl Transport for StubTransport {
    fn new(_unicast_locator: Locator, _multicast_locator: Option<Locator>) -> crate::transport::Result<Self> {
        Ok(Self {
            read: Mutex::new(VecDeque::new()),
            write: Mutex::new(VecDeque::new()),
        })
    }

    fn read(&self) -> crate::transport::Result<Option<(RtpsMessage, Locator)>> {
        match self.read.lock().unwrap().pop_front() {
            Some((message, locator)) => Ok(Some((message, locator))),
            None => Ok(None),
        }
    }

    fn write(&self, message: RtpsMessage, unicast_locator_list: &[Locator], multicast_locator_list: &[Locator]) {
        let mut locator_list = Vec::new();
        locator_list.extend_from_slice(unicast_locator_list);
        locator_list.extend_from_slice(multicast_locator_list);
        
        self.write.lock().unwrap().push_back((message, locator_list));
    }
}
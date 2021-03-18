use super::Transport;
use rust_dds_api::return_type::DDSResult;
use rust_rtps::{messages::RtpsMessage, types::Locator};

pub struct MemoryTransport {
    // read: Mutex<VecDeque<(RtpsMessage<'a>, Locator)>>,
    // write: Mutex<VecDeque<(RtpsMessage<'a>, Locator)>>,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
}

impl MemoryTransport {
    pub fn new(unicast_locator: Locator, multicast_locator_list: Vec<Locator>) -> DDSResult<Self> {
        Ok(Self {
            // read: Mutex::new(VecDeque::new()),
            // write: Mutex::new(VecDeque::new()),
            unicast_locator_list: vec![unicast_locator],
            multicast_locator_list,
        })
    }

    // pub fn push_read(&self, message: RtpsMessage<'a>, locator: Locator) {
    //     self.read.lock().unwrap().push_back((message, locator));
    // }

    // pub fn pop_write(&self) -> Option<(RtpsMessage, Locator)> {
    //     self.write.lock().unwrap().pop_front()
    // }

    pub fn receive_from(&self, _transport: &MemoryTransport) {
        // while let Some((message, dst_locator)) = transport.pop_write() {
        //     // If the message destination is the multicast, then its source has to be the same multicast as well
        //     // otherwise the source is the
        //     if transport.multicast_locator_list().contains(&dst_locator) {
        //         // self.push_read(message, dst_locator);
        //     } else {
        //         // self.push_read(message, transport.unicast_locator_list()[0]);
        //     }
        // }
    }
}

impl Transport for MemoryTransport {
    fn read<'a>(&'a self) -> DDSResult<Option<(RtpsMessage<'a>, Locator)>> {
        // match self.read.lock().unwrap().pop_front() {
        //     Some((message, locator)) => Ok(Some((message, locator))),
        //     None => Ok(None),
        // }
        todo!()
    }

    fn write<'a>(&'a self, _message: RtpsMessage<'a>, _destination_locator: &Locator) {
        // self.write
        //     .lock()
        //     .unwrap()
        //     .push_back((message, destination_locator.clone()));
        todo!()
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        &self.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        &self.multicast_locator_list
    }
}

// #[cfg(test)]
// mod tests {
// use rust_rtps::{messages::{RtpsSubmessage, submessages::InfoTs}, types::constants::{PROTOCOL_VERSION_2_4, VENDOR_ID}};

// use super::*;

// #[test]
// fn receive_from_transport_unicast_and_multicast() {
//     let unicast_locator1 = Locator::new_udpv4(7400, [192, 168, 0, 5]);
//     let multicast_locator1 = Locator::new_udpv4(7400, [239, 255, 0, 1]);
//     let transport1 = MemoryTransport::new(unicast_locator1, vec![multicast_locator1]).unwrap();

//     let unicast_locator2 = Locator::new_udpv4(7400, [192, 168, 0, 25]);
//     let multicast_locator2 = Locator::new_udpv4(7400, [239, 255, 0, 1]);
//     let transport2 = MemoryTransport::new(unicast_locator2, vec![multicast_locator2]).unwrap();

//     // Write to the unicast locator
//     let message = RtpsMessage::new(
//         PROTOCOL_VERSION_2_4,
//         VENDOR_ID,
//         [1; 12],
//         vec![RtpsSubmessage::InfoTs(InfoTs::new(
//             Endianness::LittleEndian,
//             None,
//         ))],
//     );
//     transport2.write(message, &unicast_locator1);

//     transport1.receive_from(&transport2);
//     let (message_received, src_message) = transport1.read().unwrap().unwrap();
//     assert_eq!(src_message, unicast_locator2);
//     assert_eq!(message_received.submessages().len(), 1);
//     assert!(transport2.pop_write().is_none());

//     // Write to the multicast locator
//     let message = RtpsMessage::new(
//         PROTOCOL_VERSION_2_4,
//         VENDOR_ID,
//         [1; 12],
//         vec![RtpsSubmessage::InfoTs(InfoTs::new(
//             Endianness::LittleEndian,
//             None,
//         ))],
//     );
//     transport2.write(message, &multicast_locator1);

//     transport1.receive_from(&transport2);
//     let (message_received, src_message) = transport1.read().unwrap().unwrap();
//     assert_eq!(src_message, multicast_locator2);
//     assert_eq!(message_received.submessages().len(), 1);
//     assert!(transport2.pop_write().is_none());
// }
// }

use crate::implementation::rtps::messages::{
    overall_structure::{
        RtpsMap, Submessage, SubmessageHeader, SubmessageHeaderRead, SubmessageHeaderWrite,
    },
    submessage_elements::{LocatorList, SubmessageElement},
    types::{SubmessageFlag, SubmessageKind},
};

#[derive(Debug, PartialEq, Eq)]
pub struct InfoReplySubmessageRead<'a> {
    data: &'a [u8],
}

impl SubmessageHeader for InfoReplySubmessageRead<'_> {
    fn submessage_header(&self) -> SubmessageHeaderRead {
        SubmessageHeaderRead::new(self.data)
    }
}

impl<'a> InfoReplySubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn multicast_flag(&self) -> bool {
        self.submessage_header().flags()[1]
    }

    pub fn unicast_locator_list(&self) -> LocatorList {
        self.map(&self.data[4..])
    }

    pub fn multicast_locator_list(&self) -> LocatorList {
        if self.multicast_flag() {
            let num_locators: u32 = self.map(&self.data[4..]);
            let octets_to_multicat_loctor_list = num_locators as usize * 24 + 8;
            self.map(&self.data[octets_to_multicat_loctor_list..])
        } else {
            LocatorList::new(vec![])
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoReplySubmessageWrite<'a> {
    multicast_flag: SubmessageFlag,
    submessage_elements: Vec<SubmessageElement<'a>>,
}

impl<'a> InfoReplySubmessageWrite<'a> {
    pub fn new(
        multicast_flag: SubmessageFlag,
        unicast_locator_list: LocatorList,
        multicast_locator_list: LocatorList,
    ) -> Self {
        let mut submessage_elements = vec![SubmessageElement::LocatorList(unicast_locator_list)];
        if multicast_flag {
            submessage_elements.push(SubmessageElement::LocatorList(multicast_locator_list));
        }
        Self {
            multicast_flag,
            submessage_elements,
        }
    }
}

impl Submessage for InfoReplySubmessageWrite<'_> {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeaderWrite {
        SubmessageHeaderWrite::new(SubmessageKind::INFO_REPLY, &[], octets_to_next_header)
    }

    fn submessage_elements(&self) -> &[SubmessageElement] {
        &self.submessage_elements
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::{
        messages::overall_structure::into_bytes_vec, types::Locator,
    };

    #[test]
    fn serialize_info_reply() {
        let locator = Locator::new(11, 12, [1; 16]);
        let submessage = InfoReplySubmessageWrite::new(
            false,
            LocatorList::new(vec![locator]),
            LocatorList::new(vec![]),
        );
        #[rustfmt::skip]
        assert_eq!(into_bytes_vec(submessage), vec![
                0x0f, 0b_0000_0001, 28, 0, // Submessage header
                1, 0, 0, 0, //numLocators
                11, 0, 0, 0, //kind
                12, 0, 0, 0, //port
                1, 1, 1, 1, //address
                1, 1, 1, 1, //address
                1, 1, 1, 1, //address
                1, 1, 1, 1, //address
            ]
        );
    }

    #[test]
    fn deserialize_info_reply() {
        #[rustfmt::skip]
        let submessage = InfoReplySubmessageRead::new(&[
            0x0f, 0b_0000_0001, 28, 0, // Submessage header
            1, 0, 0, 0, //numLocators
            11, 0, 0, 0, //kind
            12, 0, 0, 0, //port
            1, 1, 1, 1, //address
            1, 1, 1, 1, //address
            1, 1, 1, 1, //address
            1, 1, 1, 1, //address
        ]);
        let locator = Locator::new(11, 12, [1; 16]);
        let expected_multicast_flag = false;
        let expected_unicast_locator_list = LocatorList::new(vec![locator]);
        let expected_multicast_locator_list = LocatorList::new(vec![]);

        assert_eq!(expected_multicast_flag, submessage.multicast_flag());
        assert_eq!(
            expected_unicast_locator_list,
            submessage.unicast_locator_list()
        );
        assert_eq!(
            expected_multicast_locator_list,
            submessage.multicast_locator_list()
        );
    }

    #[test]
    fn deserialize_info_reply_with_multicast() {
        #[rustfmt::skip]
        let submessage = InfoReplySubmessageRead::new(&[
            0x0f, 0b_0000_0011, 56, 0, // Submessage header
            0, 0, 0, 0, //numLocators
            2, 0, 0, 0, //numLocators
            11, 0, 0, 0, //kind
            12, 0, 0, 0, //port
            1, 1, 1, 1, //address
            1, 1, 1, 1, //address
            1, 1, 1, 1, //address
            1, 1, 1, 1, //address
            11, 0, 0, 0, //kind
            12, 0, 0, 0, //port
            1, 1, 1, 1, //address
            1, 1, 1, 1, //address
            1, 1, 1, 1, //address
            1, 1, 1, 1, //address
        ]);
        let locator = Locator::new(11, 12, [1; 16]);
        let expected_multicast_flag = true;
        let expected_unicast_locator_list = LocatorList::new(vec![]);
        let expected_multicast_locator_list = LocatorList::new(vec![locator, locator]);

        assert_eq!(expected_multicast_flag, submessage.multicast_flag());
        assert_eq!(
            expected_unicast_locator_list,
            submessage.unicast_locator_list()
        );
        assert_eq!(
            expected_multicast_locator_list,
            submessage.multicast_locator_list()
        );
    }
}

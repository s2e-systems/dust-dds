use super::{SubmessageKind, SubmessageFlag,};
use super::{Submessage, SubmessageHeader, };
use super::submessage_elements;
use crate::messages::parameter_list::ParameterList;
use crate::types::constants::SEQUENCE_NUMBER_UNKNOWN;


#[derive(PartialEq, Debug)]
pub struct DataFrag {
    endianness_flag: SubmessageFlag,
    inline_qos_flag: SubmessageFlag,   
    non_standard_payload_flag: SubmessageFlag, 
    key_flag: SubmessageFlag,
    reader_id: submessage_elements::EntityId,
    writer_id: submessage_elements::EntityId,
    writer_sn: submessage_elements::SequenceNumber,
    fragment_starting_num: submessage_elements::FragmentNumber,
    fragments_in_submessage: submessage_elements::UShort,
    data_size: submessage_elements::ULong,
    fragment_size: submessage_elements::UShort,
    inline_qos: Option<ParameterList>,
    serialized_payload: Option<submessage_elements::SerializedDataFragment>,
}


impl Submessage for DataFrag {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeader {
        let submessage_id = SubmessageKind::DataFrag;
    
        const X: SubmessageFlag = false;
        let e = self.endianness_flag; 
        let q = self.inline_qos_flag;
        let k = self.key_flag; 
        let n = self.non_standard_payload_flag;
        let flags = [e, q, k, n, X, X, X, X];

        SubmessageHeader::new(submessage_id, flags, octets_to_next_header)
    }

    fn is_valid(&self) -> bool {
        let serialized_data_size = match &self.serialized_payload {
            Some(data) => data.0.len(),
            None => 0,
        };

        if (self.writer_sn.0 < 1 || self.writer_sn.0 == SEQUENCE_NUMBER_UNKNOWN) ||
           (self.fragment_starting_num.0 < 1) ||
           (self.fragment_size.0 as u32 > self.data_size.0) ||
           (serialized_data_size > self.fragments_in_submessage.0 as usize * self.fragment_size.0 as usize)
        {
            // TODO: Check total number of fragments
            // TODO: Check validity of inline_qos
            false
        } else {
            false
        }
    }
}

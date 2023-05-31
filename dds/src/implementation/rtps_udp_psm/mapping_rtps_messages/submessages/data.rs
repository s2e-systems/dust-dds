use crate::implementation::{
    rtps::messages::{
        overall_structure::SubmessageHeaderWrite, submessages::data::DataSubmessageWrite,
        types::SubmessageKind,
    },
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes},
};
use byteorder::ByteOrder;
use std::io::{Error, Write};

use super::submessage::MappingWriteSubmessage;

// impl MappingWriteSubmessage for DataSubmessageWrite<'_> {
//     fn submessage_header(&self) -> SubmessageHeaderWrite {
//         let inline_qos_len = if self.inline_qos_flag {
//             self.inline_qos.number_of_bytes()
//         } else {
//             0
//         };
//         let serialized_payload_len_padded = (self.serialized_payload.number_of_bytes() + 3) & !3; //ceil to multiple of 4
//         let octets_to_next_header = 20 + inline_qos_len + serialized_payload_len_padded;
//         SubmessageHeaderWrite {
//             submessage_id: SubmessageKind::DATA,
//             flags: [
//                 self.endianness_flag,
//                 self.inline_qos_flag,
//                 self.data_flag,
//                 self.key_flag,
//                 self.non_standard_payload_flag,
//                 false,
//                 false,
//                 false,
//             ],
//             submessage_length: octets_to_next_header as u16,
//         }
//     }
//     fn mapping_write_submessage_elements<W: Write, B: ByteOrder>(
//         &self,
//         mut writer: W,
//     ) -> Result<(), Error> {
//         const OCTETS_TO_INLINE_QOS: u16 = 16;
//         const EXTRA_FLAGS: u16 = 0;
//         EXTRA_FLAGS.mapping_write_byte_ordered::<_, B>(&mut writer)?;
//         OCTETS_TO_INLINE_QOS.mapping_write_byte_ordered::<_, B>(&mut writer)?;
//         self.reader_id
//             .mapping_write_byte_ordered::<_, B>(&mut writer)?;
//         self.writer_id
//             .mapping_write_byte_ordered::<_, B>(&mut writer)?;
//         self.writer_sn
//             .mapping_write_byte_ordered::<_, B>(&mut writer)?;
//         if self.inline_qos_flag {
//             self.inline_qos
//                 .mapping_write_byte_ordered::<_, B>(&mut writer)?;
//         }
//         if self.data_flag || self.key_flag {
//             self.serialized_payload
//                 .mapping_write_byte_ordered::<_, B>(&mut writer)?;
//             // Pad to 32bit boundary
//             let padding: &[u8] = match self.serialized_payload.number_of_bytes() % 4 {
//                 1 => &[0; 3],
//                 2 => &[0; 2],
//                 3 => &[0; 1],
//                 _ => &[],
//             };
//             writer.write_all(padding)?;
//         }
//         Ok(())
//     }
// }

#[cfg(test)]
mod tests {

    use crate::implementation::{
        rtps::{
            messages::{
                submessage_elements::{Parameter, ParameterList},
                types::{ParameterId, SerializedPayload},
            },
            types::{
                EntityId, EntityKey, SequenceNumber, USER_DEFINED_READER_GROUP,
                USER_DEFINED_READER_NO_KEY,
            },
        },
        rtps_udp_psm::mapping_traits::to_bytes,
    };

    use super::*;


}

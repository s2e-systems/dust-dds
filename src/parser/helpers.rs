use cdr::{BigEndian, LittleEndian};
use num_derive::FromPrimitive;
use serde_derive::{Deserialize, Serialize};
use std::cmp;
use std::convert::TryInto; /*CdrLe, CdrBe, PlCdrLe, PlCdrBe, Error, Infinite,*/

use crate::parser::InlineQosParameter;
use crate::types::{
    FragmentNumber, FragmentNumberSet, InlineQosParameterList, Parameter, SequenceNumber,
    SequenceNumberSet,
};

use super::{ErrorMessage, InlineQosPid, ProtocolVersion, Result, RTPS_MINOR_VERSION};

// All sizes are in octets
pub const MINIMUM_RTPS_MESSAGE_SIZE: usize = 20;

#[derive(FromPrimitive, PartialEq, Debug)]
pub enum EndianessFlag {
    BigEndian = 0,
    LittleEndian = 1,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SequenceNumberSerialization {
    high: i32,
    low: u32,
}

impl From<i64> for SequenceNumberSerialization {
    fn from(value: i64) -> Self {
        SequenceNumberSerialization {
            high: (value >> 32) as i32,
            low: (value & 0x0000_0000_FFFF_FFFF) as u32,
        }
    }
}

impl From<SequenceNumberSerialization> for i64 {
    fn from(value: SequenceNumberSerialization) -> Self {
        ((value.high as i64) << 32) + value.low as i64
    }
}

pub fn endianess(flags: &u8) -> Result<EndianessFlag> {
    const ENDIANESS_FLAG_MASK: u8 = 0x01;

    num::FromPrimitive::from_u8((*flags) & ENDIANESS_FLAG_MASK)
        .ok_or(ErrorMessage::InvalidTypeConversion)
}

pub fn deserialize<'de, T>(
    message: &[u8],
    start_index: &usize,
    end_index: &usize,
    endianess: &EndianessFlag,
) -> Result<T>
where
    T: serde::de::Deserialize<'de>,
{
    if message.len() <= *end_index {
        return Err(ErrorMessage::DeserializationMessageSizeTooSmall);
    }

    if *endianess == EndianessFlag::BigEndian {
        cdr::de::deserialize_data::<T, BigEndian>(&message[*start_index..=*end_index])
            .map_err(ErrorMessage::CdrError)
    } else {
        cdr::de::deserialize_data::<T, LittleEndian>(&message[*start_index..=*end_index])
            .map_err(ErrorMessage::CdrError)
    }
}

pub fn parse_sequence_number_set(
    submessage: &[u8],
    sequence_number_set_first_index: &usize,
    endianess_flag: &EndianessFlag,
) -> Result<(SequenceNumberSet, usize)> {
    const SEQUENCE_NUMBER_TYPE_SIZE: usize = 8;
    const NUM_BITS_TYPE_SIZE: usize = 4;
    const BITMAP_FIELD_SIZE: usize = 4;

    let bitmap_base_first_index = *sequence_number_set_first_index;
    let bitmap_base_last_index = bitmap_base_first_index + SEQUENCE_NUMBER_TYPE_SIZE - 1;

    let bitmap_base: i64 = deserialize::<SequenceNumberSerialization>(
        submessage,
        &bitmap_base_first_index,
        &bitmap_base_last_index,
        endianess_flag,
    )?
    .into();
    if bitmap_base < 1 {
        return Err(ErrorMessage::InvalidSubmessage);
    }

    let num_bits_first_index = bitmap_base_last_index + 1;
    let num_bits_last_index = num_bits_first_index + NUM_BITS_TYPE_SIZE - 1;

    let num_bits = deserialize::<u32>(
        submessage,
        &num_bits_first_index,
        &num_bits_last_index,
        &endianess_flag,
    )?;
    if num_bits < 1 || num_bits > 256 {
        return Err(ErrorMessage::InvalidSubmessage);
    }

    let num_bitmap_fields = ((num_bits + 31) >> 5) as usize;

    let mut sequence_number_set = SequenceNumberSet::new();

    for bitmap_field_index in 0..num_bitmap_fields {
        let field_first_index = num_bits_last_index + 1 + bitmap_field_index * BITMAP_FIELD_SIZE;
        let field_last_index = field_first_index + BITMAP_FIELD_SIZE - 1;
        let bitmap_field = deserialize::<u32>(
            submessage,
            &field_first_index,
            &field_last_index,
            &endianess_flag,
        )?;

        let number_bits_in_field = cmp::min(
            num_bits as usize - (BITMAP_FIELD_SIZE * 8) * bitmap_field_index,
            32,
        );
        for sequence_number_index in 0..number_bits_in_field {
            let sequence_number: i64 = bitmap_base
                + (sequence_number_index + (BITMAP_FIELD_SIZE * 8) * bitmap_field_index) as i64;
            let sequence_bit_mask = 1 << sequence_number_index;
            let sequence_bit = (bitmap_field & sequence_bit_mask) == sequence_bit_mask;
            sequence_number_set.insert(sequence_number, sequence_bit);
        }
    }

    Ok((
        sequence_number_set,
        SEQUENCE_NUMBER_TYPE_SIZE + NUM_BITS_TYPE_SIZE + BITMAP_FIELD_SIZE * num_bitmap_fields,
    ))
}

pub fn parse_inline_qos_parameter_list(
    submessage: &[u8],
    parameter_list_first_index: &usize,
    endianess: &EndianessFlag,
) -> Result<(InlineQosParameterList, usize)> {
    const MINIMUM_PARAMETER_VALUE_LENGTH: usize = 4;
    const PARAMETER_ID_OFFSET: usize = 1;
    const LENGTH_FIRST_OFFSET: usize = 2;
    const LENGTH_LAST_OFFSET: usize = 3;
    const VALUE_FIRST_OFFSET: usize = 4;

    let mut parameter_id_first_index = *parameter_list_first_index;

    let mut parameter_list = Vec::new();
    let parameter_list_size: usize;

    loop {
        let parameter_id_last_index = parameter_id_first_index + PARAMETER_ID_OFFSET;
        let length_first_index = parameter_id_first_index + LENGTH_FIRST_OFFSET;
        let length_last_index = parameter_id_first_index + LENGTH_LAST_OFFSET;

        let value_first_index = parameter_id_first_index + VALUE_FIRST_OFFSET;
        let value_last_index;

        let parameter_id = deserialize::<u16>(
            submessage,
            &parameter_id_first_index,
            &parameter_id_last_index,
            endianess,
        )?;
        if parameter_id == InlineQosPid::Sentinel as u16 {
            parameter_list_size = length_last_index - *parameter_list_first_index + 1;
            break;
        }

        let length = deserialize::<u16>(
            submessage,
            &length_first_index,
            &length_last_index,
            endianess,
        )? as usize;
        if length < MINIMUM_PARAMETER_VALUE_LENGTH {
            return Err(ErrorMessage::InvalidSubmessage);
        }

        value_last_index = value_first_index + length - 1;
        if value_last_index >= submessage.len() {
            return Err(ErrorMessage::InvalidSubmessage);
        }

        let value = match num::FromPrimitive::from_u16(parameter_id) {
            Some(InlineQosPid::KeyHash) => Some(InlineQosParameter::KeyHash(
                submessage[value_first_index..=value_last_index]
                    .try_into()
                    .map_err(|_| ErrorMessage::InvalidSubmessageHeader)?,
            )),
            Some(InlineQosPid::StatusInfo) => Some(InlineQosParameter::StatusInfo(
                submessage[value_first_index..=value_last_index]
                    .try_into()
                    .map_err(|_| ErrorMessage::InvalidSubmessageHeader)?,
            )),
            _ => None,
        };

        if let Some(qos_parameter) = value {
            parameter_list.push(qos_parameter);
        }

        parameter_id_first_index = value_last_index + 1;
    }

    Ok((parameter_list, parameter_list_size))
}

pub fn parse_fragment_number_set(
    submessage: &[u8],
    sequence_number_set_first_index: &usize,
    endianess_flag: &EndianessFlag,
) -> Result<(FragmentNumberSet, usize)> {
    const FRAGMENT_NUMBER_TYPE_SIZE: usize = 4;
    const NUM_BITS_TYPE_SIZE: usize = 4;
    const BITMAP_FIELD_SIZE: usize = 4;

    let bitmap_base_first_index = *sequence_number_set_first_index;
    let bitmap_base_last_index = bitmap_base_first_index + FRAGMENT_NUMBER_TYPE_SIZE - 1;

    let bitmap_base = deserialize::<FragmentNumber>(
        submessage,
        &bitmap_base_first_index,
        &bitmap_base_last_index,
        endianess_flag,
    )?;
    if bitmap_base < 1 {
        return Err(ErrorMessage::InvalidSubmessage);
    }

    let num_bits_first_index = bitmap_base_last_index + 1;
    let num_bits_last_index = num_bits_first_index + NUM_BITS_TYPE_SIZE - 1;

    let num_bits = deserialize::<u32>(
        submessage,
        &num_bits_first_index,
        &num_bits_last_index,
        &endianess_flag,
    )?;
    if num_bits < 1 || num_bits > 256 {
        return Err(ErrorMessage::InvalidSubmessage);
    }

    let num_bitmap_fields = ((num_bits + 31) >> 5) as usize;

    let mut fragment_number_set = FragmentNumberSet::with_capacity(num_bitmap_fields);

    for bitmap_field_index in 0..num_bitmap_fields {
        let field_first_index = num_bits_last_index + 1 + bitmap_field_index * BITMAP_FIELD_SIZE;
        let field_last_index = field_first_index + BITMAP_FIELD_SIZE - 1;
        let bitmap_field = deserialize::<u32>(
            submessage,
            &field_first_index,
            &field_last_index,
            &endianess_flag,
        )?;

        let number_bits_in_field = cmp::min(
            num_bits as usize - (BITMAP_FIELD_SIZE * 8) * bitmap_field_index,
            32,
        );
        for fragment_number_index in 0..number_bits_in_field {
            let fragment_number = bitmap_base
                + (fragment_number_index + (BITMAP_FIELD_SIZE * 8) * bitmap_field_index) as u32;
            let sequence_bit_mask = 1 << fragment_number_index;
            let sequence_bit = (bitmap_field & sequence_bit_mask) == sequence_bit_mask;
            fragment_number_set.push((fragment_number, sequence_bit));
        }
    }

    Ok((
        fragment_number_set,
        FRAGMENT_NUMBER_TYPE_SIZE + NUM_BITS_TYPE_SIZE + BITMAP_FIELD_SIZE * num_bitmap_fields,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_sequence_number_set() {
        {
            // Test for example in standard "1234:/12:00110"
            let submessage_test_1_big_endian = [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD2, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00,
                0x00, 0x0C,
            ];

            let (sequence_set_1, sequence_set_size) = parse_sequence_number_set(
                &submessage_test_1_big_endian,
                &0,
                &EndianessFlag::BigEndian,
            )
            .unwrap();
            assert_eq!(sequence_set_1.len(), 12);
            assert_eq!(sequence_set_size, 16);
            for (index, item) in sequence_set_1.iter().enumerate() {
                assert_eq!(item.0, &(1234 + index as i64));
                if item.0 == &1236 || item.0 == &1237 {
                    assert_eq!(item.1, &true);
                } else {
                    assert_eq!(item.1, &false);
                }
            }
        }

        {
            // Test for example in standard "1234:/12:00110"
            let submessage_test_1_little_endian = [
                0x00, 0x00, 0x00, 0x00, 0xD2, 0x04, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x0C, 0x00,
                0x00, 0x00,
            ];

            let (sequence_set_1, sequence_set_size) = parse_sequence_number_set(
                &submessage_test_1_little_endian,
                &0,
                &EndianessFlag::LittleEndian,
            )
            .unwrap();
            assert_eq!(sequence_set_1.len(), 12);
            assert_eq!(sequence_set_size, 16);
            for (index, item) in sequence_set_1.iter().enumerate() {
                assert_eq!(item.0, &(1234 + index as i64));
                if item.0 == &1236 || item.0 == &1237 {
                    assert_eq!(item.1, &true);
                } else {
                    assert_eq!(item.1, &false);
                }
            }
        }

        {
            // Test too high num bits
            let submessage_test_high_num_bits = [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD2, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
                0x00, 0x0C,
            ];

            let sequence_set_result = parse_sequence_number_set(
                &submessage_test_high_num_bits,
                &0,
                &EndianessFlag::BigEndian,
            );
            if let Err(ErrorMessage::InvalidSubmessage) = sequence_set_result {
                assert!(true);
            } else {
                assert!(false);
            }
        }

        {
            // Negative bitmap base
            let submessage_test_negative_base = [
                0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xD2, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00,
                0x00, 0x0C,
            ];

            let sequence_set_result = parse_sequence_number_set(
                &submessage_test_negative_base,
                &0,
                &EndianessFlag::BigEndian,
            );
            if let Err(ErrorMessage::InvalidSubmessage) = sequence_set_result {
                assert!(true);
            } else {
                assert!(false);
            }
        }

        {
            // Zero bitmap base
            let submessage_test_zero_base = [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00,
                0x00, 0x0C,
            ];

            let sequence_set_result = parse_sequence_number_set(
                &submessage_test_zero_base,
                &0,
                &EndianessFlag::BigEndian,
            );
            if let Err(ErrorMessage::InvalidSubmessage) = sequence_set_result {
                assert!(true);
            } else {
                assert!(false);
            }
        }

        {
            // Full size bitmap with base > 32bit
            let submessage_test_large = [
                0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x04, 0xD2, 0x00, 0x00, 0x01, 0x00, 0xAA, 0xAA,
                0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                0xAA, 0xAA,
            ];

            let (sequence_set, sequence_set_size) =
                parse_sequence_number_set(&submessage_test_large, &0, &EndianessFlag::BigEndian)
                    .unwrap();
            assert_eq!(sequence_set.len(), 256);
            assert_eq!(sequence_set_size, 44);
            for (index, item) in sequence_set.iter().enumerate() {
                assert_eq!(item.0, &(4294968530i64 + index as i64));
                if (index + 1) % 2 == 0 {
                    assert_eq!(item.1, &true);
                } else {
                    assert_eq!(item.1, &false);
                }
            }
        }

        {
            // Middle size bitmap with base > 32bit
            let submessage_test_middle = [
                0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x04, 0xD2, 0x00, 0x00, 0x00, 0x28, 0xAA, 0xAA,
                0xAA, 0xAA, 0xFF, 0x00, 0xFF, 0xAA,
            ];

            let (sequence_set, sequence_set_size) =
                parse_sequence_number_set(&submessage_test_middle, &0, &EndianessFlag::BigEndian)
                    .unwrap();
            assert_eq!(sequence_set.len(), 40);
            assert_eq!(sequence_set_size, 20);
            for (index, item) in sequence_set.iter().enumerate() {
                assert_eq!(item.0, &(4294968530i64 + index as i64));
                if (index + 1) % 2 == 0 {
                    assert_eq!(item.1, &true);
                } else {
                    assert_eq!(item.1, &false);
                }
            }
        }

        {
            // Middle size bitmap with base > 32bit with start not at 0
            let submessage_test_middle = [
                0xFA, 0xAF, 0x00, 0x00, 0x01, 0x01, 0x00, 0x00, 0x04, 0xD2, 0x00, 0x00, 0x00, 0x28,
                0xAA, 0xAA, 0xAA, 0xAA, 0xFF, 0x00, 0xFF, 0xAA, 0xAB, 0x56,
            ];

            let (sequence_set, sequence_set_size) =
                parse_sequence_number_set(&submessage_test_middle, &2, &EndianessFlag::BigEndian)
                    .unwrap();
            assert_eq!(sequence_set.len(), 40);
            assert_eq!(sequence_set_size, 20);
            for (index, item) in sequence_set.iter().enumerate() {
                assert_eq!(item.0, &(1103806596306i64 + index as i64));
                if (index + 1) % 2 == 0 {
                    assert_eq!(item.1, &true);
                } else {
                    assert_eq!(item.1, &false);
                }
            }
        }

        {
            let wrong_submessage_test = [0xFA, 0xAF];

            let sequence_set_result =
                parse_sequence_number_set(&wrong_submessage_test, &0, &EndianessFlag::BigEndian);

            if let Err(ErrorMessage::DeserializationMessageSizeTooSmall) = sequence_set_result {
                assert!(true);
            } else {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_parse_inline_qos_parameter_list() {
        {
            let submessage_big_endian = [
                0x00, 0x70, 0x00, 0x10, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A,
                0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x00, 0x10, 0x00, 0x08, 0x10, 0x11, 0x12, 0x13,
                0x14, 0x15, 0x16, 0x17, 0x00, 0x71, 0x00, 0x04, 0x10, 0x20, 0x30, 0x40, 0x00, 0x01,
                0x00, 0x00,
            ];

            let (param_list_big_endian, param_list_size) = parse_inline_qos_parameter_list(
                &submessage_big_endian,
                &0,
                &EndianessFlag::BigEndian,
            )
            .unwrap();
            assert_eq!(param_list_size, 44);
            assert_eq!(param_list_big_endian.len(), 2);
            assert_eq!(
                param_list_big_endian[0],
                InlineQosParameter::KeyHash([
                    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
                    0x0E, 0x0F, 0x10,
                ])
            );
            assert_eq!(
                param_list_big_endian[1],
                InlineQosParameter::StatusInfo([0x10, 0x20, 0x30, 0x40,])
            );
        }

        {
            let submessage_little_endian = [
                0x70, 0x00, 0x10, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A,
                0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x10, 0x00, 0x08, 0x00, 0x10, 0x11, 0x12, 0x13,
                0x14, 0x15, 0x16, 0x17, 0x71, 0x00, 0x04, 0x00, 0x10, 0x20, 0x30, 0x40, 0x01, 0x00,
                0x00, 0x00,
            ];

            let (param_list_little_endian, param_list_size) = parse_inline_qos_parameter_list(
                &submessage_little_endian,
                &0,
                &EndianessFlag::LittleEndian,
            )
            .unwrap();
            assert_eq!(param_list_size, 44);
            assert_eq!(param_list_little_endian.len(), 2);
            assert_eq!(
                param_list_little_endian[0],
                InlineQosParameter::KeyHash([
                    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
                    0x0E, 0x0F, 0x10,
                ])
            );
            assert_eq!(
                param_list_little_endian[1],
                InlineQosParameter::StatusInfo([0x10, 0x20, 0x30, 0x40,])
            );
        }

        {
            // Test no sentinel message
            let submessage = [
                0x00, 0x05, 0x00, 0x04, 0x01, 0x02, 0x03, 0x04, 0x00, 0x10, 0x00, 0x08, 0x10, 0x11,
                0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x10, 0x11, 0x00, 0x00,
            ];

            let param_list =
                parse_inline_qos_parameter_list(&submessage, &0, &EndianessFlag::LittleEndian);
            if let Err(ErrorMessage::InvalidSubmessage) = param_list {
                assert!(true);
            } else {
                assert!(false);
            }
        }

        {
            // Test length below minimum
            let submessage = [
                0x00, 0x05, 0x00, 0x03, 0x01, 0x02, 0x03, 0x00, 0x10, 0x00, 0x08, 0x10, 0x11, 0x12,
                0x13, 0x14, 0x15, 0x16, 0x17, 0x00, 0x01, 0x00, 0x00,
            ];

            let param_list =
                parse_inline_qos_parameter_list(&submessage, &0, &EndianessFlag::BigEndian);
            if let Err(ErrorMessage::InvalidSubmessage) = param_list {
                assert!(true);
            } else {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_parse_fragment_number_set() {
        {
            // Test for example in standard "1234:/12:00110"
            let submessage_test_1_big_endian = [
                0x00, 0x00, 0x04, 0xD2, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x0C,
            ];

            let (fragment_number_set_1, fragment_number_set_size) = parse_fragment_number_set(
                &submessage_test_1_big_endian,
                &0,
                &EndianessFlag::BigEndian,
            )
            .unwrap();
            assert_eq!(fragment_number_set_1.len(), 12);
            assert_eq!(fragment_number_set_size, 12);
            for (index, item) in fragment_number_set_1.iter().enumerate() {
                assert_eq!(item.0, 1234 + index as u32);
                if item.0 == 1236 || item.0 == 1237 {
                    assert_eq!(item.1, true);
                } else {
                    assert_eq!(item.1, false);
                }
            }
        }

        {
            // Test for example in standard "1234:/12:00110"
            let submessage_test_1_little_endian = [
                0xD2, 0x04, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00,
            ];

            let (fragment_number_set_1, fragment_number_set_size) = parse_fragment_number_set(
                &submessage_test_1_little_endian,
                &0,
                &EndianessFlag::LittleEndian,
            )
            .unwrap();
            assert_eq!(fragment_number_set_1.len(), 12);
            assert_eq!(fragment_number_set_size, 12);
            for (index, item) in fragment_number_set_1.iter().enumerate() {
                assert_eq!(item.0, 1234 + index as u32);
                if item.0 == 1236 || item.0 == 1237 {
                    assert_eq!(item.1, true);
                } else {
                    assert_eq!(item.1, false);
                }
            }
        }

        {
            // Test too high num bits
            let submessage_test_high_num_bits = [
                0x00, 0x00, 0x04, 0xD2, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x0C,
            ];

            let fragment_number_set_result = parse_fragment_number_set(
                &submessage_test_high_num_bits,
                &0,
                &EndianessFlag::BigEndian,
            );
            if let Err(ErrorMessage::InvalidSubmessage) = fragment_number_set_result {
                assert!(true);
            } else {
                assert!(false);
            }
        }

        {
            // Zero bitmap base
            let submessage_test_zero_base = [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x0C,
            ];

            let fragment_number_set_result = parse_fragment_number_set(
                &submessage_test_zero_base,
                &0,
                &EndianessFlag::BigEndian,
            );
            if let Err(ErrorMessage::InvalidSubmessage) = fragment_number_set_result {
                assert!(true);
            } else {
                assert!(false);
            }
        }

        {
            // Full size bitmap
            let submessage_test_large = [
                0x00, 0x00, 0x04, 0xD2, 0x00, 0x00, 0x01, 0x00, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
                0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA,
            ];

            let (fragment_number_set, fragment_number_set_size) =
                parse_fragment_number_set(&submessage_test_large, &0, &EndianessFlag::BigEndian)
                    .unwrap();
            assert_eq!(fragment_number_set.len(), 256);
            assert_eq!(fragment_number_set_size, 40);
            for (index, item) in fragment_number_set.iter().enumerate() {
                assert_eq!(item.0, 1234u32 + index as u32);
                if (index + 1) % 2 == 0 {
                    assert_eq!(item.1, true);
                } else {
                    assert_eq!(item.1, false);
                }
            }
        }

        {
            // Middle size bitmap
            let submessage_test_middle = [
                0x00, 0x00, 0x04, 0xD2, 0x00, 0x00, 0x00, 0x28, 0xAA, 0xAA, 0xAA, 0xAA, 0xFF, 0x00,
                0xFF, 0xAA,
            ];

            let (fragment_number_set, fragment_number_set_size) =
                parse_fragment_number_set(&submessage_test_middle, &0, &EndianessFlag::BigEndian)
                    .unwrap();
            assert_eq!(fragment_number_set.len(), 40);
            assert_eq!(fragment_number_set_size, 16);
            for (index, item) in fragment_number_set.iter().enumerate() {
                assert_eq!(item.0, 1234u32 + index as u32);
                if (index + 1) % 2 == 0 {
                    assert_eq!(item.1, true);
                } else {
                    assert_eq!(item.1, false);
                }
            }
        }

        {
            // Middle size bitmap with start not at 0
            let submessage_test_middle = [
                0xFA, 0xAF, 0x00, 0x01, 0x04, 0xD2, 0x00, 0x00, 0x00, 0x28, 0xAA, 0xAA, 0xAA, 0xAA,
                0xFF, 0x00, 0xFF, 0xAA, 0xAB, 0x56,
            ];

            let (fragment_number_set, fragment_number_set_size) =
                parse_fragment_number_set(&submessage_test_middle, &2, &EndianessFlag::BigEndian)
                    .unwrap();
            assert_eq!(fragment_number_set.len(), 40);
            assert_eq!(fragment_number_set_size, 16);
            for (index, item) in fragment_number_set.iter().enumerate() {
                assert_eq!(item.0, 66770u32 + index as u32);
                if (index + 1) % 2 == 0 {
                    assert_eq!(item.1, true);
                } else {
                    assert_eq!(item.1, false);
                }
            }
        }

        {
            let wrong_submessage_test = [0xFA, 0xAF];

            let fragment_number_set_result =
                parse_fragment_number_set(&wrong_submessage_test, &0, &EndianessFlag::BigEndian);

            if let Err(ErrorMessage::DeserializationMessageSizeTooSmall) =
                fragment_number_set_result
            {
                assert!(true);
            } else {
                assert!(false);
            }
        }
    }
}

use crate::xtypes::error::XTypesError;
use dust_dds_derive::TypeSupport;
use std::str::FromStr;

#[derive(Clone, Copy, Debug)]
struct U256 {
    hi: u128,
    lo: u128,
}

impl U256 {
    fn from_u128(v: u128) -> Self {
        Self { hi: 0, lo: v }
    }

    fn shl1(self) -> Self {
        Self {
            hi: (self.hi << 1) | (self.lo >> 127),
            lo: self.lo << 1,
        }
    }

    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.hi.cmp(&other.hi) {
            std::cmp::Ordering::Equal => self.lo.cmp(&other.lo),
            x => x,
        }
    }

    fn sub(self, other: Self) -> Self {
        let (lo, borrow) = self.lo.overflowing_sub(other.lo);

        Self {
            hi: self.hi - other.hi - (borrow as u128),
            lo,
        }
    }

    fn add(self, other: Self) -> Self {
        let (lo, carry) = self.lo.overflowing_add(other.lo);

        Self {
            hi: self.hi + other.hi + (carry as u128),
            lo,
        }
    }

    fn mul10(self) -> Self {
        // x * 10 = x*8 + x*2
        let x2 = self.shl1();
        let x8 = x2.shl1().shl1();
        x8.add(x2)
    }

    fn pow10(exp: usize) -> Self {
        let mut v = Self::from_u128(1);

        for _ in 0..exp {
            v = v.mul10();
        }

        v
    }

    fn leading_bit(self) -> i32 {
        if self.hi != 0 {
            128 + (127 - self.hi.leading_zeros() as i32)
        } else {
            127 - self.lo.leading_zeros() as i32
        }
    }
}

#[derive(Debug, Clone, PartialEq, TypeSupport)]
pub struct F128(pub [u8; 16]);

impl FromStr for F128 {
    type Err = XTypesError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        const EXP_BIAS: i32 = 16383;

        let input = input.trim();

        if input.is_empty() {
            return Err(XTypesError::InvalidData);
        }

        let negative = input.starts_with('-');
        let input = input.trim_start_matches(['+', '-']);

        let (base, exp10_part) = match input.find(['e', 'E']) {
            Some(pos) => (&input[..pos], &input[pos + 1..]),
            None => (input, "0"),
        };

        let exp10_extra: i32 = exp10_part.parse().map_err(|_| XTypesError::InvalidData)?;

        let parts: Vec<_> = base.split('.').collect();

        if parts.len() > 2 {
            return Err(XTypesError::InvalidData);
        }

        let int_part = parts[0];
        let frac_part = if parts.len() == 2 { parts[1] } else { "" };

        let scale = frac_part.len() as i32 - exp10_extra;

        let digits = format!("{}{}", int_part, frac_part);

        let mut mantissa: u128 = 0;

        for ch in digits.bytes() {
            if !ch.is_ascii_digit() {
                return Err(XTypesError::InvalidData);
            }

            mantissa = mantissa.checked_mul(10).ok_or(XTypesError::InvalidData)?;

            mantissa += (ch - b'0') as u128;
        }

        if mantissa == 0 {
            return Ok(if negative {
                Self((1u128 << 127).to_be_bytes())
            } else {
                Self([0; 16])
            });
        }

        let mut num = U256::from_u128(mantissa);
        let mut den = U256::from_u128(1);

        if scale > 0 {
            den = U256::pow10(scale as usize);
        } else if scale < 0 {
            num = num.mul10();
            for _ in 1..(-scale as usize) {
                num = num.mul10();
            }
        }

        // Normalize into [1,2)
        let mut exp2 = num.leading_bit() - den.leading_bit();

        if exp2 > 0 {
            for _ in 0..exp2 {
                den = den.shl1();
            }
        } else {
            for _ in 0..(-exp2) {
                num = num.shl1();
            }
        }

        while num.cmp(&den) == std::cmp::Ordering::Less {
            num = num.shl1();
            exp2 -= 1;
        }

        while num.cmp(&den.shl1()) != std::cmp::Ordering::Less {
            den = den.shl1();
            exp2 += 1;
        }

        num = num.sub(den);

        // Generate fraction bits
        let mut frac: u128 = 0;

        for i in (0..112).rev() {
            num = num.shl1();

            if num.cmp(&den) != std::cmp::Ordering::Less {
                num = num.sub(den);
                frac |= 1u128 << i;
            }
        }

        // Guard bit
        num = num.shl1();

        let guard = num.cmp(&den) != std::cmp::Ordering::Less;

        if guard {
            num = num.sub(den);
        }

        let sticky = num.hi != 0 || num.lo != 0;

        // Round-to-nearest-even
        if guard && (sticky || (frac & 1) != 0) {
            frac += 1;

            if frac >> 112 != 0 {
                frac &= (1u128 << 112) - 1;
                exp2 += 1;
            }
        }

        let biased_exp = exp2 + EXP_BIAS;

        if biased_exp <= 0 {
            return Err(XTypesError::InvalidData);
        }

        if biased_exp >= 0x7fff {
            return Err(XTypesError::InvalidData);
        }

        let bits = ((negative as u128) << 127) | ((biased_exp as u128) << 112) | frac;

        Ok(Self(bits.to_be_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_parse() {
        assert_eq!(
            F128::from_str("3.10").unwrap(),
            F128([
                0x40, 0x00, 0x8C, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC,
                0xCC, 0xCD
            ])
        );

        assert_eq!(
            F128::from_str("3").unwrap(),
            F128([
                0x40, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00
            ])
        );

        assert_eq!(
            F128::from_str("3.").unwrap(),
            F128([
                0x40, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00
            ])
        );
    }

    #[test]
    fn basic_negative_parse() {
        assert_eq!(
            F128::from_str("-3.10").unwrap(),
            F128([
                0xC0, 0x00, 0x8C, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC,
                0xCC, 0xCD
            ])
        );

        assert_eq!(
            F128::from_str("-3").unwrap(),
            F128([
                0xC0, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00
            ])
        );

        assert_eq!(
            F128::from_str("-3.").unwrap(),
            F128([
                0xC0, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00
            ])
        );
    }
}

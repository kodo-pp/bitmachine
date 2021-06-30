use itertools::Itertools;
use std::iter::{self, FromIterator};
use std::str::FromStr;

#[derive(Clone, Eq, PartialEq)]
pub struct BitString {
    bytes: Vec<u8>,
    length: usize,
}

impl std::fmt::Debug for BitString {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", "BitString { ")?;
        for bit in self.iter() {
            write!(fmt, "{}", bit)?;
        }
        write!(fmt, "{}", " }")
    }
}

impl FromIterator<Bit> for BitString {
    fn from_iter<I: IntoIterator<Item = Bit>>(iter: I) -> BitString {
        let iter = iter.into_iter();

        let mut length = 0;
        let bytes = iter
            .chunks(8)
            .into_iter()
            .map(|chunk| {
                chunk.fold(0, |num, bit| {
                    let shift_magnitude = 7 - length % 8;
                    length += 1;
                    num | (bit.as_number() << shift_magnitude)
                })
            })
            .collect();

        BitString { bytes, length }
    }
}

impl BitString {
    pub fn bit_at(&self, index: usize) -> Option<Bit> {
        if index >= self.length {
            None
        } else {
            let byte_index = index / 8;
            let byte = self.bytes[byte_index];
            let bit_subindex = index % 8;
            Some(Bit::from_number_indexed(byte, bit_subindex as u8))
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn bytes(&self) -> &Vec<u8> {
        &self.bytes
    }

    pub fn bytes_mut(&mut self) -> &mut Vec<u8> {
        &mut self.bytes
    }

    pub fn concat(&self, other: &BitString) -> BitString {
        self.iter().chain(other.iter()).collect()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Bit> + 'a {
        let len = self.len();
        Self::iter_adapter(self.bytes.iter().copied(), len)
    }

    // TODO: implement this as part of the IntoIterator trait.
    // This is not yet done, because it doesn't seem like I can
    // write `-> impl Iterator<...>` as the return type, and
    // the real type of the iterator returned is so complex it's
    // better left unnamed.
    pub fn into_iter(self) -> impl Iterator<Item = Bit> {
        let len = self.len();
        Self::iter_adapter(self.bytes.into_iter(), len)
    }

    fn iter_adapter<'a, Iter>(byte_iter: Iter, len: usize) -> impl Iterator<Item = Bit> + 'a
    where
        Iter: Iterator<Item = u8> + 'a,
    {
        byte_iter.map(iter_bits_in_byte).flatten().take(len)
    }

    pub fn empty() -> BitString {
        iter::empty().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_usize(&self) -> usize {
        self.iter()
            .fold(0, |a, b| (a << 1) | (b.as_number() as usize))
    }

    pub fn from_u64(num: u64) -> BitString {
        (0..64)
            .map(|i| {
                let i = i as u64;
                let shift_magnitude = 63 - i;
                let bit_num = (num & (1 << shift_magnitude)) >> shift_magnitude;
                Bit::from_number(bit_num as u8).unwrap()
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct BitStringFromStringError;

impl FromStr for BitString {
    type Err = BitStringFromStringError;

    fn from_str(string: &str) -> Result<BitString, BitStringFromStringError> {
        if string == "." {
            Ok(BitString::empty())
        } else {
            string
                .chars()
                .map(|c| Bit::from_char(c).ok_or(BitStringFromStringError))
                .try_collect()
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Bit {
    Zero,
    One,
}

impl Bit {
    pub fn from_number(number: u8) -> Option<Bit> {
        match number {
            0 => Some(Bit::Zero),
            1 => Some(Bit::One),
            _ => None,
        }
    }

    pub fn from_number_indexed(number: u8, index: u8) -> Bit {
        if index >= 8 {
            panic!("Invalid bit index in an octet: {}", index);
        } else {
            let shift_magnitude = 7 - index;
            let masked = number & (1 << shift_magnitude);
            let result = masked >> shift_magnitude;
            Bit::from_number(result).unwrap()
        }
    }

    pub fn as_number(self) -> u8 {
        match self {
            Bit::Zero => 0,
            Bit::One => 1,
        }
    }

    pub fn from_char(c: char) -> Option<Bit> {
        match c {
            '0' => Some(Bit::Zero),
            '1' => Some(Bit::One),
            _ => None,
        }
    }
}

fn iter_bits_in_byte(byte: u8) -> impl Iterator<Item = Bit> {
    (0..8).map(move |index| Bit::from_number_indexed(byte, index))
}

#[derive(Debug)]
pub struct BitFromStringError;

impl FromStr for Bit {
    type Err = BitFromStringError;

    fn from_str(string: &str) -> Result<Bit, BitFromStringError> {
        match string {
            "0" => Ok(Bit::Zero),
            "1" => Ok(Bit::One),
            _ => Err(BitFromStringError),
        }
    }
}

impl std::fmt::Display for Bit {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.as_number())
    }
}

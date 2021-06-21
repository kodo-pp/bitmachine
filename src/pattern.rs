use crate::bindings::Bindings;
use crate::bitstring::{Bit, BitString};
use crate::value::Value;
use itertools::Itertools;
use std::iter;

pub trait PatternParseMulti {
    fn parse(&self, arg: Vec<Value>) -> Option<Bindings>;
}

pub trait PatternParse {
    fn parse(&self, arg: Value) -> Option<Bindings>;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MultiPattern(pub Vec<Pattern>);

impl PatternParseMulti for MultiPattern {
    fn parse(&self, args: Vec<Value>) -> Option<Bindings> {
        args.into_iter()
            .zip(self.0.iter())
            .map(|(arg, pat)| pat.parse(arg))
            .fold_options(Bindings::empty(), |b1, b2| b1.union_with(b2))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Pattern {
    Anything { name: String },
    ConstLen(ConstLenPattern),
    VarLen(VarLenPattern),
}

impl Pattern {
    pub fn empty() -> Pattern {
        Pattern::ConstLen(ConstLenPattern::empty())
    }
}

impl PatternParse for Pattern {
    fn parse(&self, arg: Value) -> Option<Bindings> {
        match self {
            Pattern::Anything { name } => {
                let mut bindings = Bindings::empty();
                bindings.add(name.clone(), arg);
                Some(bindings)
            }
            Pattern::ConstLen(pat) => pat.parse(arg),
            Pattern::VarLen(pat) => pat.parse(arg),
        }
    }
}

impl From<ConstLenPattern> for Pattern {
    fn from(pattern: ConstLenPattern) -> Pattern {
        Pattern::ConstLen(pattern)
    }
}

impl From<VarLenPattern> for Pattern {
    fn from(pattern: VarLenPattern) -> Pattern {
        Pattern::VarLen(pattern)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConstLenPattern {
    pub elements: Vec<ConstLenPatternElement>,
}

impl ConstLenPattern {
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn empty() -> ConstLenPattern {
        ConstLenPattern { elements: vec![] }
    }
}

impl PatternParse for ConstLenPattern {
    fn parse(&self, arg: Value) -> Option<Bindings> {
        let bitstring = arg.into_bit_string()?;
        let iter = self.elements
            .iter()
            .zip(bitstring.iter())
            .map(|(elem, bit)| elem.parse(bit));
        
        let mut bindings = Bindings::empty();
        for item in iter {
            if let Some((name, value)) = item? {
                bindings.add(name, value.into());
            }
        }

        Some(bindings)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConstLenPatternElement {
    ConstBit(Bit),
    AnyBit { var_name: String }
}

impl ConstLenPatternElement {
    pub fn parse(&self, arg: Bit) -> Option<Option<(String, BitString)>> {
        match self {
            Self::ConstBit(bit) => {
                if arg == *bit {
                    Some(None)
                } else {
                    None
                }
            },
            Self::AnyBit { var_name } => {
                let string = iter::once(arg).collect();
                let tuple = (var_name.clone(), string);
                Some(Some(tuple))
            },
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VarLenPattern {
    pub left: ConstLenPattern,
    pub bit_string_var_name: String,
    pub right: ConstLenPattern,
}

impl PatternParse for VarLenPattern {
    fn parse(&self, arg: Value) -> Option<Bindings> {
        let bitstring = arg.into_bit_string()?;

        let left_len = self.left.len();
        let right_len = self.right.len();
        let total_len = bitstring.len();
        let middle_len: usize = total_len.checked_sub(left_len + right_len)?;

        let mut iter = bitstring.iter();
        let left_str: BitString = iter.by_ref().take(left_len).collect();
        let middle_str: BitString = iter.by_ref().take(middle_len).collect();
        let right_str: BitString = iter.collect();

        let left_parsed = self.left.parse(left_str.into())?;
        let middle_parsed = self.parse_middle(middle_str.into());
        let right_parsed = self.right.parse(right_str.into())?;

        Some(left_parsed.union_with(middle_parsed).union_with(right_parsed))
    }
}

impl VarLenPattern {
    fn parse_middle(&self, middle_str: BitString) -> Bindings {
        Bindings::new(
            iter::once((self.bit_string_var_name.clone(), middle_str.into()))
            .collect()
        )
    }
}


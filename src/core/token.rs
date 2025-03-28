use std::fmt::Display;

use alloy::primitives::{Address, address};

use strum::EnumIter;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, EnumIter, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    KILO,
    XKILO,
}

impl Token {
    pub const fn decimals(&self) -> u8 {
        match self {
            Token::KILO => 18,
            Token::XKILO => 18,
        }
    }

    pub const fn address(&self) -> Address {
        match self {
            Token::KILO => address!("0x503Fa24B7972677F00C4618e5FBe237780C1df53"),
            Token::XKILO => address!("0xA586438a641bF1D44938Dabe819249D55E88C040"),
        }
    }

    pub const fn is_native(&self) -> bool {
        match self {
            Token::KILO => false,
            Token::XKILO => false,
        }
    }

    pub fn ticker(&self) -> &'static str {
        match self {
            Token::KILO => "KILO",
            Token::XKILO => "xKILO",
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ticker())
    }
}

use std::fmt::Display;
use strum::EnumIter;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, EnumIter, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    KILO,
    XKILO,
}

impl Token {
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

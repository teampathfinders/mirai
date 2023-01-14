use crate::error::VexError;
use crate::raknet::Reliability::{
    Reliable, ReliableOrdered, ReliableSequenced, UnreliableSequenced,
};

use Reliability::*;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Reliability {
    Unreliable,
    UnreliableSequenced,
    Reliable,
    ReliableOrdered,
    ReliableSequenced,
}

impl TryFrom<u8> for Reliability {
    type Error = VexError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Unreliable,
            1 => UnreliableSequenced,
            2 => Reliable,
            3 => ReliableOrdered,
            4 => ReliableSequenced,
            _ => {
                return Err(VexError::InvalidRequest(
                    "Invalid reliability ID".to_string(),
                ))
            }
        })
    }
}

impl Reliability {
    pub fn reliable(self) -> bool {
        match self {
            Unreliable | UnreliableSequenced => false,
            _ => true,
        }
    }

    pub fn ordered(self) -> bool {
        match self {
            ReliableOrdered => true,
            _ => false,
        }
    }

    pub fn sequenced(self) -> bool {
        match self {
            UnreliableSequenced | ReliableSequenced => true,
            _ => false,
        }
    }
}

use Reliability::*;

use crate::error::VexError;
use crate::raknet::Reliability::{
    Reliable, ReliableOrdered, ReliableSequenced, UnreliableSequenced,
};

/// Describes how reliable transport of this packet should be.
/// Higher reliability takes more resources, but also has more reliability guarantees.
#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub enum Reliability {
    /// Send the frame using raw UDP.
    /// These packets can arrive in the wrong order or not arrive at all.
    /// Absolutely no guarantees are made and therefore this is also the least reliable.
    #[default]
    Unreliable,
    /// Same guarantees as [`Unreliable`],
    /// but this makes sure that old packets are discarded
    /// by keeping track of the ID of the newest packets.
    /// This reliability will cause the most packet loss.
    UnreliableSequenced,
    /// Makes sure that packets arrive using acknowledgements.
    /// This does not guarantee proper order of packets.
    Reliable,
    /// Guarantees that packets actually arrive and are also processed in the correct order.
    /// Unlike sequenced reliabilities, this does not discard old packets.
    /// Instead it waits for the older packets to arrive before processing new ones.
    /// This option is the most reliable.
    ReliableOrdered,
    /// Guarantees that packets arrive and discards old packets.
    ReliableSequenced,
}

/// Converts a byte to reliability.
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
                return Err(VexError::InvalidRequest(format!(
                    "Invalid reliability ID {value}"
                )))
            }
        })
    }
}

impl Reliability {
    /// Returns whether this reliability is reliable.
    pub fn is_reliable(self) -> bool {
        !matches!(self, Unreliable | UnreliableSequenced)
    }

    /// Returns whether this reliability is ordered.
    pub fn is_ordered(self) -> bool {
        matches!(
            self,
            ReliableOrdered | ReliableSequenced | UnreliableSequenced
        )
    }

    /// Returns whether this reliability is sequenced.
    pub fn is_sequenced(self) -> bool {
        matches!(self, UnreliableSequenced | ReliableSequenced)
    }
}

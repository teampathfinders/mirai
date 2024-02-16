use util::bail;

/// Describes how reliable transport of this packet should be.
/// Higher reliability takes more resources, but also has more reliability guarantees.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Default)]
pub enum Reliability {
    /// Send the frame using raw UDP.
    /// These raknet can arrive in the wrong order or not arrive at all.
    /// Absolutely no guarantees are made and therefore this is also the least reliable.
    #[default]
    Unreliable,
    /// Same guarantees as [`Unreliable`](Reliability::Unreliable),
    /// but this makes sure that old raknet are discarded
    /// by keeping track of the ID of the newest raknet.
    /// This reliability will cause the most packet loss.
    UnreliableSequenced,
    /// Makes sure that raknet arrive using acknowledgements.
    /// This does not guarantee proper order of raknet.
    Reliable,
    /// Guarantees that raknet actually arrive and are also processed in the correct order.
    /// Unlike sequenced reliabilities, this does not discard old raknet.
    /// Instead it waits for the older raknet to arrive before processing new ones.
    /// This option is the most reliable.
    ReliableOrdered,
    /// Guarantees that raknet arrive and discards old raknet.
    ReliableSequenced,
}

/// Converts a byte to reliability.
impl TryFrom<u8> for Reliability {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Unreliable,
            1 => Self::UnreliableSequenced,
            2 => Self::Reliable,
            3 => Self::ReliableOrdered,
            4 => Self::ReliableSequenced,
            _ => {
                bail!(
                    Malformed,
                    "Invalid reliability ID {value}, expected 0-4"
                );
            }
        })
    }
}

impl Reliability {
    /// Returns whether this reliability is reliable.
    #[inline]
    pub const fn is_reliable(self) -> bool {
        !matches!(self, Self::Unreliable | Self::UnreliableSequenced)
    }

    /// Returns whether this reliability is ordered.
    #[inline]
    pub const fn is_ordered(self) -> bool {
        matches!(
            self,
            Self::ReliableOrdered
                | Self::ReliableSequenced
                | Self::UnreliableSequenced
        )
    }

    /// Returns whether this reliability is sequenced.
    #[inline]
    pub const fn is_sequenced(self) -> bool {
        matches!(self, Self::UnreliableSequenced | Self::ReliableSequenced)
    }
}

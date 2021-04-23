use std::convert::{TryFrom, TryInto};

use serde::{Deserialize, Serialize};

use crate::node::Error;

/// A type representing in-memory connections (ICS 003).
#[derive(Serialize, Deserialize)]
pub struct Connections {
    pub connections: Vec<String>,
}

impl Connections {
    pub fn new() -> Self {
        Connections {
            connections: Vec::new(),
        }
    }
}

/// A counter type for representation of client, connection, or channel counters.
/// The primary use-case for this type is for interfacing with the storage: a counter can be read
/// or written easily due to its support for serialization to/from `[u8]`.
#[derive(Debug, PartialEq, Clone)]
pub struct Counter(u64);

impl From<u64> for Counter {
    fn from(v: u64) -> Self {
        Self(v)
    }
}

impl From<Counter> for u64 {
    fn from(c: Counter) -> Self {
        c.0
    }
}

impl From<Counter> for Vec<u8> {
    fn from(c: Counter) -> Self {
        Vec::from(c.0.to_ne_bytes())
    }
}

impl TryFrom<Vec<u8>> for Counter {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        // Ignores overflowing bytes
        // TODO: avoid unwrap below & use typed error to return upon problematic input
        let (bytes, _) = value.split_at(std::mem::size_of::<u64>());
        let res = u64::from_ne_bytes(bytes.try_into().unwrap());
        Ok(Counter(res))
    }
}

impl std::fmt::Display for Counter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::node::objects::Counter;

    #[test]
    fn counter_to_from_bytes() {
        let start = 6789_u64;
        let counter: Counter = start.into();
        let end_bytes: Vec<u8> = counter.clone().into();
        let counter_2: Counter = end_bytes.try_into().unwrap();
        assert_eq!(counter, counter_2);
    }
}

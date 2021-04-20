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

#[derive(Debug, PartialEq, Clone)]
pub struct ClientCounter(u64);

impl From<u64> for ClientCounter {
    fn from(v: u64) -> Self {
        Self(v)
    }
}

impl From<ClientCounter> for u64 {
    fn from(c: ClientCounter) -> Self {
        c.0
    }
}

impl From<ClientCounter> for Vec<u8> {
    fn from(c: ClientCounter) -> Self {
        Vec::from(c.0.to_ne_bytes())
    }
}

impl TryFrom<Vec<u8>> for ClientCounter {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        // Ignores overflowing bytes
        // TODO: avoid unwrap below & use typed error to return upon problematic input
        let (bytes, _) = value.split_at(std::mem::size_of::<u64>());
        let res = u64::from_ne_bytes(bytes.try_into().unwrap());
        Ok(ClientCounter(res))
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::node::objects::ClientCounter;

    #[test]
    fn counter_to_from_bytes() {
        let start = 6789_u64;
        let counter: ClientCounter = start.into();
        let end_bytes: Vec<u8> = counter.clone().into();
        let counter_2: ClientCounter = end_bytes.try_into().unwrap();
        assert_eq!(counter, counter_2);
    }
}

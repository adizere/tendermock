use serde::{Deserialize, Serialize};

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

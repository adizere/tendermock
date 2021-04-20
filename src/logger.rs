//! # A message logger to prettify outputs
//!
//! This module defines a `log!` macro that takes care of formatting logs output. The macro takes
//! the identity of the logger (see `Log` enum) and behaves as `format!` for the remaining arguments.
//!
//! ```ignore
//! # Doctest ignored as I can't figure out how to bring the macro in scope...
//! log!(Log::Jrpc, "query: {}", "/example");
//! ```

use chrono::Utc;
use colored::*;

/// The list of modules that can emit logs.
pub enum Log {
    Jrpc,
    Grpc,
    Chain,
    Websocket,
    Node,
}

impl Log {
    pub fn to_colored_string(&self) -> ColoredString {
        match self {
            Log::Websocket => "[Websocket]".cyan(),
            Log::Jrpc => "[JsonRPC]".yellow(),
            Log::Chain => "[Chain]".magenta(),
            Log::Grpc => "[gRPC]".green(),
            Log::Node => "[Node]".bright_magenta(),
        }
    }
}

/// Return a formatted string of the curent time.
pub fn now() -> ColoredString {
    let now = Utc::now().format("%H:%M:%S");
    now.to_string().blue()
}

#[macro_escape]
macro_rules! log {
    ($logger:expr, $str:expr, $($params:expr),*) => {
        let fmt_str = format!($str, $($params,)*);
        println!("{} {:>11} {}",crate::logger::now(),$logger.to_colored_string(), fmt_str);
    };
    ($logger:expr, $str:expr) => {
        println!("{} {:>11} {}", crate::logger::now(), $logger.to_colored_string(), $str);
    };
}

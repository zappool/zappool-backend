//! `nsec-tool` is a command-line utility to create or check secret files.

mod nsecstore_tool;

use crate::nsecstore_tool::NsecStoreTool;
use std::env;

/// Top-level executable implementation for seedstore-tool.
fn main() {
    let args: Vec<String> = env::args().collect();
    NsecStoreTool::run(&args);
}

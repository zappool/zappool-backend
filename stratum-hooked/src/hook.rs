use stratumv1_proxy_rs::{CommandMessage, Direction, Hook, ResponseMessage};

use anyhow::Result;
use serde_json::Value;

///  Our hook for proxying (username conversion, workstat saving)
pub struct ZPHook {}

impl ZPHook {
    pub fn new() -> Self {
        Self {}
    }
}

impl Hook for ZPHook {
    fn process_command(
        &self,
        _dir: Direction,
        _client_addr: std::net::SocketAddr,
        _message: &CommandMessage,
    ) -> Result<Option<Value>> {
        Ok(None)
    }

    /// Hook to use of a response before forwarding.
    fn process_response(
        &self,
        _dir: Direction,
        _client_addr: std::net::SocketAddr,
        _response: &ResponseMessage,
    ) {
    }
}

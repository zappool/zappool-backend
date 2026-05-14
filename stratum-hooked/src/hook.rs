use common_rs::username::map_full_username;
use stratumv1_proxy_rs::{CommandMessage, Direction, Hook, ResponseMessage};

use anyhow::Result;
use serde_json::Value;

/// Configuration for the hooked Zappool Stratum proxy
#[derive(Debug, Clone)]
pub struct StratumHookedConfig {
    pub upstream_user: String,
}

impl StratumHookedConfig {
    pub fn new(upstream_user: String) -> Self {
        Self { upstream_user }
    }
}

///  Our hook for proxying (username conversion, workstat saving)
pub struct ZPHook {
    config: StratumHookedConfig,
}

impl ZPHook {
    pub fn new(config: StratumHookedConfig) -> Self {
        Self { config }
    }
}

impl Hook for ZPHook {
    fn process_command(
        &self,
        dir: Direction,
        _client_addr: std::net::SocketAddr,
        message: &CommandMessage,
    ) -> Result<Option<Value>> {
        if dir == Direction::ClientToUpstream
            && message.method.to_ascii_lowercase() == "mining.submit"
        {
            let mut params = message.params.clone();
            if let Some(params_arr) = params.as_array() {
                if params_arr.iter().len() >= 1 {
                    let user_o_full = params_arr
                        .first()
                        .unwrap_or_default()
                        .as_str()
                        .unwrap_or_default();
                    if !user_o_full.is_empty() {
                        let user_us_full =
                            map_full_username(user_o_full, &self.config.upstream_user);
                        params[0] = Value::String(user_us_full);
                        return Ok(Some(params));
                    }
                }
            }
            // TODO: hook for accept, save to workstat
            return Ok(Some(params));
        }
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

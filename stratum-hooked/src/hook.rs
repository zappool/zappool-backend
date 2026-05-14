use common_rs::username::map_full_username;
use stratumv1_proxy_rs::{CommandMessage, Direction, Hook, ResponseMessage};

use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;

/// Configuration for the hooked Zappool Stratum proxy
#[derive(Debug, Clone)]
pub struct StratumHookedConfig {
    pub upstream_user: String,
    pub workstat_api_url: String,
    pub workstat_secret: String,
    pub us_pool: u16,
}

impl StratumHookedConfig {
    pub fn new(
        upstream_user: String,
        workstat_api_url: String,
        workstat_secret: String,
        us_pool: u16,
    ) -> Self {
        Self {
            upstream_user,
            workstat_api_url,
            workstat_secret,
            us_pool,
        }
    }
}

#[derive(Serialize)]
pub struct WorkInsertRequest {
    uname_o: String,
    uname_u: String,
    tdiff: u32,
    sec: String,
    pool: u8,
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

#[async_trait]
impl Hook for ZPHook {
    async fn process_command(
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
                        let user_u_full =
                            map_full_username(user_o_full, &self.config.upstream_user);

                        // hook for accept, save to workstat
                        // TODO: it's more precise to have this in datum_protocol_share_response(), but we don't have the username there
                        if !self.config.workstat_api_url.is_empty() {
                            let workstat_payload = WorkInsertRequest {
                                uname_o: user_o_full.to_string(),
                                uname_u: user_u_full.clone(),
                                // TODO proper diff, where from?
                                tdiff: 131072,
                                sec: self.config.workstat_secret.clone(),
                                pool: self.config.us_pool as u8,
                            };
                            let workstat_url =
                                format!("{}/api/work-insert", self.config.workstat_api_url);
                            match reqwest::Client::new()
                                .post(&workstat_url)
                                .json(&workstat_payload)
                                .send()
                                .await
                            {
                                Err(err) => println!(
                                    "ERROR! couldn't connect to Workstat server {} {:?}",
                                    workstat_url, err
                                ),
                                Ok(resp) => match resp.json::<serde_json::Value>().await {
                                    Err(err) => {
                                        println!(
                                            "ERROR! couldn't read Workstat response {:?}",
                                            err
                                        );
                                    }
                                    Ok(body) => {
                                        println!("Workstat response: {:?}", body);
                                    }
                                },
                            }
                        }

                        // Substitute the username
                        params[0] = Value::String(user_u_full);
                        return Ok(Some(params));
                    }
                }
            }
            return Ok(Some(params));
        }
        Ok(None)
    }

    /// Hook to use of a response before forwarding.
    async fn process_response(
        &self,
        _dir: Direction,
        _client_addr: std::net::SocketAddr,
        _response: &ResponseMessage,
    ) {
    }
}

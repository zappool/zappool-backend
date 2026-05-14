use stratumv1_proxy_rs::ClientStub as ClientStub_base;
use stratumv1_proxy_rs::Message;

use anyhow::Result;
use serde_json::Value;

/// A customized ClientStub (Stratum V1 client stub) for testing.
#[derive(Clone)]
pub struct ClientStub {
    cli: ClientStub_base,
}

impl ClientStub {
    pub fn new(server_addr: &str, username: &str) -> Self {
        let cli = ClientStub_base::new(server_addr, username);
        Self { cli }
    }

    pub async fn start(&mut self) -> Result<()> {
        self.cli.start().await
    }

    pub async fn stop(&self, wait_for_read: bool) -> Result<()> {
        self.cli.stop(wait_for_read).await
    }

    pub async fn send_command(&mut self, method: String, params: Value) -> Result<()> {
        self.cli.send_command(method, params).await
    }

    pub async fn get_message_count(&self) -> usize {
        self.cli.get_message_count().await
    }

    pub async fn get_message_by_index(&self, index: usize) -> Option<Message> {
        self.cli.get_message_by_index(index).await
    }

    pub async fn get_message_by_id(&self, id: &str) -> Option<Message> {
        self.cli.get_message_by_id(id).await
    }

    pub async fn send_mining_configure(&mut self) -> Result<()> {
        self.cli.send_mining_configure().await
    }

    pub async fn send_mining_subscribe(&mut self) -> Result<()> {
        self.cli.send_mining_subscribe().await
    }

    pub async fn send_mining_authorize(&mut self) -> Result<()> {
        self.cli.send_mining_authorize().await
    }

    pub async fn send_mining_suggest_difficulty(&mut self, difficluty: u64) -> Result<()> {
        self.cli.send_mining_suggest_difficulty(difficluty).await
    }
}

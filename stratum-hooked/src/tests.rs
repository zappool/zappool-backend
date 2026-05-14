use crate::my_hooks;
use crate::{client_stub::ClientStub, hook::StratumHookedConfig};

use serde_json::json;
use serial_test::serial;
use stratumv1_proxy_rs::{Proxy, ProxyConfig, ServerStub};

const UPSTREAM_USER: &str = "upstreamuser";

#[tokio::test]
#[serial]
async fn test_proxy_connect_only() {
    let server_addr = "127.0.0.1:43333";
    let proxy_addr = "127.0.0.1:53333";

    let server = ServerStub::new(server_addr);
    let _ = server.start().await.unwrap();

    let proxy_config = ProxyConfig::new(proxy_addr.to_string(), server_addr.to_string());
    let proxy = Proxy::new(
        proxy_config,
        my_hooks(&StratumHookedConfig::new(UPSTREAM_USER.into())),
    );
    let _ = proxy.start().await.unwrap();

    let mut client = ClientStub::new(proxy_addr, "username.device");
    let _ = client.start().await.unwrap();

    let _ = client.stop(false).await.unwrap();
    let _ = proxy.stop(false).await.unwrap();
    let _ = server.stop(true).await.unwrap();

    assert_eq!(server.get_connect_count().await, 1);
    assert_eq!(server.get_message_count().await, 0);
    assert_eq!(client.get_message_count().await, 0);
}

#[tokio::test]
#[serial]
async fn test_proxy_mining_notify_submit() {
    let server_addr = "127.0.0.1:43333";
    let proxy_addr = "127.0.0.1:53333";

    let server = ServerStub::new(server_addr);
    let _ = server.start().await.unwrap();

    let proxy_config = ProxyConfig::new(proxy_addr.to_string(), server_addr.to_string());
    let proxy = Proxy::new(
        proxy_config,
        my_hooks(&StratumHookedConfig::new(UPSTREAM_USER.into())),
    );
    let _ = proxy.start().await.unwrap();

    let username = "username1.device1";
    let mut client = ClientStub::new(proxy_addr, username);
    let _ = client.start().await.unwrap();
    let _ = client.send_mining_configure().await.unwrap();
    let _ = client.send_mining_subscribe().await.unwrap();
    let _ = client.send_mining_authorize().await.unwrap();
    let _ = client.send_mining_suggest_difficulty(1000).await.unwrap();
    let submit_params = json![[
        username,
        "699f6b4c00008ff1",
        "010000000090ce3f",
        "69afeeea",
        "7a300274",
        "05eb4000"
    ]];
    let _ = client
        .send_command("mining.submit".to_string(), submit_params)
        .await
        .unwrap();

    let _ = client.stop(true).await.unwrap();
    let _ = proxy.stop(true).await.unwrap();
    let _ = server.stop(true).await.unwrap();

    // Now check what did the stub receive
    assert_eq!(server.get_connect_count().await, 1);
    assert_eq!(server.get_message_count().await, 5);
    let msg1 = server.get_message_by_id("1").await.unwrap();
    assert_eq!(msg1.method().unwrap(), "mining.configure");
    let msg2 = server.get_message_by_id("2").await.unwrap();
    assert_eq!(msg2.method().unwrap(), "mining.subscribe");
    let msg3 = server.get_message_by_id("3").await.unwrap();
    assert_eq!(msg3.method().unwrap(), "mining.authorize");
    let msg4 = server.get_message_by_id("4").await.unwrap();
    assert_eq!(msg4.method().unwrap(), "mining.suggest_difficulty");
    let msg5 = server.get_message_by_id("5").await.unwrap();
    assert_eq!(msg5.method().unwrap(), "mining.submit");
    assert_eq!(
        msg5.to_json()
            .as_object()
            .unwrap()
            .get("params")
            .unwrap()
            .to_string(),
        "[\"upstreamuser.cb504b8c\",\"699f6b4c00008ff1\",\"010000000090ce3f\",\"69afeeea\",\"7a300274\",\"05eb4000\"]"
    );

    assert_eq!(client.get_message_count().await, 6);
    let resp1 = client.get_message_by_id("1").await.unwrap();
    assert_eq!(
        resp1.to_string(),
        "1 null {\"version-rolling.mask\":\"1fffe000\"}"
    );
    let resp3 = client.get_message_by_index(3).await.unwrap();
    assert_eq!(resp3.to_string(), "null mining.set_difficulty 1000");
    let resp4 = client.get_message_by_index(4).await.unwrap();
    assert_eq!(resp4.method().unwrap(), "mining.notify");
    let resp5 = client.get_message_by_id("5").await.unwrap();
    assert_eq!(resp5.to_string(), "5 null true");
}

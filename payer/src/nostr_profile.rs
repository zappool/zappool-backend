use bech32::{FromBase32, decode};
use futures_util::{SinkExt, StreamExt};
use hex_conservative::DisplayHex;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

use std::error::Error;
use std::time::{Duration, Instant};

#[derive(Debug, Serialize, Deserialize)]
struct NostrEvent {
    id: String,
    pubkey: String,
    created_at: u64,
    kind: u32,
    tags: Vec<Vec<String>>,
    content: String,
    sig: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProfileData {
    name: Option<String>,
    display_name: Option<String>,
    about: Option<String>,
    picture: Option<String>,
    banner: Option<String>,
    nip05: Option<String>,
    lud16: Option<String>,
    website: Option<String>,
}

/// Convert npub to hex format
fn npub_to_hex(npub: &str) -> Result<String, Box<dyn Error>> {
    let (hrp, data, _variant) = decode(npub)?;

    if hrp != "npub" {
        return Err(format!("Expected hrp to be 'npub', got '{}'", hrp).into());
    }

    // Convert from base32 to bytes
    let bytes = Vec::<u8>::from_base32(&data)?;
    let hex_pubkey = bytes.to_lower_hex_string();
    Ok(hex_pubkey)
}

/// Get profile data from a Nostr relay
async fn get_profile(relay_url: &str, npub: &str) -> Result<Option<ProfileData>, Box<dyn Error>> {
    // Convert npub to hex format
    let pubkey_hex = npub_to_hex(npub)?;
    println!("Converted npub to hex: {}", pubkey_hex);

    // Create a subscription ID
    let subscription_id = Uuid::new_v4().to_string()[..8].to_string();

    // Create the request to get user metadata
    let request = json!([
        "REQ",
        subscription_id,
        {
            "kinds": [0],  // Kind 0 is for metadata events
            "authors": [pubkey_hex]
        }
    ]);

    // Connect to the relay
    println!("Connecting to relay: {}", relay_url);
    let (ws_stream, _) = connect_async(relay_url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Send the request
    let request_str = serde_json::to_string(&request)?;
    write.send(Message::Text(request_str.clone())).await?;
    println!("Sent request: {}", request_str);

    // Wait for the response with a timeout
    let timeout_duration = Duration::from_secs(10);
    let start_time = Instant::now();

    while start_time.elapsed() < timeout_duration {
        match timeout(Duration::from_secs(1), read.next()).await {
            Ok(Some(Ok(msg))) => {
                if let Message::Text(text) = msg {
                    let response_data: Value = serde_json::from_str(&text)?;
                    println!("{}", serde_json::to_string_pretty(&response_data)?);

                    // Check if this is an EVENT message for our subscription
                    if let Some(array) = response_data.as_array() {
                        if array.len() >= 3
                            && array[0].as_str() == Some("EVENT")
                            && array[1].as_str() == Some(&subscription_id)
                        {
                            if let Some(event_obj) = array[2].as_object() {
                                if event_obj.get("kind") == Some(&json!(0)) {
                                    // Parse the content which contains the profile data
                                    if let Some(content_str) =
                                        event_obj.get("content").and_then(|c| c.as_str())
                                    {
                                        let profile_data: ProfileData =
                                            serde_json::from_str(content_str)?;
                                        println!("\nProfile Data:");
                                        println!(
                                            "{}",
                                            serde_json::to_string_pretty(&profile_data)?
                                        );

                                        // Send CLOSE to end the subscription
                                        let close_msg = json!(["CLOSE", subscription_id]);
                                        write
                                            .send(Message::Text(serde_json::to_string(&close_msg)?))
                                            .await?;
                                        return Ok(Some(profile_data));
                                    }
                                }
                            }
                        }

                        // Check if we got an EOSE (End of Stored Events)
                        if array.len() >= 2
                            && array[0].as_str() == Some("EOSE")
                            && array[1].as_str() == Some(&subscription_id)
                        {
                            println!("End of stored events received, no profile data found.");

                            // Send CLOSE to end the subscription
                            let close_msg = json!(["CLOSE", subscription_id]);
                            write
                                .send(Message::Text(serde_json::to_string(&close_msg)?))
                                .await?;
                            return Ok(None);
                        }
                    }
                }
            }
            Ok(Some(Err(e))) => {
                return Err(format!("WebSocket error: {}", e).into());
            }
            Ok(None) => {
                return Err(format!("WebSocket connection closed").into());
            }
            Err(_) => {
                // Timeout on individual message, continue the loop
                continue;
            }
        }
    }

    println!("Timeout reached, no profile data received.");
    Ok(None)
}

/// Get Lightning Address from profile
async fn get_nostr_ln_address_relay(
    npub: &str,
    relay: &str,
) -> Result<Option<String>, Box<dyn Error>> {
    let profile = get_profile(relay, npub).await?;

    if let Some(profile_data) = profile {
        println!("profile: {:?}", profile_data);
        if let Some(lud16) = &profile_data.lud16 {
            if !lud16.is_empty() {
                return Ok(Some(lud16.clone()));
            }
        }
        println!(
            "ERROR: User profile for '{}' doesn't contain 'lud16' (relay: {}, profile: {:?})",
            npub, relay, profile_data
        );
    }

    Ok(None)
}

/// Get Lightning Address from multiple relays
pub async fn get_nostr_ln_address(npub: &str) -> Result<String, Box<dyn Error>> {
    let relays = [
        "relay.damus.io",
        "relay.primal.net",
        "nostr.wine",
        "nos.lol",
        "relay.snort.social",
        "nostr.land",
        "nostr.mom",
        "relay.nostr.band",
        "nostr.oxtr.dev",
    ];

    for relay in &relays {
        let relay_url = format!("wss://{}", relay);
        if let Ok(Some(ln_addr)) = get_nostr_ln_address_relay(npub, &relay_url).await {
            return Ok(ln_addr);
        }
    }

    let err_msg = format!(
        "ERROR: Could not obtain LNAddr for '{}' (relays: {:?})",
        npub, relays
    );
    println!("{err_msg}");
    Err(err_msg.into())
}

#[allow(dead_code)]
async fn do_try() -> Result<(), Box<dyn Error>> {
    // Default values
    let default_relay_url = "wss://relay.damus.io";
    let default_npub = "npub12rv5lskctqxxs2c8rf2zlzc7xx3qpvzs3w4etgemauy9thegr43sf485vg";

    // Get command line arguments
    let args: Vec<String> = std::env::args().collect();
    let relay_url = args.get(1).map(|s| s.as_str()).unwrap_or(default_relay_url);
    let npub = args.get(2).map(|s| s.as_str()).unwrap_or(default_npub);

    println!("Fetching profile for {} from {}", npub, relay_url);

    // Test Lightning Address functionality
    if let Ok(lnaddr) = get_nostr_ln_address(npub).await {
        println!("Lightning Address: {}", lnaddr);
    }

    // Get and display profile
    match get_profile(relay_url, npub).await {
        Ok(Some(profile)) => {
            println!("\nProfile Summary:");
            if let Some(name) = &profile.name {
                println!("Name: {}", name);
            }
            if let Some(display_name) = &profile.display_name {
                println!("Display_name: {}", display_name);
            }
            if let Some(about) = &profile.about {
                println!("About: {}", about);
            }
            if let Some(picture) = &profile.picture {
                println!("Profile Picture URL: {}", picture);
            }
            if let Some(banner) = &profile.banner {
                println!("Banner URL: {}", banner);
            }
            if let Some(nip05) = &profile.nip05 {
                println!("Nip05: {}", nip05);
            }
            if let Some(lud16) = &profile.lud16 {
                println!("Lud16: {}", lud16);
            }
            if let Some(website) = &profile.website {
                println!("Website: {}", website);
            }
        }
        Ok(None) => {
            println!("No profile found for the given npub.");
        }
        Err(e) => {
            eprintln!("Error fetching profile: {}", e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_npub_to_hex() {
        let npub = "npub12rv5lskctqxxs2c8rf2zlzc7xx3qpvzs3w4etgemauy9thegr43sf485vg";
        let hex = npub_to_hex(npub).unwrap();
        assert_eq!(
            hex,
            "50d94fc2d8580c682b071a542f8b1e31a200b0508bab95a33bef0855df281d63"
        );
    }
}

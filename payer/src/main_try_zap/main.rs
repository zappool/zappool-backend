use payer::nostr_zap::nostr_zap;

use dotenv;
use nostr::SecretKey;
use seedstore::KeyStore;

use std::env;

const DEFAULT_SECRET_FILE: &str = "secret.nsec";

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    let nsec_password = env::var("NOSTR_NSEC_FILE_PASSWORD").unwrap_or("MISSING".to_owned());

    let keystore = KeyStore::new_from_encrypted_file(DEFAULT_SECRET_FILE, &nsec_password).unwrap();
    let nsec1 = keystore
        .get_secret_private_key()
        .map_err(|e| e.to_string())
        .unwrap();
    let nsec = SecretKey::from_slice(&nsec1.secret_bytes()).unwrap();

    let rec_npub = "npub12rv5lskctqxxs2c8rf2zlzc7xx3qpvzs3w4etgemauy9thegr43sf485vg";
    let relays = vec![
        "wss://relay.primal.net/",
        "wss://relay.damus.io/",
        "wss://nos.lol/",
    ];
    match nostr_zap(2000, &nsec, rec_npub, &relays).await {
        Err(e) => println!("ERROR: {:?}", e),
        Ok(_) => {}
    }
}

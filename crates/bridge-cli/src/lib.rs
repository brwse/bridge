use std::process;

use brwse_bridge_registry::client::ClientBuilder;
use chrono::Duration;
use clap::Args;
use jsonwebtoken::DecodingKey;
use tracing::{error, info};

#[derive(Args, Clone)]
pub struct RegistryArgs {
    /// Registry endpoint
    #[arg(long, env = "BRWSE_REGISTRY_ENDPOINT")]
    pub registry_endpoint: String,

    /// Public key for JWT token validation (PEM format)
    #[arg(long, env = "BRWSE_REGISTRY_PUBLIC_KEY")]
    pub public_key: String,

    /// Token refresh interval in seconds
    #[arg(long, default_value = "300", env = "BRWSE_REGISTRY_REFRESH_INTERVAL")]
    pub refresh_interval: u64,

    /// Token refresh leeway in seconds
    #[arg(long, default_value = "30", env = "BRWSE_REGISTRY_REFRESH_LEEWAY")]
    pub refresh_leeway: u64,

    /// Bridge registration token
    #[arg(long, env = "BRWSE_REGISTRY_TOKEN")]
    pub br_token: String,
}

#[derive(Args, Clone)]
pub struct BridgeArgs {
    /// Bridge listen address
    #[arg(long, default_value = "127.0.0.1:9000", env = "BRWSE_BRIDGE_LISTEN")]
    pub listen: String,

    #[command(flatten)]
    pub registry: Option<RegistryArgs>,
}

pub async fn setup_registry(args: &RegistryArgs) {
    // Parse the public key for JWT validation
    let decoding_key = DecodingKey::from_rsa_pem(args.public_key.as_bytes()).unwrap_or_else(|e| {
        error!("Failed to parse public key: {}", e);
        process::exit(1);
    });

    // Build the registry client
    let registry_client = ClientBuilder::default()
        .endpoint(args.registry_endpoint.clone())
        .decoding_key(decoding_key)
        .refresh_leeway(Duration::seconds(args.refresh_leeway as i64))
        .build()
        .await
        .unwrap_or_else(|e| {
            error!("Failed to build registry client: {}", e);
            process::exit(1);
        });

    // Register the bridge
    info!("Registering bridge with registry...");
    if let Err(e) = registry_client.register(args.br_token.clone()).await {
        error!("Failed to register bridge: {}", e);
        process::exit(1);
    }
    info!("Bridge registered successfully");

    // Start the token refresh task
    let _refresh_handle = registry_client.spawn_refresh_task();
}

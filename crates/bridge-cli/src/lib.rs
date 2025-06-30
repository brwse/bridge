use clap::Args;

#[derive(Args, Clone)]
pub struct RegistryArgs {
    /// Registry endpoint
    #[arg(long, default_value = "https://registry.brwse.ai", env = "BRWSE_REGISTRY_ENDPOINT")]
    pub registry_endpoint: String,

    /// Public key for JWT token validation (PEM format)
    #[arg(long, env = "BRWSE_REGISTRY_PUBLIC_KEY")]
    pub public_key: Option<String>,

    /// Token refresh interval in seconds
    #[arg(long, default_value = "300", env = "BRWSE_REGISTRY_REFRESH_INTERVAL")]
    pub refresh_interval: u64,

    /// Token refresh leeway in seconds
    #[arg(long, default_value = "30", env = "BRWSE_REGISTRY_REFRESH_LEEWAY")]
    pub refresh_leeway: u64,

    /// Bridge registration token
    #[arg(long, env = "BRWSE_REGISTRY_TOKEN", requires = "public_key")]
    pub br_token: Option<String>,
}

#[derive(Args, Clone)]
pub struct BridgeArgs {
    /// Bridge listen address
    #[arg(long, default_value = "127.0.0.1:9000", env = "BRWSE_BRIDGE_LISTEN")]
    pub listen: String,
}

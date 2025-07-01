use std::{process, sync::Arc};

use brwse_bridge_cli::BridgeArgs;
use brwse_bridge_postgres::bridge::PostgresBridge;
use clap::Parser;
use tracing::{error, info};

#[derive(Parser)]
#[command(author, version, about = "Postgres Bridge - PostgreSQL protocol bridge")]
struct Args {
    /// PostgreSQL server address
    #[arg(
        long,
        env = "BRWSE_DATABASE_URL",
        default_value = "postgres://postgres:postgres@localhost:5432/postgres"
    )]
    database_url: String,

    #[command(flatten)]
    bridge: BridgeArgs,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Build the PostgreSQL bridge
    info!("Starting PostgreSQL bridge on {} -> {:?}", args.bridge.listen, args.database_url);

    let (client, connection) =
        match tokio_postgres::connect(&args.database_url, tokio_postgres::NoTls).await {
            Ok((client, connection)) => (client, connection),
            Err(e) => {
                error!("Failed to connect to PostgreSQL: {}", e);
                process::exit(1);
            }
        };
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("PostgreSQL connection error: {}", e);
        }
    });

    let bridge = PostgresBridge::new(Arc::new(client));

    let mcp_ct = brwse_bridge_mcp::bridge::start(&args.bridge.listen, bridge)
        .await
        .expect("failed to start MCP server");

    let _result = tokio::signal::ctrl_c().await;
    info!("Received shutdown signal, stopping bridge...");

    mcp_ct.cancel();
}

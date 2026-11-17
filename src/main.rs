use clap::Parser;
use mcp_openproject::client::OpenProjectClient;
use mcp_openproject::config::Config;
use mcp_openproject::server::OpenProjectServer;
use rmcp::ServiceExt;
use std::process::ExitCode;
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(
    name = "mcp-openproject",
    about = "Model Context Protocol (MCP) server for OpenProject",
    version
)]
struct Cli {
    /// Test connection to OpenProject API v3 and exit
    #[arg(short = 't', long = "test-connection")]
    test_connection: bool,

    /// OpenProject base URL (overrides OPENPROJECT_URL env var)
    #[arg(long, env = "OPENPROJECT_URL")]
    url: Option<String>,

    /// OpenProject API Key (overrides OPENPROJECT_API_KEY env var)
    #[arg(long, env = "OPENPROJECT_API_KEY")]
    api_key: Option<String>,
}

fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("mcp_openproject=info,warn"));

    // CRITICAL for MCP: All logs MUST go to stderr, stdout is reserved for JSON-RPC messages.
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();
}

#[tokio::main]
async fn main() -> ExitCode {
    init_logging();

    let cli = Cli::parse();

    let config = match Config::load(cli.url, cli.api_key) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("Configuration error: {err}");
            eprintln!(
                "Please provide OPENPROJECT_URL and OPENPROJECT_API_KEY via environment variables, .env file, or CLI flags."
            );
            return ExitCode::FAILURE;
        }
    };

    let client = match OpenProjectClient::new(config.clone()) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Failed to initialize HTTP client: {err}");
            return ExitCode::FAILURE;
        }
    };

    if cli.test_connection {
        info!(
            "Testing connection to OpenProject at {}...",
            config.base_url
        );
        match client.test_connection().await {
            Ok(root_json) => {
                let instance_name = root_json
                    .get("instanceName")
                    .and_then(|v| v.as_str())
                    .unwrap_or("OpenProject");
                eprintln!(
                    "Successfully connected to {instance_name} at {}",
                    config.base_url
                );
                return ExitCode::SUCCESS;
            }
            Err(err) => {
                eprintln!("Connection failed: {err}");
                return ExitCode::FAILURE;
            }
        }
    }

    info!("Starting mcp-openproject server over stdio...");
    let server = OpenProjectServer::new(client);
    let running = match server.serve(rmcp::transport::io::stdio()).await {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Failed to start MCP server: {err}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = running.waiting().await {
        eprintln!("MCP server stopped with error: {err}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

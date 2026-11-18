use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use nexora_mcp::{GatewaySettings, GatewayState, McpGateway};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("nexora-mcp: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|argument| argument == "--help" || argument == "-h") {
        println!(
            "nexora-mcp {}\nStateless MCP gateway\n\nConfiguration: NEXORA_MCP_UPSTREAM, NEXORA_MCP_BIND",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }
    if std::env::args().any(|argument| argument == "--version" || argument == "-V") {
        println!("nexora-mcp {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let settings = GatewaySettings::load()?;
    let state = GatewayState::new(settings.upstream)?;
    let app = Router::new()
        .route(
            "/mcp",
            post(McpGateway::post).layer(DefaultBodyLimit::max(29 * 1024 * 1024)),
        )
        .route("/health", get(McpGateway::health))
        .with_state(std::sync::Arc::new(state));
    let listener = tokio::net::TcpListener::bind(settings.bind).await?;
    eprintln!("nexora-mcp listening on {}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("register SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = terminate.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

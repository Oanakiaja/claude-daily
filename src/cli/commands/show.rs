use anyhow::{Context, Result};
use colored::Colorize;
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;
use tokio::signal;

use crate::config::load_config;
use crate::server::{create_router, handlers::AppState};

const DEFAULT_PORT: u16 = 31456;
const MAX_PORT_ATTEMPTS: u16 = 100;

/// Run the web dashboard server
pub async fn run(port: Option<u16>, host: String, open_browser: bool) -> Result<()> {
    let config = load_config()?;
    let state = Arc::new(AppState {
        config: RwLock::new(config),
    });

    // Find available port
    let (listener, actual_port) = find_available_port(&host, port).await?;
    let url = format!("http://{}:{}", host, actual_port);

    println!("{}", "Starting Daily Dashboard...".green().bold());
    println!();
    println!("  {} {}", "URL:".dimmed(), url.cyan());
    println!();
    println!("{}", "Press Ctrl+C to stop the server".dimmed());
    println!();

    // Open browser
    if open_browser {
        if let Err(e) = open::that(&url) {
            eprintln!("{} Failed to open browser: {}", "Warning:".yellow(), e);
        }
    }

    // Create router and start server
    let app = create_router(state);

    // Run server with graceful shutdown on Ctrl+C
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("Server error")?;

    println!();
    println!("{}", "Server stopped.".dimmed());

    Ok(())
}

/// Find an available port, starting from the specified port or DEFAULT_PORT
async fn find_available_port(host: &str, port: Option<u16>) -> Result<(TcpListener, u16)> {
    let start_port = port.unwrap_or(DEFAULT_PORT);

    // If user specified a port, only try that one
    if port.is_some() {
        let addr = format!("{}:{}", host, start_port);
        let listener = TcpListener::bind(&addr)
            .await
            .context(format!("Port {} is not available", start_port))?;
        return Ok((listener, start_port));
    }

    // Auto-increment to find available port
    for offset in 0..MAX_PORT_ATTEMPTS {
        let try_port = start_port + offset;
        let addr = format!("{}:{}", host, try_port);

        match TcpListener::bind(&addr).await {
            Ok(listener) => return Ok((listener, try_port)),
            Err(_) => continue,
        }
    }

    anyhow::bail!(
        "Could not find available port in range {}-{}",
        start_port,
        start_port + MAX_PORT_ATTEMPTS - 1
    )
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

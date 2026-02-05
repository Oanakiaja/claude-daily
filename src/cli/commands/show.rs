use anyhow::{Context, Result};
use colored::Colorize;
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;
use tokio::signal;

use crate::auto_summarize::{
    find_unsummarized_transcripts, should_trigger_auto_summarize,
    should_trigger_auto_summarize_on_show,
};
use crate::config::{load_config, save_config};
use crate::server::{create_router, handlers::AppState};

const DEFAULT_PORT: u16 = 31456;
const MAX_PORT_ATTEMPTS: u16 = 100;

/// Run the web dashboard server
pub async fn run(port: Option<u16>, host: String, open_browser: bool) -> Result<()> {
    let mut config = load_config()?;

    // Check if we should trigger auto-summarization
    // Either: on_show is enabled (triggers every time) OR time-based trigger is due
    let should_trigger =
        should_trigger_auto_summarize_on_show(&config) || should_trigger_auto_summarize(&config)?;

    if should_trigger {
        // Spawn background jobs for unsummarized transcripts
        match trigger_auto_summarize(&config).await {
            Ok(count) => {
                if count > 0 {
                    println!(
                        "{} {} unsummarized session(s)",
                        "Auto-summarizing".yellow(),
                        count
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "{} Failed to trigger auto-summarization: {}",
                    "Warning:".yellow(),
                    e
                );
            }
        }

        // Update last check time (only for time-based trigger tracking)
        if !should_trigger_auto_summarize_on_show(&config) {
            config.summarization.last_auto_summarize_check =
                Some(chrono::Local::now().to_rfc3339());
            save_config(&config)?;
        }
    }

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

/// Trigger auto-summarization for unsummarized transcripts
async fn trigger_auto_summarize(config: &crate::config::Config) -> Result<usize> {
    use crate::jobs::{JobManager, JobType};
    use std::process::{Command, Stdio};

    #[cfg(unix)]
    use std::os::unix::process::CommandExt;

    // Find unsummarized transcripts
    let unsummarized = find_unsummarized_transcripts(config)?;

    if unsummarized.is_empty() {
        return Ok(0);
    }

    // Initialize job manager
    let job_manager = JobManager::new(config)?;

    let mut spawned_count = 0;

    for transcript in unsummarized {
        // Generate task name from session ID
        let task_name = format!("auto-{}", transcript.session_id);

        // Generate job ID
        let job_id = JobManager::generate_job_id(&task_name);
        let transcript_path_str = transcript.path.to_string_lossy().to_string();

        // Create log file for the job
        let (stdout_file, stderr_file) = match job_manager.create_log_file(&job_id) {
            Ok(f) => {
                let f2 = f.try_clone().unwrap_or_else(|_| {
                    std::fs::File::create("/dev/null").expect("Failed to open /dev/null")
                });
                (Stdio::from(f), Stdio::from(f2))
            }
            Err(_) => (Stdio::null(), Stdio::null()),
        };

        // Default CWD for auto-summarize jobs
        let cwd_str = transcript
            .cwd
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());

        // Spawn background process for summarization
        let mut cmd = Command::new("daily");
        cmd.args([
            "summarize",
            "--transcript",
            &transcript_path_str,
            "--task-name",
            &task_name,
            "--cwd",
            &cwd_str,
            "--job-id",
            &job_id,
            "--foreground",
        ])
        .stdin(Stdio::null())
        .stdout(stdout_file)
        .stderr(stderr_file);

        // Create a new process group so it doesn't get killed
        #[cfg(unix)]
        cmd.process_group(0);

        match cmd.spawn() {
            Ok(child) => {
                // Register the job with AutoSummarize type
                if let Err(e) = job_manager.register(
                    &job_id,
                    child.id(),
                    &task_name,
                    &transcript.path,
                    JobType::AutoSummarize,
                ) {
                    eprintln!("[daily] Failed to register auto-summarize job: {}", e);
                } else {
                    spawned_count += 1;
                }
            }
            Err(e) => {
                eprintln!(
                    "[daily] Failed to spawn auto-summarize process for {}: {}",
                    transcript.session_id, e
                );
            }
        }
    }

    Ok(spawned_count)
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

use anyhow::Result;
use std::process::{Command, Stdio};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use crate::config::load_config;
use crate::hooks::read_hook_input;
use crate::jobs::JobManager;

/// Handle SessionEnd hook from Claude Code
/// Spawns background process for summarization
pub async fn handle() -> Result<()> {
    let config = load_config()?;

    // Check if hooks are enabled
    if !config.hooks.enable_session_end {
        return Ok(());
    }

    // Read hook input from stdin
    let input = match read_hook_input() {
        Ok(input) => input,
        Err(e) => {
            eprintln!("[daily] Failed to read hook input: {}", e);
            return Ok(()); // Don't block session exit
        }
    };

    // Archive on all session end reasons to collect complete history
    // Reasons: "prompt_input_exit" (Ctrl+D), "logout", "clear", "other"
    eprintln!(
        "[daily] Session ended with {:?}, starting archive",
        input.reason
    );

    // Generate task name from working directory
    let task_name = generate_task_name(&input.cwd);

    // Initialize job manager
    let job_manager = match JobManager::new(&config) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[daily] Failed to initialize job manager: {}", e);
            return Ok(()); // Don't block session exit
        }
    };

    // Generate job ID
    let job_id = JobManager::generate_job_id(&task_name);
    let transcript_path = input.transcript_path.to_string_lossy().to_string();

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

    // Get cwd as string for passing to summarize command
    let cwd_str = input.cwd.to_string_lossy().to_string();

    // Spawn background process for summarization
    // This ensures Claude Code can exit immediately
    let mut cmd = Command::new("daily");
    cmd.args([
        "summarize",
        "--transcript",
        &transcript_path,
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

    // Create a new process group so Ctrl+C/Cmd+C doesn't kill the background process
    #[cfg(unix)]
    cmd.process_group(0);

    match cmd.spawn()
    {
        Ok(child) => {
            // Register the job
            if let Err(e) =
                job_manager.register(&job_id, child.id(), &task_name, &input.transcript_path)
            {
                eprintln!("[daily] Failed to register job: {}", e);
            }
            eprintln!(
                "[daily] Background summarization started: {} (PID: {})",
                job_id,
                child.id()
            );
        }
        Err(e) => {
            eprintln!("[daily] Failed to spawn summarization process: {}", e);
        }
    }

    Ok(())
}

/// Generate a task name from the working directory
fn generate_task_name(cwd: &std::path::Path) -> String {
    // Extract project name from path
    let name = cwd
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed-session".into());

    // Add timestamp to make unique
    let timestamp = chrono::Local::now().format("%H%M%S");
    format!("{}-{}", name, timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_generate_task_name() {
        let cwd = PathBuf::from("/home/user/my-project");
        let name = generate_task_name(&cwd);
        assert!(name.starts_with("my-project-"));
    }

    #[test]
    fn test_generate_task_name_empty() {
        let cwd = PathBuf::from("/");
        let name = generate_task_name(&cwd);
        // Root directory has no file_name, should use default
        assert!(name.contains("-"));
    }
}

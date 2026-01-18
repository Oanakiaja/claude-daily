use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::process::Command;

const REPO: &str = "oanakiaja/claude-daily";
const BINARY_NAME: &str = "daily";

/// Update daily binary to the latest version
pub async fn run(check_only: bool, version: Option<String>) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("[daily] Current version: v{}", current_version);

    // Get target version
    let target_version = match version {
        Some(v) => {
            let v = if v.starts_with('v') {
                v
            } else {
                format!("v{}", v)
            };
            println!("[daily] Target version: {}", v);
            v
        }
        None => {
            println!("[daily] Checking for latest version...");
            get_latest_version().await?
        }
    };

    // Compare versions
    let target_version_num = target_version.trim_start_matches('v');
    if target_version_num == current_version {
        println!("[daily] Already up to date!");
        return Ok(());
    }

    println!("[daily] New version available: {}", target_version);

    if check_only {
        println!();
        println!("Run `daily update` to install the update.");
        return Ok(());
    }

    // Detect platform
    let platform = detect_platform()?;
    println!("[daily] Platform: {}", platform);

    // Download and install
    download_and_install(&target_version, &platform).await?;

    println!();
    println!("[daily] Successfully updated to {}", target_version);

    Ok(())
}

async fn get_latest_version() -> Result<String> {
    let api_url = format!("https://api.github.com/repos/{}/releases/latest", REPO);

    let output = Command::new("curl")
        .args([
            "-fsSL",
            "-H",
            "Accept: application/vnd.github.v3+json",
            &api_url,
        ])
        .output()
        .context("Failed to execute curl")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("404") {
            anyhow::bail!(
                "No releases found. The project may not have published any releases yet.\n\
                 You can install from source using: cargo install --path ."
            );
        }
        anyhow::bail!("Failed to fetch latest version: {}", stderr);
    }

    let body = String::from_utf8_lossy(&output.stdout);

    // Parse JSON to extract tag_name
    let json: serde_json::Value =
        serde_json::from_str(&body).context("Failed to parse GitHub API response")?;

    let tag_name = json["tag_name"]
        .as_str()
        .context("Failed to get tag_name from response")?;

    Ok(tag_name.to_string())
}

fn detect_platform() -> Result<String> {
    let os = match env::consts::OS {
        "linux" => "linux",
        "macos" => "darwin",
        "windows" => "windows",
        other => anyhow::bail!("Unsupported operating system: {}", other),
    };

    let arch = match env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        other => anyhow::bail!("Unsupported architecture: {}", other),
    };

    Ok(format!("{}-{}", os, arch))
}

async fn download_and_install(version: &str, platform: &str) -> Result<()> {
    let artifact_name = if platform.starts_with("windows") {
        format!("{}-{}.exe", BINARY_NAME, platform)
    } else {
        format!("{}-{}", BINARY_NAME, platform)
    };

    let download_url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        REPO, version, artifact_name
    );

    println!("[daily] Downloading from: {}", download_url);

    // Get current binary path
    let current_exe = env::current_exe().context("Failed to get current executable path")?;
    let install_dir = current_exe
        .parent()
        .context("Failed to get install directory")?;

    // Create temp file
    let tmp_file = install_dir.join(format!("{}.tmp", BINARY_NAME));

    // Download binary using curl
    let output = Command::new("curl")
        .args(["-fsSL", "-o", tmp_file.to_str().unwrap(), &download_url])
        .output()
        .context("Failed to execute curl")?;

    if !output.status.success() {
        // Clean up temp file if it exists
        let _ = fs::remove_file(&tmp_file);
        anyhow::bail!(
            "Failed to download binary: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Make executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&tmp_file)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&tmp_file, perms)?;
    }

    // Backup current binary
    let backup_file = install_dir.join(format!("{}.bak", BINARY_NAME));
    if backup_file.exists() {
        fs::remove_file(&backup_file)?;
    }

    // On Windows, we can't replace a running binary directly
    // On Unix, we can rename and the old file handle stays valid
    #[cfg(unix)]
    {
        fs::rename(&current_exe, &backup_file).context("Failed to backup current binary")?;
        fs::rename(&tmp_file, &current_exe).context("Failed to install new binary")?;
        // Remove backup on success
        let _ = fs::remove_file(&backup_file);
    }

    #[cfg(windows)]
    {
        // On Windows, rename current to .bak and new to target
        let target = install_dir.join(format!("{}.exe", BINARY_NAME));
        fs::rename(&current_exe, &backup_file).context("Failed to backup current binary")?;
        fs::rename(&tmp_file, &target).context("Failed to install new binary")?;
    }

    Ok(())
}

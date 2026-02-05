use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};

/// Delete the daily binary itself
pub async fn run() -> Result<()> {
    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;
    let exe_path = current_exe
        .canonicalize()
        .unwrap_or_else(|_| current_exe.clone());

    println!("[daily] Binary location: {}", exe_path.display());

    // Confirm deletion
    print!("[daily] Delete this binary? [y/N] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() == "y" {
        fs::remove_file(&exe_path).context("Failed to delete binary")?;
        println!("[daily] Binary deleted: {}", exe_path.display());
        println!("[daily] Goodbye!");
    } else {
        println!("[daily] Binary deletion cancelled.");
    }

    Ok(())
}

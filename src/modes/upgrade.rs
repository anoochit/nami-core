use std::env;
use std::fs::{File, set_permissions, rename};
use std::io::Write;
use reqwest::Client;
use inquire::Confirm;
use serde::Deserialize;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[derive(Deserialize, Debug)]
struct GithubRelease {
    tag_name: String,
}

pub async fn run_upgrade() -> anyhow::Result<()> {
    println!("🔍 Detecting current platform and architecture...");

    let target_os = env::consts::OS;
    let target_arch = env::consts::ARCH;

    println!("💻 OS: {}, Arch: {}", target_os, target_arch);

    // Map the OS and Architecture to the release asset naming convention
    let asset_name = match (target_os, target_arch) {
        ("linux", "x86_64") => "nami-linux-x86_64",
        ("linux", "aarch64") => "nami-linux-aarch64",
        ("macos", "x86_64") => "nami-macos-x86_64",
        ("macos", "aarch64") => "nami-macos-aarch64",
        ("windows", "x86_64") => "nami-windows-x86_64.exe",
        _ => {
            anyhow::bail!("Unsupported platform: {}-{}. Please download the binary manually.", target_os, target_arch);
        }
    };

    println!("📥 Contacting GitHub to fetch latest release tags...");

    let client = Client::builder()
        .user_agent("nami-cli")
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    // Fetch the list of releases from GitHub API
    let releases_url = "https://api.github.com/repos/anoochit/nami-core/releases";
    let api_response = client.get(releases_url).send().await?;

    let tag = if api_response.status().is_success() {
        let releases: Vec<GithubRelease> = api_response.json().await?;
        // Find the first release tag name starting with "nightly-"
        if let Some(nightly_release) = releases.iter().find(|r| r.tag_name.starts_with("nightly-")) {
            nightly_release.tag_name.clone()
        } else {
            println!("⚠️ No nightly release found in release history. Falling back to default tag.");
            "nightly-2026-07-09".to_string()
        }
    } else {
        println!("⚠️ Failed to fetch release list from GitHub API: {}. Falling back to default tag.", api_response.status());
        "nightly-2026-07-09".to_string()
    };

    let download_url = format!(
        "https://github.com/anoochit/nami-core/releases/download/{}/{}",
        tag, asset_name
    );

    println!("📢 Target release tag: {}", tag);
    println!("🔗 Download URL: {}", download_url);

    let proceed = Confirm::new(&format!("Do you want to upgrade Nami to nightly snapshot '{}'?", tag))
        .with_default(true)
        .prompt()?;

    if !proceed {
        println!("❌ Upgrade canceled by user.");
        return Ok(());
    }

    println!("📥 Downloading upgrade binary...");

    let response = client.get(&download_url).send().await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to download binary: HTTP {}. Please check if the release asset exists.",
            response.status()
        );
    }

    let bytes = response.bytes().await?;
    println!("💾 Downloaded {} bytes successfully.", bytes.len());

    let current_exe_path = env::current_exe()?;
    println!("📍 Current executable path: {}", current_exe_path.display());

    let temp_exe_path = current_exe_path.with_extension("tmp-upgrade");
    println!("🔧 Writing temporary binary to: {}", temp_exe_path.display());

    {
        let mut temp_file = File::create(&temp_exe_path)?;
        temp_file.write_all(&bytes)?;
        temp_file.flush()?;
    }

    // Set executable permissions on Unix platforms
    #[cfg(unix)]
    {
        println!("🔑 Setting executable permissions...");
        let mut perms = std::fs::metadata(&temp_exe_path)?.permissions();
        perms.set_mode(0o755);
        set_permissions(&temp_exe_path, perms)?;
    }

    println!("🔄 Replacing active executable...");
    rename(&temp_exe_path, &current_exe_path)?;

    println!("🎉 Nami has been successfully upgraded to nightly snapshot {}!", tag);

    Ok(())
}

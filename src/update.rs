// src/update.rs
//
// Self-update via GitHub Releases (no API key required for public repos).

use serde::Deserialize;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::process::Command;

const DEFAULT_REPO: &str = "";

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub fn run(repo_override: Option<String>, check_only: bool) -> Result<(), String> {
    let repo = resolve_repo(repo_override)?;

    let release = fetch_latest_release(&repo)?;
    if is_current_version(&release.tag_name) {
        println!("Already up to date ({})", release.tag_name);
        return Ok(());
    }

    if check_only {
        let current = env!("CARGO_PKG_VERSION");
        println!("Update available: {} (current {})", release.tag_name, current);
        return Ok(());
    }

    let asset_name = current_asset_name()?;
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| {
            format!(
                "No matching asset '{}' in latest release {}",
                asset_name, release.tag_name
            )
        })?;

    let exe_path = env::current_exe().map_err(|e| e.to_string())?;
    let tmp_path = temp_path_next_to(&exe_path)?;

    download_to(&asset.browser_download_url, &tmp_path)?;
    make_executable(&tmp_path)?;

    let deferred = replace_current_exe(&exe_path, &tmp_path)?;
    if deferred {
        println!("Update downloaded. Relaunch A to use {}", release.tag_name);
    } else {
        println!("Updated to {}", release.tag_name);
    }
    Ok(())
}

fn resolve_repo(repo_override: Option<String>) -> Result<String, String> {
    if let Some(r) = repo_override {
        if !r.trim().is_empty() {
            return Ok(r);
        }
    }
    if let Ok(r) = env::var("A_UPDATE_REPO") {
        if !r.trim().is_empty() {
            return Ok(r);
        }
    }
    if !DEFAULT_REPO.trim().is_empty() {
        return Ok(DEFAULT_REPO.to_string());
    }
    Err("No repo configured. Set A_UPDATE_REPO or pass --repo owner/name.".to_string())
}

fn current_asset_name() -> Result<String, String> {
    let os = match env::consts::OS {
        "windows" => "windows",
        "macos" => "macos",
        "linux" => "linux",
        other => return Err(format!("Unsupported OS: {}", other)),
    };

    let arch = match env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        other => return Err(format!("Unsupported architecture: {}", other)),
    };

    let mut name = format!("a-{}-{}", os, arch);
    if os == "windows" {
        name.push_str(".exe");
    }
    Ok(name)
}

fn fetch_latest_release(repo: &str) -> Result<Release, String> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    let resp = ureq::get(&url)
        .set("User-Agent", "a-updater")
        .call()
        .map_err(ureq_err)?;

    let mut body = String::new();
    resp.into_reader()
        .read_to_string(&mut body)
        .map_err(|e| e.to_string())?;

    serde_json::from_str::<Release>(&body).map_err(|e| e.to_string())
}

fn download_to(url: &str, path: &Path) -> Result<(), String> {
    let resp = ureq::get(url)
        .set("User-Agent", "a-updater")
        .call()
        .map_err(ureq_err)?;

    let mut reader = resp.into_reader();
    let mut file = fs::File::create(path).map_err(|e| e.to_string())?;
    io::copy(&mut reader, &mut file).map_err(|e| e.to_string())?;
    file.flush().map_err(|e| e.to_string())?;
    Ok(())
}

fn is_current_version(tag: &str) -> bool {
    let current = env!("CARGO_PKG_VERSION");
    let clean = tag.trim_start_matches('v');
    clean == current
}

fn temp_path_next_to(exe_path: &Path) -> Result<PathBuf, String> {
    let parent = exe_path
        .parent()
        .ok_or_else(|| "Could not determine executable directory".to_string())?;
    let file_name = exe_path
        .file_name()
        .ok_or_else(|| "Could not determine executable name".to_string())?
        .to_string_lossy()
        .to_string();

    let tmp_name = format!("{}.new", file_name);
    Ok(parent.join(tmp_name))
}

fn make_executable(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn replace_current_exe(exe_path: &Path, tmp_path: &Path) -> Result<bool, String> {
    #[cfg(windows)]
    {
        let exe = exe_path.to_string_lossy();
        let tmp = tmp_path.to_string_lossy();

        // Replace after this process exits.
        let cmd = format!(
            "ping 127.0.0.1 -n 2 >NUL & move /Y \"{}\" \"{}\"",
            tmp, exe
        );
        Command::new("cmd")
            .args(["/C", &cmd])
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(true);
    }

    #[cfg(not(windows))]
    {
        fs::rename(tmp_path, exe_path).map_err(|e| e.to_string())?;
        Ok(false)
    }
}

fn ureq_err(e: ureq::Error) -> String {
    match e {
        ureq::Error::Status(code, resp) => {
            let status = resp.status_text().to_string();
            let body = resp.into_string().unwrap_or_default();
            format!("HTTP {} {}: {}", code, status, body)
        }
        ureq::Error::Transport(t) => t.to_string(),
    }
}

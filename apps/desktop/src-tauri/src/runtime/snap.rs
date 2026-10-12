use crate::domain::{
    discovery::{
        ApplicationSource, DiscoveredSnapPackage, DiscoveryStatus, MatchConfidence,
        ProviderDiscoveryResult,
    },
    error::RuntimeError,
    provider::ProviderDefinition,
};
use std::{
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::Duration,
};

/// Check whether the `snap` CLI executable exists on the system.
pub fn is_snap_available() -> bool {
    if Path::new("/usr/bin/snap").exists()
        || Path::new("/bin/snap").exists()
        || Path::new("/snap/bin/snap").exists()
    {
        return true;
    }

    // Check system PATH defensively
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            if Path::new(dir).join("snap").exists() {
                return true;
            }
        }
    }

    false
}

/// Find the exact path to the `snap` executable.
pub fn get_snap_executable_path() -> Option<String> {
    if Path::new("/usr/bin/snap").exists() {
        return Some("/usr/bin/snap".to_string());
    }
    if Path::new("/bin/snap").exists() {
        return Some("/bin/snap".to_string());
    }
    if Path::new("/snap/bin/snap").exists() {
        return Some("/snap/bin/snap".to_string());
    }
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            let p = Path::new(dir).join("snap");
            if p.exists() {
                return Some(p.to_string_lossy().to_string());
            }
        }
    }
    None
}

/// Parse raw `snap list` output into strongly typed `DiscoveredSnapPackage` records.
/// Pure parsing function for testability with fixtures without needing Snap installed.
pub fn parse_snap_list_output(output: &str) -> Result<Vec<DiscoveredSnapPackage>, RuntimeError> {
    let mut packages = Vec::new();
    let lines: Vec<&str> = output.lines().collect();

    if lines.is_empty() {
        return Ok(packages);
    }

    let mut header_found = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Header detection: starts with "Name"
        if !header_found {
            if trimmed.starts_with("Name") {
                header_found = true;
                continue;
            } else if trimmed.starts_with("No snaps are installed") {
                return Ok(packages);
            }
        }

        // Split columns by whitespace
        let cols: Vec<&str> = trimmed.split_whitespace().collect();

        // Valid snap list data row requires at least Name, Version, Rev, Tracking
        if cols.len() < 4 {
            // Skip malformed lines gracefully to preserve resilience
            continue;
        }

        let name = cols[0].to_string();
        let version = cols[1].to_string();
        let revision = cols[2].to_string();
        let tracking = cols[3].to_string();

        let publisher = if cols.len() > 4 && cols[4] != "-" {
            // Strip any trailing publisher verification asterisks for cleaner presentation
            Some(cols[4].trim_end_matches('*').to_string())
        } else {
            None
        };

        let notes = if cols.len() > 5 && cols[5] != "-" {
            Some(cols[5..].join(" "))
        } else {
            None
        };

        packages.push(DiscoveredSnapPackage {
            name,
            version,
            revision,
            tracking,
            publisher,
            notes,
        });
    }

    Ok(packages)
}

/// Deterministically match an AI-Hub provider definition against discovered Snap packages.
/// Distinguishes Confirmed match vs Ambiguous match vs No match.
/// Guards against generic words (`ai`, `chat`, `assistant`, `desktop`).
pub fn match_provider_against_snaps(
    provider: &ProviderDefinition,
    packages: &[DiscoveredSnapPackage],
) -> ProviderDiscoveryResult {
    let provider_slug = provider.id.as_str().to_lowercase();

    // 1. Check for Confirmed match against candidate snap names
    for pkg in packages {
        let pkg_lower = pkg.name.to_lowercase();

        // Check configured candidate snap names
        let is_candidate = provider
            .candidate_snap_names
            .iter()
            .any(|cand| cand.eq_ignore_ascii_case(&pkg.name));

        // Exact match with provider slug or standard desktop pattern
        let is_exact = pkg_lower == provider_slug
            || pkg_lower == format!("{provider_slug}-desktop")
            || pkg_lower == format!("{provider_slug}desktop");

        if is_candidate || is_exact {
            let exe_path = format!("/snap/bin/{}", pkg.name);
            return ProviderDiscoveryResult {
                provider_id: provider.id,
                status: DiscoveryStatus::Confirmed,
                match_confidence: MatchConfidence::Confirmed,
                source: Some(ApplicationSource::Snap),
                matched_package: Some(pkg.name.clone()),
                version: Some(pkg.version.clone()),
                executable_path: Some(exe_path),
                details: Some(format!(
                    "Discovered installed Snap package '{}' (rev {})",
                    pkg.name, pkg.revision
                )),
            };
        }
    }

    // 2. Check for Ambiguous match: contains provider slug but with unexpected affixes
    // Strict safeguard: provider slug must be at least 4 characters to avoid substring false positives
    if provider_slug.len() >= 4 {
        for pkg in packages {
            let pkg_lower = pkg.name.to_lowercase();

            // Ignore generic keywords that should never cause a match
            if is_generic_keyword(&pkg_lower) {
                continue;
            }

            // If package contains the provider slug (e.g. "chatgpt-wrapper" or "gemini-cli")
            if pkg_lower.contains(&provider_slug) {
                return ProviderDiscoveryResult {
                    provider_id: provider.id,
                    status: DiscoveryStatus::Ambiguous,
                    match_confidence: MatchConfidence::Ambiguous,
                    source: Some(ApplicationSource::Snap),
                    matched_package: Some(pkg.name.clone()),
                    version: Some(pkg.version.clone()),
                    executable_path: None,
                    details: Some(format!(
                        "Ambiguous package '{}' contains provider keyword '{}' but is not a verified candidate",
                        pkg.name, provider.name
                    )),
                };
            }
        }
    }

    // 3. No match found
    ProviderDiscoveryResult {
        provider_id: provider.id,
        status: DiscoveryStatus::NotInstalled,
        match_confidence: MatchConfidence::None,
        source: None,
        matched_package: None,
        version: None,
        executable_path: None,
        details: Some(format!(
            "No installed Snap package found matching {}",
            provider.name
        )),
    }
}

/// Generic words that must never trigger ambiguous matches.
fn is_generic_keyword(name: &str) -> bool {
    const GENERIC_WORDS: &[&str] = &[
        "ai",
        "chat",
        "bot",
        "assistant",
        "desktop",
        "hub",
        "client",
        "tool",
        "core",
        "base",
        "gnome",
        "gtk",
        "mesa",
    ];

    GENERIC_WORDS.iter().any(|&w| name == w)
}

/// Query installed Snap metadata via the bounded, non-interactive `snap list` command.
pub fn query_installed_snaps(
    timeout: Duration,
) -> Result<Vec<DiscoveredSnapPackage>, RuntimeError> {
    let snap_bin = get_snap_executable_path().ok_or_else(|| {
        RuntimeError::DiscoveryUnavailable(
            "Snap command-line tooling is not installed on this system".to_string(),
        )
    })?;

    let (sender, receiver) = mpsc::channel();

    let snap_bin_clone = snap_bin.clone();
    let worker = thread::spawn(move || {
        let result = Command::new(&snap_bin_clone)
            .args(["list", "--color=never", "--unicode=never"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        let _ = sender.send(result);
    });

    match receiver.recv_timeout(timeout) {
        Ok(cmd_result) => {
            let output = cmd_result.map_err(|e| {
                RuntimeError::DiscoveryFailed(format!("Failed to execute '{snap_bin} list': {e}"))
            })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let code = output.status.code().unwrap_or(-1);
                return Err(RuntimeError::DiscoveryFailed(format!(
                    "Snap list command exited with status code {code}: {stderr}"
                )));
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            parse_snap_list_output(&stdout)
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // In a low-memory / constrained environment, command hung or timed out
            drop(worker);
            Err(RuntimeError::DiscoveryTimeout(format!(
                "Timed out waiting for '{snap_bin} list' after {} seconds",
                timeout.as_secs()
            )))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(RuntimeError::DiscoveryFailed(
            "Discovery worker thread unexpectedly terminated".to_string(),
        )),
    }
}

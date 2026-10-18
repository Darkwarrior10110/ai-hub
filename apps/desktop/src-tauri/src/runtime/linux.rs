use super::{
    memory::parse_meminfo,
    snap::{is_snap_available, match_provider_against_snaps, query_installed_snaps},
    traits::RuntimePlatform,
};
use crate::domain::{
    discovery::{
        DiscoverySnapshot, DiscoveryStatus, InstalledApplication, MatchConfidence,
        ProviderDiscoveryResult,
    },
    error::RuntimeError,
    launch::{LaunchResult, LaunchStatus},
    lifecycle::{ProcessIdentity, ProcessOwnership},
    provider::{LaunchType, ProviderDefinition},
    ResourceSnapshot,
};
use std::{
    path::Path,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProcessInspection {
    pub pid: u32,
    pub exists: bool,
    pub uid: Option<u32>,
    pub start_time_ticks: Option<u64>,
    pub comm: Option<String>,
    pub cmdline: Option<String>,
    pub exe_path: Option<String>,
    pub state: Option<char>,
    pub children: Vec<u32>,
}

pub fn parse_stat_starttime(stat_content: &str) -> Option<u64> {
    let rparen_idx = stat_content.rfind(')')?;
    let remainder = stat_content.get(rparen_idx + 1..)?.trim_start();
    // remainder tokens start at field 3 (state)
    // token 19 is field 22 (starttime)
    let tokens: Vec<&str> = remainder.split_whitespace().collect();
    if tokens.len() > 19 {
        tokens[19].parse::<u64>().ok()
    } else {
        None
    }
}

pub fn parse_stat_state(stat_content: &str) -> Option<char> {
    let rparen_idx = stat_content.rfind(')')?;
    let remainder = stat_content.get(rparen_idx + 1..)?.trim_start();
    remainder.chars().next()
}

pub trait ProcessLauncher: Send + Sync + std::fmt::Debug {
    fn spawn(&self, executable: &str, args: &[String]) -> Result<u32, RuntimeError>;
    fn is_running(&self, pid: u32) -> bool {
        if pid == 0 || pid > 4_194_304 {
            return false;
        }

        #[cfg(unix)]
        {
            let mut status: libc::c_int = 0;
            let res = unsafe { libc::waitpid(pid as libc::pid_t, &mut status, libc::WNOHANG) };
            if res == pid as libc::pid_t {
                return false;
            }
        }

        let proc_path = format!("/proc/{pid}");
        if !Path::new(&proc_path).exists() {
            return false;
        }

        if let Ok(stat) = std::fs::read_to_string(format!("{proc_path}/stat")) {
            if let Some(state) = parse_stat_state(&stat) {
                if state == 'Z' || state == 'X' {
                    return false;
                }
            }
        }

        true
    }
    fn inspect_process(&self, pid: u32) -> Result<ProcessInspection, RuntimeError>;
    fn send_signal(&self, pid: u32, signal: i32) -> Result<(), RuntimeError>;
    fn get_current_uid(&self) -> u32;
    fn read_meminfo(&self) -> Result<String, RuntimeError>;
}

#[derive(Debug, Default)]
pub struct OsProcessLauncher;

impl ProcessLauncher for OsProcessLauncher {
    fn spawn(&self, executable: &str, args: &[String]) -> Result<u32, RuntimeError> {
        let mut cmd = std::process::Command::new(executable);
        cmd.args(args);
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }

        let child = cmd.spawn().map_err(|e| {
            RuntimeError::LaunchFailed(format!("Failed to spawn process '{executable}': {e}"))
        })?;

        Ok(child.id())
    }

    fn is_running(&self, pid: u32) -> bool {
        if pid == 0 || pid > 4_194_304 {
            return false;
        }

        #[cfg(unix)]
        {
            let mut status: libc::c_int = 0;
            let res = unsafe { libc::waitpid(pid as libc::pid_t, &mut status, libc::WNOHANG) };
            if res == pid as libc::pid_t {
                return false;
            }
        }

        let proc_path = format!("/proc/{pid}");
        if !Path::new(&proc_path).exists() {
            return false;
        }

        if let Ok(stat) = std::fs::read_to_string(format!("{proc_path}/stat")) {
            if let Some(state) = parse_stat_state(&stat) {
                if state == 'Z' || state == 'X' {
                    return false;
                }
            }
        }

        true
    }

    fn inspect_process(&self, pid: u32) -> Result<ProcessInspection, RuntimeError> {
        let proc_path = format!("/proc/{pid}");
        if !Path::new(&proc_path).exists() {
            return Ok(ProcessInspection {
                pid,
                exists: false,
                ..Default::default()
            });
        }

        #[cfg(unix)]
        let uid = std::fs::metadata(&proc_path)
            .map(|m| std::os::unix::fs::MetadataExt::uid(&m))
            .ok();
        #[cfg(not(unix))]
        let uid = None;

        let comm = std::fs::read_to_string(format!("{proc_path}/comm"))
            .map(|s| s.trim().to_string())
            .ok();

        let cmdline = std::fs::read(format!("{proc_path}/cmdline"))
            .map(|bytes| {
                String::from_utf8_lossy(&bytes)
                    .replace('\0', " ")
                    .trim()
                    .to_string()
            })
            .ok();

        let exe_path = std::fs::read_link(format!("{proc_path}/exe"))
            .map(|p| p.to_string_lossy().to_string())
            .ok();

        let stat_content = std::fs::read_to_string(format!("{proc_path}/stat")).ok();
        let start_time_ticks = stat_content.as_deref().and_then(parse_stat_starttime);
        let state = stat_content.as_deref().and_then(parse_stat_state);

        let mut children = Vec::new();
        if let Ok(children_str) =
            std::fs::read_to_string(format!("{proc_path}/task/{pid}/children"))
        {
            for tok in children_str.split_whitespace() {
                if let Ok(child_pid) = tok.parse::<u32>() {
                    children.push(child_pid);
                }
            }
        }

        Ok(ProcessInspection {
            pid,
            exists: true,
            uid,
            start_time_ticks,
            comm,
            cmdline,
            exe_path,
            state,
            children,
        })
    }

    fn send_signal(&self, pid: u32, signal: i32) -> Result<(), RuntimeError> {
        if pid == 0 || pid > 4_194_304 {
            return Err(RuntimeError::Internal(format!(
                "Refusing to signal invalid process PID {pid}"
            )));
        }

        #[cfg(unix)]
        {
            let ret = unsafe { libc::kill(pid as libc::pid_t, signal) };
            if ret == 0 {
                // If the process is a leader of its own process group, deliver signal to group as well
                let pgrp = unsafe { libc::getpgid(pid as libc::pid_t) };
                let my_pgrp = unsafe { libc::getpgrp() };
                if pgrp == pid as libc::pid_t && pgrp != my_pgrp {
                    unsafe { libc::kill(-pgrp, signal) };
                }
                Ok(())
            } else {
                let err = std::io::Error::last_os_error();
                if err.raw_os_error() == Some(libc::ESRCH) {
                    // Process already terminated
                    Ok(())
                } else {
                    Err(RuntimeError::ShutdownFailed(format!(
                        "Failed to send signal {signal} to PID {pid}: {err}"
                    )))
                }
            }
        }
        #[cfg(not(unix))]
        {
            Err(RuntimeError::PlatformNotSupported(
                "Signal delivery is only supported on Unix systems".to_string(),
            ))
        }
    }

    fn get_current_uid(&self) -> u32 {
        #[cfg(unix)]
        {
            unsafe { libc::getuid() }
        }
        #[cfg(not(unix))]
        {
            0
        }
    }

    fn read_meminfo(&self) -> Result<String, RuntimeError> {
        std::fs::read_to_string("/proc/meminfo")
            .map_err(|e| RuntimeError::Internal(format!("Failed to read /proc/meminfo: {e}")))
    }
}

pub struct LinuxRuntime {
    launcher: Box<dyn ProcessLauncher>,
}

impl LinuxRuntime {
    pub fn new() -> Self {
        Self {
            launcher: Box::new(OsProcessLauncher),
        }
    }

    pub fn with_launcher(launcher: Box<dyn ProcessLauncher>) -> Self {
        Self { launcher }
    }
}

impl Default for LinuxRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for LinuxRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinuxRuntime")
            .field("launcher", &self.launcher)
            .finish()
    }
}

impl RuntimePlatform for LinuxRuntime {
    fn name(&self) -> &'static str {
        "linux"
    }

    fn discover_all_applications(
        &self,
        providers: &[ProviderDefinition],
    ) -> Result<DiscoverySnapshot, RuntimeError> {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        if !is_snap_available() {
            let results = providers
                .iter()
                .map(|p| ProviderDiscoveryResult {
                    provider_id: p.id,
                    status: DiscoveryStatus::Unavailable,
                    match_confidence: MatchConfidence::None,
                    source: None,
                    matched_package: None,
                    version: None,
                    executable_path: None,
                    details: Some(
                        "Snap command-line tooling is not installed or available on this system"
                            .to_string(),
                    ),
                })
                .collect();

            return Ok(DiscoverySnapshot {
                is_snap_available: false,
                scanned_at_ms: now_ms,
                total_packages_scanned: 0,
                results,
                diagnostics: Some("Snap tooling unavailable in PATH".to_string()),
            });
        }

        // Query Snap metadata with bounded timeout (15s to tolerate system load on constrained hosts)
        match query_installed_snaps(Duration::from_secs(15)) {
            Ok(packages) => {
                let results = providers
                    .iter()
                    .map(|p| match_provider_against_snaps(p, &packages))
                    .collect();

                Ok(DiscoverySnapshot {
                    is_snap_available: true,
                    scanned_at_ms: now_ms,
                    total_packages_scanned: packages.len(),
                    results,
                    diagnostics: None,
                })
            }
            Err(e) => {
                let error_desc = e.to_string();
                let results = providers
                    .iter()
                    .map(|p| ProviderDiscoveryResult {
                        provider_id: p.id,
                        status: DiscoveryStatus::Failed,
                        match_confidence: MatchConfidence::None,
                        source: None,
                        matched_package: None,
                        version: None,
                        executable_path: None,
                        details: Some(format!("Snap discovery query failed: {error_desc}")),
                    })
                    .collect();

                Ok(DiscoverySnapshot {
                    is_snap_available: true,
                    scanned_at_ms: now_ms,
                    total_packages_scanned: 0,
                    results,
                    diagnostics: Some(error_desc),
                })
            }
        }
    }

    fn detect_application(
        &self,
        provider: &ProviderDefinition,
    ) -> Result<Option<InstalledApplication>, RuntimeError> {
        // First, check Snap metadata if snap is available
        if is_snap_available() {
            if let Ok(packages) = query_installed_snaps(Duration::from_secs(15)) {
                let match_result = match_provider_against_snaps(provider, &packages);
                if match_result.status == DiscoveryStatus::Confirmed {
                    if let Some(pkg_name) = match_result.matched_package {
                        let exe_path = match_result
                            .executable_path
                            .unwrap_or_else(|| format!("/snap/bin/{pkg_name}"));

                        return Ok(Some(InstalledApplication {
                            provider_id: provider.id,
                            display_name: provider.name.clone(),
                            version: match_result.version,
                            launch_type: LaunchType::Snap,
                            executable_path: exe_path,
                            launch_args: Vec::new(),
                            icon_path: None,
                            package_id: Some(pkg_name),
                        }));
                    }
                } else if match_result.status == DiscoveryStatus::Ambiguous {
                    return Err(RuntimeError::AmbiguousMatch(format!(
                        "Ambiguous snap package match for {}: {:?}",
                        provider.name, match_result.matched_package
                    )));
                }
            }
        }

        // Fallback: check candidate executables directly (only absolute paths to prevent CWD hijacking)
        for exe in &provider.candidate_executables {
            let p = Path::new(exe);
            if p.is_absolute() && p.exists() {
                return Ok(Some(InstalledApplication {
                    provider_id: provider.id,
                    display_name: provider.name.clone(),
                    version: None,
                    launch_type: if exe.starts_with("/snap/") {
                        LaunchType::Snap
                    } else {
                        LaunchType::Executable
                    },
                    executable_path: exe.clone(),
                    launch_args: Vec::new(),
                    icon_path: None,
                    package_id: provider.candidate_snap_names.first().cloned(),
                }));
            }
        }

        Ok(None)
    }

    fn launch_application(&self, app: &InstalledApplication) -> Result<LaunchResult, RuntimeError> {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        // Security check: executable path must be absolute
        let exe_path = Path::new(&app.executable_path);
        if !exe_path.is_absolute() {
            return Err(RuntimeError::InvalidLaunchTarget(format!(
                "Executable path must be absolute: {}",
                app.executable_path
            )));
        }

        // Safety allowlist based on launch type
        match app.launch_type {
            LaunchType::Snap => {
                if !app.executable_path.starts_with("/snap/") {
                    return Err(RuntimeError::InvalidLaunchTarget(format!(
                        "Snap executable path must reside under /snap/: {}",
                        app.executable_path
                    )));
                }
            }
            LaunchType::Executable | LaunchType::AppImage | LaunchType::DesktopEntry => {
                // Ensure no shell command injection in executable path
                if app.executable_path.contains(';')
                    || app.executable_path.contains('|')
                    || app.executable_path.contains('&')
                    || app.executable_path.contains('`')
                    || app.executable_path.contains('$')
                {
                    return Err(RuntimeError::InvalidLaunchTarget(format!(
                        "Executable path contains invalid characters: {}",
                        app.executable_path
                    )));
                }
            }
            LaunchType::WindowsApp => {
                return Err(RuntimeError::PlatformNotSupported(
                    "Windows applications cannot be launched on Linux runtime".to_string(),
                ));
            }
        }

        if !exe_path.exists() {
            return Err(RuntimeError::LaunchFailed(format!(
                "Target executable not found on filesystem: {}",
                app.executable_path
            )));
        }

        // Spawn child process through launcher
        let pid = self
            .launcher
            .spawn(&app.executable_path, &app.launch_args)?;

        let inspection = self.launcher.inspect_process(pid).ok();
        let start_time_ticks = inspection.as_ref().and_then(|i| i.start_time_ticks);

        let proc_name = exe_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| app.executable_path.clone());

        let proc_identity = ProcessIdentity {
            pid,
            process_name: proc_name,
            start_time_ms: Some(now_ms),
            start_time_ticks,
            executable_path: Some(app.executable_path.clone()),
            ownership: ProcessOwnership::StartedByAIHub,
        };

        // Check whether process is alive
        let status = if self.launcher.is_running(pid) {
            LaunchStatus::Running
        } else {
            LaunchStatus::LaunchRequested
        };

        Ok(LaunchResult {
            provider_id: app.provider_id,
            status,
            process: Some(proc_identity),
            details: Some(format!(
                "Application successfully spawned via {} with PID {}",
                match app.launch_type {
                    LaunchType::Snap => "Snap",
                    LaunchType::Executable => "Native Executable",
                    _ => "OS Launcher",
                },
                pid
            )),
            launched_at_ms: now_ms,
        })
    }

    fn is_process_running(&self, process: &ProcessIdentity) -> Result<bool, RuntimeError> {
        if !self.launcher.is_running(process.pid) {
            return Ok(false);
        }
        if let Some(recorded_ticks) = process.start_time_ticks {
            if let Ok(inspection) = self.launcher.inspect_process(process.pid) {
                if inspection.start_time_ticks != Some(recorded_ticks) {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    fn stop_process(
        &self,
        process: &ProcessIdentity,
        graceful_timeout_secs: u32,
        _allow_force: bool,
    ) -> Result<bool, RuntimeError> {
        if process.ownership != ProcessOwnership::StartedByAIHub {
            return Err(RuntimeError::ProcessOwnershipUnknown(format!(
                "Cannot stop process PID {}: ownership is not StartedByAIHub",
                process.pid
            )));
        }

        if !self.launcher.is_running(process.pid) {
            return Ok(true);
        }

        // Phase 1: Deliver SIGTERM (signal 15) for standard graceful shutdown
        self.launcher.send_signal(process.pid, 15)?;

        let start = Instant::now();
        // Allocate up to 75% of graceful timeout for Phase 1 (SIGTERM), reserving at least 1.5s
        let total_secs = graceful_timeout_secs.max(3) as u64;
        let phase1_ms = ((total_secs * 1000) * 3 / 4).max(1500);
        let phase1_timeout = Duration::from_millis(phase1_ms);

        while start.elapsed() < phase1_timeout {
            if !self.launcher.is_running(process.pid) {
                return Ok(true);
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        // Phase 2: If still running, the application (e.g. Electron apps that intercept window close
        // with preventDefault to minimize to tray) did not exit on SIGTERM.
        // Deliver SIGQUIT (signal 3) for graceful quit without SIGKILL.
        if self.launcher.is_running(process.pid) {
            let _ = self.launcher.send_signal(process.pid, 3);
        }

        let total_timeout = Duration::from_secs(total_secs);
        while start.elapsed() < total_timeout {
            if !self.launcher.is_running(process.pid) {
                return Ok(true);
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        if !self.launcher.is_running(process.pid) {
            Ok(true)
        } else {
            Err(RuntimeError::ShutdownTimeout(format!(
                "Process PID {} did not terminate within {} seconds after graceful termination signals",
                process.pid, graceful_timeout_secs
            )))
        }
    }

    fn validate_process(
        &self,
        process: &ProcessIdentity,
        provider: &ProviderDefinition,
    ) -> Result<bool, RuntimeError> {
        if !self.launcher.is_running(process.pid) {
            return Ok(false);
        }

        let inspection = self.launcher.inspect_process(process.pid)?;
        if !inspection.exists {
            return Ok(false);
        }

        // 1. UID verification: must match current user UID
        let current_uid = self.launcher.get_current_uid();
        if inspection.uid != Some(current_uid) {
            return Err(RuntimeError::ProcessVerificationFailed(format!(
                "Process PID {} has UID {:?}, which does not match current user UID {}",
                process.pid, inspection.uid, current_uid
            )));
        }

        // 2. Start time verification: detect PID recycling
        if let Some(expected_ticks) = process.start_time_ticks {
            if inspection.start_time_ticks != Some(expected_ticks) {
                return Err(RuntimeError::ProcessVerificationFailed(format!(
                    "Process PID {} start time ticks ({:?}) do not match recorded ticks ({}) - PID reuse detected",
                    process.pid, inspection.start_time_ticks, expected_ticks
                )));
            }
        }

        // 3. Provider identity matching
        let matches =
            is_process_matching_provider(&inspection, provider, process.executable_path.as_deref());

        if !matches {
            return Err(RuntimeError::ProcessVerificationFailed(format!(
                "Process PID {} (comm: {:?}, exe: {:?}) does not match provider definition {}",
                process.pid,
                inspection.comm,
                inspection.exe_path,
                provider.id.as_str()
            )));
        }

        Ok(true)
    }

    fn get_system_resources(&self) -> Result<ResourceSnapshot, RuntimeError> {
        let meminfo_text = self.launcher.read_meminfo()?;
        let sys_mem = parse_meminfo(&meminfo_text)?;

        let total_mb = sys_mem.total_bytes / (1024 * 1024);
        let available_mb = sys_mem.available_bytes / (1024 * 1024);
        let _used_mb = sys_mem.used_bytes / (1024 * 1024);
        let used_pct = if sys_mem.total_bytes > 0 {
            (sys_mem.used_bytes as f64 / sys_mem.total_bytes as f64) * 100.0
        } else {
            0.0
        };

        let (warning, message) = if available_mb < 512 || used_pct >= 85.0 {
            (
                true,
                Some(format!(
                    "Resource Saver advisory: memory pressure detected ({:.1}% used, {} MB available of {} MB total)",
                    used_pct, available_mb, total_mb
                )),
            )
        } else {
            (false, None)
        };

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Ok(ResourceSnapshot {
            is_available: true,
            diagnostics: Some("System memory parsed from /proc/meminfo".to_string()),
            system_memory: Some(sys_mem),
            memory_pressure_warning: warning,
            memory_pressure_message: message,
            timestamp_ms: now_ms,
            ..Default::default()
        })
    }
}

pub fn is_process_matching_provider(
    inspection: &ProcessInspection,
    provider: &ProviderDefinition,
    recorded_exe_path: Option<&str>,
) -> bool {
    let mut candidates = Vec::new();
    for snap in &provider.candidate_snap_names {
        candidates.push(snap.to_lowercase());
    }
    for exe in &provider.candidate_executables {
        candidates.push(exe.to_lowercase());
    }
    candidates.push(provider.id.as_str().to_lowercase());

    // 1. Check comm
    if let Some(comm) = &inspection.comm {
        let comm_lower = comm.to_lowercase();
        for cand in &candidates {
            if comm_lower == *cand || (comm_lower.len() == 15 && cand.starts_with(&comm_lower)) {
                return true;
            }
        }
    }

    // 2. Check exe_path
    if let Some(exe_str) = &inspection.exe_path {
        let exe_lower = exe_str.to_lowercase();
        let exe_path = Path::new(exe_str);
        if let Some(file_name) = exe_path.file_name().and_then(|f| f.to_str()) {
            let file_lower = file_name.to_lowercase();
            for cand in &candidates {
                if file_lower == *cand {
                    return true;
                }
            }
        }
        for snap in &provider.candidate_snap_names {
            let snap_marker = format!("/snap/{}/", snap.to_lowercase());
            let snap_bin_marker = format!("/snap/bin/{}", snap.to_lowercase());
            if exe_lower.contains(&snap_marker) || exe_lower.contains(&snap_bin_marker) {
                return true;
            }
        }
        if let Some(recorded) = recorded_exe_path {
            if exe_str == recorded {
                return true;
            }
        }
    }

    // 3. Check cmdline
    if let Some(cmdline) = &inspection.cmdline {
        let cmdline_lower = cmdline.to_lowercase();
        for snap in &provider.candidate_snap_names {
            let snap_marker = format!("/snap/{}/", snap.to_lowercase());
            let snap_bin_marker = format!("/snap/bin/{}", snap.to_lowercase());
            if cmdline_lower.contains(&snap_marker) || cmdline_lower.contains(&snap_bin_marker) {
                return true;
            }
        }
        for cand in &candidates {
            let tokens = cmdline_lower.split_whitespace();
            for tok in tokens {
                let tok_path = Path::new(tok);
                if let Some(name) = tok_path.file_name().and_then(|n| n.to_str()) {
                    if name == cand {
                        return true;
                    }
                }
            }
        }
    }

    false
}

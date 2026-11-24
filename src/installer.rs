use crate::builder::{build_payload, BuildOptions, DEFAULT_MAX_DIM};
use crate::cdp::{cleanup_open_pages, fetch_targets, filter_antigravity_pages};
use crate::paths::{read_port_info, resolve_daemon_exe, resolve_port_file, AppPaths};
use crate::sys::{is_pid_alive, process_memory_stats, spawn_detached, terminate_pid};
use crate::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

pub const MACOS_SERVICE_LABEL: &str = "io.antigravity.custom-theme";
pub const MACOS_LAUNCHER_APP_NAME: &str = "Antigravity Theme.app";
pub const MACOS_LAUNCHER_BUNDLE_ID: &str = "io.antigravity.custom-theme.launcher";
#[cfg(all(unix, not(target_os = "macos")))]
const SYSTEMD_UNIT: &str = "antigravity-custom-theme.service";

pub fn xml_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}

pub fn sh_single_quote(input: &str) -> String {
    format!("'{}'", input.replace('\'', "'\\''"))
}

pub fn render_macos_plist(
    daemon_exe: &Path,
    root: &Path,
    log_file: &Path,
    watch_themes: bool,
) -> String {
    let extra_arg = if watch_themes {
        "        <string>--watch-themes</string>\n"
    } else {
        ""
    };
    let daemon_xml = xml_escape(&daemon_exe.to_string_lossy());
    let root_xml = xml_escape(&root.to_string_lossy());
    let log_xml = xml_escape(&log_file.to_string_lossy());
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{MACOS_SERVICE_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{daemon_xml}</string>
        <string>--root</string>
        <string>{root_xml}</string>
{extra_arg}    </array>
    <key>WorkingDirectory</key>
    <string>{root_xml}</string>
    <key>ProcessType</key>
    <string>Background</string>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{log_xml}</string>
    <key>StandardErrorPath</key>
    <string>{log_xml}</string>
</dict>
</plist>
"#
    )
}

pub fn render_macos_launcher_info_plist(icon_name: Option<&str>) -> String {
    let icon_block = match icon_name {
        Some(name) if !name.trim().is_empty() => format!(
            "    <key>CFBundleIconFile</key>\n    <string>{}</string>\n",
            xml_escape(name.trim())
        ),
        _ => String::new(),
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDisplayName</key>
    <string>Antigravity Theme</string>
    <key>CFBundleExecutable</key>
    <string>antigravity-theme-launcher</string>
    <key>CFBundleIdentifier</key>
    <string>{MACOS_LAUNCHER_BUNDLE_ID}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Antigravity Theme</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>LSUIElement</key>
    <true/>
{icon_block}</dict>
</plist>
"#
    )
}

pub fn render_macos_launcher_script(cli_exe: &Path, root: &Path) -> String {
    let cli_q = sh_single_quote(&cli_exe.to_string_lossy());
    let root_q = sh_single_quote(&root.to_string_lossy());
    format!(
        r#"#!/usr/bin/env bash
set -euo pipefail

CLI_BIN={cli_q}
REPO_ROOT={root_q}
PID_FILE="$REPO_ROOT/daemon.pid"

daemon_alive=0
if [ -f "$PID_FILE" ]; then
    pid="$(tr -d '[:space:]' < "$PID_FILE" || true)"
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
        daemon_alive=1
    fi
fi

if [ "$daemon_alive" -eq 0 ] && [ -x "$CLI_BIN" ]; then
    "$CLI_BIN" --root "$REPO_ROOT" start >/dev/null 2>&1 || true
fi

APP_PATH=""
for candidate in "/Applications/Antigravity.app" "$HOME/Applications/Antigravity.app"; do
    if [ -d "$candidate" ]; then
        APP_PATH="$candidate"
        break
    fi
done

if [ -z "$APP_PATH" ]; then
    osascript -e 'display alert "Antigravity.app not found" message "Install Antigravity in /Applications/Antigravity.app and try again." as warning' >/dev/null 2>&1 || true
    exit 1
fi

exec open -a "$APP_PATH" --args --remote-debugging-port=0 "$@"
"#
    )
}

pub fn write_macos_launcher_app(
    apps_dir: &Path,
    cli_exe: &Path,
    root: &Path,
    icon_source: Option<&Path>,
) -> Result<PathBuf> {
    let app_dir = apps_dir.join(MACOS_LAUNCHER_APP_NAME);
    let macos_dir = app_dir.join("Contents").join("MacOS");
    let resources_dir = app_dir.join("Contents").join("Resources");
    fs::create_dir_all(&macos_dir)?;
    fs::create_dir_all(&resources_dir)?;

    let mut icon_name: Option<&str> = None;
    if let Some(src) = icon_source {
        if src.is_file() {
            let dst = resources_dir.join("icon.icns");
            if fs::copy(src, &dst).is_ok() {
                icon_name = Some("icon.icns");
            }
        }
    }

    let info_plist = render_macos_launcher_info_plist(icon_name);
    fs::write(app_dir.join("Contents").join("Info.plist"), info_plist)?;

    let launcher_path = macos_dir.join("antigravity-theme-launcher");
    let script = render_macos_launcher_script(cli_exe, root);
    fs::write(&launcher_path, script)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&launcher_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&launcher_path, perms)?;
    }

    Ok(app_dir)
}

pub fn remove_macos_launcher_app(apps_dir: &Path) -> Result<bool> {
    let app_dir = apps_dir.join(MACOS_LAUNCHER_APP_NAME);
    if app_dir.exists() {
        fs::remove_dir_all(&app_dir)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn read_running_pid(paths: &AppPaths) -> Option<u32> {
    let raw = fs::read_to_string(&paths.pid_file).ok()?;
    let pid: u32 = raw.trim().parse().ok()?;
    if pid > 0 && is_pid_alive(pid) {
        Some(pid)
    } else {
        None
    }
}

pub fn stop_daemon(paths: &AppPaths) -> Result<Option<u32>> {
    if !paths.pid_file.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&paths.pid_file).unwrap_or_default();
    let pid: u32 = raw.trim().parse().unwrap_or(0);
    let stopped = if pid > 0 && is_pid_alive(pid) {
        terminate_pid(pid);
        for _ in 0..10 {
            thread::sleep(Duration::from_millis(80));
            if !is_pid_alive(pid) {
                break;
            }
        }
        Some(pid)
    } else {
        None
    };
    let _ = fs::remove_file(&paths.pid_file);
    Ok(stopped)
}

pub fn start_daemon(paths: &AppPaths, watch_themes: bool) -> Result<u32> {
    if !paths.payload_js.is_file() {
        println!("[agy-theme] dist/theme_payload.js missing; building payload...");
        let rep = build_payload(&BuildOptions {
            themes_path: paths.themes_json.clone(),
            engine_path: paths.engine_js.clone(),
            out_path: paths.payload_js.clone(),
            max_dim: DEFAULT_MAX_DIM,
        })?;
        println!(
            "[agy-theme] Built {} ({:.1} KB in {} ms).",
            rep.out_path.display(),
            rep.size_bytes as f64 / 1024.0,
            rep.elapsed_ms
        );
    }

    if let Some(old_pid) = stop_daemon(paths)? {
        println!("[agy-theme] Stopped previous daemon (PID {old_pid}).");
    }

    let daemon_exe = resolve_daemon_exe();
    let root_str = paths.root.to_string_lossy().to_string();
    let is_dedicated_daemon = daemon_exe
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case("agy-theme-daemon"))
        .unwrap_or(false);

    let mut args: Vec<&str> = Vec::new();
    if !is_dedicated_daemon {
        args.push("daemon");
    }
    args.push("--root");
    args.push(&root_str);
    if watch_themes {
        args.push("--watch-themes");
    }

    let _spawned_pid = spawn_detached(&daemon_exe, &args, &paths.root)?;

    for _ in 0..20 {
        thread::sleep(Duration::from_millis(150));
        if let Some(pid) = read_running_pid(paths) {
            return Ok(pid);
        }
    }

    Err(format!(
        "Daemon did not write daemon.pid in time. Check {}",
        paths.log_file.display()
    )
    .into())
}

pub fn print_status(paths: &AppPaths) {
    println!("Root:        {}", paths.root.display());
    println!(
        "Payload:     {} ({})",
        paths.payload_js.display(),
        if let Ok(m) = fs::metadata(&paths.payload_js) {
            format!("{:.1} KB", m.len() as f64 / 1024.0)
        } else {
            "not built".to_string()
        }
    );

    if let Some(pid) = read_running_pid(paths) {
        let mem_str = match process_memory_stats(pid) {
            Some(ms) => match ms.private_kb {
                Some(priv_kb) => format!(
                    "WorkingSet {:.2} MB | Private {:.2} MB",
                    ms.working_set_kb as f64 / 1024.0,
                    priv_kb as f64 / 1024.0
                ),
                None => format!("RSS {:.2} MB", ms.working_set_kb as f64 / 1024.0),
            },
            None => "RAM unavailable".to_string(),
        };
        println!("Daemon:      RUNNING (PID {pid} | {mem_str})");
    } else {
        println!("Daemon:      STOPPED");
    }

    match read_port_info() {
        Some(info) => {
            let pages = fetch_targets(info.port, Duration::from_millis(1200))
                .map(filter_antigravity_pages)
                .map(|v| v.len())
                .unwrap_or(0);
            println!(
                "Antigravity: CDP port {} active ({} matching page(s)) [{}]",
                info.port,
                pages,
                info.port_file.display()
            );
        }
        None => {
            println!(
                "Antigravity: DevToolsActivePort not found ({})",
                resolve_port_file().display()
            );
        }
    }
}

pub fn install(paths: &AppPaths, no_startup: bool, watch_themes: bool) -> Result<()> {
    if !paths.payload_js.is_file() {
        println!("[install] Building dist/theme_payload.js...");
        let rep = build_payload(&BuildOptions {
            themes_path: paths.themes_json.clone(),
            engine_path: paths.engine_js.clone(),
            out_path: paths.payload_js.clone(),
            max_dim: DEFAULT_MAX_DIM,
        })?;
        println!(
            "[install] Built {} ({:.1} KB in {} ms) with {} theme(s).",
            rep.out_path.display(),
            rep.size_bytes as f64 / 1024.0,
            rep.elapsed_ms,
            rep.theme_names.len()
        );
    }

    if let Some(old_pid) = stop_daemon(paths)? {
        println!("[install] Stopped existing daemon (PID {old_pid}).");
    }

    // Remove legacy VBS launcher if migrating from the Node.js version.
    if paths.legacy_vbs.is_file() {
        let _ = fs::remove_file(&paths.legacy_vbs);
    }

    #[cfg(windows)]
    {
        install_windows(paths, no_startup, watch_themes)?;
    }

    #[cfg(target_os = "macos")]
    {
        install_macos(paths, no_startup, watch_themes)?;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        install_linux(paths, no_startup, watch_themes)?;
    }

    Ok(())
}

pub fn uninstall(paths: &AppPaths) -> Result<()> {
    remove_autostart(paths);

    if let Some(pid) = stop_daemon(paths)? {
        println!("[uninstall] Stopped daemon PID {pid}.");
    } else {
        println!("[uninstall] No active daemon running.");
    }

    match cleanup_open_pages() {
        Ok(0) => println!("[uninstall] No open Antigravity CDP pages to clear."),
        Ok(n) => println!("[uninstall] Cleared theme from {n} open Antigravity page(s)."),
        Err(e) => println!("[uninstall] CDP cleanup skipped ({e})."),
    }

    println!("[uninstall] Done. Antigravity custom theme removed.");
    Ok(())
}

#[cfg(windows)]
fn ps_single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

#[cfg(windows)]
const WIN_TASK_NAME: &str = "AntigravityCustomTheme";

#[cfg(windows)]
fn install_windows(paths: &AppPaths, no_startup: bool, watch_themes: bool) -> Result<()> {
    use std::process::Command;

    let daemon_exe = resolve_daemon_exe();
    let root_str = paths.root.to_string_lossy();
    let daemon_str = daemon_exe.to_string_lossy();
    let mut daemon_args = format!("--root \"{}\"", root_str);
    if watch_themes {
        daemon_args.push_str(" --watch-themes");
    }

    // Patch Antigravity.lnk shortcuts with --remote-debugging-port=0, and register
    // Windows Task Scheduler service (with Startup .lnk fallback) so the daemon survives
    // closing & reopening Antigravity immediately without requiring a reboot.
    let setup_persistence_block = if no_startup {
        String::new()
    } else {
        format!(
            r#"
$TaskName = {task_q}
$StartupDir = [Environment]::GetFolderPath('Startup')
$ShortcutPath = Join-Path $StartupDir 'AntigravityCustomTheme.lnk'
$TaskRegistered = $false

try {{
    $Action = New-ScheduledTaskAction -Execute {daemon_q} -Argument {args_q} -WorkingDirectory {root_q}
    $Trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
    $Settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit ([TimeSpan]::Zero) -MultipleInstances IgnoreNew
    Register-ScheduledTask -TaskName $TaskName -Action $Action -Trigger $Trigger -Settings $Settings -Description 'Antigravity Custom Theme Rust Daemon' -Force -ErrorAction Stop | Out-Null
    Start-ScheduledTask -TaskName $TaskName -ErrorAction Stop
    $TaskRegistered = $true
    if (Test-Path $ShortcutPath) {{ Remove-Item $ShortcutPath -Force -ErrorAction SilentlyContinue }}
    Write-Host "[install] Windows Scheduled Task registered & started: $TaskName (independent of Antigravity lifecycle)"
}} catch {{
    Write-Host "[install] Task Scheduler unavailable ($($_.Exception.Message)); falling back to Startup shortcut."
    $Sc = $WshShell.CreateShortcut($ShortcutPath)
    $Sc.TargetPath = {daemon_q}
    $Sc.Arguments = {args_q}
    $Sc.WorkingDirectory = {root_q}
    $Sc.WindowStyle = 7
    $Sc.Description = 'Antigravity Custom Theme Rust Daemon'
    $Sc.Save()
    Write-Host "[install] Startup shortcut created (direct native Rust binary): $ShortcutPath"
}}
"#,
            task_q = ps_single_quote(WIN_TASK_NAME),
            daemon_q = ps_single_quote(&daemon_str),
            args_q = ps_single_quote(&daemon_args),
            root_q = ps_single_quote(&root_str),
        )
    };

    let ps_script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$WshShell = New-Object -ComObject WScript.Shell
$SearchDirs = @(
    [Environment]::GetFolderPath('DesktopDirectory'),
    [Environment]::GetFolderPath('Programs'),
    [Environment]::GetFolderPath('Startup'),
    (Join-Path $env:APPDATA 'Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar')
) | Where-Object {{ $_ -and (Test-Path $_) }}

foreach ($Dir in $SearchDirs) {{
    Get-ChildItem -Path $Dir -Recurse -Filter '*.lnk' -ErrorAction SilentlyContinue | ForEach-Object {{
        try {{
            $Sc = $WshShell.CreateShortcut($_.FullName)
            if ($Sc.TargetPath -and ($Sc.TargetPath -imatch '\\Antigravity\.exe$')) {{
                if ($Sc.Arguments -notmatch '--remote-debugging-port') {{
                    $Sc.Arguments = ("$($Sc.Arguments) --remote-debugging-port=0").Trim()
                    $Sc.Save()
                    Write-Host "[install] Patched CDP flag on shortcut: $($_.FullName)"
                }}
            }}
        }} catch {{}}
    }}
}}
{setup_persistence_block}
"#
    );

    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &ps_script,
        ])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        print!("{stdout}");
    }
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Windows shortcut/service setup failed: {stderr}").into());
    }

    // Wait briefly for Task Scheduler to spawn daemon.pid; otherwise spawn detached directly.
    let mut running_pid = None;
    if !no_startup {
        for _ in 0..12 {
            if let Some(pid) = read_running_pid(paths) {
                running_pid = Some(pid);
                break;
            }
            thread::sleep(Duration::from_millis(150));
        }
    }
    let pid = match running_pid {
        Some(p) => p,
        None => start_daemon(paths, watch_themes)?,
    };

    let mem_note = process_memory_stats(pid)
        .map(|m| format!(" (~{:.2} MB RAM)", m.working_set_kb as f64 / 1024.0))
        .unwrap_or_default();
    println!("[install] Rust daemon running silently (PID {pid}){mem_note}.");

    if read_port_info().is_none() {
        println!(
            "[install] Note: DevToolsActivePort not found yet. Quit Antigravity completely (System Tray -> Quit) and reopen from Start Menu (if using a Taskbar pin, unpin and re-pin from Start Menu)."
        );
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn current_macos_uid() -> u32 {
    extern "C" {
        fn getuid() -> u32;
    }
    unsafe { getuid() }
}

#[cfg(target_os = "macos")]
fn macos_home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(target_os = "macos")]
fn find_antigravity_app() -> Option<PathBuf> {
    if let Some(custom) = std::env::var_os("AGY_APP_PATH").filter(|v| !v.is_empty()) {
        let p = PathBuf::from(custom);
        if p.exists() {
            return Some(p);
        }
    }
    let system_app = PathBuf::from("/Applications/Antigravity.app");
    if system_app.is_dir() {
        return Some(system_app);
    }
    let user_app = macos_home_dir()
        .join("Applications")
        .join("Antigravity.app");
    if user_app.is_dir() {
        return Some(user_app);
    }
    None
}

#[cfg(target_os = "macos")]
fn bootout_macos_agent(plist_path: &Path) {
    use std::process::Command;
    let uid = current_macos_uid();
    let domain = format!("gui/{uid}");
    let service = format!("gui/{uid}/{MACOS_SERVICE_LABEL}");
    let _ = Command::new("launchctl")
        .args(["bootout", &service])
        .output();
    let _ = Command::new("launchctl")
        .args(["bootout", &domain, &plist_path.to_string_lossy()])
        .output();
    let _ = Command::new("launchctl")
        .args(["unload", &plist_path.to_string_lossy()])
        .output();
}

#[cfg(target_os = "macos")]
fn install_macos(paths: &AppPaths, no_startup: bool, watch_themes: bool) -> Result<()> {
    use crate::paths::resolve_cli_exe;
    use std::process::Command;

    let home = macos_home_dir();
    let apps_dir = home.join("Applications");
    let cli_exe = resolve_cli_exe();
    let icon_source = find_antigravity_app()
        .map(|app| app.join("Contents").join("Resources").join("icon.icns"));
    let launcher_app =
        write_macos_launcher_app(&apps_dir, &cli_exe, &paths.root, icon_source.as_deref())?;
    println!("[install] macOS CDP launcher ready: {}", launcher_app.display());

    let pid = if no_startup {
        start_daemon(paths, watch_themes)?
    } else {
        let daemon_exe = resolve_daemon_exe();
        let plist_dir = home.join("Library").join("LaunchAgents");
        fs::create_dir_all(&plist_dir)?;
        let plist_path = plist_dir.join(format!("{MACOS_SERVICE_LABEL}.plist"));

        let plist = render_macos_plist(&daemon_exe, &paths.root, &paths.log_file, watch_themes);
        fs::write(&plist_path, plist)?;

        bootout_macos_agent(&plist_path);
        let uid = current_macos_uid();
        let domain = format!("gui/{uid}");
        let service = format!("gui/{uid}/{MACOS_SERVICE_LABEL}");

        let bootstrap = Command::new("launchctl")
            .args(["bootstrap", &domain, &plist_path.to_string_lossy()])
            .output();
        let loaded = match bootstrap {
            Ok(ref o) if o.status.success() => true,
            _ => Command::new("launchctl")
                .args(["load", "-w", &plist_path.to_string_lossy()])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false),
        };
        if loaded {
            let _ = Command::new("launchctl")
                .args(["kickstart", "-k", &service])
                .output();
        }
        println!("[install] LaunchAgent installed: {}", plist_path.display());

        let mut running_pid = None;
        for _ in 0..20 {
            if let Some(p) = read_running_pid(paths) {
                running_pid = Some(p);
                break;
            }
            thread::sleep(Duration::from_millis(150));
        }
        match running_pid {
            Some(p) => p,
            None => start_daemon(paths, watch_themes)?,
        }
    };

    let mem_note = process_memory_stats(pid)
        .map(|m| format!(" (~{:.2} MB RSS)", m.working_set_kb as f64 / 1024.0))
        .unwrap_or_default();
    println!("[install] Rust daemon running silently (PID {pid}){mem_note}.");

    if read_port_info().is_none() {
        println!(
            "[install] Note: DevToolsActivePort not found yet. Quit Antigravity (Cmd+Q) and reopen via \"{}\" (or run `agy-theme launch`).",
            launcher_app.display()
        );
    }
    Ok(())
}

pub fn launch_antigravity(paths: &AppPaths, restart: bool) -> Result<()> {
    if read_running_pid(paths).is_none() {
        let pid = start_daemon(paths, false)?;
        println!("[launch] Started Rust theme daemon (PID {pid}).");
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let app_path = find_antigravity_app().ok_or(
            "Antigravity.app not found in /Applications or ~/Applications. Install Antigravity or set AGY_APP_PATH.",
        )?;

        if restart {
            println!("[launch] Requesting graceful quit of Antigravity...");
            let _ = Command::new("osascript")
                .args(["-e", "tell application \"Antigravity\" to quit"])
                .output();
            for _ in 0..25 {
                let still_running = Command::new("pgrep")
                    .args(["-x", "Antigravity"])
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);
                if !still_running {
                    break;
                }
                thread::sleep(Duration::from_millis(200));
            }
        }

        let status = Command::new("open")
            .args([
                "-a",
                &app_path.to_string_lossy(),
                "--args",
                "--remote-debugging-port=0",
            ])
            .status()?;
        if !status.success() {
            return Err(format!("Failed to launch {} via `open -a`.", app_path.display()).into());
        }
        println!(
            "[launch] Launched {} with --remote-debugging-port=0.",
            app_path.display()
        );

        if !restart {
            for _ in 0..10 {
                if read_port_info().is_some() {
                    break;
                }
                thread::sleep(Duration::from_millis(150));
            }
            if read_port_info().is_none() {
                println!(
                    "[launch] Note: DevToolsActivePort not active yet. If Antigravity was already open without CDP, quit it (Cmd+Q) or run `agy-theme launch --restart`."
                );
            }
        }
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = restart;
        println!(
            "[launch] `agy-theme launch` is tailored for macOS. On Windows open the patched Antigravity shortcut; on Linux run `antigravity --remote-debugging-port=0`."
        );
        Ok(())
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn install_linux(paths: &AppPaths, no_startup: bool, watch_themes: bool) -> Result<()> {
    use std::process::Command;
    if no_startup {
        let pid = start_daemon(paths, watch_themes)?;
        println!("[install] Rust daemon running (PID {pid}).");
        return Ok(());
    }
    let daemon_exe = resolve_daemon_exe();
    let has_systemd = Command::new("systemctl")
        .args(["--user", "status"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_systemd {
        let xdg = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            format!("{}/.config", std::env::var("HOME").unwrap_or_default())
        });
        let unit_dir = Path::new(&xdg).join("systemd").join("user");
        fs::create_dir_all(&unit_dir)?;
        let unit_path = unit_dir.join(SYSTEMD_UNIT);
        let extra = if watch_themes { " --watch-themes" } else { "" };
        let unit = format!(
            "[Unit]\nDescription=Antigravity Custom Theme Rust Daemon\nAfter=graphical-session.target\n\n[Service]\nType=simple\nWorkingDirectory={}\nExecStart={} --root \"{}\"{extra}\nRestart=always\nRestartSec=3\n\n[Install]\nWantedBy=default.target\n",
            paths.root.display(),
            daemon_exe.display(),
            paths.root.display()
        );
        fs::write(&unit_path, unit)?;
        let _ = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status();
        let _ = Command::new("systemctl")
            .args(["--user", "enable", "--now", SYSTEMD_UNIT])
            .status();
        println!(
            "[install] systemd user service active: {}",
            unit_path.display()
        );
    } else {
        let pid = start_daemon(paths, watch_themes)?;
        println!("[install] systemd --user unavailable; started detached Rust daemon (PID {pid}).");
    }
    Ok(())
}

fn remove_autostart(paths: &AppPaths) {
    let _ = paths;
    #[cfg(windows)]
    {
        use std::process::Command;
        let ps = format!(
            "if (Get-ScheduledTask -TaskName {t} -ErrorAction SilentlyContinue) {{ Stop-ScheduledTask -TaskName {t} -ErrorAction SilentlyContinue; Unregister-ScheduledTask -TaskName {t} -Confirm:$false -ErrorAction SilentlyContinue; Write-Host '[uninstall] Removed Windows Scheduled Task: {name}' }}",
            t = ps_single_quote(WIN_TASK_NAME),
            name = WIN_TASK_NAME
        );
        if let Ok(out) = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &ps,
            ])
            .output()
        {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if !stdout.trim().is_empty() {
                print!("{stdout}");
            }
        }

        if let Some(app_data) = std::env::var_os("APPDATA") {
            let lnk = Path::new(&app_data)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
                .join("Startup")
                .join("AntigravityCustomTheme.lnk");
            if lnk.is_file() && fs::remove_file(&lnk).is_ok() {
                println!(
                    "[uninstall] Removed Windows Startup shortcut: {}",
                    lnk.display()
                );
            }
        }
        if paths.legacy_vbs.is_file() && fs::remove_file(&paths.legacy_vbs).is_ok() {
            println!(
                "[uninstall] Removed legacy helper script: {}",
                paths.legacy_vbs.display()
            );
        }
    }

    #[cfg(target_os = "macos")]
    {
        let home = macos_home_dir();
        let plist = home
            .join("Library")
            .join("LaunchAgents")
            .join(format!("{MACOS_SERVICE_LABEL}.plist"));
        bootout_macos_agent(&plist);
        if plist.is_file() && fs::remove_file(&plist).is_ok() {
            println!("[uninstall] Removed LaunchAgent plist: {}", plist.display());
        }
        let apps_dir = home.join("Applications");
        if let Ok(true) = remove_macos_launcher_app(&apps_dir) {
            println!(
                "[uninstall] Removed macOS CDP launcher: {}",
                apps_dir.join(MACOS_LAUNCHER_APP_NAME).display()
            );
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        use std::process::Command;
        let _ = Command::new("systemctl")
            .args(["--user", "disable", "--now", SYSTEMD_UNIT])
            .output();
        let xdg = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            format!("{}/.config", std::env::var("HOME").unwrap_or_default())
        });
        let unit = Path::new(&xdg)
            .join("systemd")
            .join("user")
            .join(SYSTEMD_UNIT);
        if unit.is_file() && fs::remove_file(&unit).is_ok() {
            let _ = Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .output();
            println!("[uninstall] Removed systemd user unit: {}", unit.display());
        }
    }
}

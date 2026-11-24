use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub root: PathBuf,
    pub themes_json: PathBuf,
    pub engine_js: PathBuf,
    pub dist_dir: PathBuf,
    pub payload_js: PathBuf,
    pub config_json: PathBuf,
    pub pid_file: PathBuf,
    pub log_file: PathBuf,
    pub legacy_vbs: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortInfo {
    pub port: u16,
    pub token: String,
    pub port_file: PathBuf,
}

fn looks_like_repo_root(dir: &Path) -> bool {
    dir.join("themes").join("themes.json").is_file()
        || dir.join("engine").join("core.js").is_file()
        || dir.join("dist").join("theme_payload.js").is_file()
}

impl AppPaths {
    pub fn discover(explicit_root: Option<&Path>) -> Self {
        let root = resolve_root_dir(explicit_root);
        let config_json = env::var_os("AGY_CONFIG_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("theme_config.json"));
        let dist_dir = root.join("dist");
        Self {
            themes_json: root.join("themes").join("themes.json"),
            engine_js: root.join("engine").join("core.js"),
            payload_js: dist_dir.join("theme_payload.js"),
            dist_dir,
            config_json,
            pid_file: root.join("daemon.pid"),
            log_file: root.join("daemon.log"),
            legacy_vbs: root.join("silent-start.vbs"),
            root,
        }
    }
}

fn normalize_path(p: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let s = p.to_string_lossy();
        if let Some(stripped) = s.strip_prefix(r"\\?\") {
            if !stripped.starts_with("UNC\\") {
                return PathBuf::from(stripped);
            }
        }
    }
    p
}

pub fn resolve_root_dir(explicit_root: Option<&Path>) -> PathBuf {
    if let Some(p) = explicit_root {
        let resolved = fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
        return normalize_path(resolved);
    }

    if let Some(env_root) = env::var_os("AGY_ROOT_DIR").filter(|v| !v.is_empty()) {
        let p = PathBuf::from(env_root);
        let resolved = fs::canonicalize(&p).unwrap_or(p);
        return normalize_path(resolved);
    }

    // 1. Check current working directory and its immediate ancestors.
    if let Ok(cwd) = env::current_dir() {
        for ancestor in cwd.ancestors().take(4) {
            if looks_like_repo_root(ancestor) {
                return normalize_path(ancestor.to_path_buf());
            }
        }
    }

    // 2. Walk up from the executable path (handles target/release/agy-theme-daemon.exe, bin/, etc.).
    if let Ok(exe) = env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            for ancestor in exe_dir.ancestors().take(5) {
                if looks_like_repo_root(ancestor) {
                    return normalize_path(ancestor.to_path_buf());
                }
            }
            return normalize_path(exe_dir.to_path_buf());
        }
    }

    normalize_path(env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

pub fn resolve_daemon_exe() -> PathBuf {
    let daemon_name = if cfg!(windows) {
        "agy-theme-daemon.exe"
    } else {
        "agy-theme-daemon"
    };

    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            let sibling = dir.join(daemon_name);
            if sibling.is_file() {
                return sibling;
            }
        }
        return exe;
    }

    PathBuf::from(daemon_name)
}

pub fn resolve_cli_exe() -> PathBuf {
    let cli_name = if cfg!(windows) {
        "agy-theme.exe"
    } else {
        "agy-theme"
    };

    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            let sibling = dir.join(cli_name);
            if sibling.is_file() {
                return sibling;
            }
        }
        return exe;
    }

    PathBuf::from(cli_name)
}

pub fn resolve_port_file() -> PathBuf {
    if let Some(custom) = env::var_os("AGY_PORT_FILE").filter(|v| !v.is_empty()) {
        return PathBuf::from(custom);
    }

    let candidates = default_port_file_candidates();
    candidates
        .iter()
        .find(|p| p.is_file())
        .cloned()
        .unwrap_or_else(|| {
            candidates
                .into_iter()
                .next()
                .unwrap_or_else(|| PathBuf::from("DevToolsActivePort"))
        })
}

fn default_port_file_candidates() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        let base = env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| env::var_os("USERPROFILE").map(|h| PathBuf::from(h).join("AppData").join("Roaming")))
            .unwrap_or_else(|| PathBuf::from("."));
        vec![
            base.join("antigravity").join("DevToolsActivePort"),
            base.join("Antigravity").join("DevToolsActivePort"),
        ]
    }
    #[cfg(target_os = "macos")]
    {
        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let app_support = home.join("Library").join("Application Support");
        vec![
            app_support.join("antigravity").join("DevToolsActivePort"),
            app_support.join("Antigravity").join("DevToolsActivePort"),
        ]
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let xdg = env::var_os("XDG_CONFIG_HOME")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                env::var_os("HOME")
                    .map(|h| PathBuf::from(h).join(".config"))
                    .unwrap_or_else(|| PathBuf::from(".config"))
            });
        vec![
            xdg.join("antigravity").join("DevToolsActivePort"),
            xdg.join("Antigravity").join("DevToolsActivePort"),
        ]
    }
}

pub fn read_port_info() -> Option<PortInfo> {
    let port_file = resolve_port_file();
    let content = fs::read_to_string(&port_file).ok()?;
    let mut lines = content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty());
    let port: u16 = lines.next()?.parse().ok()?;
    if port == 0 {
        return None;
    }
    let token = lines.next().unwrap_or("").to_string();
    Some(PortInfo {
        port,
        token,
        port_file,
    })
}

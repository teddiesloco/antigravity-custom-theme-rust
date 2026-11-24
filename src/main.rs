use agy_theme::builder::{build_payload, list_themes, BuildOptions, DEFAULT_MAX_DIM};
use agy_theme::cdp::verify_live;
use agy_theme::daemon::{run as run_daemon, DaemonOptions};
use agy_theme::installer::{
    install, launch_antigravity, print_status, start_daemon, stop_daemon, uninstall,
};
use agy_theme::paths::AppPaths;
use agy_theme::sys::process_memory_stats;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn print_help() {
    println!(
        r#"agy-theme v{} — Ultra-Fast, Low-RAM Rust Custom Theme Engine for Antigravity

USAGE:
  agy-theme <COMMAND> [OPTIONS]

COMMANDS:
  build       Compile themes/themes.json + engine/core.js -> dist/theme_payload.js
                [--themes <path>] [--engine <path>] [--out <path>] [--max-dim <px>]
  daemon      Run CDP theme daemon in foreground (Ctrl+C to stop)
                [--watch-themes]
  start       Launch silent background Rust daemon (zero console window)
                [--watch-themes]
  stop        Stop running background daemon
  status      Show daemon PID, live RAM usage, payload status, and CDP port
  install     Build payload, patch Antigravity shortcuts / macOS launcher, register auto-start, and start daemon
                [--no-startup] [--watch-themes]
  launch      Ensure daemon is running and open Antigravity with --remote-debugging-port=0 (macOS)
                [--restart]
  uninstall   Remove auto-start, macOS launcher, stop daemon, and restore Antigravity default UI
  verify      Live CDP inspection (themeActive, zero live blur, pills, browser hole)
  themes      List all configured presets in themes/themes.json

GLOBAL OPTIONS:
  --root <dir>    Override repository root directory (default: auto-detected)
  -h, --help      Show this help message
  -V, --version   Show version"#,
        env!("CARGO_PKG_VERSION")
    );
}

fn take_value(args: &[String], idx: &mut usize, flag: &str) -> Result<String, String> {
    *idx += 1;
    args.get(*idx)
        .cloned()
        .ok_or_else(|| format!("Missing value for {flag}"))
}

fn run_cli() -> Result<ExitCode, String> {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    if raw_args.is_empty() {
        print_help();
        return Ok(ExitCode::SUCCESS);
    }

    let mut explicit_root: Option<PathBuf> = None;
    let mut filtered: Vec<String> = Vec::with_capacity(raw_args.len());
    let mut i = 0;
    while i < raw_args.len() {
        match raw_args[i].as_str() {
            "--root" => {
                let val = take_value(&raw_args, &mut i, "--root")?;
                explicit_root = Some(PathBuf::from(val));
            }
            "-h" | "--help" => {
                print_help();
                return Ok(ExitCode::SUCCESS);
            }
            "-V" | "--version" => {
                println!("agy-theme {}", env!("CARGO_PKG_VERSION"));
                return Ok(ExitCode::SUCCESS);
            }
            other => filtered.push(other.to_string()),
        }
        i += 1;
    }

    let Some(cmd) = filtered.first().map(String::as_str) else {
        print_help();
        return Ok(ExitCode::SUCCESS);
    };
    let rest = &filtered[1..];
    let paths = AppPaths::discover(explicit_root.as_deref());

    match cmd {
        "build" => {
            let mut opts = BuildOptions {
                themes_path: paths.themes_json.clone(),
                engine_path: paths.engine_js.clone(),
                out_path: paths.payload_js.clone(),
                max_dim: DEFAULT_MAX_DIM,
            };
            let mut idx = 0;
            while idx < rest.len() {
                match rest[idx].as_str() {
                    "--themes" => opts.themes_path = PathBuf::from(take_value(rest, &mut idx, "--themes")?),
                    "--engine" => opts.engine_path = PathBuf::from(take_value(rest, &mut idx, "--engine")?),
                    "--out" => opts.out_path = PathBuf::from(take_value(rest, &mut idx, "--out")?),
                    "--max-dim" => {
                        let v = take_value(rest, &mut idx, "--max-dim")?;
                        opts.max_dim = v
                            .parse()
                            .map_err(|_| format!("Invalid --max-dim value: {v}"))?;
                    }
                    other => return Err(format!("Unknown build option: {other}")),
                }
                idx += 1;
            }
            let report = build_payload(&opts).map_err(|e| e.to_string())?;
            println!(
                "[build] Built {} ({:.1} KB in {} ms) with {} theme(s): {}",
                report.out_path.display(),
                report.size_bytes as f64 / 1024.0,
                report.elapsed_ms,
                report.theme_names.len(),
                report.theme_names.join(", ")
            );
            Ok(ExitCode::SUCCESS)
        }
        "daemon" => {
            let watch_themes = rest.iter().any(|a| a == "--watch-themes");
            run_daemon(DaemonOptions {
                paths,
                echo_stdout: true,
                watch_themes,
            })
            .map_err(|e| e.to_string())?;
            Ok(ExitCode::SUCCESS)
        }
        "start" => {
            let watch_themes = rest.iter().any(|a| a == "--watch-themes");
            let pid = start_daemon(&paths, watch_themes).map_err(|e| e.to_string())?;
            let mem = process_memory_stats(pid)
                .map(|m| format!(" (~{:.2} MB RAM)", m.working_set_kb as f64 / 1024.0))
                .unwrap_or_default();
            println!("[start] Rust daemon running silently (PID {pid}){mem}.");
            Ok(ExitCode::SUCCESS)
        }
        "stop" => {
            match stop_daemon(&paths).map_err(|e| e.to_string())? {
                Some(pid) => println!("[stop] Stopped daemon (PID {pid})."),
                None => println!("[stop] Daemon is not running."),
            }
            Ok(ExitCode::SUCCESS)
        }
        "status" => {
            print_status(&paths);
            Ok(ExitCode::SUCCESS)
        }
        "install" => {
            let no_startup = rest.iter().any(|a| a == "--no-startup" || a == "-NoStartup");
            let watch_themes = rest.iter().any(|a| a == "--watch-themes");
            install(&paths, no_startup, watch_themes).map_err(|e| e.to_string())?;
            Ok(ExitCode::SUCCESS)
        }
        "launch" => {
            let restart = rest.iter().any(|a| a == "--restart");
            launch_antigravity(&paths, restart).map_err(|e| e.to_string())?;
            Ok(ExitCode::SUCCESS)
        }
        "uninstall" => {
            uninstall(&paths).map_err(|e| e.to_string())?;
            Ok(ExitCode::SUCCESS)
        }
        "verify" => {
            let outcome = verify_live().map_err(|e| e.to_string())?;
            println!(
                "{}",
                serde_json::to_string_pretty(&outcome.results).unwrap_or_default()
            );
            if outcome.themed_count == 0 {
                eprintln!("[verify] FAIL: #antigravity-custom-theme-active not active on any page.");
                return Ok(ExitCode::FAILURE);
            }
            if outcome.has_live_blur {
                eprintln!("[verify] FAIL: Detected backdrop-filter blur rule in injected style.");
                return Ok(ExitCode::FAILURE);
            }
            println!(
                "[verify] PASS: {}/{} page(s) themed on port {} with zero live blur.",
                outcome.themed_count, outcome.total_count, outcome.port
            );
            Ok(ExitCode::SUCCESS)
        }
        "themes" => {
            let list = list_themes(&paths.themes_json).map_err(|e| e.to_string())?;
            println!("Configured themes ({}):", list.len());
            for (idx, t) in list.iter().enumerate() {
                let mode = if t.is_light { "Light" } else { "Dark " };
                let kind = if t.has_wallpaper { "Wallpaper" } else { "Pure-CSS " };
                let def = if idx == 0 { " [default]" } else { "" };
                println!("  {}. {:<22} [{} | {}]{} — {}", idx + 1, t.name, mode, kind, def, t.concept);
            }
            Ok(ExitCode::SUCCESS)
        }
        other => Err(format!("Unknown command '{other}'. Run `agy-theme --help` for usage.")),
    }
}

fn main() -> ExitCode {
    match run_cli() {
        Ok(code) => code,
        Err(msg) => {
            eprintln!("[agy-theme] Error: {msg}");
            ExitCode::FAILURE
        }
    }
}

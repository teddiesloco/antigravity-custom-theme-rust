use agy_theme::builder::{build_payload, list_themes, percent_encode_svg, BuildOptions, DEFAULT_MAX_DIM};
use agy_theme::cdp::{
    evaluate_once, fetch_targets, filter_antigravity_pages, CdpTarget,
};
use agy_theme::config::{read_config, save_config_merge};
use agy_theme::installer::{
    remove_macos_launcher_app, render_macos_plist, sh_single_quote, write_macos_launcher_app,
    MACOS_LAUNCHER_APP_NAME, MACOS_LAUNCHER_BUNDLE_ID, MACOS_SERVICE_LABEL,
};
use agy_theme::ws::compute_accept_key;
use serde_json::{json, Map, Value};
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn mock_ws_accept(stream: &mut TcpStream) {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 512];
    loop {
        let n = stream.read(&mut tmp).unwrap();
        buf.extend_from_slice(&tmp[..n]);
        if buf.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }
    let req = String::from_utf8_lossy(&buf);
    let key = req
        .lines()
        .find_map(|l| {
            let (k, v) = l.split_once(':')?;
            if k.trim().eq_ignore_ascii_case("sec-websocket-key") {
                Some(v.trim().to_string())
            } else {
                None
            }
        })
        .expect("Sec-WebSocket-Key header");
    let accept = compute_accept_key(&key);
    let resp = format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {accept}\r\n\r\n"
    );
    stream.write_all(resp.as_bytes()).unwrap();
}

fn mock_ws_read_client_text(stream: &mut TcpStream) -> String {
    let mut hdr = [0u8; 2];
    stream.read_exact(&mut hdr).unwrap();
    let masked = (hdr[1] & 0x80) != 0;
    assert!(masked, "client frames must be masked per RFC 6455");
    let len7 = (hdr[1] & 0x7F) as usize;
    let len = match len7 {
        0..=125 => len7,
        126 => {
            let mut b = [0u8; 2];
            stream.read_exact(&mut b).unwrap();
            u16::from_be_bytes(b) as usize
        }
        127 => {
            let mut b = [0u8; 8];
            stream.read_exact(&mut b).unwrap();
            u64::from_be_bytes(b) as usize
        }
        _ => unreachable!(),
    };
    let mut mask = [0u8; 4];
    stream.read_exact(&mut mask).unwrap();
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).unwrap();
    for (i, b) in payload.iter_mut().enumerate() {
        *b ^= mask[i & 3];
    }
    String::from_utf8(payload).unwrap()
}

fn mock_ws_send_server_text(stream: &mut TcpStream, text: &str) {
    let bytes = text.as_bytes();
    let mut hdr = Vec::with_capacity(10);
    hdr.push(0x81);
    if bytes.len() <= 125 {
        hdr.push(bytes.len() as u8);
    } else if bytes.len() <= 65_535 {
        hdr.push(126);
        hdr.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
    } else {
        hdr.push(127);
        hdr.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
    }
    stream.write_all(&hdr).unwrap();
    stream.write_all(bytes).unwrap();
}

#[test]
fn test_rfc6455_accept_key_vector() {
    // Official RFC 6455 Section 1.3 test vector
    assert_eq!(
        compute_accept_key("dGhlIHNhbXBsZSBub25jZQ=="),
        "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
    );
}

#[test]
fn test_svg_percent_encoding_matches_browser_data_uri() {
    let encoded = percent_encode_svg("<svg viewBox=\"0 0 10 10\"></svg>");
    assert!(encoded.starts_with(
        "data:image/svg+xml,%3Csvg%20viewBox%3D%220%200%2010%2010%22%3E%3C/svg%3E"
    ));
}

#[test]
fn test_build_payload_preserves_order_and_inlines_assets() {
    let root = repo_root();
    let out_path = std::env::temp_dir().join(format!(
        "agy_theme_test_payload_{}.js",
        std::process::id()
    ));
    let _ = fs::remove_file(&out_path);

    let report = build_payload(&BuildOptions {
        themes_path: root.join("themes").join("themes.json"),
        engine_path: root.join("engine").join("core.js"),
        out_path: out_path.clone(),
        max_dim: DEFAULT_MAX_DIM,
    })
    .expect("build_payload should succeed");

    assert_eq!(report.theme_names.first().map(String::as_str), Some("WLOP Cosmic"));
    assert!(report.theme_names.len() >= 5);

    let compiled = fs::read_to_string(&out_path).expect("compiled payload readable");
    assert!(!compiled.contains("/*__THEMES_PLACEHOLDER__*/{}"));
    assert!(compiled.contains("data:image/webp;base64,"));
    assert!(compiled.contains("data:image/svg+xml,"));
    assert!(compiled.contains("Kyoto Matcha Light"));

    let _ = fs::remove_file(&out_path);
}

#[test]
fn test_list_themes_reports_all_presets() {
    let themes = list_themes(&repo_root().join("themes").join("themes.json"))
        .expect("list_themes should succeed");
    assert_eq!(themes[0].name, "WLOP Cosmic");
    assert!(themes[0].has_wallpaper);
    assert!(themes.iter().any(|t| t.name == "Kyoto Matcha Light" && t.is_light));
}

#[test]
fn test_config_atomic_merge_roundtrip() {
    let cfg_path = std::env::temp_dir().join(format!(
        "agy_theme_test_cfg_{}.json",
        std::process::id()
    ));
    let _ = fs::remove_file(&cfg_path);

    let mut first = Map::new();
    first.insert("themeName".into(), json!("WLOP Cosmic"));
    first.insert("dimming".into(), json!(55));
    save_config_merge(&cfg_path, &first).expect("save first");

    let mut second = Map::new();
    second.insert("glassBlur".into(), json!(6));
    let merged = save_config_merge(&cfg_path, &second).expect("save merge");
    assert_eq!(merged.get("themeName"), Some(&json!("WLOP Cosmic")));
    assert_eq!(merged.get("dimming"), Some(&json!(55)));
    assert_eq!(merged.get("glassBlur"), Some(&json!(6)));

    let loaded = read_config(&cfg_path);
    assert_eq!(loaded, merged);
    let _ = fs::remove_file(&cfg_path);
}

#[test]
fn test_filter_antigravity_pages() {
    let targets = vec![
        CdpTarget {
            id: "1".into(),
            target_type: "page".into(),
            title: "Antigravity".into(),
            url: "http://127.0.0.1:12345/workbench".into(),
            web_socket_debugger_url: "ws://127.0.0.1:9222/devtools/page/1".into(),
        },
        CdpTarget {
            id: "2".into(),
            target_type: "page".into(),
            title: "DevTools".into(),
            url: "devtools://devtools/bundled/inspector.html".into(),
            web_socket_debugger_url: "ws://127.0.0.1:9222/devtools/page/2".into(),
        },
        CdpTarget {
            id: "3".into(),
            target_type: "service_worker".into(),
            title: "SW".into(),
            url: "http://127.0.0.1:12345/sw.js".into(),
            web_socket_debugger_url: "ws://127.0.0.1:9222/devtools/sw/3".into(),
        },
    ];
    let filtered = filter_antigravity_pages(targets);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "1");
}

#[test]
fn test_mock_cdp_http_and_websocket_evaluate() {
    let http_listener = TcpListener::bind("127.0.0.1:0").expect("bind mock http");
    let http_port = http_listener.local_addr().unwrap().port();

    let ws_listener = TcpListener::bind("127.0.0.1:0").expect("bind mock ws");
    let ws_port = ws_listener.local_addr().unwrap().port();
    let ws_url = format!("ws://127.0.0.1:{ws_port}/devtools/page/MOCK1");
    let ws_url_for_http = ws_url.clone();

    let http_handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = http_listener.accept() {
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let body = json!([{
                "id": "MOCK1",
                "type": "page",
                "title": "Antigravity Mock",
                "url": "http://127.0.0.1:9999/launchpad",
                "webSocketDebuggerUrl": ws_url_for_http
            }])
            .to_string();
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
            // Keep socket open (simulates Chromium DevTools HTTP keep-alive behavior!)
            thread::sleep(Duration::from_millis(500));
        }
    });

    let ws_handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = ws_listener.accept() {
            mock_ws_accept(&mut stream);
            let txt = mock_ws_read_client_text(&mut stream);
            let req: Value = serde_json::from_str(&txt).unwrap();
            let id = req["id"].as_u64().unwrap();
            let reply = json!({
                "id": id,
                "result": {
                    "result": {
                        "type": "object",
                        "value": { "themeActive": true, "hasLiveBlurRule": false }
                    }
                }
            });
            mock_ws_send_server_text(&mut stream, &reply.to_string());
        }
    });

    let targets = fetch_targets(http_port, Duration::from_secs(2)).expect("fetch_targets");
    let pages = filter_antigravity_pages(targets);
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].web_socket_debugger_url, ws_url);

    let val = evaluate_once(&ws_url, "1+1", true, true, Duration::from_secs(2))
        .expect("evaluate_once");
    assert_eq!(val["themeActive"], json!(true));
    assert_eq!(val["hasLiveBlurRule"], json!(false));

    let _ = http_handle.join();
    let _ = ws_handle.join();
}

#[test]
fn test_raster_downscale_and_alpha_avatar_encoding() {
    use base64::engine::general_purpose::STANDARD as BASE64;
    use base64::Engine;
    use image::{ImageBuffer, ImageFormat, Rgb, Rgba};

    let tmp = std::env::temp_dir().join(format!("agy_img_test_{}", std::process::id()));
    let assets_dir = tmp.join("themes").join("assets");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&assets_dir).unwrap();

    // 1. Oversized opaque wallpaper (1200x800, built with max_dim = 600 -> resized to 600x400 JPEG)
    let wp_img = ImageBuffer::from_fn(1200, 800, |x, y| {
        Rgb([(x % 255) as u8, (y % 255) as u8, ((x + y) % 255) as u8])
    });
    wp_img
        .save_with_format(assets_dir.join("big_wp.png"), ImageFormat::Png)
        .unwrap();

    // 2. Oversized transparent avatar (400x400 RGBA -> resized to 256x256 lossless WebP/PNG)
    let av_img = ImageBuffer::from_fn(400, 400, |x, y| {
        Rgba([(x % 255) as u8, (y % 255) as u8, 180, if x < 200 { 255 } else { 128 }])
    });
    av_img
        .save_with_format(assets_dir.join("avatar.png"), ImageFormat::Png)
        .unwrap();

    let themes_json = tmp.join("themes").join("themes.json");
    fs::write(
        &themes_json,
        json!({
            "Test Theme": {
                "concept": "Synthetic",
                "wallpaper": "assets/big_wp.png",
                "userAvatar": "assets/avatar.png"
            }
        })
        .to_string(),
    )
    .unwrap();

    let engine_js = tmp.join("core.js");
    fs::write(&engine_js, "const THEMES = /*__THEMES_PLACEHOLDER__*/{};").unwrap();
    let out_js = tmp.join("dist").join("theme_payload.js");

    build_payload(&BuildOptions {
        themes_path: themes_json,
        engine_path: engine_js,
        out_path: out_js.clone(),
        max_dim: 600,
    })
    .expect("build with synthetic rasters");

    let compiled = fs::read_to_string(&out_js).unwrap();
    let json_part = compiled
        .strip_prefix("const THEMES = ")
        .unwrap()
        .strip_suffix(';')
        .unwrap();
    let v: Value = serde_json::from_str(json_part).unwrap();

    // Verify resized wallpaper dimensions == 600x400
    let wp_uri = v["Test Theme"]["wallpaper"].as_str().unwrap();
    assert!(wp_uri.starts_with("data:image/jpeg;base64,"));
    let wp_bytes = BASE64
        .decode(wp_uri.split_once(',').unwrap().1)
        .unwrap();
    let wp_dec = image::load_from_memory(&wp_bytes).unwrap();
    assert_eq!((wp_dec.width(), wp_dec.height()), (600, 400));

    // Verify resized avatar dimensions == 256x256 and retains alpha
    let av_uri = v["Test Theme"]["userAvatar"].as_str().unwrap();
    assert!(
        av_uri.starts_with("data:image/webp;base64,")
            || av_uri.starts_with("data:image/png;base64,")
    );
    let av_bytes = BASE64
        .decode(av_uri.split_once(',').unwrap().1)
        .unwrap();
    let av_dec = image::load_from_memory(&av_bytes).unwrap();
    assert_eq!((av_dec.width(), av_dec.height()), (256, 256));
    assert!(av_dec.color().has_alpha());

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_macos_plist_rendering_and_xml_escaping() {
    let daemon = PathBuf::from("/tmp/My & <App>/agy-theme-daemon");
    let root = PathBuf::from("/tmp/Teddy's \"Repo\"");
    let log = PathBuf::from("/tmp/My & <App>/daemon.log");

    let plist = render_macos_plist(&daemon, &root, &log, true);
    assert!(plist.contains(&format!("<string>{MACOS_SERVICE_LABEL}</string>")));
    assert!(plist.contains("/tmp/My &amp; &lt;App&gt;/agy-theme-daemon"));
    assert!(plist.contains("/tmp/Teddy&apos;s &quot;Repo&quot;"));
    assert!(plist.contains("<string>--watch-themes</string>"));
    assert!(plist.contains("<key>ProcessType</key>"));
    assert!(plist.contains("<key>SuccessfulExit</key>"));
    assert!(plist.contains("<false/>"));

    let plist_no_watch = render_macos_plist(&daemon, &root, &log, false);
    assert!(!plist_no_watch.contains("--watch-themes"));
}

#[test]
fn test_macos_launcher_app_create_and_remove_idempotent() {
    let tmp = std::env::temp_dir().join(format!("agy_macos_app_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    let apps_dir = tmp.join("Applications");
    let fake_icon = tmp.join("icon.icns");
    fs::create_dir_all(&tmp).unwrap();
    fs::write(&fake_icon, b"icns-stub").unwrap();

    let cli = PathBuf::from("/tmp/dir with space/Teddy's/agy-theme");
    let root = PathBuf::from("/tmp/dir with space/Teddy's/repo");
    assert_eq!(sh_single_quote("a'b c"), "'a'\\''b c'");

    let app_path = write_macos_launcher_app(&apps_dir, &cli, &root, Some(&fake_icon))
        .expect("write_macos_launcher_app should succeed");
    assert_eq!(app_path, apps_dir.join(MACOS_LAUNCHER_APP_NAME));

    let info_plist = fs::read_to_string(app_path.join("Contents").join("Info.plist")).unwrap();
    assert!(info_plist.contains(MACOS_LAUNCHER_BUNDLE_ID));
    assert!(info_plist.contains("<key>LSUIElement</key>"));
    assert!(info_plist.contains("<string>icon.icns</string>"));
    assert!(app_path.join("Contents").join("Resources").join("icon.icns").is_file());

    let launcher = app_path
        .join("Contents")
        .join("MacOS")
        .join("antigravity-theme-launcher");
    let script = fs::read_to_string(&launcher).unwrap();
    assert!(script.contains("--remote-debugging-port=0"));
    assert!(script.contains("CLI_BIN='/tmp/dir with space/Teddy'\\''s/agy-theme'"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&launcher).unwrap().permissions().mode();
        assert_ne!(mode & 0o111, 0, "launcher script must be executable");
        let syntax = std::process::Command::new("bash")
            .args(["-n", &launcher.to_string_lossy()])
            .status()
            .expect("bash -n should run");
        assert!(syntax.success(), "launcher script must pass bash -n");
    }

    assert!(remove_macos_launcher_app(&apps_dir).unwrap());
    assert!(!app_path.exists());
    assert!(!remove_macos_launcher_app(&apps_dir).unwrap());

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_bearrrr_eye_comfort_preset_contract() {
    let themes_path = repo_root().join("themes").join("themes.json");
    let themes = list_themes(&themes_path).expect("list_themes should succeed");
    assert_eq!(themes[0].name, "WLOP Cosmic");
    assert!(themes.len() >= 6);

    let bearrrr = themes
        .iter()
        .find(|t| t.name == "Bearrrr Eye-Comfort Dark")
        .expect("Bearrrr Eye-Comfort Dark preset must exist");
    assert!(!bearrrr.is_light);
    assert!(!bearrrr.has_wallpaper);
    assert!(!bearrrr.has_sticker);

    let raw: Value = serde_json::from_str(&fs::read_to_string(&themes_path).unwrap()).unwrap();
    let cfg = &raw["Bearrrr Eye-Comfort Dark"];
    assert_eq!(cfg["wallpaper"], json!(""));
    assert_eq!(cfg["primary"], json!("#e5a93b"));
    assert_eq!(cfg["secondary"], json!("#63c5da"));
    assert_eq!(cfg["textColor"], json!("#eceff4"));
    let serialized = cfg.to_string().to_ascii_lowercase();
    for forbidden in ["#a78bfa", "#a855f7", "#f472b6"] {
        assert!(
            !serialized.contains(forbidden),
            "Bearrrr Eye-Comfort Dark must not contain glaring purple {forbidden}"
        );
    }
}

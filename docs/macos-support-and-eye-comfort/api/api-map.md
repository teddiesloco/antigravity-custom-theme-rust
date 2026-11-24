---
type: api-map
feature: "macos-support-and-eye-comfort"
status: approved
version: 1.0.0
owner: "@ags-ba"
updated: 2026-11-24
links:
  - "srs/spec.md"
  - "ascii-screen/main.md"
  - "html-design/prototype.html"
changelog:
  - "2026-11-24 | /init | Khởi tạo tài liệu chuẩn AGS-BA"
  - "2026-11-24 | /ba | Ánh xạ hợp đồng Local CLI cho macOS installer, launcher và preset theme"
---

# Ma Trận Hợp Đồng Local CLI — macOS CDP Launcher, Modern LaunchAgent & Bearrrr Eye-Comfort Dark Theme

<!-- ags-ba-interface: local-cli-mcp -->

## 1. Bảng Hợp đồng Local CLI

| Surface | CLI | MCP | Input | Output | Persistence | Errors |
|---|---|---|---|---|---|---|
| macos-install-action | agy-theme install | none | --no-startup, --watch-themes, --root | stdout install report (PID, RAM MB, LaunchAgent & App path) | theme_config.json, ~/Library/LaunchAgents/ (plist), ~/Applications/ (launcher bundle), dist payload | E-macos-support-and-eye-comfort-002, E-macos-support-and-eye-comfort-003, E-macos-support-and-eye-comfort-004 |
| macos-launch-action | agy-theme launch | none | --restart, --root | stdout launch report + spawned Antigravity process with --remote-debugging-port=0 | theme_config.json (reads active config & starts daemon) | E-macos-support-and-eye-comfort-001, E-macos-support-and-eye-comfort-002 |
| macos-status-panel | agy-theme status | none | --root | stdout status lines (Root, Payload KB, Daemon PID/RSS, CDP port) | none (read-only) | E-macos-support-and-eye-comfort-003 |
| macos-uninstall-action | agy-theme uninstall | none | --root | stdout uninstall report (removed LaunchAgent, launcher app, cleared CDP pages) | theme_config.json (preserved), removes ~/Library/LaunchAgents/ plist & ~/Applications/ launcher bundle | E-macos-support-and-eye-comfort-002 |
| bearrrr-theme-card | agy-theme build | none | themes/themes.json, engine/core.js, --max-dim | dist payload + build elapsed_ms & theme list | themes/themes.json, theme_config.json | E-macos-support-and-eye-comfort-004 |

## 2. Quy tắc Biến đổi & Định dạng Dữ liệu

| Mã ID | Tham chiếu Nguồn | Nội dung Đặc tả / Quy tắc Nghiệp vụ | Điều kiện / Ràng buộc | Trạng thái / Ưu tiên |
|---|---|---|---|---|
| `AMP-macos-support-and-eye-comfort-001` | `FR-macos-support-and-eye-comfort-001` | Đường dẫn đưa vào LaunchAgent XML plist (`ProgramArguments`, `WorkingDirectory`, `StandardOutPath`, `StandardErrorPath`) phải được escape XML (`&`, `<`, `>`, `"`, `'`). | Đảm bảo plist hợp lệ cả khi thư mục dự án chứa dấu cách hoặc ký tự đặc biệt. | `P0` |
| `AMP-macos-support-and-eye-comfort-002` | `FR-macos-support-and-eye-comfort-002` | Đường dẫn đưa vào shell script bên trong `Antigravity Theme.app/Contents/MacOS/antigravity-theme-launcher` phải được single-quote shell escape an toàn (`'` -> `'\''`). | Tránh lỗi dấu cách hoặc injection khi sinh wrapper macOS `.app`. | `P0` |
| `AMP-macos-support-and-eye-comfort-003` | `FR-macos-support-and-eye-comfort-004` | Thứ tự khóa trong `themes/themes.json` được bảo toàn bởi `serde_json` `preserve_order`; `"WLOP Cosmic"` đứng vị trí 0, `"Bearrrr Eye-Comfort Dark"` dùng `wallpaper: ""` và `pureBg` CSS thuần. | `theme_config.json` cục bộ lưu `{"themeName": "Bearrrr Eye-Comfort Dark", "bubbleName": "Bearrrr Eye-Comfort Dark"}` (được `.gitignore`). | `P0` |

## 3. Danh sách Câu hỏi Mở (Open Questions — OQ)

- [x] **OQ-001** *(Owner: `@teddy`, Đã chốt: `2026-11-24`, Mức độ: `Resolved`)*: Giao tiếp hoàn toàn qua Local CLI `agy-theme` và CDP nội bộ `127.0.0.1`, không dựng HTTP API hay DB ngoài.

## 4. Checklist Tự Kiểm Tra Trước Khi Chuyển `approved`

- [x] Bao phủ 100% 5 bề mặt (`data-testid`) từ `ascii-screen/main.md` và `html-design/prototype.html`.
- [x] Toàn bộ mã lỗi khớp với SRS `E-macos-support-and-eye-comfort-001..004`.

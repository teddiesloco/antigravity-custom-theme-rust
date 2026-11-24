---
type: srs
feature: "macos-support-and-eye-comfort"
status: approved
version: 1.0.0
owner: "@ags-ba"
updated: 2026-11-24
links:
  - "srs/user-flow.md"
  - "ascii-screen/_index.md"
  - "ascii-screen/main.md"
  - "html-design/prototype.html"
  - "api/api-map.md"
  - "userstories/us-001.md"
changelog:
  - "2026-11-24 | /init | Khởi tạo tài liệu chuẩn AGS-BA"
  - "2026-11-24 | /ba | Hoàn thiện đặc tả macOS LaunchAgent, CDP Launcher và preset Bearrrr Eye-Comfort Dark"
---

# Đặc Tả Yêu Cầu Phần Mềm (SRS) — macOS CDP Launcher, Modern LaunchAgent & Bearrrr Eye-Comfort Dark Theme

> **Mục đích tài liệu:** Đặc tả đầy đủ yêu cầu chức năng (`FR`), quy tắc nghiệp vụ (`BR`), phi chức năng (`NFR`) và ma trận lỗi (`E`) cho bản nâng cấp macOS của `agy-theme` trên fork `teddiesloco/antigravity-custom-theme-rust`.

## 1. Tổng quan Hệ thống & Phạm vi Tính năng

| Mã ID | Tham chiếu Nguồn | Nội dung Đặc tả / Quy tắc Nghiệp vụ | Điều kiện / Ràng buộc | Trạng thái / Ưu tiên |
|---|---|---|---|---|
| `FR-macos-support-and-eye-comfort-001` | `UR-macos-support-and-eye-comfort-001` | Lệnh `agy-theme install` trên macOS tạo LaunchAgent plist (`~/Library/LaunchAgents/io.antigravity.custom-theme.plist`) có XML escaping an toàn, nạp dịch vụ bằng `launchctl bootout gui/<uid>` rồi `launchctl bootstrap gui/<uid>` (tự fallback sang `unload`/`load -w` nếu macOS cũ), chờ xác nhận PID thật + in RAM tiêu thụ giống Windows, và cảnh báo rõ nếu `DevToolsActivePort` chưa xuất hiện. | Không phá chữ ký `/Applications/Antigravity.app`; nếu `launchctl` thất bại hoặc không thấy PID sau timeout thì báo lỗi `E-macos-support-and-eye-comfort-002`; nếu thiếu `DevToolsActivePort` thì chỉ in cảnh báo hướng dẫn `E-macos-support-and-eye-comfort-003`. | `P0` |

## 2. Yêu cầu Chức năng Chi tiết

| Mã ID | Tham chiếu Nguồn | Nội dung Đặc tả / Quy tắc Nghiệp vụ | Điều kiện / Ràng buộc | Trạng thái / Ưu tiên |
|---|---|---|---|---|
| `FR-macos-support-and-eye-comfort-002` | `UR-macos-support-and-eye-comfort-001` | Lệnh `agy-theme install` trên macOS tự tạo ứng dụng launcher nhỏ gọn `~/Applications/Antigravity Theme.app` (kèm `Info.plist`, icon từ Antigravity nếu có, và executable script gọi daemon nếu chưa chạy rồi `open -a Antigravity --args --remote-debugging-port=0`). Đồng thời bổ sung lệnh CLI `agy-theme launch` (và nhánh `launch` trong `scripts/install-unix.sh`) để khởi chạy từ terminal. | Không sửa `app.asar` hay bất kỳ tệp nào trong `/Applications/Antigravity.app`. Nếu không tìm thấy `/Applications/Antigravity.app` (hoặc `~/Applications/Antigravity.app`) thì trả lỗi `E-macos-support-and-eye-comfort-001`. | `P0` |
| `FR-macos-support-and-eye-comfort-003` | `UR-macos-support-and-eye-comfort-001` | Lệnh `agy-theme uninstall` trên macOS gỡ sạch dịch vụ LaunchAgent bằng `launchctl bootout gui/<uid>/io.antigravity.custom-theme` (fallback `unload`), xoá tệp plist và xoá `~/Applications/Antigravity Theme.app` (nếu tồn tại), đồng thời tắt daemon và dọn theme khỏi các cửa sổ CDP đang mở. | Thao tác idempotent (chạy lặp lại không lỗi khi plist hoặc `.app` đã bị xoá trước đó). | `P0` |
| `FR-macos-support-and-eye-comfort-004` | `UR-macos-support-and-eye-comfort-001` | Bổ sung preset Pure-CSS không ảnh nền mang tên `"Bearrrr Eye-Comfort Dark"` vào `themes/themes.json` theo đúng bảng màu dịu mắt `bearrrr100x` OMP (Warm Amber `#e5a93b`, Frost Cyan `#63c5da`, Honey Gold `#ebcb8b`, Nord Slate `#4c566a`, Snow `#eceff4`, nền trầm obsidian/nord `#0d121a`–`#161b22`), kiểm tra qua `agy-theme themes` và `agy-theme build`. | `wallpaper` rỗng `""`, `isLight` là `false`, `unifiedGlass` là `false`, không dùng màu tím chói (`#a78bfa`, `#a855f7`, `#f472b6`). Lỗi đọc/biên dịch trả `E-macos-support-and-eye-comfort-004`. | `P0` |
| `FR-macos-support-and-eye-comfort-005` | `UR-macos-support-and-eye-comfort-001` | Cập nhật `README.md` và `scripts/install-unix.sh` ghi rõ hướng dẫn cài đặt, mở bằng `Antigravity Theme.app` hoặc `agy-theme launch`, và cách đặt preset mặc định nhẹ nhất trên macOS. | Đường dẫn trong tài liệu không chứa đường dẫn cá nhân cứng (`/Users/...`). | `P1` |

## 3. Quy tắc Nghiệp vụ

| Mã ID | Tham chiếu Nguồn | Nội dung Đặc tả / Quy tắc Nghiệp vụ | Điều kiện / Ràng buộc | Trạng thái / Ưu tiên |
|---|---|---|---|---|
| `BR-macos-support-and-eye-comfort-001` | `UR-macos-support-and-eye-comfort-001` | Tuyệt đối không chỉnh sửa `/Applications/Antigravity.app` hoặc `Resources/app.asar` để bảo toàn `ElectronAsarIntegrity` và chữ ký mã hoá của Apple. | Mọi cách bật `--remote-debugging-port=0` trên macOS chỉ đi qua `open -a` trong launcher riêng hoặc CLI `agy-theme launch`. | `P0` |
| `BR-macos-support-and-eye-comfort-002` | `UR-macos-support-and-eye-comfort-001` | Lệnh `agy-theme install` và `agy-theme launch` không tự ý tắt phiên Antigravity đang chạy nếu người dùng chưa truyền cờ tường minh `--restart` (tránh làm mất phiên chat/làm việc hiện tại). | Mặc định chỉ báo rõ cần thoát (`Cmd+Q`) và mở lại bằng `Antigravity Theme.app` nếu `DevToolsActivePort` chưa hoạt động. | `P0` |
| `BR-macos-support-and-eye-comfort-003` | `UR-macos-support-and-eye-comfort-001` | Giữ nguyên `"WLOP Cosmic"` ở vị trí đầu tiên (index 0) trong `themes/themes.json` để không phá tương thích và bài kiểm thử gốc; lựa chọn mặc định trên máy người dùng (`Bearrrr Eye-Comfort Dark`) được lưu ở `theme_config.json` (gitignored). | Đảm bảo `git pull upstream main` về sau ít xung đột nhất. | `P0` |
| `BR-macos-support-and-eye-comfort-004` | `UR-macos-support-and-eye-comfort-001` | Preset `Bearrrr Eye-Comfort Dark` tuân thủ Zero Live Blur và Zero-Image (`wallpaper: ""`, chỉ dùng `pureBg` CSS gradient) nhằm tối thiểu hoá RAM và GPU trên macOS Electron. | Kiểm định tự động trong `tests/integration_tests.rs`. | `P0` |

## 4. Yêu cầu Phi Chức năng

| Mã ID | Tham chiếu Nguồn | Nội dung Đặc tả / Quy tắc Nghiệp vụ | Điều kiện / Ràng buộc | Trạng thái / Ưu tiên |
|---|---|---|---|---|
| `NFR-macos-support-and-eye-comfort-001` | `UR-macos-support-and-eye-comfort-001` | Daemon chạy ngầm trên macOS Apple Silicon (arm64) tiêu thụ Physical Memory Footprint < 2.0 MB, CPU idle ~0.0%, tốc độ `agy-theme build` < 50 ms cho toàn bộ 6 preset. | Đo kiểm thực tế trên macOS bằng `top`/`ps` và `agy-theme status`. | `P0` |
| `NFR-macos-support-and-eye-comfort-002` | `UR-macos-support-and-eye-comfort-001` | Không còn cảnh báo biên dịch (`unused variable: paths`) trên macOS và Linux; `cargo test --locked` và `cargo build --release --locked` thoát mã `0` cả cục bộ lẫn trên GitHub Actions CI. | Tách các hàm render plist/launcher thuần để kiểm thử được trên mọi hệ điều hành. | `P0` |

## 5. Ma trận Mã lỗi & Thông báo Hiển thị

| Mã ID | Tham chiếu Nguồn | Nội dung Đặc tả / Quy tắc Nghiệp vụ | Điều kiện / Ràng buộc | Trạng thái / Ưu tiên |
|---|---|---|---|---|
| `E-macos-support-and-eye-comfort-001` | `FR-macos-support-and-eye-comfort-002` | `Antigravity.app not found`: không tìm thấy ứng dụng tại `/Applications/Antigravity.app` hoặc `~/Applications/Antigravity.app` khi chạy `agy-theme launch`. | Trả mã thoát khác `0` và in hướng dẫn cài Antigravity hoặc chỉ định đúng đường dẫn. | `P0` |
| `E-macos-support-and-eye-comfort-002` | `FR-macos-support-and-eye-comfort-001` | `LaunchAgent start failed`: cả `launchctl bootstrap` lẫn fallback đều không khởi chạy được daemon hoặc không xuất hiện `daemon.pid` hợp lệ trong 3 giây. | Tự fallback sang `start_daemon` nếu cần hoặc trả mã lỗi khác `0` kèm đường dẫn `daemon.log`. | `P0` |
| `E-macos-support-and-eye-comfort-003` | `FR-macos-support-and-eye-comfort-001` | `DevToolsActivePort missing` (Cảnh báo không dừng cài đặt): phiên Antigravity hiện tại mở không có cờ `--remote-debugging-port=0`. | In hướng dẫn rõ: bấm `Cmd+Q` rồi mở `Antigravity Theme` từ Spotlight/Applications hoặc chạy `agy-theme launch`. | `P0` |
| `E-macos-support-and-eye-comfort-004` | `FR-macos-support-and-eye-comfort-004` | `Theme payload build failed`: tệp `themes/themes.json` sai JSON hoặc thiếu asset. | Trả mã thoát khác `0` kèm chi tiết dòng/tệp lỗi; không ghi đè hỏng `dist/theme_payload.js` cũ nhờ `atomic_write`. | `P0` |

## 6. Giả định & Phụ thuộc

| Mã ID | Tham chiếu Nguồn | Nội dung Đặc tả / Quy tắc Nghiệp vụ | Điều kiện / Ràng buộc | Trạng thái / Ưu tiên |
|---|---|---|---|---|
| `FR-macos-support-and-eye-comfort-006` | `UR-macos-support-and-eye-comfort-001` | Người dùng đang chạy macOS 13+ (Apple Silicon hoặc Intel) với Antigravity v2.33.0+ cài ở `/Applications/Antigravity.app` và bộ công cụ Rust `cargo`/`rustc` khi tự biên dịch từ mã nguồn. | Không yêu cầu Node.js, Python hay quyền root (`sudo`); mọi tệp cài đặt nằm trong thư mục người dùng (`~/Library/LaunchAgents`, `~/Applications`). | `P0` |

## 7. Danh sách Câu hỏi Mở (Open Questions — OQ)

- [x] **OQ-001** *(Owner: `@teddy`, Đã chốt: `2026-11-24`, Mức độ: `Resolved`)*: Có giữ `"WLOP Cosmic"` ở vị trí đầu trong `themes/themes.json` và dùng `theme_config.json` làm mặc định `Bearrrr Eye-Comfort Dark` trên máy local không? — **Đã chốt Option A** để giữ tương thích 100% test upstream.

## 8. Checklist Tự Kiểm Tra Trước Khi Chuyển `approved`

- [x] 100% các mục yêu cầu đã gắn mã định danh chuẩn (`FR/BR/NFR/E-macos-support-and-eye-comfort-NNN`).
- [x] Không còn từ ngữ định tính mơ hồ; mọi trường hợp lỗi đều đã gắn mã `E-macos-support-and-eye-comfort-NNN`.
- [x] Toàn bộ câu hỏi mở `OQ-NNN` đã được giải quyết.

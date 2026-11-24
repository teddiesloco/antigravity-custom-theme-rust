---
type: user-flow
feature: "macos-support-and-eye-comfort"
status: approved
version: 1.0.0
owner: "@ags-ba"
updated: 2026-11-24
links:
  - "srs/spec.md"
  - "ascii-screen/_index.md"
  - "ascii-screen/main.md"
changelog:
  - "2026-11-24 | /init | Khởi tạo tài liệu chuẩn AGS-BA"
  - "2026-11-24 | /ba | Hoàn thiện luồng vận hành macOS installer, launcher và chọn theme"
---

# Sơ Đồ Luồng Vận Hành (User Flow) — macOS CDP Launcher, Modern LaunchAgent & Bearrrr Eye-Comfort Dark Theme

> **Mục đích tài liệu:** Đặc tả hành trình người dùng macOS từ lúc biên dịch/cài đặt đến khi mở Antigravity có sẵn theme dịu mắt và gỡ bỏ sạch khi cần.

## 1. Tổng quan Điểm vào (Entry Points) & Điểm kết thúc (Exit States)

| Mã ID | Tham chiếu Nguồn | Nội dung Đặc tả / Quy tắc Nghiệp vụ | Điều kiện / Ràng buộc | Trạng thái / Ưu tiên |
|---|---|---|---|---|
| `UFL-macos-support-and-eye-comfort-001` | `FR-macos-support-and-eye-comfort-001` | Điểm vào gồm: (1) chạy `agy-theme install` hoặc `scripts/install-unix.sh`, (2) bấm `Antigravity Theme.app` từ Spotlight/Dock hoặc chạy `agy-theme launch`, (3) xem trạng thái qua `agy-theme status` và gỡ qua `agy-theme uninstall`. Điểm kết thúc thành công là daemon đang chạy (< 2 MB RAM) và Antigravity hiển thị theme `Bearrrr Eye-Comfort Dark`. | Không yêu cầu `sudo`; không tự tắt phiên Antigravity đang chạy nếu không có cờ `--restart` (`BR-macos-support-and-eye-comfort-002`). | `P0` |

## 2. Sơ đồ Luồng Vận hành (Pele Flowchart)

```pele
flowchart TD
  Entry([Entry: agy-theme install]) --> Build[Dựng dist/theme_payload.js gồm Bearrrr Eye-Comfort Dark]
  Build -->|E-macos-support-and-eye-comfort-004| ErrBuild([Exit: lỗi build, giữ nguyên payload cũ])
  Build --> Agent[Cài LaunchAgent bằng launchctl bootstrap + tạo Antigravity Theme.app]
  Agent -->|E-macos-support-and-eye-comfort-002| ErrAgent([Exit: lỗi nạp LaunchAgent, xem daemon.log])
  Agent --> CheckCDP{Có DevToolsActivePort?}
  CheckCDP -->|Có| Done([Exit: Daemon active + Theme áp tức thì])
  CheckCDP -->|Chưa - E-macos-support-and-eye-comfort-003| Guide[Nhắc mở lại bằng Antigravity Theme.app hoặc agy-theme launch]
  Guide --> Launch[Người dùng mở Antigravity Theme.app / agy-theme launch]
  Launch -->|E-macos-support-and-eye-comfort-001| ErrApp([Exit: thiếu Antigravity.app])
  Launch -->|BR-macos-support-and-eye-comfort-001 thỏa| Done
```

## 3. Bảng Điều kiện Chuyển Bước & Quyền Truy cập

| Mã ID | Tham chiếu Nguồn | Nội dung Đặc tả / Quy tắc Nghiệp vụ | Điều kiện / Ràng buộc | Trạng thái / Ưu tiên |
|---|---|---|---|---|
| `UFL-macos-support-and-eye-comfort-003` | `FR-macos-support-and-eye-comfort-002` | Chỉ cho phép ghi tệp trong vùng người dùng hiện tại (`~/Library/LaunchAgents`, `~/Applications/Antigravity Theme.app`, và thư mục repo `dist/`, `daemon.pid`, `daemon.log`, `theme_config.json`). | Cấm ghi vào `/Applications/Antigravity.app` (`BR-macos-support-and-eye-comfort-001`). Nếu thiếu `Antigravity.app` khi chạy `launch` thì dừng với mã `E-macos-support-and-eye-comfort-001`. | `P0` |

## 4. Xử lý Cài Lại (Idempotency), Cập nhật App & Gỡ Bỏ

| Mã ID | Tham chiếu Nguồn | Nội dung Đặc tả / Quy tắc Nghiệp vụ | Điều kiện / Ràng buộc | Trạng thái / Ưu tiên |
|---|---|---|---|---|
| `UFL-macos-support-and-eye-comfort-004` | `FR-macos-support-and-eye-comfort-003` | Chạy `agy-theme install` nhiều lần sẽ `bootout` bản cũ, ghi đè sạch plist & `Antigravity Theme.app` và nạp lại bản mới không sinh tiến trình trùng. Khi Antigravity tự cập nhật, người dùng chỉ cần tiếp tục mở qua `Antigravity Theme.app` vì wrapper nằm ngoài bundle gốc. Chạy `agy-theme uninstall` dọn sạch LaunchAgent, plist và `Antigravity Theme.app`. | Không để sót tiến trình zombie hoặc lỗi khi file đã xoá sẵn (`E-macos-support-and-eye-comfort-002` chỉ báo khi cài mới không khởi động được). | `P0` |

## 5. Danh sách Câu hỏi Mở (Open Questions — OQ)

- [x] **OQ-001** *(Owner: `@teddy`, Đã chốt: `2026-11-24`, Mức độ: `Resolved`)*: Launcher macOS đặt tại `~/Applications/Antigravity Theme.app` để không cần quyền quản trị hệ thống và hiện ngay trong Spotlight.

## 6. Checklist Tự Kiểm Tra Trước Khi Chuyển `approved`

- [x] 100% các bước đã gắn mã `UFL-macos-support-and-eye-comfort-NNN` và liên kết về `FR/BR/E`.
- [x] Không còn placeholder hay câu hỏi mở `Blocking`.

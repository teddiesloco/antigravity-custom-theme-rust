---
type: screen-index
feature: "macos-support-and-eye-comfort"
status: approved
version: 1.0.0
owner: "@ags-ba"
updated: 2026-11-24
links:
  - "srs/spec.md"
  - "ascii-screen/main.md"
  - "html-design/prototype.html"
  - "api/api-map.md"
changelog:
  - "2026-11-24 | /init | Khởi tạo tài liệu chuẩn AGS-BA"
  - "2026-11-24 | /ba | Hoàn thiện danh mục bề mặt vận hành macOS & thẻ xem trước Bearrrr Eye-Comfort Dark"
---

# Danh Mục Bề Mặt (Screen Index) — macOS CDP Launcher, Modern LaunchAgent & Bearrrr Eye-Comfort Dark Theme

## 1. Bảng Master Danh mục Bề mặt (Screens Table)

| Mã ID | Tham chiếu Nguồn | Nội dung Đặc tả / Quy tắc Nghiệp vụ | Điều kiện / Ràng buộc | Trạng thái / Ưu tiên |
|---|---|---|---|---|
| `SCR-macos-support-and-eye-comfort-001` | `FR-macos-support-and-eye-comfort-001` | Bề mặt điều khiển macOS CLI & thẻ xem trước preset `Bearrrr Eye-Comfort Dark` (`ascii-screen/main.md`, `html-design/prototype.html`), gồm 5 thành phần có `data-testid` khớp chặt với hợp đồng local CLI trong `api/api-map.md`. | Đầy đủ 4 trạng thái `Loading`, `Empty`, `Error`, `Populated` và 3 ngưỡng Responsive (Desktop/Tablet/Mobile). | `P0` |

## 2. Liên kết Bề mặt <-> FR <-> User Story

| Mã ID | Tham chiếu Nguồn | Nội dung Đặc tả / Quy tắc Nghiệp vụ | Điều kiện / Ràng buộc | Trạng thái / Ưu tiên |
|---|---|---|---|---|
| `SCR-macos-support-and-eye-comfort-002` | `US-macos-support-and-eye-comfort-001` | `main.md` liên kết trực tiếp tới `FR-macos-support-and-eye-comfort-001..005`, `BR-macos-support-and-eye-comfort-001..004`, và các mã lỗi `E-macos-support-and-eye-comfort-001..004`. | Mọi `data-testid` trong `main.md` có mặt nguyên vẹn trong `html-design/prototype.html` và cột `Surface` của `api/api-map.md`. | `P0` |

## 3. Trạng thái Đồng bộ ASCII / HTML Prototype

| Mã ID | Tham chiếu Nguồn | Nội dung Đặc tả / Quy tắc Nghiệp vụ | Điều kiện / Ràng buộc | Trạng thái / Ưu tiên |
|---|---|---|---|---|
| `SCR-macos-support-and-eye-comfort-003` | `FR-macos-support-and-eye-comfort-004` | `ascii-screen/main.md` và `html-design/prototype.html` đã đồng bộ 100%, hiển thị đúng màu thực tế của `Bearrrr Eye-Comfort Dark` (`#e5a93b`, `#63c5da`, `#ebcb8b`, `#0d121a`, `#eceff4`, không tím chói). | Không sử dụng ảnh nền hoặc `backdrop-filter: blur()`. | `P0` |

## 4. Mô tả Tóm tắt Mục đích Từng Bề mặt

| Mã ID | Tham chiếu Nguồn | Nội dung Đặc tả / Quy tắc Nghiệp vụ | Điều kiện / Ràng buộc | Trạng thái / Ưu tiên |
|---|---|---|---|---|
| `SCR-macos-support-and-eye-comfort-004` | `FR-macos-support-and-eye-comfort-002` | Màn hình `main.md` đại diện cho trải nghiệm macOS: cài LaunchAgent + `Antigravity Theme.app`, mở Antigravity kèm cờ CDP, kiểm tra trạng thái PID/RAM/CDP, chọn preset `Bearrrr Eye-Comfort Dark`, và gỡ cài đặt sạch. | Khớp 1-1 với hành vi CLI `agy-theme` và modal chọn theme trong `engine/core.js`. | `P0` |

## 5. Danh sách Câu hỏi Mở (Open Questions — OQ)

- [x] **OQ-001** *(Owner: `@teddy`, Đã chốt: `2026-11-24`, Mức độ: `Resolved`)*: Đã thống nhất dùng bảng màu `bearrrr100x` Eye-Comfort Dark (Amber + Frost Cyan + Nord Slate, không tím).

## 6. Checklist Tự Kiểm Tra Trước Khi Chuyển `approved`

- [x] Đã đồng bộ ID `SCR-macos-support-and-eye-comfort-NNN` và `data-testid` giữa ASCII, HTML prototype và API map.
- [x] Không còn placeholder hay câu hỏi mở `Blocking`.

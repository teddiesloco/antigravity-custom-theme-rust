# Bề Mặt Vận Hành macOS & Preset Bearrrr Eye-Comfort Dark

## 1. Wireframe ASCII

Trạng thái bắt buộc: `Loading` · `Empty` · `Error` · `Populated`.
Responsive bắt buộc: Desktop >=1280px · Tablet 768-1279px · Mobile <768px.

```text
Populated (Desktop >=1280px)
+----------------------------------------------------------------------+
| Antigravity Theme — macOS Control & Bearrrr Eye-Comfort Dark         |
+----------------------------------+-----------------------------------+
| [1] Trạng thái Daemon & CDP      | [2] Preset: Bearrrr Eye-Comfort   |
|     PID: active | RAM: ~1.0 MB   |     Warm Amber #e5a93b            |
|     Port: DevToolsActivePort     |     Frost Cyan #63c5da            |
|                                  |     Zero-Image Pure CSS (0 ảnh)   |
+----------------------------------+-----------------------------------+
| [3] Cài macOS + LaunchAgent      | [4] Mở Antigravity (CDP)          |
| [5] Gỡ sạch LaunchAgent & App    |                                   |
+----------------------------------+-----------------------------------+
Loading: [1] Đang kiểm tra PID/RAM và DevToolsActivePort...
Empty:   [1] Daemon STOPPED · Chưa mở Antigravity với cờ CDP
Error:   [1] Banner mã lỗi E-macos-support-and-eye-comfort-001..004 + nút thử lại
Tablet 768-1279px: 1 cột xếp dọc (Trạng thái -> Preset -> 3 nút hành động)
Mobile <768px: 1 cột gọn, toàn bộ nút hành động full-width
```

## 2. Đặc tả Thành phần & Logic (Screen Specification)

| # | Thành phần (Element) | Kiểu (Type) | Quy tắc Hiển thị, Ràng buộc & Xử lý Sự kiện (Logic & Validation) |
|---|---|---|---|
| 1 | Bảng trạng thái daemon & CDP `data-testid="macos-status-panel"` | Status Card | • Nguồn dữ liệu từ `agy-theme status` (`daemon.pid`, `ps` RSS, `DevToolsActivePort`).<br>• Hiển thị PID, RAM thực tế và trạng thái cổng CDP; khi thiếu `DevToolsActivePort` hiển thị cảnh báo `E-macos-support-and-eye-comfort-003`. |
| 2 | Thẻ preset dịu mắt `data-testid="bearrrr-theme-card"` | Theme Card | • Đọc từ `themes/themes.json` qua `agy-theme themes` và biên dịch bởi `agy-theme build`.<br>• Hiển thị bảng màu `Bearrrr Eye-Comfort Dark` (Amber `#e5a93b`, Cyan `#63c5da`, Nord `#0d121a`, không tím, 0 ảnh); lỗi biên dịch báo `E-macos-support-and-eye-comfort-004`. |
| 3 | Nút cài đặt macOS `data-testid="macos-install-action"` | Button | • Gọi `agy-theme install`: dựng payload, nạp LaunchAgent bằng `launchctl bootstrap`, tạo `~/Applications/Antigravity Theme.app`, xác minh PID + RAM.<br>• Lỗi nạp LaunchAgent báo `E-macos-support-and-eye-comfort-002`; thiếu CDP cảnh báo `E-macos-support-and-eye-comfort-003`. |
| 4 | Nút mở Antigravity kèm CDP `data-testid="macos-launch-action"` | Button | • Gọi `agy-theme launch` (hoặc mở `~/Applications/Antigravity Theme.app`): đảm bảo daemon đang chạy rồi chạy `open -a Antigravity --args --remote-debugging-port=0` không phá `app.asar`.<br>• Không tìm thấy ứng dụng báo `E-macos-support-and-eye-comfort-001`. |
| 5 | Nút gỡ cài đặt sạch `data-testid="macos-uninstall-action"` | Button | • Gọi `agy-theme uninstall`: `launchctl bootout`, xoá plist và `~/Applications/Antigravity Theme.app`, dừng daemon và xoá CSS theme trên trang CDP đang mở.<br>• Chạy lặp lại an toàn (idempotent). |

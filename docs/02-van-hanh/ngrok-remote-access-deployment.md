---
title: "Hướng dẫn Vận hành Triển khai Đường hầm Truy cập Từ xa (ngrok Remote Access Tunnel)"
updated: 2026-09-14
commit: d4e1f7c
status: living
owns:
  - ngrok-remote-access-deployment
  - treasury-workbench-ingress
covers:
  - deploy/ngrok/ngrok-treasury-workbench.yml
  - scripts/deploy-ngrok-tunnel.ps1
  - scripts/ram-guard.ps1
  - liva-ui/src/BankingApp.vue
---
# Hướng dẫn Vận hành Triển khai Đường hầm Truy cập Từ xa (ngrok Remote Access Tunnel)
## LIVA Banking Harness — Bàn làm việc Kế toán Nguồn vốn (Treasury Workbench Ingress)

[⬆ Mục lục](../README.md) · [Cấu hình Môi trường](01-cau-hinh-va-bien-moi-truong.md) · [Bản vẽ Kiến trúc](../01-kien-truc/system-architecture-blueprint.md) · [Lộ trình Master Remake](../06-ke-hoach/master-remake-roadmap.md) · [Thông số Kết nối Phase 2](../01-kien-truc/phase2-connectivity-specifications.md)

---

## 1. Tuyên bố Mục đích & Tổng quan Kiến trúc

### 1.1 Mục đích Triển khai
Trong môi trường doanh nghiệp và ngân hàng, Bàn làm việc Kế toán Nguồn vốn (**Treasury & Reconciliation Workbench**) của LIVA Banking Harness cần được truy cập từ xa bởi Ban Giám đốc Tài chính (CFO), Kế toán trưởng (Chief Accountant) và Kiểm toán viên độc lập khi làm việc lưu động, phê duyệt ngoại lệ hai pha (Maker-Checker HITL) trên thiết bị di động hoặc máy tính bảo mật bên ngoài mạng LAN nội bộ.

Để đáp ứng nhu cầu này mà **không cần can thiệp mở cổng Firewall doanh nghiệp (Inbound Port Forwarding)** và **không phơi nhiễm trực tiếp CSDL kế toán**, LIVA Banking Harness triển khai kiến trúc **Đường hầm Ingress kết thúc TLS (TLS-Terminated Ingress Tunnel)** qua giải pháp ngrok Enterprise:
- **Tài khoản vận hành**: `duong.na.2719@aptechlearning.edu.vn`
- **Domain gán cố định**: `rd_3JHcwHJK1iUTdwx1D22XRMeUA5K` (`rd_3JHcwHJK1iUTdwx1D22XRMeUA5K.ngrok-free.app`)
- **Dịch vụ Upstream mục tiêu**: Cổng giao diện Treasury Workbench (`http://127.0.0.1:5173` đối với bản Vite Dev, `http://127.0.0.1:4173` đối với bản Vite Preview, hoặc `http://127.0.0.1:8080` đối với bản Web Gateway phát hành On-Premise).

### 1.2 Mô hình Ranh giới An ninh: Ingress Tunnel vs. Zero-Egress Local Core

```
+---------------------------------------------------------------------------------------------------------+
|                                    INTERNET & NGƯỜI DÙNG TỪ XA                                           |
|       CFO / Kế toán trưởng duyệt ngoại lệ HITL qua Mobile / Tablet / Laptop ngoài văn phòng               |
+----------------------------------------------------+----------------------------------------------------+
                                                     │ HTTPS (TLS 1.3, HSTS Preload)
                                                     ▼
+---------------------------------------------------------------------------------------------------------+
|                                 NGROK EDGE CLOUD GATEWAY INFRASTRUCTURE                                 |
|   • Miền gán: rd_3JHcwHJK1iUTdwx1D22XRMeUA5K.ngrok-free.app (duong.na.2719@aptechlearning.edu.vn)       |
|   • Chặn Clickjacking (X-Frame-Options: DENY), Chặn Sniffing (X-Content-Type-Options: nosniff)          |
|   • Giới hạn quyền thiết bị (Permissions-Policy: camera=(), mic=(), geo=(), pay=())                     |
+----------------------------------------------------+----------------------------------------------------+
                                                     │ Mã hóa Kênh ngrok Secure Agent Tunnel
                                                     ▼
+=========================================================================================================+
|                           MÁY CHỦ BẢO MẬT DOANH NGHIỆP (LOCAL AIR-GAPPED ON-PREMISE)                    |
|                                                                                                         |
|  [ngrok Agent Process (deploy-ngrok-tunnel.ps1)] ──(HTTP Reverse Proxy)──> [Treasury Workbench UI]     |
|       │                                                                       (Port 5173 / 8080)        |
|       │ Local Web Inspector (http://127.0.0.1:4040)                                   │                 |
|       │ Giám sát Request / Response / IP / Headers                                    │ IPC / Local WS  |
|                                                                                       ▼                 |
|  [HÀNG RÀO PHÁP LÝ & AN NINH DỮ LIỆU ZERO-EGRESS]                               [LIVA NATIVE CORE]      |
|  ------------------------------------------------                               • SQLite WAL Cục bộ     |
|  • TUYỆT ĐỐI KHÔNG rò rỉ dữ liệu tài chính ra Internet                           • SLM Cục bộ (GGUF)     |
|  • Sao kê gốc, số dư tài khoản, mã CCCD, số hóa đơn chỉ nằm tại RAM máy cục bộ • Merkle Audit Chain   |
|  • Chỉ truyền tải giao diện web HTML/CSS/JS được mã hóa ra kênh Ingress          • Zero Egress Netfilter|
+=========================================================================================================+
```

---

## 2. Ranh giới Pháp lý & An toàn Thông tin (Compliance Boundaries)

### 2.1 Tuân thủ Nghị định 13/2023/NĐ-CP (Bảo vệ Dữ liệu Cá nhân & Hồ sơ DPIA)
1. **Nguyên tắc Bất khả Egress Dữ liệu Cá nhân (Zero PII Egress Principle)**:
   - Toàn bộ dữ liệu nhạy cảm bao gồm: Số định danh cá nhân (CCCD 12 số), Số tài khoản ngân hàng, Tên người thụ hưởng, và Nội dung diễn giải giao dịch được xử lý hoàn toàn trong bộ nhớ RAM và CSDL SQLite WAL cục bộ On-Premise của máy chủ doanh nghiệp.
   - Kênh ngrok chỉ đóng vai trò là **kênh dẫn truyền hình ảnh hiển thị (Ingress Presentation Layer)**; tuyệt đối không gửi các bản ghi sao kê dạng bảng thô hay tệp cơ sở dữ liệu lên đám mây của bên thứ ba.
2. **Khử định danh & Mặt nạ hiển thị (Client-Side Redaction & Masking)**:
   - Module `compliance::sanitizer` trên lõi native tự động ẩn 6 chữ số giữa của số tài khoản ngân hàng (`9704********1234`) và che hoàn toàn chuỗi số định danh trước khi đẩy lên giao diện web từ xa.

### 2.2 Tuân thủ Thông tư 09/2020/TT-NHNN (An toàn Hệ thống Thông tin Ngân hàng)
1. **Cưỡng chế Giao thức Mã hóa Vận chuyển (Encryption in Transit)**:
   - Toàn bộ lưu lượng truy cập qua ngrok được mã hóa bắt buộc bằng **TLS 1.3 / TLS 1.2**.
   - Cưỡng chế header HTTP bảo mật cấp cao:
     * `Strict-Transport-Security: max-age=31536000; includeSubDomains; preload`
     * `Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self'; img-src 'self' data: asset:; connect-src 'self' wss:; object-src 'none'; frame-ancestors 'none';`
     * `X-Frame-Options: DENY`: Chống tuyệt đối tấn công Clickjacking chiếm quyền click nút duyệt lệnh chi tiền.
     * `X-Content-Type-Options: nosniff`: Ngăn chặn trình duyệt tự ý đoán định dạng MIME.
2. **Nguyên tắc Bốn Mắt & Token Phê duyệt HITL Dùng Một Lần (Maker-Checker Protocol)**:
   - Người dùng từ xa chỉ có quyền xem xét chênh lệch (Diff Preview) và ký duyệt ngoại lệ thông qua Token UUIDv4 dùng một lần kèm mã hóa HMAC-SHA256. Mọi hành động duyệt đều được ghi nhận vào `banking_audit_chain` với dấu vết IP nguồn và dấu thời gian UNIX chính xác.

---

## 3. Cấu hình Kỹ thuật Chi tiết (`deploy/ngrok/ngrok-treasury-workbench.yml`)

Tệp cấu hình ngrok Agent phiên bản 3 được chuẩn hóa tại `deploy/ngrok/ngrok-treasury-workbench.yml`:

```yaml
# ==============================================================================
# LIVA Banking Harness — ngrok Remote Access Tunnel Configuration
# Deployment Target: Treasury & Reconciliation Workbench (Remote Ingress)
# ==============================================================================
# Account: duong.na.2719@aptechlearning.edu.vn
# Domain: rd_3JHcwHJK1iUTdwx1D22XRMeUA5K
# Upstream Service: Treasury Workbench (Default: 5173 [Vite Dev], 4173 [Preview], 8080 [Prod])
# Compliance: Decree 13/2023/ND-CP (DPIA) & Circular 09/2020/TT-NHNN
# ==============================================================================

version: "3"

agent:
  authtoken: 31tkcGzS8HjU6flkQnMhuqJYMZM_3R8LtfJScAEbRpSZdEPnf
  metadata: "LIVA Banking Treasury Workbench - Operator: duong.na.2719@aptechlearning.edu.vn"

tunnels:
  treasury-workbench:
    proto: http
    addr: 5173
    domain: rd_3JHcwHJK1iUTdwx1D22XRMeUA5K
    inspect: true
    metadata: '{"app": "liva-banking", "component": "treasury-workbench", "env": "remote-preview", "domain_id": "rd_3JHcwHJK1iUTdwx1D22XRMeUA5K"}'
    traffic_policy:
      on_http_response:
        - actions:
            - type: add-headers
              config:
                headers:
                  Strict-Transport-Security: "max-age=31536000; includeSubDomains; preload"
                  X-Frame-Options: "DENY"
                  X-Content-Type-Options: "nosniff"
                  X-XSS-Protection: "1; mode=block"
                  Referrer-Policy: "strict-origin-when-cross-origin"
                  Content-Security-Policy: "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self'; img-src 'self' data: asset:; connect-src 'self' wss:; object-src 'none'; frame-ancestors 'none';"
                  Permissions-Policy: "camera=(), microphone=(), geolocation=(), payment=()"
                  Cache-Control: "no-store, no-cache, must-revalidate, proxy-revalidate"
```

### Các Thông số Cấu hình Trọng yếu:
| Trường Cấu hình | Giá trị Thiết lập | Ý nghĩa Kỹ thuật |
|---|---|---|
| `version` | `"3"` | Sử dụng chuẩn cú pháp cấu hình ngrok Agent v3 hiện đại. |
| `agent.authtoken` | `31tkcGzS8...` | Token định danh gắn liền với tài khoản `duong.na.2719@aptechlearning.edu.vn`. |
| `proto` | `http` | Chuyển tiếp lưu lượng HTTP/HTTPS và WebSocket nâng cấp (WSS). |
| `addr` | `5173` | Cổng upstream cục bộ của Treasury Workbench. |
| `domain` | `rd_3JHcwHJK1iUTdwx1D22XRMeUA5K` | Tài nguyên miền cố định đã đăng ký trên ngrok dashboard. |
| `inspect` | `true` | Kích hoạt bộ kiểm định lưu lượng nội bộ qua giao diện quản trị `127.0.0.1:4040`. |
| `traffic_policy` | Response Headers Policy | Tự động tiêm các tiêu đề an ninh ngân hàng vào mọi phản hồi HTTP trả về client. |

---

## 4. Hướng dẫn Tự động hóa Vận hành (`scripts/deploy-ngrok-tunnel.ps1`)

Kịch bản PowerShell quản trị vòng đời đường hầm được đặt tại `scripts/deploy-ngrok-tunnel.ps1`. Kịch bản này tích hợp đầy đủ các chốt kiểm định RAM Guardrails, kiểm tra cổng cục bộ, quản lý tiến trình nền (PID File), và trích xuất nhật ký chi tiết.

### 4.1 Bảng Lệnh Vận hành Chuẩn (Operator Command Cheat Sheet)

```powershell
# 1. Kiểm tra toàn diện điều kiện tiên quyết (Pre-flight Check)
powershell -ExecutionPolicy Bypass -File scripts/deploy-ngrok-tunnel.ps1 -Action test

# 2. Khởi động đường hầm truy cập từ xa (Chạy ngầm với PID Tracking)
powershell -ExecutionPolicy Bypass -File scripts/deploy-ngrok-tunnel.ps1 -Action start

# 3. Khởi động với cổng thay đổi (Ví dụ: cổng Preview 4173 hoặc Production 8080)
powershell -ExecutionPolicy Bypass -File scripts/deploy-ngrok-tunnel.ps1 -Action start -Port 8080

# 4. Kiểm tra trạng thái hoạt động & URL công khai đang mở
powershell -ExecutionPolicy Bypass -File scripts/deploy-ngrok-tunnel.ps1 -Action status

# 5. Mở bảng điều khiển Web Inspector kiểm tra gói tin HTTP
powershell -ExecutionPolicy Bypass -File scripts/deploy-ngrok-tunnel.ps1 -Action inspect

# 6. Dừng an toàn toàn bộ các phiên tunnel ngrok đang chạy
powershell -ExecutionPolicy Bypass -File scripts/deploy-ngrok-tunnel.ps1 -Action stop
```

### 4.2 Cơ chế Kiểm soát RAM Pre-flight Guardrail
Theo quy tắc an toàn tài nguyên trong `AGENTS.md`, mọi thao tác triển khai dịch vụ mạng hoặc biên dịch đều phải qua chốt chặn bộ nhớ:
- Kịch bản tự động triệu gọi `scripts/ram-guard.ps1`.
- Cưỡng chế: Hệ thống phải có **tối thiểu 4.0 GB RAM khả dụng (Free RAM)** và **mức tải bộ nhớ không vượt quá 80%**.
- Nếu ngưỡng an toàn bị vi phạm, lệnh triển khai sẽ bị hủy tức thì (`exit 1`) nhằm bảo vệ sự ổn định của hệ điều hành và CSDL SQLite WAL.

---

## 5. Giám sát Lưu lượng & Giao diện Thanh tra (Web Inspector Port 4040)

Khi đường hầm được khởi chạy với tham số `inspect: true`, ngrok mở một máy chủ giám sát nội bộ tại địa chỉ:
$$\text{URL Quản trị}: \mathbf{http://127.0.0.1:4040}$$

### 5.1 Các Khả năng Giám sát Thời gian Thực:
1. **Request/Response Inspection**:
   - Xem chi tiết từng yêu cầu HTTP gửi từ thiết bị di động của CFO về máy chủ.
   - Kiểm tra mã phản hồi HTTP (200 OK, 304 Not Modified, 403 Forbidden).
   - Kiểm định sự hiện diện của các tiêu đề an ninh (`Strict-Transport-Security`, `X-Frame-Options`, `Content-Security-Policy`).
2. **Replay Requests**:
   - Cho phép Quản trị viên hệ thống phát lại (replay) một yêu cầu API đối soát để tái hiện lỗi mà không cần thao tác lại trên thiết bị từ xa.
3. **Đo lường Độ trễ & Thông lượng (Latency & Throughput)**:
   - Theo dõi thời gian phản hồi (Round-Trip Time RTT, p50, p90, p99).
   - Số lượng kết nối đồng thời và tốc độ truyền tải byte/giây.

---

## 6. Sổ tay Xử lý Sự cố & Khắc phục Lỗi (Troubleshooting Runbook)

### Sự cố 1: Lỗi `ERR_NGROK_3200` — Upstream Service Connection Refused
- **Hiện tượng**: Truy cập URL công khai trên trình duyệt nhận được thông báo lỗi `ERR_NGROK_3200: The connection to http://localhost:5173 was refused by the webserver`.
- **Nguyên nhân**: Đường hầm ngrok đã mở thành công, nhưng ứng dụng Treasury Workbench (`liva-ui`) chưa được khởi động trên cổng 5173.
- **Biện pháp xử lý**:
  ```powershell
  # Bước 1: Mở một PowerShell riêng biệt và khởi động giao diện Vite
  cd e:\Project\01_AI_Agents\LIVA_Banking\liva-ui
  npm run dev

  # Bước 2: Kiểm tra lại cổng 5173 đã LISTENING
  Get-NetTCPConnection -LocalPort 5173 -State Listen
  ```

### Sự cố 2: Lỗi `ERR_NGROK_313` / `ERR_NGROK_314` — Custom Subdomain / Hostname Restriction
- **Hiện tượng**: Log ghi nhận `Only paid plans may create endpoints with custom subdomains... ERR_NGROK_313`.
- **Nguyên nhân**: Định dạng chuỗi domain trong file YAML sử dụng tên miền phụ không khớp với gói cước tài khoản ngrok hiện tại.
- **Biện pháp xử lý**:
  * Kiểm tra đúng cấu hình tài nguyên `rd_3JHcwHJK1iUTdwx1D22XRMeUA5K` trong `deploy/ngrok/ngrok-treasury-workbench.yml`.
  * Trong trường hợp cần kiểm thử nhanh khẩn cấp với URL tạm thời (Ephemeral URL), sử dụng cờ:
    ```powershell
    powershell -ExecutionPolicy Bypass -File scripts/deploy-ngrok-tunnel.ps1 -Action start -FallbackEphemeral
    ```

### Sự cố 3: Lỗi Port 4040 bị xung đột hoặc ngrok chạy ngầm không tắt được
- **Hiện tượng**: Kịch bản báo `[WARN] ngrok tunnel is ALREADY running!` nhưng giao diện không truy cập được.
- **Biện pháp xử lý**:
  ```powershell
  # Dừng dứt điểm toàn bộ tiến trình ngrok bằng lệnh stop
  powershell -ExecutionPolicy Bypass -File scripts/deploy-ngrok-tunnel.ps1 -Action stop

  # Xóa file PID mồ côi nếu có
  Remove-Item "deploy/ngrok/.ngrok.pid" -Force -ErrorAction SilentlyContinue
  ```

### Sự cố 4: Kích hoạt Chốt chặn Bộ nhớ RAM Guard (`exit 1`)
- **Hiện tượng**: Kịch bản in thông báo đỏ: `[RAM-GUARD] CRITICAL: Free RAM (< 4.0GB) is below safety threshold`.
- **Nguyên nhân**: Máy tính đang mở nhiều ứng dụng nặng chiếm dụng bộ nhớ RAM.
- **Biện pháp xử lý**:
  * Tắt các ứng dụng đồ họa hoặc máy ảo không cần thiết để giải phóng dung lượng RAM $\ge 4.0\text{ GB}$.
  * Chạy lại kiểm tra: `powershell -ExecutionPolicy Bypass -File scripts/ram-guard.ps1`.

---

## 7. Kết luận & Khuyến nghị Vận hành

Việc triển khai **ngrok Remote Access Tunnel** cho **LIVA Treasury Workbench** kết hợp giữa tính linh hoạt của điện toán đám mây và sự an toàn tuyệt đối của mô hình On-Premise Air-Gapped. Bằng cách giữ toàn bộ cơ sở dữ liệu và lõi tính toán đối soát trong ranh giới mạng nội bộ, giải pháp hoàn toàn thỏa mãn các yêu cầu khắt khe nhất của **Nghị định 13/2023/NĐ-CP** và **Thông tư 09/2020/TT-NHNN**, sẵn sàng phục vụ các phiên trình diễn giải pháp tại Demo Day INNOSTART 2026.

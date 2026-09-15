---
title: P100_P106_System_Settings_PRD
tags:
  - liva/banking
  - liva/settings
  - liva/rbac
  - liva/openbanking
  - liva/erp
author: LIVA Core Architecture Team
last_update: 2026-09-15T22:10:00+07:00
---

# P100–P106: Cài Đặt Hệ Thống, Quản Trị Phân Quyền RBAC & Kết Nối Ngân Hàng / ERP (PRD)

Tài liệu đặc tả kỹ thuật kiến trúc cho cụm phân hệ **P100–P106: Quản trị cấu hình hệ thống, Phân quyền người dùng theo vai trò (RBAC), Cổng kết nối Open Banking, Cầu nối đồng bộ ERP, Tham số dung sai đối soát và Bảo mật chữ ký số HSM** trong LIVA Banking Harness.

---

## 1. Mục Tiêu & Cơ Sở Pháp Lý

Cụm phân hệ P100–P106 thiết lập hạ tầng quản trị tập trung bảo vệ tính toàn vẹn và bảo mật của toàn bộ quy trình đối soát ngân hàng:
- **Thông tư 09/2020/TT-NHNN**: Quy định về an toàn hệ thống thông tin trong hoạt động ngân hàng, bắt buộc cơ chế kiểm soát kép (Dual Control / Maker-Checker) và quản lý phân quyền chặt chẽ.
- **Quy tắc Phân Nhiệm (Segregation of Duties - SoD)**: Nghiêm cấm tuyệt đối một người dùng vừa nắm quyền lập lệnh/đối soát (Maker) vừa nắm quyền phê duyệt giải ngân (Checker).
- **Nghị định 13/2023/NĐ-CP**: Giám sát Zero Cloud Egress, quản lý khóa bảo mật trên phần cứng HSM / USB Token On-Premise.
- **Tiêu chuẩn ISO 20022 & Open Banking API**: Chuẩn hóa các kết nối ngân hàng B2B (Vietcombank, Techcombank, BIDV, MBBank) và các phần mềm ERP phổ biến (MISA AMIS, FAST Accounting, SAP).

---

## 2. P101: Quản Trị Người Dùng & Ma Trận Phân Quyền RBAC

### 2.1. Danh Mục Vai Trò (Roles)
1. **`ADMIN` (Quản trị hệ thống)**: Cấu hình kết nối, quản lý tài khoản, giám sát sao lưu. Không có quyền duyệt giao dịch tài chính.
2. **`MAKER` (Kế toán viên / Đối soát viên)**: Nhập sao kê, chạy đối soát tự động, giải quyết lệch Tier 2 & Tier 3, lập dự thảo lệnh chi.
3. **`CHECKER` (Kế toán trưởng / Giám đốc tài chính)**: Phê duyệt đề xuất đối soát từ hàng đợi cách ly P44, ký duyệt lệnh giải ngân thanh toán P62.
4. **`COMPLIANCE_OFFICER` (Cán bộ tuân thủ AML)**: Tiếp nhận cảnh báo rửa tiền P81, ký số và gửi báo cáo STR Phụ lục II TT 09/2023.
5. **`AUDITOR` (Kiểm toán viên nội bộ / độc lập)**: Quyền chỉ đọc (Read-Only) toàn bộ sổ cái, báo cáo tài chính F01, B02, B03 và xác thực gốc Merkle.

### 2.2. Ma Trận Quyền Hạn (Permissions Matrix)
| Nghiệp Vụ | ADMIN | MAKER | CHECKER | COMPLIANCE | AUDITOR |
|---|---|---|---|---|---|
| Nhập sao kê & chạy đối soát | ✕ | ✓ | ✕ | ✕ | ✕ |
| Lập đề xuất khớp thủ công (HITL) | ✕ | ✓ | ✕ | ✕ | ✕ |
| Duyệt đề xuất đối soát P44 | ✕ | ✕ | ✓ | ✕ | ✕ |
| Lập lệnh giải ngân ngân quỹ P62 | ✕ | ✓ | ✕ | ✕ | ✕ |
| Ký duyệt giải ngân tiền ngân hàng | ✕ | ✕ | ✓ | ✕ | ✕ |
| Ký duyệt báo cáo STR (Phòng chống rửa tiền) | ✕ | ✕ | ✕ | ✓ | ✕ |
| Xem sổ cái & báo cáo tài chính VAS | ✓ | ✓ | ✓ | ✓ | ✓ |
| Cấu hình API ngân hàng & ERP | ✓ | ✕ | ✕ | ✕ | ✕ |

---

## 3. P102: Cấu Hình Kết Nối Open Banking API

Hỗ trợ 4 ngân hàng trọng điểm tại Việt Nam:
1. **Vietcombank (VCB B2B API)**:
   - Endpoint: `https://api.vietcombank.com.vn/b2b/v2/statements`
   - Xác thực: mTLS Certificate + OAuth2 Client Credentials + HMAC-SHA256 Request Signing.
2. **Techcombank (TCB Corporate Banking API)**:
   - Endpoint: `https://corporate-api.techcombank.com.vn/v1/cash-management`
   - Xác thực: X.509 Digital Certificate + API Secret Key.
3. **BIDV (BIDV iBank Open API)**:
   - Endpoint: `https://open.bidv.com.vn/api/corporate/v3`
4. **MBBank (MBB Open Banking)**:
   - Endpoint: `https://api.mbbank.com.vn/corporate/v1`

Hệ thống cung cấp cơ chế chuyển đổi tức thì giữa chế độ Giả lập an toàn (Mock Sandbox) và Môi trường sản xuất (Production Live), hiển thị độ trễ ping và nhịp tim (Heartbeat latency).

---

## 4. P103: Cấu Hình Cầu Nối Đồng Bộ ERP

1. **MISA AMIS Accounting**:
   - Cổng giao tiếp: REST OpenAPI (JSON format).
   - Đồng bộ chứng từ thu/chi tự động qua endpoint `/api/v1/voucher/bank-payment`.
2. **FAST Accounting**:
   - Cổng giao tiếp: FAST Web Service API (XML format).
   - Đồng bộ bảng cân đối tài khoản và sổ cái tiền gửi ngân hàng.
3. **SAP Business One (SAP B1)**:
   - Cổng giao tiếp: SAP DI-Server / Service Layer.

---

## 5. P104: Tham Số Dung Sai & Quy Tắc Đối Soát

| Tham Số | Mặc Định | Mô Tả Nghiệp Vụ |
|---|---|---|
| Ngưỡng lệch tuyệt đối (Tier 1) | $0\text{ VND}$ | Khớp chính xác 100% số tiền đến từng đồng |
| Dung sai phí ngân hàng tối đa (Tier 2) | $50{,}000\text{ VND}$ | Tự động tách phí giao dịch khi số tiền lệch $\le 50\text{K}$ |
| Độ sâu giải thuật chia tách (Tier 3 Split Solver) | $k = 8$ | Tìm kiếm tổ hợp con (Subset Sum) tối đa 8 hóa đơn |
| Thời gian hết hạn hàng đợi cách ly P44 | $900\text{ giây (15p)}$ | Tự động trả về hàng đợi nếu Checker không thao tác |
| Ngưỡng cảnh báo AML giá trị lớn | $400{,}000{,}000\text{ VND}$ | Ngưỡng luật định theo Quyết định 11/2023/QĐ-TTg |

---

## 6. P105 & P106: Bảo Mật Chữ Ký Số HSM & Sao Lưu Thảm Họa

- **Quản lý khóa ký số X.509**: Tích hợp các nhà cung cấp chứng thư số công cộng hợp chuẩn Việt Nam (VNPT-CA, Viettel-CA, FPT-CA). Mọi thao tác phê duyệt của Checker đều được ký số lưu vào chuỗi Merkle.
- **Bảo chứng Merkle Log bất biến**: Cập nhật SHA-256 tree root sau mỗi chu kỳ ghi sổ.
- **Sao lưu phục hồi thảm họa (Disaster Recovery)**: Sao lưu tự động định kỳ (Snapshot + WAL logs) với cơ chế mã hóa AES-256 tại chỗ (On-Premise), cam kết RPO $< 1\text{ phút}$, RTO $< 30\text{ giây}$.

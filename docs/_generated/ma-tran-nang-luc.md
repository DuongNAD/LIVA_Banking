---
title: "Ma trận năng lực LIVA Banking Harness"
updated: 2026-09-13
commit: 3688b5f
status: index
owns:
  - ma-tran-nang-luc-banking-harness
covers:
  - docs/_data/capabilities.json
  - scripts/docs-capabilities.mjs
---
# Ma trận năng lực LIVA Banking Harness

[⬆ Mục lục](../README.md) · [Tầm nhìn](../00-san-pham/tam-nhin-banking-harness.md) · [Lộ trình Master Remake](../06-ke-hoach/master-remake-roadmap.md)

> File này được sinh từ [`docs/_data/capabilities.json`](../_data/capabilities.json).
> Không sửa tay. Chạy `npm run docs:capabilities` để sinh lại hoặc
> `npm run docs:capabilities:check` để kiểm tra drift.

## Tóm tắt

| Trạng thái | Số năng lực |
|---|---:|
| [OK] | 7 |
| [MỘT PHẦN] | 3 |
| [THỬ NGHIỆM] | 0 |
| [THIẾU] | 0 |
| [BỊ CHẶN] | 0 |
| [CÔ LẬP/LOẠI BỎ] | 9 |
| **Tổng** | **19** |

## Danh sách

| ID | Năng lực | Trạng thái | Ưu tiên | Đích | Hiện trạng | Bằng chứng | Mốc tiếp theo |
|---|---|---|---|---|---|---|---|
| `banking.statement-parsing` | Bóc tách sao kê đa định dạng (Universal Multi-Format Ingestion) | [MỘT PHẦN] | P0 | GĐ1 | Đã hoàn thành parser cho 3 ngân hàng lớn nhất: VCB (Excel ô gộp), TCB (CSV BOM/Napas), BIDV (PDF bảng biểu); tự động sniff định dạng file; đang mở rộng sang 32+ ngân hàng còn lại và chuẩn SWIFT MT940. | `liva-native-core/src/banking/parser/vcb_excel.rs`<br>`liva-native-core/src/banking/parser/tcb_csv.rs`<br>`liva-native-core/src/banking/parser/bidv_pdf.rs`<br>`liva-native-core/src/banking/parser/mod.rs`<br>`liva-native-core/src/banking/tests.rs` | Hoàn thiện parser chuẩn SWIFT MT940 (:61:, :86:) và ISO 20022 CAMT.053 XML. |
| `banking.deterministic-reconciliation` | Động cơ đối soát xác định 3 chiều (Deterministic 3-Way Reconciliation) | [OK] | P0 | GĐ1 | Đã hoàn thiện giải thuật so khớp 3 tầng: Tier 1 Hash match O(1), Tier 2 Fuzzy heuristic (xử lý sai lệch phí 11.000đ), Tier 3 Composite split allocation solver; kiểm soát bất biến số học kế toán kép, 0% ảo giác. | `liva-native-core/src/banking/reconciliation/hash_matcher.rs`<br>`liva-native-core/src/banking/reconciliation/fuzzy_matcher.rs`<br>`liva-native-core/src/banking/reconciliation/split_solver.rs`<br>`liva-native-core/src/banking/reconciliation/mod.rs`<br>`liva-native-core/src/banking/tests.rs` | Tối ưu hóa bảng băm AHash cho lô lớn hơn 100.000 dòng giao dịch phân tán đa tài khoản. |
| `banking.erp-connectors` | Kết nối & Đồng bộ ERP Doanh nghiệp (ERP Integration Connectors) | [MỘT PHẦN] | P0 | GĐ2 | Đã có mô hình dữ liệu InternalLedgerEntry và bộ nạp fixture JSON; cơ chế Closed-Loop Confirmation đã thiết kế; chưa có live adapter REST/ODBC cho MISA AMIS và FAST Business Online. | `liva-native-core/src/banking/models.rs`<br>`fixtures/statements/open_invoices.json`<br>`liva-native-core/src/banking/tests.rs` | Xây dựng module xuất file chứng từ kế toán chuẩn XML/Excel tương thích 1-click import cho MISA và FAST. |
| `security.zero-egress` | Rào chắn an ninh cục bộ & Cách ly mạng (Zero-Egress Guardrails) | [OK] | P0 | GĐ1 | Cam kết 100% Zero Cloud Leakage; bộ lọc PII khử định danh CCCD (12 số), số tài khoản; kiểm toán mạng xác nhận 0 byte rò rỉ ra ngoài loopback 127.0.0.1; mã hóa AES-256-GCM niêm phong DPAPI. | `liva-native-core/src/banking/compliance/sanitizer.rs`<br>`liva-native-core/src/banking/compliance/security.rs`<br>`liva-native-core/src/crypto.rs`<br>`liva-native-core/src/banking/tests.rs` | Hoàn thiện hồ sơ Đánh giá Tác động Chuyển dữ liệu (DPIA) gửi Cục A05 theo Nghị định 13. |
| `compliance.audit-ledger` | Sổ cái kiểm toán chống giả mạo (Immutable Audit Ledger) | [OK] | P0 | GĐ1 | Đã hoàn thành chuỗi log kiểm toán forward-chaining sử dụng HMAC-SHA256; phát hiện tức thì hành vi can thiệp trái phép cơ sở dữ liệu SQLite cục bộ; đáp ứng Thông tư 09/2020/TT-NHNN Cấp 3-5. | `liva-native-core/src/banking/compliance/audit_ledger.rs`<br>`liva-native-core/src/banking/tests.rs` | Tích hợp chữ ký số phần cứng PKI / Token USB của Kế toán trưởng vào chuỗi audit. |
| `treasury.cashflow-sentinel` | Tháp canh giám sát & Dự báo dòng tiền (Treasury Sentinel & Forecast) | [MỘT PHẦN] | P1 | GĐ2 | Đã có giao diện hiển thị số dư đa ngân hàng và biểu đồ chênh lệch; logic tính toán số dư thực tế hoạt động; mô hình dự báo Rolling Cash Flow 30-90 ngày đang được hoàn thiện thuật toán hồi quy. | `liva-ui/src/stores/bankingStore.ts`<br>`liva-ui/src/components/banking/ReconciliationTrendChart.vue`<br>`liva-native-core/src/commands/banking.rs` | Tự động kích hoạt cảnh báo thâm hụt số dư trước 24-48 giờ dựa trên lịch công nợ phải thu/phải trả. |
| `runtime.native-core` | Lõi Rust Native thống nhất (liva-native-core) | [OK] | P0 | GĐ0 | Vận hành hoàn toàn bằng mã máy Rust compiled, nhúng SQLite WAL, llama.cpp in-process và ONNX Runtime; quản lý bộ nhớ nghiêm ngặt RAM <= 4GB. | `liva-native-core/src/boot.rs`<br>`liva-native-core/src/lib.rs`<br>`liva-desktop/src-tauri/src/lib.rs` | Duy trì CI build profile release phục vụ môi trường On-Premise ngân hàng. |
| `data.persistence-integrity` | Dữ liệu bền vững và bảo mật tầng cơ sở dữ liệu | [OK] | P0 | GĐ0 | SQLite schema có WAL mode, dedicated writer actor, mã hóa trường nhạy cảm bằng AES-256-GCM v2 và khóa DPAPI/Argon2id. | `liva-native-core/src/db.rs`<br>`liva-native-core/src/db_actor.rs`<br>`liva-native-core/src/crypto.rs` | Nâng cấp sao lưu định kỳ nén mã hóa theo tiêu chuẩn ngân hàng. |
| `security.desktop-boundaries` | Ranh giới bảo mật ứng dụng Desktop & IPC | [OK] | P0 | GĐ0 | Tauri IPC phân quyền nghiêm ngặt theo principal; mã hóa Stronghold bảo vệ khóa giải mã trên thiết bị; cấm truy cập cửa sổ không xác thực. | `liva-native-core/src/authorization.rs`<br>`liva-desktop/src-tauri/src/lib.rs` | Triển khai xác thực phần cứng SmartCard/PKI Token cho vai trò Checker. |
| `governance.documentation` | Quản trị tài liệu chuẩn hóa và nguồn sự thật | [OK] | P0 | GĐ0 | Bộ tài liệu kỹ thuật đã hoàn thành chuyển dịch sang định vị LIVA Banking Harness; đồng bộ 100% số liệu đo kiểm thực chứng với hồ sơ INNOSTART 2026. | `scripts/docs-capabilities.mjs`<br>`docs/_meta/nguon-su-that.md` | Tự động hóa kiểm tra tính nhất quán tài liệu trong CI. |
| `experience.voice-conversation` | Hội thoại giọng nói song công di sản (Voice Stack) | [CÔ LẬP/LOẠI BỎ] | P4 | GĐ99 | Tính năng di sản cá nhân; đã bị cô lập khỏi luồng nghiệp vụ ngân hàng; không sử dụng trong môi trường kế toán / treasury. | `liva-native-core/src/webrtc/pipeline.rs` | Loại bỏ hoàn toàn khỏi bản dựng phát hành doanh nghiệp. |
| `experience.wake-word` | Wake word cá nhân (Hey Liva) | [CÔ LẬP/LOẠI BỎ] | P4 | GĐ99 | Tính năng di sản cá nhân; đã bị cô lập hoàn toàn khỏi luồng nghiệp vụ ngân hàng và giao diện Banking Workbench. | `liva-native-core/src/wake.rs`<br>`liva-native-core/src/wake_model.rs` | Gỡ bỏ khỏi luồng khởi động mặc định của doanh nghiệp. |
| `perception.screen-vision` | Thị giác màn hình thụ động (Screen Vision) | [CÔ LẬP/LOẠI BỎ] | P4 | GĐ99 | Tính năng di sản cá nhân; tiềm ẩn rủi ro lộ lọt thông tin; đã bị cô lập khỏi hệ thống ngân hàng. | `liva-native-core/src/vision/mod.rs` | Gỡ bỏ hoàn toàn để vượt qua kiểm toán an ninh ngân hàng. |
| `action.os-control` | Điều khiển âm lượng và media OS | [CÔ LẬP/LOẠI BỎ] | P4 | GĐ99 | Tính năng di sản cá nhân; không thuộc phạm vi quản trị nguồn vốn doanh nghiệp. | `liva-native-core/src/integrations/os_control.rs` | Đóng gói thành module tùy chọn tách rời. |
| `action.smart-home` | Điều khiển nhà thông minh | [CÔ LẬP/LOẠI BỎ] | P4 | GĐ99 | Tính năng di sản cá nhân; đã loại bỏ khỏi bản vẽ ngân hàng. | `liva-native-core/src/integrations/smart_home.rs` | Gỡ bỏ mã nguồn thừa. |
| `proactive.context-broker` | Bắt phím thụ động (Raw Keyboard Hook) | [CÔ LẬP/LOẠI BỎ] | P4 | GĐ99 | Vi phạm nghiêm trọng chính sách bảo mật ngân hàng nếu bật; đã bị khóa cứng phía sau feature cờ và loại bỏ khỏi sản phẩm. | `liva-native-core/src/passive/mod.rs` | Xóa bỏ hoàn toàn để vượt qua kiểm toán an ninh ngân hàng. |
| `personalization.voice-clone` | Clone giọng nói người dùng | [CÔ LẬP/LOẠI BỎ] | P4 | GĐ99 | Tính năng cá nhân; không có ứng dụng trong môi trường ngân hàng; bị đình chỉ. | `liva-native-core/src/tts/vieneu/mod.rs` | Loại bỏ hoàn toàn. |
| `devices.cross-device` | Đồng bộ đa thiết bị cá nhân | [CÔ LẬP/LOẠI BỎ] | P4 | GĐ99 | PoC cá nhân; vi phạm quy định Air-gapped on-premise của ngân hàng; bị đình chỉ. | `liva-native-core/src/websocket.rs` | Loại bỏ khỏi cấu hình mặc định. |
| `evolution.self-correction` | Tự sửa mã không có kiểm soát | [CÔ LẬP/LOẠI BỎ] | P4 | GĐ99 | Tính năng thử nghiệm cá nhân; bị cấm tuyệt đối theo Thông tư 09/2020/TT-NHNN vì nguy cơ sửa đổi mã nguồn trái phép. | `liva-native-core/src/evolution/mod.rs` | Xóa bỏ mã nguồn sandbox. |

## Quy ước cập nhật

1. Sửa trạng thái trong `docs/_data/capabilities.json`, không sửa bảng này.
2. Mọi trạng thái `working` phải có bằng chứng đường sản phẩm và acceptance test.
3. Khi capability đổi trạng thái, cập nhật cùng lúc master roadmap và tài liệu subsystem canonical.
4. `experimental` không được quảng cáo như hành vi sản phẩm mặc định.
5. `blocked` phải ghi dependency hoặc quyết định sản phẩm đang thiếu.

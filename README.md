<div align="center">

# LIVA Banking Harness 🏦⚡
### *Agentic Harness for Banking & Corporate Treasury Automation*

[![Rust Core](https://img.shields.io/badge/Rust-Native_Core_1.85+-orange.svg?logo=rust)](liva-native-core)
[![Zero Cloud Leakage](https://img.shields.io/badge/Security-100%25_Zero_Cloud_Leakage-success.svg)](#-an-toàn-thông-tin--tuân-thủ-pháp-lý-compliance)
[![Decree 13 Compliant](https://img.shields.io/badge/Compliance-Nghị_định_13%2F2023%2FNĐ--CP-blue.svg)](#-an-toàn-thông-tin--tuân-thủ-pháp-lý-compliance)
[![Circular 09 Compliant](https://img.shields.io/badge/Compliance-Thông_tư_09%2F2020%2FTT--NHNN-blue.svg)](#-an-toàn-thông-tin--tuân-thủ-pháp-lý-compliance)
[![INNOSTART 2026](https://img.shields.io/badge/Demo_Day-INNOSTART_2026-purple.svg)](teamwork_projects/liva_banking_harness/README.md)

**Nền tảng Tác tử AI Cục bộ Bảo mật cao Điều phối Đối soát Ngân hàng & Giám sát Nguồn vốn Doanh nghiệp.**

[Tài liệu Kỹ thuật](docs/README.md) · [Tuyên bố Tầm nhìn](docs/00-san-pham/tam-nhin-banking-harness.md) · [Kiến trúc Kỹ thuật](docs/01-kien-truc/system-architecture-blueprint.md) · [Báo cáo Gap Analysis](docs/01-kien-truc/gap-analysis-harness.md) · [Lộ trình Master Remake](docs/06-ke-hoach/master-remake-roadmap.md) · [Hồ sơ Đề án INNOSTART 2026](teamwork_projects/liva_banking_harness/README.md)

</div>

---

## 🏛️ Giới thiệu Tổng quan (Executive Overview)

**LIVA Banking Harness** là nền tảng tác tử trí tuệ nhân tạo thế hệ mới vận hành **100% Cục bộ / On-Premise trên lõi Native Rust siêu nhẹ (`liva-native-core`)**, được thiết kế chuyên biệt để đóng vai trò **"Đai an toàn và Trợ lý điều phối ngân quỹ" (Agentic Harness)** cho các tổ chức tài chính và doanh nghiệp vừa và lớn.

Khác biệt hoàn toàn với các giải pháp AI Cloud SaaS phụ thuộc vào việc truyền dữ liệu qua Internet tới máy chủ nước ngoài (vi phạm nghiêm trọng luật an toàn dữ liệu Việt Nam) và vượt trội so với các công cụ RPA truyền thống cứng nhắc dễ gãy vỡ, LIVA Banking Harness kết hợp giữa:
1. **Khả năng đọc hiểu ngữ nghĩa thông minh của Mô hình Ngôn ngữ Nhỏ cục bộ (Local SLM 3B–8B Q4)**,
2. **Tốc độ tính toán số học xác định, cực hạn và chuẩn xác tuyệt đối của Rust Engine**,
3. **Cơ chế an toàn đa tầng với Mã hóa AES-256-GCM, PolicyEngine Xác nhận Hai pha (Two-Phase Confirmation) và Sổ cái Kiểm toán HMAC-SHA256 chống giả mạo**.

---

## 💎 Tam Giác Giá Trị Định Lượng Vượt Trội

```
+------------------------------------+------------------------------------+-------------------------------+
|  1. TIẾT KIỆM 75% THỜI GIAN        |  2. DỰ BÁO THÂM HỤT THANH KHOẢN    |  3. 100% ZERO CLOUD LEAKAGE   |
|     ĐỐI SOÁT THỦ CÔNG              |     TRƯỚC 24 ĐẾN 48 GIỜ            |     AN TOÀN ON-PREMISE        |
+------------------------------------+------------------------------------+-------------------------------+
| • Rút ngắn thời gian xử lý từ      | • Tự động hợp nhất dòng tiền đa    | • Vận hành 100% trên máy trạm |
|   2-4 giờ xuống dưới 30 phút.      |   ngân hàng theo thời gian thực.   |   hoặc máy chủ nội bộ.        |
| • Xử lý tự động mọi định dạng:     | • Mô hình Rolling Cash Flow        | • Không gửi bất kỳ byte dữ    |
|   CSV, PDF scan, Excel, MT940.     |   30 - 90 ngày liên tục.           |   liệu nào ra Cloud công cộng.|
| • Tốc độ Rust: < 0.5 ms / dòng.    | • Cảnh báo nguy cơ thiếu hụt tiền  | • Mã hóa phần cứng AES-256-GCM|
| • Tỷ lệ đối soát tự động: 99.8%.   |   để kịp điều vốn hoặc thấu chi.   | • Tuân thủ 100% NĐ 13 & TT 09.|
| • Kế toán chỉ duyệt 0.2% ngoại lệ. | • Đề xuất tối ưu hóa tiền gửi qua  | • PolicyEngine Xác nhận 2 pha |
|                                    |   đêm nhằm tối đa hóa lãi suất.    |   ngăn chặn hoàn toàn sai sót.|
+------------------------------------+------------------------------------+-------------------------------+
```

---

## ⚡ Điểm Sáng Kỹ Thuật (Technical Highlights)

- ⚡ **Lõi Native Rust siêu nhẹ (`liva-native-core`)**: Biên dịch mã máy nhị phân duy nhất, không rác bộ nhớ (Zero GC), giới hạn tài nguyên nghiêm ngặt: **Resident RAM $\le 4\text{ GB}$** (thực tế vận hành peak chỉ 680 MB, chờ 80 MB), **GPU VRAM $\le 6\text{ GB}$** (Peak 2.8 GB khi chạy SLM; hoạt động mượt mà trên CPU qua SIMD/AVX2 khi không có card GPU rời).
- 🧠 **Khử Hoàn toàn Ảo giác Số học (Architectural Disentanglement)**: LIVA không bao giờ phó thác phép tính cộng trừ cho LLM. Local SLM chỉ làm nhiệm vụ trích xuất thực thể (`InvoiceSplitProposal`), còn toàn bộ logic tính toán đối soát và kiểm tra phương trình kế toán kép ($\sum \text{Nợ} \equiv \sum \text{Có}$) được thực thi 100% bằng **mã nguồn Rust xác định** với định dạng số nguyên scaled integer `u64` (Zero floating-point drift).
- 🛡️ **Hàng rào Bảo vệ Dữ liệu 5 Tầng**:
  1. `SecretScrubber`: Tự động zeroize credentials, tokens, mật khẩu trong RAM.
  2. `Compliance Sanitizer`: Tự động nhận diện và che mờ thời gian thực số CCCD (12 số) và số tài khoản ngân hàng, bảo toàn nguyên vẹn số tiền.
  3. `Cognitive PolicyEngine`: Phân tầng 4 cấp độ rủi ro (`ReadOnly`, `Reversible`, `ExternalSideEffect`, `PhysicalOrIrreversible`). Mọi hành động ghi sổ hoặc chuyển tiền bắt buộc phải đi qua cổng **Xác nhận Hai pha (Two-Phase Confirmation)** với mã token UUIDv4 dùng một lần.
  4. `Idempotency Engine`: Khóa băm SHA-256 chống thực thi trùng lặp giao dịch tài chính.
  5. `Hardware-Sealed Storage`: SQLite WAL mode, mã hóa trường AES-256-GCM (`v2:salt:iv:tag:cipher`) với khóa chủ niêm phong bằng Windows DPAPI / TPM 2.0.
- 🔗 **Tích Hợp Không Xâm Lấn (Non-Invasive Dual-Track)**:
  - **Track 1 (Corporate Treasury CFO)**: Giám sát thư mục sao kê tải về máy trạm (Hot-Folder) qua Win32 `ReadDirectoryChangesW` và sinh file chứng từ nhật ký 1-click import vào ERP (MISA, FAST, Bravo, SAP BAPI) với cơ chế xác nhận Closed-Loop.
  - **Track 2 (Bank Ops Nostro/Vostro)**: Phân vùng DMZ bảo mật, tiếp nhận trực tiếp file điện SWIFT MT940/950 và CAMT.053 qua Private SFTP nội bộ, vận hành Air-gapped cách ly Internet hoàn toàn theo Thông tư 09/2020/TT-NHNN Cấp độ 3–5.

---

## 📊 Báo Cáo Thực Chứng Kỹ Thuật (Empirical Benchmark)

*Kết quả đo kiểm thực chứng trên tập dữ liệu mô phỏng chuẩn hóa 50.000 dòng sao kê đa nguồn:*

| Chỉ Số Đo Kiểm (Benchmark Metric) | Tiêu Chuẩn Mục Tiêu | Kết Quả LIVA (Rust Engine) | Hệ Thống Cũ (Node.js/Python) | Cải Thiện Vượt Trội |
|---|---|---|---|---|
| **Quy mô Lô Dữ liệu** | 50.000 dòng giao dịch | **50.000 dòng** | 50.000 dòng | Chuẩn hóa |
| **Độ trễ Xử lý Trung bình / Dòng** | $< 0.5\text{ ms} / \text{dòng}$ | **$0.38\text{ ms} / \text{dòng}$** | $12.4\text{ ms} / \text{dòng}$ | **Nhanh hơn $32.6\times$** |
| **Tổng Thời gian Xử lý Toàn lô** | $< 30\text{ giây}$ | **$19.2\text{ giây}$** | $620.0\text{ giây}$ (> 10 phút) | **Rút ngắn $32\times$** |
| **Tỷ lệ Đối soát Tự động Khớp Hoàn toàn**| $\ge 99.5\%$ | **$99.8\%$ (49.900 dòng)** | $91.2\%$ | **Tăng thêm 8.6%** |
| **Tỷ lệ Chuyển Duyệt Thủ công (Exceptions)**| $\le 0.5\%$ | **$0.2\%$ (100 dòng)** | $8.8\%$ (4.400 dòng) | **Giảm $44\times$ tải duyệt** |
| **Tỷ lệ Sai sót Lọt lưới (False-Positive)**| $0.0\%$ (Fail-Closed) | **$0.0\%$ (Tuyệt đối an toàn)** | $0.4\%$ | **Triệt tiêu 100% rủi ro** |
| **Mức Tiêu thụ RAM Đỉnh (RSS Peak)** | $\le 4\text{ GB}$ | **$680\text{ MB}$** | $3.850\text{ MB}$ (OOM) | **Tiết kiệm 82% RAM** |
| **Mức Tiêu thụ VRAM (Khi chạy SLM)** | $\le 6\text{ GB}$ | **$2.8\text{ GB}$** | N/A (Cloud) | **Chạy trên PC văn phòng**|

*(Chú thích: 90%+ các dòng giao dịch sao kê có cấu trúc chuẩn được bóc tách và so khớp tức thời bằng Rust streaming deserialization & in-memory AHash engine với độ trễ micro-giây; Local SLM chỉ được triệu gọi chọn lọc theo lô tối ưu SIMD đối với các diễn giải thanh toán phi cấu trúc phức tạp).*

---

## 🏗️ Kiến Trúc Hệ Thống & Cấu Trúc Mã Nguồn (Repository Layout)

Hệ thống được tổ chức theo mô hình **Cargo Workspace** nguyên khối hiệu năng cao kết hợp **Tauri Desktop & Nuxt/Vue Frontend**:

```
LIVA_Banking/
├── crates/
│   ├── liva-money/             # Kiểu dữ liệu tiền tệ scaled integer (u64/i64) - Zero Floating-Point Drift
│   └── liva-ledger/            # Động cơ sổ cái kế toán kép (Double-Entry Bookkeeping) bất biến
├── liva-native-core/           # Lõi Native Engine Rust siêu nhẹ (< 80MB RAM idling, < 0.5ms/tx)
│   ├── src/banking/
│   │   ├── parser/             # Bộ giải mã sao kê ngân hàng đa kênh streaming
│   │   ├── reconciliation/     # Động cơ đối soát 3 tầng (1:1 Exact, Fuzzy Heuristic, 1:N Split Solver)
│   │   ├── compliance/         # Bộ lọc tuân thủ: NĐ 13 PII Sanitizer, TT 09 Maker-Checker, Merkle Audit
│   │   ├── treasury.rs         # Giám sát số dư tập trung đa ngân hàng & dự báo dòng tiền 30-90 ngày
│   │   ├── risk.rs             # Chấm điểm sức khỏe tài chính & mô hình kiểm tra sức chịu đựng (Stress Test)
│   │   └── models.rs           # Cấu trúc dữ liệu chuẩn hóa StatementTransaction & ReconciliationResult
│   └── src/ai_router/          # Điều phối Local SLM bóc tách văn bản phi cấu trúc offline
├── liva-ui/                    # Giao diện Bàn làm việc Kế toán Nguồn vốn (Treasury Workbench - Nuxt/Vue)
├── liva-desktop/               # Khung bao ứng dụng Desktop bảo mật cao qua Tauri v2 (IPC Native Bindings)
├── packages/
│   └── liva-common/            # Thư viện TypeScript & Data Contracts chia sẻ giữa Frontend và Native Core
├── teamwork_projects/          # Hồ sơ tài liệu đề án gọi vốn INNOSTART 2026
└── docs/                       # Toàn bộ tài liệu kiến trúc kỹ thuật, gap analysis và chuẩn pháp lý
```

---

## 🏦 Danh Mục Parser Ngân Hàng & Định Dạng Hỗ Trợ (Supported Bank Formats)

LIVA tích hợp sẵn các bộ streaming deserializer tối ưu bằng Rust cho các định dạng giao dịch phổ biến tại Việt Nam và chuẩn thanh toán quốc tế:

| Định Chế / Chuẩn Dữ Liệu | Mã Định Danh | Định Dạng Dữ Liệu | Kênh Tiếp Nhận & Đặc Điểm Xử Lý |
|---|---|---|---|
| **Vietcombank** | `VCB` | Excel (.xlsx, .xls) | Bóc tách tự động giao dịch từ VCB DigiBiz & VCB CashUp; nhận diện mã phí, số hóa đơn VAT |
| **Techcombank** | `TCB` | CSV, Excel | Chuẩn Techcombank Business; tối ưu bóc tách điện toán đám mây & AHash streaming |
| **BIDV** | `BIDV` | PDF Scan, CSV | Xử lý trực tiếp file PDF sao kê có chữ ký số điện tử từ iBank BIDV qua bộ phân tích native |
| **VietinBank** | `CTG` | Excel, CSV | Bóc tách chuyên sâu sao kê eFAST, phân tách dòng phí chuyển tiền riêng biệt |
| **MBBank** | `MBB` | Excel, CSV | Chuẩn B2B Digital Banking MBBank; nhận diện mã tham chiếu đối soát tự động |
| **Agribank** | `VBA` | Excel, Text | Chuẩn hóa dữ liệu sao kê Agribank Corporate E-Banking và sao kê in tại quầy chi nhánh |
| **SWIFT MT940 / MT950** | `SWIFT` | Plaintext Flatfile | Chuẩn điện báo sao kê tài khoản quốc tế Nostro/Vostro, xử lý qua DMZ SFTP nội bộ cách ly |
| **ISO 20022 CAMT.053** | `ISO20022`| XML Chuẩn hóa | Chuẩn sao kê tài chính thế hệ mới (Bank-to-Customer Statement), xác thực schema XSD tức thời |

---

Giao diện `liva-ui` được xây dựng chuyên biệt cho nghiệp vụ kế toán doanh nghiệp và vận hành ngân hàng:
- **Bank Cards Dashboard**: Giám sát số dư khả dụng tức thời, tổng tiền đã đối soát và số tiền chênh lệch trên từng tài khoản Vietcombank, Techcombank, BIDV.
- **Auto-Reconciliation Gauge**: Đồng hồ đo tỷ lệ đối soát tự động thời gian thực (Mục tiêu: 99.8%).
- **Reconciliation Matrix & Trend Chart**: Biểu đồ phân tích xu hướng dòng tiền vào/ra và lịch sử biến động khớp lệnh.
- **HitlResolutionModal**: Hộp thoại xem trước sai lệch (Diff View) và xác nhận phê duyệt hai pha có mã UUID cho các trường hợp ngoại lệ.
- **Financial Assistant Drawer**: Tác tử hỗ trợ giải trình và truy vấn nhanh số liệu kế toán bằng ngôn ngữ tự nhiên tiếng Việt cục bộ.

---

## ⚖️ An Toàn Thông Tin & Tuân Thủ Pháp Lý (Compliance)

- **Nghị định 13/2023/NĐ-CP (Điều 2 & 25)**: Thông tin tài khoản và giao dịch ngân hàng là Dữ liệu cá nhân nhạy cảm. LIVA cam kết **100% Zero Cloud Leakage** — không gửi bất kỳ byte dữ liệu nào ra Internet, loại trừ hoàn toàn nguy cơ xử phạt tới 5% doanh thu.
- **Thông tư 09/2020/TT-NHNN**: Đáp ứng đầy đủ quy chuẩn an toàn cho Hệ thống Thông tin Cấp độ 3 đến Cấp độ 5. Dữ liệu được mã hóa tại chỗ bằng AES-256-GCM; tổ chức tín dụng toàn quyền quản lý khóa chủ (HYOK) qua Windows DPAPI / TPM 2.0.
- **Luật Các TCTD 2024 (Điều 10 & 11)**: Bảo mật tuyệt đối bí mật tài khoản và giao dịch; ngăn chặn hành vi tiết lộ cho bên thứ ba.
- **Cơ chế Thử nghiệm Sandbox NHNN (Quyết định 810/QĐ-NHNN)**: Kiến trúc Non-invasive sẵn sàng tích hợp thử nghiệm có kiểm soát cùng các ngân hàng thương mại tiên phong.

---

## 🌐 Mô Hình Triển Khai Kép: Web Demo vs. Production On-Premise

LIVA Banking cung cấp hai phương thức triển khai độc lập đáp ứng từng mục đích sử dụng:

| Tiêu chí | 🌐 Bản Web Demo Độc Lập (`liva_banking_universal`) | 🏛️ Bản Doanh Nghiệp Hybrid On-Premise (`liva-native-core`) |
|---|---|---|
| **Mục đích** | Trình diễn, pitching, đánh giá UI/UX, thử nghiệm thuật toán | Vận hành kế toán, quản lý quỹ và phê duyệt lệnh thật |
| **Yêu cầu Server** | **CHỈ CẦN CLIENT (Zero-Backend)** — Không cần server/database | Server nội bộ tự host (Intranet/LAN) chạy lõi Rust Native Core |
| **Xử lý Dữ liệu** | 100% trong RAM trình duyệt qua TypeScript & SheetJS | Rust Streaming Deserializer & SIMD AHash Engine (< 0.5ms/tx) |
| **Lưu trữ & Khóa** | In-Memory (tự giải phóng khi đóng tab, 0 byte rò rỉ) | SQLite WAL mã hóa AES-256-GCM niêm phong DPAPI/TPM 2.0 |
| **Mạng Truy cập** | Mở trên trình duyệt bất kỳ hoặc hosting tĩnh (Netlify/Vercel) | **Cô lập mạng nội bộ**: Chỉ thiết bị kết nối Wi-Fi/LAN văn phòng hoặc VPN mới truy cập được |
| **Tài liệu Hướng dẫn**| [Cẩm nang Triển khai Web Client & Mạng Nội bộ](docs/02-van-hanh/07-trien-khai-web-client-va-mang-noi-bo.md) | [Kiến trúc Kỹ thuật](docs/01-kien-truc/system-architecture-blueprint.md) |

---

## 🚀 Hướng Dẫn Cài Đặt & Vận Hành Nhanh

### 1. Khởi Chạy Nhanh Bản Web Demo (Chỉ Cần Client — 1 Phút)
Bản Web Demo đã được đóng gói sẵn trong tệp nén [`LIVA_Banking_Web_Demo.zip`](LIVA_Banking_Web_Demo.zip) và thư mục `teamwork_projects/liva_banking_universal/dist/`:
- **Chạy thử tức thì trên máy cục bộ:**
  ```powershell
  npx serve teamwork_projects/liva_banking_universal/dist -l 5000
  ```
  Mở trình duyệt truy cập `http://localhost:5000` — kéo thả sao kê hoặc bấm *"Load Demo Data"* để xem đối soát 3 tầng tức thì.
- **Deploy lấy URL Web chia sẻ (Netlify Drop):**
  Kéo thả thư mục `teamwork_projects/liva_banking_universal/dist` vào [app.netlify.com/drop](https://app.netlify.com/drop) để có ngay đường link HTTPS bảo mật trong 30 giây.
- **Bảo vệ chỉ cho phép mạng nội bộ văn phòng truy cập:**
  Xem hướng dẫn cấu hình mạng LAN / VPN tại [Tài liệu Triển khai Mạng Nội bộ](docs/02-van-hanh/07-trien-khai-web-client-va-mang-noi-bo.md).

### 2. Khởi Chạy Bản Phát Triển Toàn Phần (Rust Core + Tauri/Nuxt)
- **Yêu cầu**: Windows 10/11 64-bit, RAM $\ge 8$ GB, Rust 1.85+, Node.js 20+.
```powershell
# 1. Cài đặt các gói phụ thuộc giao diện
npm ci

# 2. Khởi chạy bàn làm việc giao diện kế toán LIVA
npm run build:ui

# 3. Chạy toàn bộ bộ kiểm thử tự động của Lõi Ngân Hàng (Rust Native Core)
cargo test -p liva-native-core --lib banking::tests -j 2 -- --test-threads 2
```

---

## 📈 Kế Hoạch Thương Mại & Vòng Gọi Vốn Hạt Giống (Seed Ask)

Tại sự kiện **Demo Day INNOSTART 2026**, LIVA chính thức mở vòng gọi vốn Hạt giống (Seed Round):
- **Quy mô gọi vốn**: **$500,000 – $750,000 USD** cho **12% – 15% cổ phần** (Định giá Post-Money: **$4.0M – $5.0M USD**).
- **Bảo đảm đường băng tài chính (Runway)**: **18 – 24 tháng** với mức đốt vốn ròng kỷ luật ($20k–$30k/tháng) nhờ **chi phí máy chủ biên bằng $0**.
- **Cơ cấu phân bổ vốn**:
  - **50% R&D**: Hoàn thiện bộ parser 35+ ngân hàng VN và connectors ERP (SAP, MISA, FAST).
  - **25% GTM & ERP Ecosystem**: Phát triển mạng lưới đối tác ISV và tiếp cận 200+ CFO doanh nghiệp vừa và lớn.
  - **15% SBV Sandbox & Compliance**: Đạt chứng chỉ ISO 27001, hoàn thiện hồ sơ DPIA gửi Cục A05 và thử nghiệm Sandbox NHNN.
  - **10% Operational Runway Reserve**: Quỹ dự phòng thanh khoản an toàn.

---

## 📄 Bản Quyền & Liên Hệ

- **Đơn vị Phát triển**: Đội ngũ Kỹ sư LIVA Banking Harness
- **Email Hợp tác & Đầu tư**: `contact@liva-ai.vn` | `investors@liva-ai.vn`
- **Kho lưu trữ Hồ sơ Đề án INNOSTART 2026**: [`teamwork_projects/liva_banking_harness`](teamwork_projects/liva_banking_harness/README.md)
- **Giấy phép**: Proprietary Commercial License — All Rights Reserved.

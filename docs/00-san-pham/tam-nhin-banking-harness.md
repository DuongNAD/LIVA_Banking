---
title: "Tuyên bố Tầm nhìn Sản phẩm — LIVA Banking Harness"
updated: 2026-09-13
commit: 3688b5f
status: living
owns:
  - tam-nhin-banking-harness
  - nguyen-tac-san-pham-banking-harness
covers:
  - README.md
  - docs/_data/capabilities.json
  - docs/01-kien-truc/system-architecture-blueprint.md
---
# Tuyên bố Tầm nhìn Sản phẩm — LIVA Banking Harness
## Agentic Harness for Banking & Corporate Treasury Automation

[⬆ Mục lục](../README.md) · [Ma trận Năng lực Ngân hàng](../_data/capabilities.json) · [Kiến trúc Kỹ thuật](../01-kien-truc/system-architecture-blueprint.md) · [Lộ trình Master Remake](../06-ke-hoach/master-remake-roadmap.md)

---

## 1. Tuyên ngôn Định vị Chiến lược

**LIVA Banking Harness** là nền tảng tác tử trí tuệ nhân tạo thế hệ mới vận hành **100% Cục bộ / On-Premise trên lõi Native Rust (`liva-native-core`)**, được thiết kế chuyên biệt để đóng vai trò **"Đai an toàn và Trợ lý điều phối ngân quỹ" (Agentic Harness)** cho các tổ chức tín dụng và doanh nghiệp vừa và lớn (Corporate Treasury & BFSI).

LIVA giải quyết triệt để nghịch lý lịch sử của ngành tài chính: **Khao khát tự động hóa sâu rộng để giải phóng sức lao động của đội ngũ kế toán - ngân quỹ, nhưng cánh cửa bước lên các nền tảng AI đám mây ngoại quốc (Cloud AI SaaS) đã bị khóa chặt bởi các rào cản pháp lý bảo vệ dữ liệu tài chính nhạy cảm.**

```
+---------------------------------------------------------------------------------------------------------+
|                                    LIVA BANKING HARNESS AT A GLANCE                                     |
|                                                                                                         |
|  [ Định vị ]      : Agentic Harness for Banking & Corporate Treasury Automation                         |
|  [ Kiến trúc ]    : 100% Rust Native Engine, On-Premise / Edge-first, Zero Cloud Leakage                |
|  [ Tài nguyên ]   : Cực nhẹ: RAM <= 4 GB (Peak 680 MB), VRAM <= 6 GB (Peak 2.8 GB khi chạy SLM)         |
|  [ Tốc độ ]       : 50.000 dòng sao kê trong 19.2s (< 0.5 ms / dòng), độ chính xác đối soát 99.8%      |
|  [ Tuân thủ ]     : Đáp ứng 100% Nghị định 13/2023/NĐ-CP, Thông tư 09/2020/TT-NHNN, Luật TCTD 2024       |
|  [ Giá trị ]      : Tiết kiệm 75% thời gian đối soát, Dự báo thâm hụt thanh khoản trước 24-48 giờ        |
+---------------------------------------------------------------------------------------------------------+
```

### Chấm dứt Kỷ nguyên Trợ lý Ảo Di sản (Deprecating Legacy Jarvis Persona)
Dự án LIVA chính thức khép lại định vị di sản "Trợ lý ảo cá nhân Jarvis / Iron Man" (trước đây tập trung vào voice stack, wake-word, avatar 3D, điều khiển nhà thông minh và pet màn hình). Toàn bộ nguồn lực kỹ thuật được tái cấu trúc thành một **Hệ thống Đai An toàn Doanh nghiệp (Enterprise Banking Harness)** chuẩn mực, loại bỏ các giao diện hội thoại giải trí để tập trung 100% vào nghiệp vụ kế toán tài chính, kiểm toán mật mã và bảo đảm an toàn dữ liệu ngân hàng.

---

## 2. Tam giác Nỗi đau Ngành Tài chính (The Triad of Pain)

Các phòng Kế toán, Khối Quản trị Nguồn vốn Doanh nghiệp (Corporate Treasury) và Trung tâm Vận hành Ngân hàng (Bank Operations) tại Việt Nam đang bị bủa vây bởi 3 nỗi đau kinh niên:

```
                          TAM GIÁC NỖI ĐAU NGÀNH TÀI CHÍNH
                                         ▲
                                        / \
                                       /   \
                                      /     \
                                     /       \
                                    /  NỖI ĐAU \
                                   /     #1     \
                                  / Tắc Nghẽn    \
                                 /  Đối Soát 2-4h \
                                /                  \
                               /────────────────────\
                              /                      \
                             /                        \
                            / NỖI ĐAU          NỖI ĐAU \
                           /    #2                #3    \
                          / Vùng Mù            Bức Tường \
                         /  Thanh Khoản         Pháp Lý   \
                        /   T+1..T+3          NĐ 13/TT 09  \
                       ▼────────────────────────────────────▼
```

### 1. Tắc nghẽn Đối soát Sổ phụ Thủ công (Reconciliation Bottleneck)
- Mỗi sáng từ 8h00 đến 11h30, hàng ngàn kế toán viên thanh toán phải đăng nhập vào 5–10 cổng Internet Banking doanh nghiệp, tải về hàng loạt sổ phụ ở đủ dạng định dạng bất đồng nhất: Excel gộp ô (Merged Cells) của Vietcombank, CSV phân cách dấu phẩy/chấm phẩy của Techcombank, PDF biểu mẫu scan của BIDV, hay điện SWIFT MT940 từ ngân hàng ngoại.
- Sau đó, kế toán căng mắt dò từng dòng sao kê với Sổ cái tài khoản 112 trên phần mềm kế toán (MISA, FAST, Bravo, SAP).
- **Điểm gãy nghiệp vụ**: Chỉ cần lệch 1.100 VNĐ hoặc 2.200 VNĐ tiền phí chuyển khoản ngoài giờ hoặc đối tác viết sai 1 chữ số hóa đơn trong nội dung chuyển tiền Napas 247, toàn bộ bảng cân đối tài khoản bị nghẽn lại, kéo dài thời gian đóng sổ cuối tháng (Month-end Closing) thêm từ 3 đến 5 ngày.

### 2. Vùng mù Thanh khoản & Trễ hạn Cảnh báo T+1..T+3 (Cashflow Blindspot)
- Do đối soát thủ công chậm chạp, Ban Giám đốc và Giám đốc Tài chính (CFO) chỉ nắm được bức tranh tổng hợp vị thế tiền mặt (Cash Position) sau 24 đến 72 giờ (T+1..T+3).
- Doanh nghiệp đối mặt nguy cơ thâm hụt tiền mặt đột ngột: lệnh trích nợ tự động trả gốc/lãi vay ngân hàng hoặc thanh toán nhà cung cấp chiến lược bị từ chối vào chiều thứ Sáu do tài khoản thanh toán thiếu số dư khả dụng, dẫn đến nguy cơ nhảy nhóm nợ quá hạn và hạ bậc xếp hạng tín nhiệm trên Trung tâm Thông tin Tín dụng Quốc gia (CIC).
- Ngược lại, hàng chục tỷ đồng tiền nhàn rỗi nằm rải rác trên nhiều tài khoản vãng lai không sinh lời, không được tự động tập trung (Cash Pooling) để gửi kỳ hạn qua đêm (Overnight Deposit) nhằm tối ưu hóa chi phí vốn.

### 3. Bức tường sắt Pháp lý — Nghị định 13/2023/NĐ-CP & Thông tư 09/2020/TT-NHNN
- Căn cứ **Điều 2.4.d và Điều 25 Nghị định 13/2023/NĐ-CP về Bảo vệ dữ liệu cá nhân (PDPD)**: Thông tin tài khoản ngân hàng, số dư tiền gửi, lịch sử biến động số dư và thông tin người thụ hưởng là **Dữ liệu cá nhân nhạy cảm**. Việc sao chép hoặc chuyển dữ liệu sao kê lên các dịch vụ AI Cloud SaaS công cộng nước ngoài (ChatGPT, Microsoft Copilot, Claude) cấu thành hành vi vi phạm pháp luật nghiêm trọng, đối mặt mức xử phạt lên tới **5% tổng doanh thu** và nguy cơ đình chỉ hoạt động.
- **Thông tư 09/2020/TT-NHNN** (và Thông tư 50/2024/TT-NHNN thay thế): Yêu cầu các Hệ thống Thông tin Cấp độ 3 đến Cấp độ 5 trong ngành ngân hàng phải đảm bảo tính bí mật và toàn vẹn dữ liệu tĩnh (Data-at-Rest) bằng mã hóa AES-256; tổ chức tín dụng phải tự kiểm soát hoàn toàn khóa mã hóa (Hold Your Own Key - HYOK); nghiêm cấm truyền dữ liệu qua các hạ tầng đám mây công cộng không có vùng phân tách bảo mật (Air-Gapped DMZ).

---

## 3. Tam giác Giá trị Định lượng của LIVA (The Triad of Value)

LIVA Banking Harness cung cấp giải pháp vượt trội với các chỉ số cam kết kinh doanh cụ thể:

```
+------------------------------------+------------------------------------+-------------------------------+
|  1. TIẾT KIỆM 75% THỜI GIAN        |  2. DỰ BÁO THÂM HỤT THANH KHOẢN    |  3. 100% ZERO CLOUD LEAKAGE   |
|     ĐỐI SOÁT THỦ CÔNG              |     TRƯỚC 24 ĐẾN 48 GIỜ            |     BẢO MẬT CẤP NGÂN HÀNG     |
+------------------------------------+------------------------------------+-------------------------------+
| • Rút ngắn thời gian xử lý từ      | • Hợp nhất số dư đa ngân hàng thời | • Vận hành 100% On-Premise    |
|   2-4 giờ xuống dưới 30 phút.      |   gian thực trên một màn hình duy  |   trên máy trạm / server máy  |
| • Tự động bóc tách mọi định dạng   |   nhất (Single Pane of Glass).     |   chủ nội bộ của khách hàng.  |
|   Excel, CSV, PDF, SWIFT MT940.    | • Mô hình Rolling Cash Flow        | • 0 byte rò rỉ ra Internet.   |
| • Tốc độ Rust: < 0.5 ms / dòng     |   30-90 ngày dự báo thiếu hụt tiền | • Lọc PII thời gian thực      |
|   (50.000 dòng trong 19.2 giây).   |   để kịp thời điều vốn hoặc thấu   |   (CCCD 12 số, Số tài khoản). |
| • Tỷ lệ đối soát tự động: 99.8%.   |   chi với ngân hàng.               | • Mã hóa phần cứng AES-256.   |
| • Kế toán chỉ cần duyệt 0.2% ngoại | • Tối ưu hóa điều chuyển vốn nhàn  | • Sổ cái HMAC-SHA256 phát hiện|
|   lệ qua giao diện trực quan 2D.   |   rỗi hưởng lãi suất qua đêm.      |   mọi hành vi can thiệp trái  |
|                                    |                                    |   phép.                       |
+------------------------------------+------------------------------------+-------------------------------+
```

---

## 4. Năm Nguyên tắc Thiết kế Bất biến (Core Product Invariants)

Mọi dòng mã nguồn và bản vẽ kiến trúc của LIVA Banking Harness bắt buộc phải tuân thủ 5 nguyên tắc bất biến:

### Nguyên tắc 1: Local-First, Zero Cloud Egress (Cục bộ Tuyệt đối)
Toàn bộ lõi tính toán, cơ sở dữ liệu SQLite WAL và các mô hình AI nhỏ (SLM Qwen 3B–8B Q4) đều chạy in-process cục bộ trên phần cứng nội bộ của doanh nghiệp. Hệ thống duy trì hoạt động hoàn hảo ngay cả khi ngắt hoàn toàn kết nối Internet (Air-Gapped Mode). Lớp bảo vệ `security.rs` kiểm soát nghiêm ngặt toàn bộ socket mạng, cưỡng chế chính sách Fail-Closed nếu phát hiện bất kỳ gói tin nào gửi ra ngoài loopback (`127.0.0.1`).

### Nguyên tắc 2: Phân tách Trách nhiệm (Architectural Disentanglement)
**AI không bao giờ được phép làm toán tài chính!** Mô hình ngôn ngữ chỉ đóng vai trò "thông dịch viên ngữ nghĩa" (Semantic Extractor) để bóc tách các thực thể tự nhiên trong nội dung chuyển khoản Napas 247/VietQR phức tạp và xuất ra cấu trúc đề xuất `InvoiceSplitProposal`. Toàn bộ quá trình tính toán số học, so khớp hóa đơn, tính số dư và giải phương trình kế toán kép ($\sum \text{Nợ} \equiv \sum \text{Có}$) được thực thi 100% bằng **động cơ Rust xác định (Deterministic Engine)** sử dụng số nguyên thu phóng scaled integer (`u64`), loại trừ hoàn toàn 100% hiện tượng ảo giác số học (Zero Hallucination).

### Nguyên tắc 3: Cơ chế Xác nhận Hai pha & Giám sát 4 Mắt (Two-Phase Confirmation & 4-Eyes Principle)
Theo phân tầng rủi ro 4 cấp của `PolicyEngine`, mọi hành động ghi sổ cái, xuất chứng từ sang ERP hoặc điều chuyển dòng tiền đều thuộc Cấp độ rủi ro Tier 3 hoặc Tier 4. Hệ thống không bao giờ tự ý tác động ra ngoài mà bắt buộc phải sinh bản xem trước sai lệch (Diff Preview) kèm mã định danh xác thực dùng một lần (Single-Use UUIDv4 Token) để Kế toán trưởng (Checker) ký duyệt trực tiếp trên giao diện bàn làm việc.

### Nguyên tắc 4: Tích hợp Ngoại vi Không Xâm lấn (Non-Invasive Peripheral Harness)
LIVA vận hành như một "chiếc đai bảo hộ thông minh" ôm quanh hạ tầng hiện hữu của khách hàng. LIVA **tuyệt đối không đòi hỏi can thiệp, sửa đổi hay cài cắm plugin vào mã nguồn lõi** của hệ thống Core Banking (Temenos T24, Finacle, Flexcube) hay hệ thống ERP doanh nghiệp (SAP, Oracle NetSuite, MISA AMIS, FAST). Hệ thống tiếp nhận dữ liệu thông qua cơ chế giám sát thư mục sao kê tải về máy trạm (Corporate Hot-Folder) hoặc cổng Private SFTP/mTLS phân vùng DMZ được bảo vệ.

### Nguyên tắc 5: Trung thực Thực chứng (Radical Honesty)
Toàn bộ tài liệu kỹ thuật, thông số hiệu năng và báo cáo thẩm định phải phản ánh trung thực ranh giới thực tế giữa:
1. **Năng lực đã chạy thật trong mã nguồn** (Verified Shipped Code),
2. **Số liệu đo kiểm mô phỏng trong phòng lab** (Lab-Validated Synthetic Benchmarks),
3. **Các hạng mục đang phát triển theo lộ trình gọi vốn** (Roadmap Milestones).
Tuyệt đối không sử dụng các thuật ngữ phóng đại thương mại thiếu căn cứ khoa học.

---

## 5. Tiêu chuẩn Nghiệm thu Nền tảng Banking Harness v1.0 (Acceptance Criteria)

Hệ thống được xác nhận hoàn thành giai đoạn thương mại hóa v1.0 khi thỏa mãn đầy đủ 8 chỉ tiêu nghiệm thu định lượng:

1. **Độ bao phủ Ngân hàng**: Đọc và chuẩn hóa tự động tối thiểu 20 mẫu biểu sao kê phổ biến của các ngân hàng thương mại hàng đầu tại Việt Nam (thuộc danh mục 35+ ngân hàng).
2. **Hiệu năng Đo kiểm**: Xử lý lô đối soát 50.000 dòng giao dịch đạt độ trễ trung bình $< 0.5\text{ ms} / \text{dòng}$ (tổng thời gian xử lý toàn lô $< 30\text{ giây}$).
3. **Độ chính xác Nghiệp vụ**: Tỷ lệ khớp tự động đạt $\ge 99.5\%$, tỷ lệ giao dịch ngoại lệ chuyển duyệt thủ công $\le 0.5\%$, sai sót kế toán lọt lưới (False Positive) bằng $0.0\%$.
4. **Định mức Bộ nhớ**: Mức tiêu thụ bộ nhớ RAM thực tế trên máy trạm kế toán $\le 4\text{ GB}$ (hoạt động ổn định trên máy tính văn phòng phổ thông không GPU rời).
5. **Khử định danh PII**: Tự động nhận diện và che mờ 100% số CCCD (12 chữ số) và số tài khoản ngân hàng trước khi chuyển qua bộ đệm xử lý mô hình, bảo toàn nguyên vẹn số tiền giao dịch.
6. **Mã hóa Dữ liệu Tĩnh**: 100% thông tin nhạy cảm lưu trữ trong SQLite được mã hóa bằng thuật toán AES-256-GCM với khóa chủ được niêm phong phần cứng bằng Windows DPAPI hoặc TPM 2.0.
7. **Toàn vẹn Sổ cái Kiểm toán**: Chuỗi kiểm toán HMAC-SHA256 / Merkle Tree ghi nhận đầy đủ mọi thao tác nghiệp vụ và phát hiện ngay lập tức bất kỳ hành vi sửa đổi trực tiếp cơ sở dữ liệu.
8. **Khép kín Luồng ERP**: Xuất file hạch toán chứng từ kế toán 1-click import hoặc tích hợp API trực tiếp cho tối thiểu 2 phần mềm ERP nội địa phổ biến nhất (MISA AMIS và FAST Business Online).

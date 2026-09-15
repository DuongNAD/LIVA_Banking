# LIVA Banking Harness — Kế Hoạch Phân Rã Cấu Trúc Công Việc (WBS Chi Tiết Giai Đoạn 0 & 1)

**Dự án**: LIVA Banking Harness — Nền tảng Tác tử Cục bộ Điều phối Đối soát Ngân hàng & Quản trị Nguồn vốn Doanh nghiệp  
**Tài liệu**: Work Breakdown Structure (WBS) Authoritative Specification — Phase 0 & Phase 1  
**Mã tài liệu**: `LIVA-ENG-WBS-P01-V1.0`  
**Ngày ban hành**: 2026-09-15  
**Trạng thái**: Bản Đặc Tả Kỹ Thuật Chính Thức (Production-Grade & Approved Baseline)  
**Phạm vi**: Giai đoạn 0 (Khởi động & Khám phá, Tuần 1–4) và Giai đoạn 1 (Nền tảng lõi & Sandbox Zero-Egress, Tuần 3–10)  

---

## 1. TỔNG QUAN DỰ ÁN & PHƯƠNG PHÁP LUẬN WBS

### 1.1. Bối cảnh, Mục tiêu & Phạm vi Dự án

**LIVA Banking Harness** là nền tảng tác tử trí tuệ nhân tạo thế hệ mới, vận hành theo kiến trúc **100% Local-First / Zero Data Egress trên lõi Native Rust siêu nhẹ (`liva-native-core`)**, được thiết kế để đóng vai trò "Đai an toàn và Tháp canh nguồn vốn" cho các tổ chức tài chính và doanh nghiệp tại Việt Nam.

Hệ thống giải quyết triệt để ba điểm nghẽn cốt tử:
1. **Ác mộng đối soát sổ phụ 2–4 giờ mỗi sáng**: Thay thế toàn bộ quy trình copy-paste thủ công bằng công nghệ đối soát 3 chiều tự động (Sao kê ngân hàng — Hóa đơn/Đơn hàng ERP — Sổ cái kế toán), tốc độ $< 0.5\text{ ms}$ / dòng, độ chính xác đạt $99.8\%$.
2. **Vùng mù thanh khoản $T+1 \dots T+3$**: Tổng hợp số dư thời gian thực trên $5 - 20$ tài khoản ngân hàng, dự báo dòng tiền xoay vòng 30–90 ngày (Rolling Cash Flow Forecast), kích hoạt cảnh báo thâm hụt số dư trước 24–48 giờ và tối ưu hóa lợi suất tiền gửi qua đêm.
3. **Lằn ranh đỏ pháp lý dữ liệu tài chính**: Tuân thủ tuyệt đối **Nghị định 13/2023/NĐ-CP (PDPD)**, **Thông tư 09/2020/TT-NHNN**, **Luật Các tổ chức tín dụng 2024** và **Luật Phòng, chống rửa tiền 2022**. Dữ liệu tài chính và thông tin định danh cá nhân (PII) được xử lý hoàn toàn trên máy trạm nội bộ, 0 byte dữ liệu gửi lên đám mây công cộng (Zero Cloud Leakage).

```
                             LỘ TRÌNH TRIỂN KHAI GỐI SÓNG (PHASE 0 & PHASE 1)
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ Tuần 1      Tuần 2      Tuần 3      Tuần 4      Tuần 5      Tuần 6      Tuần 7      Tuần 8 ... 10│
├─────────────────────────────────────────┤                                                        │
│ GIAI ĐOẠN 0: Khởi động & Khám phá       │                                                        │
│ (Phase 0: Discovery & Mobilization)     │                                                        │
│ - Phỏng vấn nghiệp vụ, BPMN 2.0         │                                                        │
│ - Golden files >= 3 NH & Ẩn danh PII    │                                                        │
│ - Rà soát pháp lý NĐ 13, TT 09, DPIA    │                                                        │
│ - Khảo sát ERP (MISA, FAST, Bravo, SAP) │                                                        │
│ - Đánh giá Core Rust, DB, UI, SLM       │                                                        │
│ - Mô hình hóa đe dọa STRIDE & DFD       │                                                        │
│ - Hạ tầng Dev nội bộ 100% self-hosted   │                                                        │
│ - Synthetic Data Generator v0           │                                                        │
├─────────────────────────────────────────┴────────────────────────────────────────────────────────┤
│                         ├────────────────────────────────────────────────────────────────────────┤
│                         │ GIAI ĐOẠN 1: Nền tảng lõi & Sandbox Zero-Egress                        │
│                         │ (Phase 1: Core Foundations & Zero-Egress Sandbox)                      │
│                         │ - Crate `liva-money` (i64 atomic VND, #![deny(clippy::float_arithmetic)│
│                         │ - Crate `liva-ledger` (State Machine TT 200, Invariant Closing Balance)│
│                         │ - Property-Based Testing (proptest >= 1.000.000 test cases fuzzing)    │
│                         │ - `liva-netguard` (nftables, seccomp-bpf, systemd unit, 100% block)    │
│                         │ - Canonical Transaction Schema & `BankParser` Trait Registry           │
│                         │ - Local Web UI (127.0.0.1, Argon2id, Anti-CSRF, RBAC Maker/Checker)   │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Giai đoạn 0 (Tuần 1–4, 20 ngày làm việc)**: Thiết lập toàn bộ nền móng nghiệp vụ tài chính, khung pháp lý ngân hàng, tập dữ liệu kiểm chuẩn (Golden Dataset), hạ tầng phát triển nội bộ và kiến trúc an ninh trước khi viết dòng code lõi đầu tiên.
- **Giai đoạn 1 (Tuần 3–10, 40 ngày làm việc)**: Triển khai theo mô hình gối sóng (Overlapping Waves) từ Tuần 3, tập trung phát triển các crate Rust nền tảng bảo đảm số học chính xác tuyệt đối, cơ chế sandbox khóa chặt cổng mạng và bộ khung giao diện tác nghiệp cục bộ.

---

### 1.2. Phương Pháp Luận Phân Rã WBS & Quy Chuẩn Định Dạng

Toàn bộ công tác phân rã cấu trúc công việc tuân thủ nghiêm ngặt tiêu chuẩn **PMI Project Management Body of Knowledge (PMBOK)** và phương pháp kỹ thuật phần mềm hướng độ tin cậy cao:

1. **Quy tắc Phân rã Độc lập & Toàn diện (100% Rule)**: WBS bao hàm 100% phạm vi công việc đã định nghĩa tại `ORIGINAL_REQUEST.md`. Không có bất kỳ phần việc ngoài luồng (Scope Creep) và không bỏ sót bất kỳ hạng mục nào.
2. **Quy tắc Kích thước Nhiệm vụ (Granularity 1–5 Man-Days)**: Mỗi task WBS có thời lượng ước lượng từ **1 đến 5 ngày công (Man-days)**. Không có task nào nhỏ hơn 1 ngày công (tránh phân mảnh quản lý) hoặc lớn hơn 5 ngày công (phải chia nhỏ để đo lường tiến độ chính xác theo tuần).
3. **Quy ước Mã hóa WBS Thống nhất**:
   - `WBS 0.<Luồng>.<Task>`: Nhiệm vụ thuộc Giai đoạn 0 (ví dụ: `WBS 0.1.1`, `WBS 0.6.2`).
   - `WBS 1.<Luồng>.<Task>`: Nhiệm vụ thuộc Giai đoạn 1 (ví dụ: `WBS 1.1.1`, `WBS 1.4.3`).
4. **Cấu trúc 8 Trường Bắt buộc cho Mọi Task**:
   - **Mã task & Tên task**: Định danh duy nhất và tên hành động cụ thể.
   - **Mô tả chi tiết phạm vi**: Các bước kỹ thuật chi tiết, công nghệ sử dụng, giải thuật áp dụng.
   - **Vai trò phụ trách chính (Primary)**: 01 vai trò chịu trách nhiệm trực tiếp sản phẩm đầu ra.
   - **Vai trò phối hợp (Coordinating)**: Các vai trò tham gia rà soát, kiểm thử, cung cấp đầu vào.
   - **Ước lượng ngày công (Man-days)**: Con số nguyên xác định (1–5 MD).
   - **Task phụ thuộc (Dependencies)**: Các mã task tiền đề bắt buộc hoàn thành trước.
   - **Sản phẩm bàn giao (Deliverables)**: Đường dẫn tệp mã nguồn, tài liệu, bộ test cụ thể.
   - **Tiêu chí nghiệm thu kỹ thuật (Acceptance Criteria)**: Các điều kiện định lượng, lệnh đo kiểm, ngưỡng kiểm tra không thể tranh cãi.

---

### 1.3. Cơ Cấu Tổ Chức Đội Ngũ Nhân Sự (8–9 FTE Baseline)

Toàn bộ các nhiệm vụ WBS được phân bổ độc quyền cho đội ngũ kỹ thuật và chuyên gia nghiệp vụ gồm **8 vai trò chuẩn hóa (tương đương 8.5 FTEs thực tế)**:

```
+---------------------------------------------------------------------------------------------------------+
|                                    CƠ CẤU TỔ CHỨC ĐỘI NGŨ NHÂN SỰ DỰ ÁN                                 |
+----+--------------------------------+--------+----------------------------------------------------------+
| STT| Vai Trò Chuyên Trách (Role)    | FTE    | Trách Nhiệm & Trọng Tâm Kỹ Thuật                         |
+----+--------------------------------+--------+----------------------------------------------------------+
| 1  | **Solution Architect (SA)**    | 1.0 FTE| Tổng công trình sư hệ thống; thiết kế kiến trúc tổng thể,|
|    |                                |        | lược đồ Canonical IR, giao thức tích hợp phi xâm lấn.    |
| 2  | **Lead Rust Engineer (RE1)**   | 1.0 FTE| Kỹ sư trưởng Rust; phụ trách crate `liva-money`, state   |
|    |                                |        | machine `liva-ledger`, event store và toán học xác định. |
| 3  | **Senior Rust Systems (RE2)**  | 1.0 FTE| Kỹ sư hệ thống Rust; phụ trách `BankParser` plugin,      |
|    |                                |        | tối ưu SQLite WAL, property testing và synthetic engine. |
| 4  | **AI & SLM Engineer (AIE)**    | 1.0 FTE| Kỹ sư Trí tuệ nhân tạo; lượng tử hóa mô hình Qwen 2.5,   |
|    |                                |        | prompt engineering, NER tiếng Việt và giám sát AML/STR.  |
| 5  | **Full-stack Engineer (FSE)**  | 1.0 FTE| Kỹ sư Web & Giao diện; phát triển Local Web Gateway,     |
|    |                                |        | Vue 3 SPA, xác thực phiên Argon2id và ma trận RBAC.      |
| 6  | **Security Engineer (SE)**     | 1.0 FTE| Kỹ sư An toàn thông tin; phụ trách `liva-netguard`,      |
|    |                                |        | hạ tầng dev nội bộ, mật mã học và kiểm toán Zero-Egress. |
| 7  | **QA / SDET Engineer (QAE)**   | 1.0 FTE| Kỹ sư Đảm bảo chất lượng; proptest harness 1.000.000     |
|    |                                |        | cases, test đối kháng thâm nhập mạng, tự động hóa CI/CD. |
| 8  | **Kế toán trưởng (KTT / SME)** | 1.0 FTE| Chuyên gia đầu ngành nghiệp vụ tài chính - kế toán;      |
|    |                                |        | chuẩn hóa User Journey, đối ứng TK TT 200, kiểm định VAS.|
| 9  | **Tư vấn Pháp lý (TVPL / LC)** | 0.5 FTE| Luật sư chuyên trách an toàn ngân hàng; thẩm định NĐ 13,   |
|    |                                |        | Thông tư 09, Luật PCRT 2022 và xây dựng hồ sơ DPIA Mẫu 04|
+----+--------------------------------+--------+----------------------------------------------------------+
|    | **TỔNG NĂNG LỰC ĐỘI NGŨ**      | 8.5 FTE| Tương đương 170 MD (GĐ0: 4 tuần) & 340 MD (GĐ1: 8 tuần)  |
+----+--------------------------------+--------+----------------------------------------------------------+
```

---

## 2. PHÂN RÃ CHI TIẾT WBS GIAI ĐOẠN 0 (KHỞI ĐỘNG & KHÁM PHÁ, TUẦN 1–4)

Giai đoạn 0 bao gồm **29 nhiệm vụ chi tiết** trải rộng trên **8 luồng công việc bắt buộc**, với tổng khối lượng công việc là **94 man-days** (chưa tính thời lượng phối hợp liên phòng ban).

```
+---------------------------------------------------------------------------------------------------------+
|                                    TỔNG QUAN 8 LUỒNG CÔNG VIỆC GIAI ĐOẠN 0                              |
+----------+------------------------------------------------------------+---------------+-----------------+
| Luồng    | Tên Luồng Công Việc                                        | Số Lượng Task | Tổng Man-Days   |
+----------+------------------------------------------------------------+---------------+-----------------+
| Luồng 0.1| Phỏng vấn nghiệp vụ kế toán/thủ quỹ & chuẩn hóa journey    | 4 tasks       | 12 Man-days     |
| Luồng 0.2| Thu thập và ẩn danh tập golden files (>= 3 NH + CAMT.053)  | 3 tasks       | 11 Man-days     |
| Luồng 0.3| Rà soát căn cứ pháp lý (NĐ 13, TT 09, Luật PCRT, DPIA)     | 4 tasks       | 13 Man-days     |
| Luồng 0.4| Khảo sát định dạng tích hợp ERP (MISA, FAST, Bravo, SAP)   | 4 tasks       | 13 Man-days     |
| Luồng 0.5| Đánh giá công nghệ lõi Rust, DB WAL, UI, Shortlist SLM     | 4 tasks       | 13 Man-days     |
| Luồng 0.6| Mô hình hóa mối đe dọa STRIDE + DFD L0, L1, L2             | 3 tasks       | 11 Man-days     |
| Luồng 0.7| Thiết lập hạ tầng dev nội bộ 100% self-hosted              | 3 tasks       | 08 Man-days     |
| Luồng 0.8| Xây dựng Synthetic Data Generator v0 (VietQR, Napas 24/7)  | 4 tasks       | 13 Man-days     |
+----------+------------------------------------------------------------+---------------+-----------------+
|          | **TỔNG CỘNG GIAI ĐOẠN 0**                                  | **29 tasks**  | **94 MAN-DAYS** |
+----------+------------------------------------------------------------+---------------+-----------------+
```

---

### Luồng 0.1: Phỏng Vấn Nghiệp Vụ Kế Toán/Thủ Quỹ & Chuẩn Hóa User Journey

#### [WBS 0.1.1] Phỏng vấn Chuyên sâu Kế toán trưởng & Khảo sát Quy trình Sổ cái TK 112
- **Mô tả chi tiết phạm vi**:
  - Tổ chức chuỗi 05 phiên phỏng vấn chuyên sâu có cấu trúc với các Kế toán trưởng thuộc khối doanh nghiệp bán lẻ, sản xuất và logistics đa tài khoản.
  - Khảo sát thực tế phương pháp đối chiếu Sổ cái TK 112 (1121 tiền Việt, 1122 ngoại tệ) với sổ phụ ngân hàng định kỳ ngày/tuần/tháng theo Thông tư 200/2014/TT-BTC và Thông tư 133/2016/TT-BTC.
  - Phân tích chi tiết quy tắc bóc tách phí ngân hàng ẩn (1.100đ, 2.200đ, 9.900đ, 11.000đ), thuế GTGT phí dịch vụ ngân hàng (hạch toán vào TK 1331), chênh lệch tỷ giá vi mô (TK 515/635) và các giao dịch cấn trừ công nợ đa bên (1:N thanh toán gộp, N:1 trả góp nhiều lần).
- **Vai trò phụ trách chính**: Kế toán trưởng (KTT / SME)
- **Vai trò phối hợp**: Solution Architect (SA)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: Không (Khởi động ngay Ngày 1)
- **Sản phẩm bàn giao**: Báo cáo Tổng hợp Khảo sát Nghiệp vụ Kế toán trưởng (`docs/survey/01_chief_accountant_interview_synthesis.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Lập danh mục tối thiểu 15 điểm đau và trường hợp ngoại lệ kèm mẫu chứng từ thực tế chứng minh.
  2. Bảng ma trận đối ứng tài khoản kế toán chi tiết cho 100% các loại nghiệp vụ ngân hàng phổ biến (rút/nộp tiền mặt, chi trả nợ vay, thu nợ khách hàng, trích phí tự động, cấn trừ công nợ).
  3. Báo cáo được Kế toán trưởng và Solution Architect ký duyệt đồng thuận.

#### [WBS 0.1.2] Khảo sát Nghiệp vụ Thủ quỹ, Giờ Cutoff & Luồng Lập Lệnh Chi
- **Mô tả chi tiết phạm vi**:
  - Khảo sát thực địa vị trí Thủ quỹ / Kế toán thanh toán tác nghiệp trên các cổng E-Banking doanh nghiệp lớn.
  - Thu thập và ghi nhận chi tiết các mốc thời gian Cutoff Time thanh toán liên ngân hàng của các kênh: CITAD (15:30–16:00), NAPAS 24/7 (hạn mức kỹ thuật 500 triệu/lệnh), chuyển tiền nội bộ cùng hệ thống (16:30), và chuyển tiền quốc tế SWIFT.
  - Khảo sát quy trình luân chuyển hồ sơ chứng từ gốc (Hóa đơn GTGT, Hợp đồng kinh tế, Giấy đề nghị thanh toán, Phiếu chi) đến khi xuất bản lệnh Ủy nhiệm chi (UNC) và trình duyệt.
- **Vai trò phụ trách chính**: Full-stack Engineer (FSE)
- **Vai trò phối hợp**: Kế toán trưởng (KTT / SME)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: Không (Khởi động Ngày 1)
- **Sản phẩm bàn giao**: Tài liệu Phân tích Nghiệp vụ Thủ quỹ & Mốc Giờ Cutoff (`docs/survey/02_treasury_cashier_workflow_analysis.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Bảng danh mục mốc thời gian Cutoff Time và hạn mức kỹ thuật chi tiết của tối thiểu 5 ngân hàng thương mại lớn (VCB, TCB, BIDV, CTG, MBB).
  2. Sơ đồ chu trình sống của lệnh Ủy nhiệm chi từ trạng thái `DRAFT` -> `PENDING_MAKER` -> `PENDING_CHECKER` -> `EXECUTED` -> `RECONCILED`.
  3. Liệt kê đầy đủ các nguyên nhân khiến lệnh chi bị ngân hàng từ chối (sai tên thụ hưởng, sai mã chi nhánh CITAD, thiếu số dư khả dụng).

#### [WBS 0.1.3] Khảo sát Định hướng Quản trị Thanh khoản & Nhu cầu của CFO
- **Mô tả chi tiết phạm vi**:
  - Phỏng vấn định hướng với 03 Giám đốc Tài chính (CFO) về kỳ vọng đối với Tháp canh Ngân quỹ (Treasury Sentinel).
  - Thu thập các công thức tính toán và chỉ số đo lường sức khỏe tài chính: Tỷ số thanh toán nhanh (Quick Ratio), Hệ số khả năng trả nợ (DSCR), Số ngày xoay vòng tiền mặt (Cash Conversion Cycle), và Đường băng tiền mặt (Cash Runway).
  - Khảo sát nhu cầu cảnh báo sớm thâm hụt trước 24h–48h và quy tắc điều chuyển vốn nội bộ tự động (Cash Concentration / Sweeping) nhằm tối ưu hóa lợi suất tiền gửi qua đêm.
- **Vai trò phụ trách chính**: Solution Architect (SA)
- **Vai trò phối hợp**: Kế toán trưởng (KTT / SME)
- **Ước lượng ngày công**: 2 Man-days
- **Task phụ thuộc**: WBS 0.1.1, WBS 0.1.2
- **Sản phẩm bàn giao**: Đặc tả Yêu cầu Quản trị Dòng tiền & Dashboard CFO (`docs/survey/03_cfo_liquidity_requirements.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Định nghĩa chuẩn xác 100% công thức toán học xác định cho các chỉ số thanh khoản, không dùng ước lượng định tính.
  2. Quy chuẩn hóa ngưỡng cảnh báo thâm hụt tiền mặt trước tối thiểu 24–48 giờ kèm hành động gợi ý cụ thể.
  3. Bảng phân quyền hạn mức duyệt chi tài chính đa tầng (Threshold Hierarchy Matrix) theo quy mô công ty.

#### [WBS 0.1.4] Chuẩn hóa User Journey & Đặc tả Luồng Tác nghiệp BPMN 2.0
- **Mô tả chi tiết phạm vi**:
  - Hợp nhất toàn bộ kết quả khảo sát từ KTT, Thủ quỹ và CFO thành bộ hồ sơ Hành trình Người dùng chuẩn mực.
  - Xây dựng sơ đồ quy trình tác nghiệp chuẩn BPMN 2.0 cho 4 hành trình cốt lõi:
    1. Tiếp nhận sao kê, bóc tách và đối soát tự động 3 tầng (Khớp 1:1, Heuristic mờ, Subset-Sum 1:N / N:1).
    2. Điều hướng và xử lý hàng đợi giao dịch sai lệch / ngoại lệ (Human-in-the-Loop Discrepancy Queue).
    3. Quy trình khởi tạo, soát xét và phê duyệt lệnh chi Maker-Checker hai vòng bảo mật.
    4. Giám sát thanh khoản tập trung, dự báo dòng tiền 30–90 ngày và khuyến nghị điều chuyển vốn.
  - Định nghĩa máy trạng thái (State Machine) toàn diện cho từng thực thể dữ liệu tài chính trong hệ thống.
- **Vai trò phụ trách chính**: Solution Architect (SA)
- **Vai trò phối hợp**: Full-stack Engineer (FSE), QA / SDET Engineer (QAE)
- **Ước lượng ngày công**: 4 Man-days
- **Task phụ thuộc**: WBS 0.1.1, WBS 0.1.2, WBS 0.1.3
- **Sản phẩm bàn giao**: Tài liệu Đặc tả Hành trình Người dùng & Sơ đồ BPMN 2.0 (`docs/specs/01_banking_user_journeys_bpmn.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. 100% các luồng tác nghiệp được mô hình hóa bằng BPMN 2.0 phân tách rõ ràng từng swimlane vai trò (Maker, Checker, Auditor, System Engine).
  2. Máy trạng thái giao dịch bao phủ 100% các trường hợp biên và điều kiện ngắt mạch an toàn (Fail-Closed).
  3. Đạt phê duyệt ký xác nhận từ cả khối Nghiệp vụ (KTT) và khối Kỹ thuật (SA, FSE).

---

### Luồng 0.2: Thu Thập & Ẩn Danh Tập Golden Files (≥ 3 Ngân Hàng + CAMT.053)

#### [WBS 0.2.1] Thu thập Tập Mẫu Sao Kê Thực tế Đa Ngân hàng & CAMT.053 XML
- **Mô tả chi tiết phạm vi**:
  - Thu thập tối thiểu 20 tập tin mẫu sao kê thực tế từ tối thiểu 04 ngân hàng thương mại lớn tại Việt Nam:
    1. Vietcombank (VCB): Mẫu Excel `.xlsx` có ô gộp nhiều cấp (merged cells), phân trang phức tạp.
    2. Techcombank (TCB): Mẫu CSV UTF-8 BOM và mẫu Excel doanh nghiệp lớn có mã giao dịch FT.
    3. BIDV: Mẫu PDF bảng biểu xuất từ iBank và mẫu Excel giao dịch.
    4. VietinBank (CTG) hoặc MBBank (MBB): Mẫu sao kê tài khoản thanh toán đa năng.
  - Thu thập bộ mẫu điện chuẩn quốc tế ISO 20022 `camt.053.001.08` XML đầy đủ các khối thẻ `<BkToCstmrStmt>`, `<Bal>`, `<Ntry>`.
  - Phân loại tập mẫu theo các kịch bản thử thách: giao dịch phí ngầm, cấn trừ công nợ, chuỗi VietQR, giao dịch đêm, giao dịch sai lệch số dư.
- **Vai trò phụ trách chính**: Kế toán trưởng (KTT / SME)
- **Vai trò phối hợp**: QA / SDET Engineer (QAE)
- **Ước lượng ngày công**: 4 Man-days
- **Task phụ thuộc**: Không (Bắt đầu Tuần 1)
- **Sản phẩm bàn giao**: Thư mục Mẫu Sao Kê Thô & Bảng Danh mục Phân loại (`data/golden_raw/manifest.json`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Đạt tối thiểu 20 tệp sao kê đại diện cho $\ge 4$ ngân hàng Việt Nam và chuẩn ISO 20022 CAMT.053 XML.
  2. Tệp `manifest.json` ghi nhận đầy đủ metadata: Tên ngân hàng, Định dạng tệp, Bảng mã ký tự, Loại kịch bản thử thách, Tổng số dòng giao dịch.
  3. Dữ liệu thô được lưu trữ an toàn trong vùng đệm cách ly mạng nội bộ.

#### [WBS 0.2.2] Thiết kế Bộ Quy tắc Nhận diện & Che mờ PII (Nghị định 13)
- **Mô tả chi tiết phạm vi**:
  - Xây dựng bộ quy tắc biểu thức chính quy (Regex Engine Specs) nhận diện toàn bộ các thực thể dữ liệu cá nhân nhạy cảm: Số CCCD 12 chữ số (`\b0\d{11}\b`), Số điện thoại VN, Số tài khoản ngân hàng cá nhân.
  - Xây dựng từ điển tên thể nhân tiếng Việt dựa trên danh mục họ và tên đệm phổ biến.
  - Thiết lập cơ chế Whitelist bảo toàn 100% các thực thể pháp nhân doanh nghiệp (CÔNG TY, TNHH, CP, TẬP ĐOÀN, NGÂN HÀNG) và mã chứng từ, mã hóa đơn.
  - Thiết kế quy tắc phân tách chặt chẽ giữa số tài khoản và số tiền giao dịch độc lập để không làm biến dạng số liệu kế toán.
- **Vai trò phụ trách chính**: Security Engineer (SE)
- **Vai trò phối hợp**: AI & SLM Engineer (AIE)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 0.2.1
- **Sản phẩm bàn giao**: Đặc tả Quy tắc Khử PII & Fixture Kiểm thử (`docs/specs/02_pii_sanitization_rules.md`, `tests/fixtures/pii_rules_spec.json`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Tỷ lệ nhận diện đúng (Recall) dữ liệu CCCD, SĐT, Số tài khoản đạt $100\%$ trên bộ fixture kiểm thử 500 mẫu biên.
  2. Tỷ lệ che mờ nhầm (False-Positive) tên công ty hoặc mã hóa đơn kinh doanh đạt đúng $0.0\%$.
  3. Vượt qua các test case phân biệt đối tượng nhạy cảm phức tạp (ví dụ: tên cá nhân bị che mờ, nhưng tên pháp nhân hoặc trường học được bảo toàn).

#### [WBS 0.2.3] Lập trình Công cụ Ẩn danh Offline & Tạo Tập Golden Dataset
- **Mô tả chi tiết phạm vi**:
  - Phát triển công cụ dòng lệnh (CLI Utility) bằng Rust chạy hoàn toàn nội bộ (Zero Network Egress) thực thi các quy tắc từ WBS 0.2.2.
  - Xử lý toàn bộ 20+ tệp sao kê thô trong `data/golden_raw/`, sinh ra tập dữ liệu chuẩn hóa `data/golden_clean/` với $\ge 5.000$ dòng giao dịch đã được khử sạch PII.
  - Tạo lập tệp ánh xạ nhãn đúng (Ground-Truth Labels) cho từng dòng: ngân hàng phát hành, số tiền gốc, phí bóc tách, hóa đơn ERP đối ứng, và kết quả mong đợi (Khớp 1:1, Khớp mờ, hoặc Ngoại lệ).
  - Tính toán mã băm SHA-256 niêm phong tập dữ liệu kiểm toán.
- **Vai trò phụ trách chính**: Senior Rust Systems Engineer (RE2)
- **Vai trò phối hợp**: QA / SDET Engineer (QAE), Security Engineer (SE)
- **Ước lượng ngày công**: 4 Man-days
- **Task phụ thuộc**: WBS 0.2.1, WBS 0.2.2
- **Sản phẩm bàn giao**: Mã nguồn Công cụ Ẩn danh Offline (`tools/golden_anonymizer/`) & Tập Dữ liệu Vàng Đã Khử PII Kèm Ground-Truth (`data/golden_clean/manifest.json`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. $100\%$ các tệp trong `data/golden_clean/` được quét độc lập và khẳng định không còn bất kỳ số CCCD, SĐT hoặc số tài khoản thực nào chưa được che mờ.
  2. Tập Ground-Truth bao phủ đầy đủ $\ge 5.000$ dòng giao dịch với nhãn phân bổ khớp 1:1, khớp mờ, chia tách 1:N và ngoại lệ.
  3. Bằng chứng giám sát mạng khẳng định công cụ chạy offline hoàn toàn, 0 byte truyền ra ngoài.

---

### Luồng 0.3: Rà Soát Căn Cứ Pháp Lý & Thiết Lập Khung Tuân Thủ Ngân Hàng

#### [WBS 0.3.1] Rà soát Căn cứ Pháp lý & Ma trận Tuân thủ Nghị định 13/2023/NĐ-CP
- **Mô tả chi tiết phạm vi**:
  - Rà soát toàn diện các điều khoản của Nghị định 13/2023/NĐ-CP liên quan đến hoạt động xử lý dữ liệu tài chính: Điều 2 Khoản 4.d (Dữ liệu cá nhân nhạy cảm), Điều 9 (Quyền của chủ thể dữ liệu), Điều 13 (Nghĩa vụ bảo vệ dữ liệu), Điều 24 (Hồ sơ Đánh giá tác động xử lý dữ liệu nội địa) và Điều 25 (Đánh giá tác động chuyển dữ liệu ra nước ngoài).
  - Xây dựng Thư tư vấn pháp lý chính thức (Legal Opinion) khẳng định: Cơ chế Local-First / Zero Data Egress của LIVA giúp doanh nghiệp không phát sinh hành vi chuyển dữ liệu xuyên biên giới, loại trừ rủi ro vi phạm Điều 25.
  - Xây dựng Ma trận Ánh xạ Tuân thủ (Compliance Traceability Matrix) liên kết từng điều khoản luật với tính năng kỹ thuật tương ứng.
- **Vai trò phụ trách chính**: Tư vấn Pháp lý (TVPL / LC)
- **Vai trò phối hợp**: Security Engineer (SE)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: Không (Khởi động Tuần 1)
- **Sản phẩm bàn giao**: Thư Tư vấn Pháp lý & Ma trận Tuân thủ Nghị định 13 (`docs/compliance/01_decree13_legal_opinion_and_matrix.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Thư tư vấn pháp lý hoàn chỉnh có xác nhận chuyên môn của luật sư thành viên Đoàn Luật sư.
  2. Ma trận đối chiếu chi tiết 100% các nghĩa vụ của Bên Kiểm soát dữ liệu và Bên Xử lý dữ liệu tương ứng với kiến trúc LIVA.
  3. Luận cứ pháp lý chứng minh rõ ràng điều kiện miễn trừ nghĩa vụ nộp hồ sơ Điều 25 khi chạy 100% on-premise.

#### [WBS 0.3.2] Thiết lập Tiêu chuẩn An toàn HTTT Ngân hàng (Thông tư 09) & Luật Các TCTD 2024
- **Mô tả chi tiết phạm vi**:
  - Rà soát các quy định về an toàn hệ thống thông tin ngành ngân hàng tại Thông tư 09/2020/TT-NHNN: Tiêu chuẩn an toàn Cấp độ 3 đến Cấp độ 5, quy định mã hóa dữ liệu nhạy cảm bằng AES-256 (Điều 17–20), nguyên tắc bảo mật khóa độc lập (HYOK).
  - Rà soát các điều khoản bảo mật thông tin khách hàng tại Điều 10, 11 Luật Các tổ chức tín dụng 2024 (có hiệu lực từ 01/07/2024).
  - Xây dựng bộ yêu cầu kỹ thuật bắt buộc cho cơ chế Kiểm soát kép (Maker-Checker / 4-Eyes Principle) và sổ cái kiểm toán bất biến (Immutable Audit Trail) đáp ứng tiêu chuẩn thanh tra của Cơ quan Thanh tra, giám sát ngân hàng (NHNN).
- **Vai trò phụ trách chính**: QA / SDET Engineer (QAE)
- **Vai trò phối hợp**: Tư vấn Pháp lý (TVPL / LC), Security Engineer (SE)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: Không (Khởi động Tuần 1)
- **Sản phẩm bàn giao**: Bộ Khung Tiêu Chuẩn Kỹ Thuật Tuân Thủ Thông Tư 09 & Luật TCTD (`docs/compliance/02_circular09_banking_security_baseline.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Bản đặc tả chi tiết 100% các tiêu chí an toàn thông tin Cấp độ 3 áp dụng cho phân hệ LIVA DMZ Node.
  2. Định nghĩa quy chuẩn giao thức Maker-Checker: 2 phiên xác thực phân quyền tách biệt, cơ chế ký băm HMAC-SHA256, và token hết hạn sau 15 phút.
  3. Được xác nhận phù hợp với quy trình kiểm toán nội bộ của khối ngân hàng thương mại.

#### [WBS 0.3.3] Thiết lập Khung Giám sát Rủi ro Rửa tiền AML & Mẫu Báo cáo STR
- **Mô tả chi tiết phạm vi**:
  - Rà soát Luật Phòng, chống rửa tiền 2022 (Luật số 14/2022/QH15), Quyết định 11/2023/QĐ-TTg về mức giao dịch lớn phải báo cáo ($\ge 400.000.000\text{ VNĐ}$), và Thông tư 09/2023/TT-NHNN hướng dẫn thi hành luật.
  - Định nghĩa chính xác tham số và logic phát hiện cho 4 mẫu hình giao dịch nghi vấn trọng yếu:
    1. Chia nhỏ dòng tiền né ngưỡng 400M (Structuring/Smurfing).
    2. Tài khoản trung chuyển thần tốc (Rapid Pass-through Churn: nhận tiền và giải ngân > 95% trong 15 phút).
    3. Đột biến doanh số giao dịch bất thường (Velocity Surge > 300% đường xu hướng 90 ngày).
    4. Giao dịch đêm bất thường (Night Velocity trong khung giờ 23:00–05:00).
  - Chuẩn hóa cấu trúc dữ liệu JSON sinh tự động Mẫu Báo cáo Giao dịch Đáng ngờ (Form STR) nộp Cục PCRT - Ngân hàng Nhà nước.
- **Vai trò phụ trách chính**: AI & SLM Engineer (AIE)
- **Vai trò phối hợp**: Tư vấn Pháp lý (TVPL / LC), Kế toán trưởng (KTT / SME)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: Không (Khởi động Tuần 1)
- **Sản phẩm bàn giao**: Đặc Tả Quy Tắc Giám Sát AML/CTF & Lược Đồ Báo Cáo STR (`docs/compliance/03_aml_ctf_regulatory_spec.md`, `templates/nhnn_str_report_template.json`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Định nghĩa chuẩn xác 100% công thức toán học và điều kiện biên của 4 dấu hiệu nghi vấn AML.
  2. Lược đồ JSON biểu diễn Form STR phản ánh đầy đủ 100% các trường thông tin theo quy định của Cục PCRT - NHNN.
  3. Kế toán trưởng và Tư vấn pháp lý phê duyệt tính pháp lý của quy trình cảnh báo.

#### [WBS 0.3.4] Xây dựng Bộ Hồ sơ Đánh giá Tác động Dữ liệu Cá nhân (DPIA) Mẫu 04
- **Mô tả chi tiết phạm vi**:
  - Soạn thảo trọn bộ hồ sơ Đánh giá Tác động Xử lý Dữ liệu Cá nhân (DPIA) theo đúng thể thức Mẫu số 04 ban hành kèm theo Nghị định 13/2023/NĐ-CP dành cho doanh nghiệp triển khai LIVA.
  - Phân tích chi tiết: Mục đích xử lý dữ liệu (đối soát tài chính), Loại dữ liệu cá nhân thu thập (số tài khoản, tên cá nhân), Biện pháp kỹ thuật bảo vệ (Mã hóa AES-256-GCM, DPAPI, PolicyEngine), Đánh giá rủi ro lộ lọt và phương án ứng phó sự cố an ninh mạng.
  - Thiết lập quy trình lưu trữ hồ sơ sẵn sàng tại doanh nghiệp để xuất trình khi có yêu cầu thanh kiểm tra từ Cục An ninh mạng và phòng, chống tội phạm sử dụng công nghệ cao (A05 - Bộ Công an).
- **Vai trò phụ trách chính**: AI & SLM Engineer (AIE)
- **Vai trò phối hợp**: Tư vấn Pháp lý (TVPL / LC), Security Engineer (SE)
- **Ước lượng ngày công**: 4 Man-days
- **Task phụ thuộc**: WBS 0.3.1, WBS 0.3.2
- **Sản phẩm bàn giao**: Bộ Hồ Sơ DPIA Hoàn Chỉnh Mẫu 04 Nghị Định 13 (`docs/compliance/04_dpia_dossier_decree13_form04.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Hồ sơ điền đầy đủ 100% các mục theo quy định của Mẫu số 04 Nghị định 13/2023/NĐ-CP.
  2. Thể hiện rõ ràng các biện pháp bảo vệ kỹ thuật chứng minh rủi ro chuyển dữ liệu xuyên biên giới là bằng 0.
  3. Đạt phê duyệt cuối cùng từ Tư vấn pháp lý trưởng của dự án.

---

### Luồng 0.4: Khảo Sát Định Dạng Tích Hợp ERP (MISA, FAST, Bravo, SAP)

#### [WBS 0.4.1] Phân tích Cấu trúc Nhập/Xuất ERP Nội địa (MISA, FAST, Bravo)
- **Mô tả chi tiết phạm vi**:
  - Thu thập và phân tích chi tiết cấu trúc tệp dữ liệu Excel, XML, CSV dùng cho chức năng Import chứng từ kế toán của 3 hệ thống ERP nội địa chiếm thị phần lớn nhất:
    1. MISA AMIS / MISA SME: Mẫu nạp Báo Nợ (Ủy nhiệm chi), Báo Có (Thu tiền gửi), Bảng kê ngân hàng.
    2. FAST Business Online / FAST Financial: Mẫu nạp chứng từ chi tiền gửi (BC1, BN1), thu tiền gửi ngân hàng.
    3. Bravo 8 ERP-VN: Mẫu nạp bảng kê ngân quỹ và cơ chế bảng trung gian Staging Table.
  - Lập bảng ma trận ánh xạ (Field Mapping Matrix) giữa các trường sao kê ngân hàng và các trường dữ liệu bắt buộc của từng hệ thống kế toán.
  - Phân tích quy tắc hạch toán tài khoản kế toán Việt Nam (TK 1121, 1122, 131, 331, 511, 642, 635, 811) và mã hóa đối tượng công nợ.
- **Vai trò phụ trách chính**: Full-stack Engineer (FSE)
- **Vai trò phối hợp**: Solution Architect (SA), Kế toán trưởng (KTT / SME)
- **Ước lượng ngày công**: 4 Man-days
- **Task phụ thuộc**: WBS 0.1.1
- **Sản phẩm bàn giao**: Đặc tả Cấu trúc Dữ liệu ERP Nội địa & Bộ Tệp Template (`docs/integration/01_domestic_erp_schemas_misa_fast_bravo.md`, `data/erp_templates/`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Bảng ánh xạ trường chi tiết 100% các cột bắt buộc, kiểu dữ liệu, định dạng ngày (`DD/MM/YYYY`) và số tiền của MISA, FAST, Bravo.
  2. Bộ tệp Excel/XML mẫu được kiểm thử nạp thực tế vào phần mềm MISA AMIS và FAST thành công không phát sinh lỗi cú pháp.
  3. Kế toán trưởng ký xác nhận tính chính xác về mặt định khoản đối ứng.

#### [WBS 0.4.2] Khảo sát Chuẩn Giao tiếp Sao kê Điện tử SAP EBS / BAPI
- **Mô tả chi tiết phạm vi**:
  - Nghiên cứu chuẩn giao tiếp sao kê điện tử ngân hàng của hệ thống SAP ERP (SAP ECC 6.0 và SAP S/4HANA):
    1. Chuẩn Multicash: Cấu trúc tệp Header `AUSZUG.TXT` và dòng giao dịch chi tiết `UMSATZ.TXT`.
    2. Chuẩn ISO 20022 CAMT.053 XML dùng cho tính năng Electronic Bank Statement (EBS) qua giao dịch `FF_5`.
  - Khảo sát giao thức hạch toán chứng từ kế toán tổng hợp qua BAPI `BAPI_ACC_DOCUMENT_POST` hoặc mẫu bảng tính upload General Ledger (`FB50`/`FB01`).
  - Tài liệu hóa quy trình hạch toán hai bước kinh điển của SAP: Bước 1 ghi nhận vào tài khoản ngân hàng trung gian (Bank Clearing Account); Bước 2 cấn trừ tự động vào tài khoản đối tác (Subledger Reconciliation).
- **Vai trò phụ trách chính**: Solution Architect (SA)
- **Vai trò phối hợp**: Lead Rust Engineer (RE1)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: Không (Bắt đầu Tuần 1)
- **Sản phẩm bàn giao**: Đặc tả Kỹ thuật Tích hợp SAP EBS & Lược đồ BAPI (`docs/integration/02_sap_ebs_and_gl_voucher_spec.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Phân tích đầy đủ cấu trúc từng trường của tệp Multicash `AUSZUG` và `UMSATZ` (Bank key, Account number, Statement number, Posting date, Amount, Note to payee).
  2. Đặc tả đầy đủ tham số đầu vào của BAPI `BAPI_ACC_DOCUMENT_POST` (Document Header, Account GL, Account Payable, Account Receivable, Currency Amount).
  3. Đạt phê duyệt kiến trúc từ Solution Architect.

#### [WBS 0.4.3] Thiết kế Lược đồ Chứng từ Kế toán Chuẩn hóa `CanonicalVoucher`
- **Mô tả chi tiết phạm vi**:
  - Thiết kế cấu trúc dữ liệu trung gian chuẩn hóa `CanonicalVoucher` bằng ngôn ngữ Rust và định nghĩa JSON Schema tương ứng.
  - Cấu trúc bao gồm:
    * Khối Header: Mã định danh duy nhất (`voucher_id`), loại chứng từ, ngày chứng từ, ngày ghi sổ, mã tài khoản ngân hàng nguồn, tổng số tiền.
    * Khối Danh sách định khoản (`entries`): Danh sách các cặp định khoản kép Nợ/Có, tài khoản đối ứng, số tiền nguyên tỷ lệ 64-bit (`i64` VND), mã đối tượng công nợ, mã hóa đơn liên kết, diễn giải thanh toán đã làm sạch.
    * Khối Bất biến kế toán (`invariants`): $\sum \text{Debit} \equiv \sum \text{Credit} \equiv \text{Total Amount}$.
  - Thiết kế các trait chuyển đổi (Converter Traits) trong Rust cho phép biến đổi một chiều từ `CanonicalVoucher` sang các định dạng xuất tương ứng: MISA Excel/XML, FAST Excel, Bravo Excel, SAP Multicash.
- **Vai trò phụ trách chính**: Solution Architect (SA)
- **Vai trò phối hợp**: Lead Rust Engineer (RE1), Kế toán trưởng (KTT / SME)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 0.4.1, WBS 0.4.2
- **Sản phẩm bàn giao**: Đặc tả Lược đồ CanonicalVoucher & JSON Schema Chuẩn (`docs/specs/03_canonical_voucher_schema.md`, `schemas/canonical_voucher.json`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Tệp JSON Schema đạt chuẩn Draft-07, vượt qua bộ kiểm tra cú pháp tự động `check-jsonschema`.
  2. Khẳng định 100% số tiền sử dụng số nguyên `i64` VND, loại trừ hoàn toàn kiểu số thực dấu phẩy động.
  3. Lược đồ có khả năng biểu diễn đầy đủ 100% các nghiệp vụ thu/chi/điều chuyển tiền gửi của cả 4 hệ thống ERP mục tiêu.

#### [WBS 0.4.4] Thiết kế Kiến trúc Tích hợp Ngoại vi Phi Xâm lấn & Vòng lặp Phản hồi Đóng
- **Mô tả chi tiết phạm vi**:
  - Thiết kế kiến trúc tích hợp ngoại vi (Peripheral Harness Architecture) phân định 2 chế độ vận hành độc lập:
    * *Track 1 (Doanh nghiệp - CFO / Kế toán)*: Module Hot-Folder Watcher sử dụng API Win32 `ReadDirectoryChangesW` (crate `notify` của Rust) giám sát thư mục tải về máy trạm; bộ sinh tệp chứng từ mẫu 1-Click Import nạp vào MISA/FAST/Bravo.
    * *Track 2 (Ngân hàng - DMZ Node)*: Lắng nghe thư mục SFTP nội bộ an toàn nhận điện CAMT.053 / MT940; kết nối cơ sở dữ liệu bản sao chỉ đọc (Read-Replica) qua bộ lọc ngữ pháp AST chỉ cho phép lệnh `SELECT`.
  - Thiết kế cơ chế Vòng lặp Phản hồi Đóng (Closed-Loop Confirmation): Đọc lại tệp log phản hồi kết quả import từ ERP để đóng trạng thái hạch toán (`POSTED_ERP`), ngăn ngừa triệt để tình trạng phân mảnh trạng thái giữa LIVA và phần mềm kế toán.
- **Vai trò phụ trách chính**: Senior Rust Systems Engineer (RE2)
- **Vai trò phối hợp**: Solution Architect (SA), Full-stack Engineer (FSE)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 0.1.4, WBS 0.4.3
- **Sản phẩm bàn giao**: Bản Thiết Kế Kiến Trúc Tích Hợp Phi Xâm Lấn Dual-Track (`docs/specs/04_non_invasive_integration_architecture.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Kiến trúc phân tách rõ ràng luồng dữ liệu của Track 1 và Track 2, khẳng định không yêu cầu quyền Admin hoặc cài driver hệ điều hành.
  2. Cơ chế Closed-Loop định nghĩa rõ luồng xử lý khi tệp import bị ERP từ chối (rollback trạng thái và cảnh báo người dùng).
  3. Được phê duyệt bởi Solution Architect và Security Engineer.

---

### Luồng 0.5: Đánh Giá Công Nghệ Lõi Rust, Database, UI Framework & Shortlist SLM

#### [WBS 0.5.1] Đánh giá & Chuẩn hóa Hệ sinh thái Rust Workspace
- **Mô tả chi tiết phạm vi**:
  - Khảo sát và thiết lập cấu hình Rust workspace (`Cargo.toml` mẹ), lựa chọn phiên bản Rust 2024 (MSRV 1.82+), chuẩn hóa các crates phụ thuộc (`tokio`, `rusqlite`, `serde`, `thiserror`, `rayon`).
  - Cấu hình `.cargo/config.toml` khống chế cờ biên dịch `-j 2`, `target-cpu=native`, và các profile `dev`, `release`.
  - Thiết lập linter clippy nghiêm ngặt với các nhóm cấm rò rỉ bộ nhớ, unwrap bừa bãi và sai số số học.
- **Vai trò phụ trách chính**: Lead Rust Engineer (RE1)
- **Vai trò phối hợp**: Solution Architect (SA), QA / SDET Engineer (QAE)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: Không (Bắt đầu Tuần 1)
- **Sản phẩm bàn giao**: Báo cáo Kiến trúc Workspace & Cấu trúc Thư mục Scaffold (`docs/architecture/01_rust_workspace_baseline.md`, `Cargo.toml`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Lệnh `cargo check -j 2` hoàn thành dưới 30 giây trên toàn bộ workspace scaffold.
  2. Phân định ranh giới crate tường minh: `crates/liva-money`, `crates/liva-ledger`, `crates/liva-netguard`, `crates/liva-core-banking`, `crates/liva-web-gateway`.
  3. 100% crates phụ thuộc không có lỗ hổng bảo mật đã biết theo `cargo audit`.

#### [WBS 0.5.2] Thực nghiệm Hiệu năng & Quyết định SQLite WAL vs PostgreSQL
- **Mô tả chi tiết phạm vi**:
  - Xây dựng harness đo lường trực tiếp thông lượng ghi/đọc (writes/sec, latency p99) giữa SQLite WAL (cấu hình `page_size=32768`, `mmap_size=268435456`) và PostgreSQL cục bộ trên 100.000 giao dịch mô phỏng.
  - Đo kiểm mức tiêu thụ RAM thường trú (RSS) và đánh giá rủi ro an ninh mạng khi mở cổng TCP socket (Port 5432) so với truy cập file descriptor thuần túy.
  - Đánh giá khả năng kiểm soát khóa mã hóa cấp trường (Field-level encryption) và cơ chế sao lưu nóng (Online Backup API).
- **Vai trò phụ trách chính**: Senior Rust Systems Engineer (RE2)
- **Vai trò phối hợp**: Solution Architect (SA), Security Engineer (SE)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 0.5.1
- **Sản phẩm bàn giao**: Báo cáo Thực nghiệm Đo kiểm DB Benchmark & Quyết định Kiến trúc ADR-001 (`docs/adr/ADR-001-sqlite-wal-storage-engine.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. SQLite WAL đạt thông lượng ghi $\ge 50.000\text{ writes/sec}$ trong batch giao dịch.
  2. Mức tiêu thụ RAM thường trú của SQLite duy trì $\le 50\text{ MB}$ (tiết kiệm hơn 85% so với daemon PostgreSQL).
  3. Xác nhận 0 socket mạng mở, đáp ứng 100% ranh giới an ninh Zero-Egress.

#### [WBS 0.5.3] Khảo sát Khung Giao diện Local UI (Tauri v2 vs Local Web)
- **Mô tả chi tiết phạm vi**:
  - Đánh giá kiến trúc Tauri v2 + Vue 3 / Vite so với Local Web Server thuần túy (`127.0.0.1`); kiểm tra mức chiếm dụng RAM, khả năng hiển thị bảng dữ liệu ảo hóa (Virtual Table Grid) 50.000 dòng mượt mà với 60fps.
  - Kiểm tra cơ chế bảo mật phân quyền IPC capabilities: vô hiệu hóa hoàn toàn `withGlobalTauri`, phân tách kênh lệnh giữa Widget tra cứu và Dashboard quản trị.
  - Thiết kế lớp trừu tượng `PlatformAdapter` cho phép chạy hoán đổi linh hoạt giữa Desktop App và Local Web Browser.
- **Vai trò phụ trách chính**: Full-stack Engineer (FSE)
- **Vai trò phối hợp**: Solution Architect (SA), Security Engineer (SE)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 0.5.1
- **Sản phẩm bàn giao**: Báo cáo Đánh giá Kiến trúc UI ADR-002 & PoC Bảng Ảo Hóa (`docs/adr/ADR-002-local-ui-framework.md`, `poc/virtual-grid/`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Bảng dữ liệu ảo hóa cuộn mượt mà 60fps với 50.000 dòng sao kê, độ trễ phản hồi $< 16\text{ ms}$.
  2. RAM tiến trình UI đo kiểm thực tế $\le 120\text{ MB}$.
  3. Phân tách rành mạch quyền hạn IPC, cấm gọi lệnh nhạy cảm từ cửa sổ chưa xác thực.

#### [WBS 0.5.4] Thẩm định & Rút gọn SLM Kèm Rà soát Bản quyền Thương mại
- **Mô tả chi tiết phạm vi**:
  - Đánh giá độ chính xác trích xuất thực thể ngân hàng Việt Nam giữa các ứng viên SLM: Qwen 2.5 (3B / 7B), PhoGPT (4B / 7B), Gemma 2 (2B / 9B) và Llama 3.2 (3B).
  - Lượng tử hóa mô hình sang định dạng GGUF chuẩn `Q4_K_M`, đo tốc độ suy luận (tokens/sec) trên CPU máy trạm phổ thông hỗ trợ tập lệnh AVX2.
  - Rà soát điều khoản bản quyền thương mại: phân tích tính mở của giấy phép Apache 2.0 so với các điều khoản hạn chế của Meta Llama Community License và Google Gemma Terms of Use.
- **Vai trò phụ trách chính**: AI & SLM Engineer (AIE)
- **Vai trò phối hợp**: Tư vấn Pháp lý (TVPL / LC), Solution Architect (SA)
- **Ước lượng ngày công**: 4 Man-days
- **Task phụ thuộc**: Không (Bắt đầu Tuần 1)
- **Sản phẩm bàn giao**: Báo cáo Đánh giá SLM & Hồ sơ Thẩm định Bản quyền Thương mại (`docs/ai/01_slm_evaluation_and_license_audit.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Qwen 2.5 (3B-Instruct) đạt độ chính xác trích xuất thực thể NER $\ge 96\%$ trên ngữ liệu diễn giải giao dịch ngân hàng Việt Nam.
  2. Tốc độ suy luận CPU đạt $\ge 35\text{ tokens/sec}$ trên CPU Intel Core i5 thế hệ 11 trở lên.
  3. Xác nhận bằng văn bản pháp lý: Giấy phép Apache 2.0 cho phép thương mại hóa, đóng gói on-premise cho khách hàng ngân hàng mà không có nghĩa vụ chia sẻ mã nguồn (non-copyleft).

---

### Luồng 0.6: Mô Hình Hóa Mối Đe Dọa STRIDE + Data Flow Diagram (DFD L0, L1, L2)

#### [WBS 0.6.1] Xây dựng Sơ đồ Luồng Dữ liệu (DFD L0, L1, L2) & Ranh giới Zero-Egress
- **Mô tả chi tiết phạm vi**:
  - Thiết kế toàn diện sơ đồ luồng dữ liệu 3 cấp độ phản ánh chính xác kiến trúc hệ thống:
    * **DFD Level 0 (Context Diagram)**: Xác lập ranh giới hệ thống LIVA cục bộ, tương tác với Kế toán viên, Cổng E-Banking và phần mềm ERP.
    * **DFD Level 1 (Subsystem Diagram)**: Thể hiện 5 phân hệ cốt lõi: Ingestion & Parser, Compliance Sanitizer, Dual-Engine Disentanglement (SLM vs. Rust Math), PolicyEngine Maker-Checker, và Secure Storage SQLite WAL.
    * **DFD Level 2 (Transaction Processing Path)**: Bóc tách chi tiết luồng dữ liệu từ nhận chuỗi thô $\rightarrow$ Che mờ PII $\rightarrow$ Trích xuất thực thể $\rightarrow$ Giải bài toán ràng buộc số học $\rightarrow$ Ghi sổ cái bất biến.
  - Định hình rõ nét các ranh giới an ninh: Untrusted Zone, Sanitization Boundary, Sandbox Execution Boundary, và Cryptographic Vault Boundary.
- **Vai trò phụ trách chính**: Security Engineer (SE)
- **Vai trò phối hợp**: Solution Architect (SA), Full-stack Engineer (FSE)
- **Ước lượng ngày công**: 4 Man-days
- **Task phụ thuộc**: WBS 0.5.1, WBS 0.5.3
- **Sản phẩm bàn giao**: Bộ Hồ sơ Kiến trúc An ninh Luồng Dữ liệu DFD L0/L1/L2 (`docs/security/01_data_flow_diagrams_l0_l1_l2.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. 100% các điểm tiếp nhận và xử lý dữ liệu nhạy cảm được bao bọc bởi ranh giới an ninh xác định.
  2. Thể hiện rõ kênh truyền loopback nội bộ (`127.0.0.1`), không có bất kỳ luồng dữ liệu nào rò rỉ ra ngoài ranh giới máy trạm.
  3. Sơ đồ vẽ bằng chuẩn Mermaid và SVG được Solution Architect phê duyệt.

#### [WBS 0.6.2] Mô hình hóa Mối đe dọa STRIDE cho Nghiệp vụ Ngân hàng
- **Mô tả chi tiết phạm vi**:
  - Phân tích chi tiết 6 trục đe dọa an ninh theo mô hình STRIDE áp dụng riêng cho nghiệp vụ ngân quỹ:
    1. **Spoofing (Giả mạo)**: Giả mạo file sao kê, giả mạo phiên IPC, giả mạo cán bộ Maker/Checker.
    2. **Tampering (Can thiệp)**: Dùng Hex Editor sửa số dư trong file SQLite, can thiệp mã nhị phân.
    3. **Repudiation (Chối bỏ)**: Kiểm soát viên duyệt chi sau đó chối bỏ trách nhiệm.
    4. **Information Disclosure (Rò rỉ thông tin)**: Rò rỉ PII, lộ số tài khoản ra ngoài Internet.
    5. **Denial of Service (Từ chối dịch vụ)**: Zip bomb, Regex DoS, làm tràn bộ nhớ máy trạm.
    6. **Elevation of Privilege (Leo quyền)**: Tấn công Prompt Injection lừa AI tự hạ cấp rủi ro hoặc tự động ký lệnh chi.
  - Thiết lập ma trận kiểm soát đối ứng chi tiết cho từng kịch bản đe dọa.
- **Vai trò phụ trách chính**: Security Engineer (SE)
- **Vai trò phối hợp**: Solution Architect (SA), Kế toán trưởng (KTT / SME)
- **Ước lượng ngày công**: 4 Man-days
- **Task phụ thuộc**: WBS 0.6.1
- **Sản phẩm bàn giao**: Báo cáo Phân tích Mối đe dọa STRIDE & Ma trận Biện pháp Đối ứng (`docs/security/02_stride_threat_modeling_banking.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Nhận diện tối thiểu 18 kịch bản tấn công/đe dọa đặc thù trong nghiệp vụ ngân quỹ và đối soát.
  2. 100% các mối đe dọa được gắn giải pháp kỹ thuật phòng vệ xác định (Mã hóa AES-256-GCM, Hash-Chain, Two-Phase Confirmation, Anti-Downgrade Invariant).
  3. Báo cáo được Security Engineer và Solution Architect ký nghiệm thu.

#### [WBS 0.6.3] Đặc tả Kỹ thuật Cơ chế Bảo mật Sandbox & Fail-Closed
- **Mô tả chi tiết phạm vi**:
  - Xây dựng đặc tả kỹ thuật chi tiết cho phân hệ `liva-netguard`: nguyên lý lọc gói mạng cấp kernel (`nftables`), bộ lọc lệnh hệ thống (`seccomp-bpf`), và hồ sơ cô lập dịch vụ (`systemd.unit`).
  - Thiết kế nguyên tắc phòng vệ **Fail-Closed**: Bất kỳ khi nào phát hiện dấu hiệu can thiệp CSDL trái phép (sai lệch chữ ký băm HMAC) hoặc sai lệch phương trình kế toán ($\Delta \ne 0$), hệ thống lập tức khóa toàn bộ giao diện và từ chối xuất chứng từ hạch toán.
- **Vai trò phụ trách chính**: Security Engineer (SE)
- **Vai trò phối hợp**: Lead Rust Engineer (RE1), Solution Architect (SA)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 0.6.2
- **Sản phẩm bàn giao**: Bản Đặc Tả Kỹ Thuật Security Sandbox & Fail-Closed Protocol (`docs/security/03_sandbox_and_fail_closed_spec.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Quy định rõ ràng bộ luật nftables chặn 100% IP đích ngoại vi ngoài loopback.
  2. Định nghĩa máy trạng thái khóa an toàn (System Lockdown State) kèm quy trình mở khóa khẩn cấp dành cho Quản trị viên.
  3. Được xác nhận phù hợp với tiêu chuẩn an toàn Cấp độ 3 Thông tư 09/2020/TT-NHNN.

---

### Luồng 0.7: Thiết Lập Hạ Tầng Dev Nội Bộ 100% Self-Hosted

#### [WBS 0.7.1] Thiết lập Máy chủ GitLab CE & Kho Quản trị Mã nguồn Nội bộ
- **Mô tả chi tiết phạm vi**:
  - Cài đặt và cấu hình máy chủ GitLab Community Edition (GitLab CE) trên máy chủ nội bộ (Private LAN), 0 kết nối ra Internet công cộng.
  - Thiết lập cơ chế xác thực đa yếu tố (2FA), phân quyền dự án theo ma trận vai trò (Maintainer, Developer, Reporter).
  - Tích hợp Git Hook kiểm soát an toàn (`pre-receive` secret scrubber): tự động quét và chặn đứng các commit chứa credentials, private keys, hoặc số CCCD/tài khoản khách hàng thực tế.
- **Vai trò phụ trách chính**: Security Engineer (SE)
- **Vai trò phối hợp**: Solution Architect (SA)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: Không (Bắt đầu Tuần 1)
- **Sản phẩm bàn giao**: Máy chủ GitLab CE Nội bộ Hoạt động & Tài liệu Vận hành Hạ tầng (`docs/infra/01_self_hosted_gitlab_setup.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Máy chủ GitLab CE trực tuyến trong mạng LAN nội bộ, ngắt kết nối Internet công cộng.
  2. Git hook `pre-receive` chặn thành công 100% các commit thử nghiệm có chứa private key hoặc số CCCD 12 số.
  3. Cấp phát tài khoản và cấu hình SSH key bảo mật cho toàn bộ 8–9 nhân sự dự án.

#### [WBS 0.7.2] Cấu hình GitLab CI Runner Giới hạn Phần cứng & Air-Gapped
- **Mô tả chi tiết phạm vi**:
  - Triển khai GitLab CI Runner chuyên dụng trên môi trường Linux nội bộ; cấu hình runner chạy tuần tự (`concurrent = 1`) để không gây nghẽn RAM.
  - Thiết lập rào chắn phần cứng qua Linux cgroups v2: khống chế `memory.max = 4G`, `memory.swap.max = 0`, và CPU affinity (tối đa 2 cores).
  - Cấu hình môi trường thực thi Air-Gapped: runner chạy ở chế độ offline, toàn bộ crates phụ thuộc được kéo từ kho đệm cục bộ (`vendor` directory hoặc local sccache mirror).
  - Xây dựng pipeline kiểm định chuẩn `.gitlab-ci.yml`:
    * Stage 1: `cargo fmt --all -- --check`
    * Stage 2: `cargo clippy --workspace --all-targets -j 2 -- -D warnings`
    * Stage 3: `cargo test --workspace -j 2 -- --test-threads 2`
- **Vai trò phụ trách chính**: Security Engineer (SE)
- **Vai trò phối hợp**: Lead Rust Engineer (RE1), QA / SDET Engineer (QAE)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 0.7.1
- **Sản phẩm bàn giao**: GitLab CI Runner Sẵn sàng & Tập tin Cấu hình Pipeline (`.gitlab-ci.yml`, `docs/infra/02_ci_runner_hardening.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Pipeline CI biên dịch hoàn toàn offline với cờ `cargo --frozen` thành công.
  2. Mức tiêu thụ RAM tối đa của pipeline đo kiểm không bao giờ vượt quá 4.0 GB.
  3. Mọi vi phạm về format hoặc warning clippy đều khiến pipeline thất bại (Red build) và chặn merge request.

#### [WBS 0.7.3] Thiết lập Issue Tracker & Kênh Trao đổi Nội bộ Bảo mật
- **Mô tả chi tiết phạm vi**:
  - Khởi tạo và chuẩn hóa hệ thống bảng quản lý công việc (GitLab Issue Boards) theo đúng danh mục WBS Phase 0 & Phase 1.
  - Triển khai ứng dụng trao đổi nội bộ tự lưu trữ (Mattermost Team Edition hoặc Zulip Self-Hosted) trên máy chủ mạng LAN.
  - Thiết lập lưu vết audit log cho toàn bộ hoạt động trao đổi, cam kết không sử dụng các nền tảng chat đám mây công cộng (Slack/Discord) để trao đổi về dữ liệu nghiệp vụ ngân hàng.
- **Vai trò phụ trách chính**: Solution Architect (SA)
- **Vai trò phối hợp**: Full-stack Engineer (FSE), Security Engineer (SE)
- **Ước lượng ngày công**: 2 Man-days
- **Task phụ thuộc**: WBS 0.7.1
- **Sản phẩm bàn giao**: Kênh Trao đổi Nội bộ & Hệ thống Quản trị WBS Boards (`docs/infra/03_internal_collaboration_setup.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. 100% nhân sự trong đội ngũ được cấp tài khoản và kích hoạt 2FA.
  2. Toàn bộ các task WBS được nhập đầy đủ vào Issue Tracker với nhãn gán vai trò và milestone tương ứng.
  3. Dữ liệu hội thoại được lưu trữ và sao lưu cục bộ hàng ngày trên máy chủ nội bộ.

---

### Luồng 0.8: Xây Dựng Synthetic Data Generator v0 Cho Giao Dịch Việt Nam

#### [WBS 0.8.1] Thiết kế Cấu trúc Dữ liệu & Động cơ Sinh VietQR / Napas 24/7
- **Mô tả chi tiết phạm vi**:
  - Xây dựng thư viện Rust sinh chuỗi payload VietQR chuẩn EMVCo TLV (Tag-Length-Value): tính toán mã kiểm tra lỗi CRC16-CCITT (đa thức $0x1021$).
  - Mô phỏng các mẫu diễn giải giao dịch chuyển tiền liên ngân hàng NAPAS 24/7 thực tế với các tiền tố ngân hàng phổ biến: `MBVCB.xxxx`, `QRIBFT.xxxx`, `Napas 247 TT`, số trace chuẩn 6–12 chữ số, mã chuẩn chi FT.
  - Tích hợp biểu đồ phân bổ phí giao dịch Napas thực tế: 1.100đ, 2.200đ, 3.300đ, 5.500đ, 7.700đ, 8.800đ, 11.000đ.
- **Vai trò phụ trách chính**: Lead Rust Engineer (RE1)
- **Vai trò phối hợp**: Senior Rust Systems Engineer (RE2), Kế toán trưởng (KTT / SME)
- **Ước lượng ngày công**: 4 Man-days
- **Task phụ thuộc**: WBS 0.5.1
- **Sản phẩm bàn giao**: Module Sinh Dữ Liệu VietQR & Napas 24/7 (`crates/liva-core-banking/src/generator/vietqr_napas.rs`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Chuỗi mã QR sinh ra được quét thử nghiệm thành công và giải mã chính xác trên các ứng dụng ngân hàng thương mại Việt Nam.
  2. Diễn giải thanh toán Napas bao phủ đủ 10 biến thể cú pháp phổ biến nhất trên thị trường.
  3. Bóc tách chính xác biểu phí ngầm liên ngân hàng mà không làm sai lệch số tiền gốc.

#### [WBS 0.8.2] Xây dựng Bộ Sinh Sao Kê Đa Ngân Hàng (VCB, TCB, BIDV, CAMT.053)
- **Mô tả chi tiết phạm vi**:
  - Lập trình bộ xuất tệp sao kê đa định dạng:
    * Vietcombank: Xuất tệp Excel `.xlsx` có ô tiêu đề gộp nhiều cấp, phân tách cột Nợ/Có riêng biệt, định dạng số chấm phẩy kiểu Việt Nam (`15.000.000,00`).
    * Techcombank: Xuất tệp CSV chuẩn UTF-8 có BOM, cột số tiền âm/dương hoặc phân tách giao dịch bằng mã FT.
    * BIDV: Xuất tệp bảng biểu dạng lưới (Text Grid / PDF) đa dòng diễn giải.
    * ISO 20022 CAMT.053: Xuất điện chuẩn XML phân cấp đầy đủ các khối `<Stmt>`, `<Ntry>`, `<Amt>`.
  - Bảo đảm số dư lũy kế từng dòng khớp hoàn hảo: $\text{Balance}_i \equiv \text{Balance}_{i-1} \pm \text{Amount}_i$.
- **Vai trò phụ trách chính**: Lead Rust Engineer (RE1)
- **Vai trò phối hợp**: Senior Rust Systems Engineer (RE2), QA / SDET Engineer (QAE)
- **Ước lượng ngày công**: 4 Man-days
- **Task phụ thuộc**: WBS 0.8.1
- **Sản phẩm bàn giao**: Module Xuất Tệp Sao Kê Đa Ngân Hàng (`crates/liva-core-banking/src/generator/statement_exporter.rs`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Xuất file định dạng XLSX, CSV, PDF, XML mở sạch sẽ trên Microsoft Excel và trình duyệt, không lỗi font hay cấu trúc.
  2. 100% các dòng giao dịch khớp đúng định dạng mẫu của từng ngân hàng.
  3. Số dư lũy kế từng dòng bảo đảm tính toàn vẹn số học tuyệt đối.

#### [WBS 0.8.3] Xây dựng Động cơ Kiểm soát Bất biến & Phân bổ Tỷ lệ Đối soát
- **Mô tả chi tiết phạm vi**:
  - Tích hợp thuật toán bảo đảm bất biến số học tổng thể: $\text{ClosingBalance} \equiv \text{OpeningBalance} + \sum \text{Credit} - \sum \text{Debit}$.
  - Cài đặt bộ điều phối tỷ lệ kịch bản đối soát thực tế:
    * $85\% - 90\%$: Khớp tuyệt đối 1:1 (cùng số tham chiếu, ngày giá trị, số tiền).
    * $7.5\% - 10\%$: Khớp Heuristic mờ (lệch phí chuyển tiền Napas 1.100đ–11.000đ, độ lệch ngày $\pm 48\text{h}$).
    * $2\% - 3\%$: Khớp tách gộp Composite (1:N thanh toán nhiều hóa đơn, N:1 trả góp).
    * $0.2\% - 0.5\%$: Ngoại lệ cảnh báo (sai lệch số tiền, giao dịch đáng ngờ AML, chuyển nhầm tài khoản).
- **Vai trò phụ trách chính**: QA / SDET Engineer (QAE)
- **Vai trò phối hợp**: Senior Rust Systems Engineer (RE2), Kế toán trưởng (KTT / SME)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 0.8.2
- **Sản phẩm bàn giao**: Động cơ Phân bổ Kịch bản Đối soát (`crates/liva-core-banking/src/generator/reconciliation_orchestrator.rs`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Bất biến số dư tổng thể đạt tỷ lệ chính xác $100.000\%$ ($\Delta = 0$).
  2. Tỷ lệ phân bổ các kịch bản kiểm thử đo kiểm đạt độ lệch chuẩn $\le \pm 0.1\%$ so với cấu hình mục tiêu.
  3. Bàn giao kèm bộ kịch bản AML kích hoạt đúng 4 loại cảnh báo đáng ngờ.

#### [WBS 0.8.4] Đóng gói CLI Tool Sinh Dữ liệu Quy mô Lớn (1k – 100k Giao dịch)
- **Mô tả chi tiết phạm vi**:
  - Xây dựng công cụ dòng lệnh `generate-banking-data` hỗ trợ các tham số: `--seed <u64>`, `--count <N>`, `--bank <VCB|TCB|BIDV|CAMT053>`, `--out-dir <PATH>`.
  - Xuất đồng thời cặp file song sinh: File sao kê ngân hàng đối ngoại và File sổ cái/hóa đơn ERP nội bộ tương ứng phục vụ nạp thẳng vào engine đối soát.
  - Tối ưu hóa hiệu năng sinh dữ liệu: sử dụng buffer đa luồng qua `rayon`, bảo đảm sinh 50.000 dòng trong thời gian dưới 10 giây.
- **Vai trò phụ trách chính**: Senior Rust Systems Engineer (RE2)
- **Vai trò phối hợp**: QA / SDET Engineer (QAE)
- **Ước lượng ngày công**: 2 Man-days
- **Task phụ thuộc**: WBS 0.8.3
- **Sản phẩm bàn giao**: Tệp Nhị phân CLI & Hướng Dẫn Sử Dụng (`tools/generate-banking-data/`, `docs/tools/synthetic_generator_guide.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Sinh 50.000 dòng giao dịch hoàn tất trong thời gian $\le 10$ giây trên PC thông thường.
  2. Tham số `--seed` bảo đảm tính tái lập byte-for-byte 100% giữa các lần chạy.
  3. Dữ liệu sinh ra nạp thành công vào engine kiểm thử mà không cần chỉnh sửa thủ công.

---

## 3. PHÂN RÃ CHI TIẾT WBS GIAI ĐOẠN 1 (NỀN TẢNG LÕI & SANDBOX ZERO-EGRESS, TUẦN 3–10)

Giai đoạn 1 bao gồm **20 nhiệm vụ chi tiết** trải rộng trên **6 luồng công việc kỹ thuật cốt lõi**, với tổng khối lượng công việc là **73 man-days** phát triển chính (chưa tính thời lượng kiểm thử mở rộng và đối soát nghiệm thu).

```
+---------------------------------------------------------------------------------------------------------+
|                                    TỔNG QUAN 6 LUỒNG CÔNG VIỆC GIAI ĐOẠN 1                              |
+----------+------------------------------------------------------------+---------------+-----------------+
| Luồng    | Tên Luồng Công Việc                                        | Số Lượng Task | Tổng Man-Days   |
+----------+------------------------------------------------------------+---------------+-----------------+
| Luồng 1.1| Crate `liva-money` (Số nguyên i64, Zero-Float, Làm tròn VAS| 4 tasks       | 12 Man-days     |
| Luồng 1.2| Crate `liva-ledger` (State Machine TT 200, Event Store)    | 4 tasks       | 19 Man-days     |
| Luồng 1.3| Property-Based Testing (proptest >= 1.000.000 test cases)  | 3 tasks       | 09 Man-days     |
| Luồng 1.4| `liva-netguard` Sandbox & Chặn đứng 100% Egress            | 3 tasks       | 11 Man-days     |
| Luồng 1.5| Canonical Transaction Schema & Trait `BankParser` Registry | 3 tasks       | 12 Man-days     |
| Luồng 1.6| Local Web UI Framework (127.0.0.1, Argon2id, RBAC 3 vai trò| 3 tasks       | 10 Man-days     |
+----------+------------------------------------------------------------+---------------+-----------------+
|          | **TỔNG CỘNG GIAI ĐOẠN 1**                                  | **20 tasks**  | **73 MAN-DAYS** |
+----------+------------------------------------------------------------+---------------+-----------------+
```

---

### Luồng 1.1: Thiết Kế & Phát Triển Crate `liva-money`

#### [WBS 1.1.1] Thiết kế Kiểu Dữ liệu Cốt lõi `Money` & Cưỡng Chế Zero-Float
- **Mô tả chi tiết phạm vi**:
  - Khởi tạo crate độc lập `crates/liva-money` trong workspace.
  - Thiết kế cấu trúc `Money`: trường `amount: i64` (đơn vị atomic đồng VND, 1 đơn vị = 1 đồng; hỗ trợ ngoại tệ với fixed-point scale) và `currency: CurrencyCode` (VND, USD, EUR theo ISO 4217).
  - Cưỡng chế nghiêm ngặt chỉ thị biên dịch tại đầu tệp `crates/liva-money/src/lib.rs`:
    ```rust
    #![deny(clippy::float_arithmetic)]
    #![deny(clippy::suboptimal_flops)]
    #![deny(clippy::imprecise_flops)]
    ```
  - *(Lưu ý Kiến trúc Phạm vi Clippy: Chỉ thị `#![deny(clippy::float_arithmetic)]` cùng các lint cấm float được cấu hình độc quyền ở cấp độ crate tài chính `crates/liva-money` và `crates/liva-ledger`, tuyệt đối không áp đặt lên root workspace `Cargo.toml` để bảo toàn khả năng xử lý số thực `f32` của các phân hệ Nhận diện giọng nói STT Parakeet, Đọc giọng nói VieNeu-TTS, và bộ lọc âm thanh FFT trong `liva-native-core`).*
  - Cài đặt các trait chuẩn của Rust: `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Display`, `Serialize`, `Deserialize`.
- **Vai trò phụ trách chính**: Lead Rust Engineer (RE1)
- **Vai trò phối hợp**: Solution Architect (SA), QA / SDET Engineer (QAE)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 0.5.1
- **Sản phẩm bàn giao**: Mã nguồn Crate `crates/liva-money/src/lib.rs` & Bộ Unit Test Cơ Sở (`crates/liva-money/tests/money_basic.rs`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Trình biên dịch Rust báo lỗi và dừng build ngay lập tức nếu xuất hiện bất kỳ biến hoặc phép tính float nào trong crate `liva-money` hoặc `liva-ledger`.
  2. Lệnh `cargo clippy -p liva-money -- -D clippy::float_arithmetic` đạt 0 warning, 0 error.
  3. Phạm vi `i64` biểu diễn an toàn tới $\pm 9.22 \times 10^{18}\text{ VND}$ mà không sợ tràn số trong các nghiệp vụ thông thường.

#### [WBS 1.1.2] Cài đặt Số học An toàn & Bộ Xử lý Tràn Số (Safe Math Arithmetic)
- **Mô tả chi tiết phạm vi**:
  - Lập trình các hàm toán học an toàn có kiểm tra tràn số: `checked_add`, `checked_sub`, `checked_mul`.
  - Định nghĩa enum lỗi chuẩn tắc `MoneyError`: `Overflow`, `Underflow`, `CurrencyMismatch`, `DivisionByZero`.
  - Nạp chồng các toán tử toán học (`std::ops::Add`, `Sub`) với cơ chế tự động panic-safe (luôn trả về `Result<Money, MoneyError>`).
  - Xây dựng macro khởi tạo nhanh tiện dụng `vnd!(100_000)` phục vụ viết code sạch và an toàn.
- **Vai trò phụ trách chính**: Lead Rust Engineer (RE1)
- **Vai trò phối hợp**: QA / SDET Engineer (QAE)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 1.1.1
- **Sản phẩm bàn giao**: Module Số Học An Toàn (`crates/liva-money/src/ops.rs`) & Bộ Test Suite Tràn Số (`crates/liva-money/tests/overflow_tests.rs`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. 100% các phép toán biên với `i64::MAX` và `i64::MIN` trả về `Err(MoneyError::Overflow)` an toàn, tuyệt đối không crash tiến trình.
  2. Phép toán giữa hai loại tiền tệ khác nhau (ví dụ VND cộng USD) trả về lỗi `CurrencyMismatch`.
  3. Đạt độ bao phủ kiểm thử (Code Coverage) $\ge 98\%$.

#### [WBS 1.1.3] Cài đặt Thuật toán Phân bổ Tiền Hare-Niemeyer & Quy tắc VAS
- **Mô tả chi tiết phạm vi**:
  - Cài đặt giải thuật phân bổ tiền không thất thoát **Hare-Niemeyer (Largest Remainder Method)**: cho phép chia một khoản tiền `Money` cho danh sách tỷ lệ phần trăm hoặc chia đều cho $N$ đối tượng mà không bị mất đồng nào do làm tròn lẻ.
  - Cài đặt quy tắc làm tròn kế toán Việt Nam (VAS / Thông tư 200/2014/TT-BTC): cơ chế làm tròn nửa chẵn (Banker's Rounding / Round-Half-to-Even) khi tính toán tỷ giá ngoại tệ.
  - Khẳng định bất biến phân bổ: $\sum_{k=1}^N \text{Allocated}_k \equiv \text{OriginalAmount}$.
- **Vai trò phụ trách chính**: Lead Rust Engineer (RE1)
- **Vai trò phối hợp**: Kế toán trưởng (KTT / SME), QA / SDET Engineer (QAE)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 1.1.2
- **Sản phẩm bàn giao**: Module Phân Bổ Tiền (`crates/liva-money/src/allocation.rs`) & Báo Cáo Đối Chiếu VAS (`crates/liva-money/tests/vas_rounding_tests.rs`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Chạy 100.000 trường hợp chia tiền ngẫu nhiên với tỷ lệ phức tạp, khẳng định bất biến $\sum \text{Allocated}_i \equiv \text{Total}$ đạt tỷ lệ đúng $100.000\%$.
  2. Số dư dôi dư phân bổ chính xác từng đồng cho các đối tượng có phần dư lớn nhất.
  3. Kế toán trưởng ký xác nhận quy tắc làm tròn tuân thủ đúng thực tiễn hạch toán thuế GTGT và quyết toán tài chính.

#### [WBS 1.1.4] Xây dựng Bộ Parser Chuỗi Tiền Tệ Định Dạng Việt Nam & Quốc Tế
- **Mô tả chi tiết phạm vi**:
  - Phát triển bộ parser chuỗi tiền tệ thông minh: nhận diện định dạng dấu chấm phân cách hàng nghìn kiểu Việt Nam (`15.000.000`), dấu phẩy kiểu quốc tế (`15,000,000`), số âm trong ngoặc đơn `(500.000)`, số tiền kèm hậu tố đơn vị `VND`, `đ`, `USD`.
  - Tích hợp hàm format tiền tệ ra chuỗi chuẩn hiển thị trên giao diện người dùng theo thị hiếu người Việt (`15.000.000 đ`).
  - Xử lý các chuỗi bẩn có khoảng trắng thừa, ký tự xuống dòng `\r\n` hoặc ký tự rác từ file scan.
- **Vai trò phụ trách chính**: Lead Rust Engineer (RE1)
- **Vai trò phối hợp**: Kế toán trưởng (KTT / SME), QA / SDET Engineer (QAE)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 1.1.1
- **Sản phẩm bàn giao**: Module Phân Tích & Định Dạng Tiền Tệ (`crates/liva-money/src/parser.rs`, `src/formatter.rs`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Phân tích chính xác 100% các định dạng số tiền trích xuất từ sao kê thực tế của VCB, TCB, BIDV, Agribank.
  2. Từ chối an toàn và trả về lỗi xác định đối với các chuỗi dị dạng không thể bóc tách số.
  3. Tốc độ parse chuỗi đạt $< 0.1\text{ µs}$ / lần gọi.

---

### Luồng 1.2: Thiết Kế & Phát Triển Crate `liva-ledger`

#### [WBS 1.2.1] Thiết kế Hệ Thống Tài Khoản Kế Toán Kép Theo Thông Tư 200
- **Mô tả chi tiết phạm vi**:
  - Định nghĩa cấu trúc danh mục hệ thống tài khoản kế toán doanh nghiệp (Chart of Accounts - COA) theo Thông tư 200/2014/TT-BTC: TK 112 (Tiền gửi ngân hàng), TK 1121, TK 1122, TK 131 (Phải thu), TK 331 (Phải trả), TK 511 (Doanh thu), TK 642 (Chi phí quản lý), TK 635 (Chi phí tài chính), TK 515 (Doanh thu tài chính), TK 811 (Chi phí khác).
  - Chuẩn hóa ánh xạ đảo ngược góc nhìn kế toán (Perspective Mapping):
    * Sao kê ngân hàng ghi CÓ (Credit) $\rightarrow$ Sổ cái doanh nghiệp ghi NỢ (Debit - tài sản tiền gửi tăng).
    * Sao kê ngân hàng ghi NỢ (Debit) $\rightarrow$ Sổ cái doanh nghiệp ghi CÓ (Credit - tài sản tiền gửi giảm).
  - Thiết kế cấu trúc định khoản kép `AccountEntry` gồm mã tài khoản, hướng phát sinh (Debit/Credit) và số tiền `Money`.
- **Vai trò phụ trách chính**: Solution Architect (SA)
- **Vai trò phối hợp**: Kế toán trưởng (KTT / SME), Lead Rust Engineer (RE1)
- **Ước lượng ngày công**: 4 Man-days
- **Task phụ thuộc**: WBS 1.1.1
- **Sản phẩm bàn giao**: Đặc Tả Nghiệp Vụ Sổ Cái Kế Toán & Cấu Trúc COA (`crates/liva-ledger/src/coa.rs`, `docs/accounting/tt200_chart_of_accounts.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Bao phủ đầy đủ 100% các tài khoản liên quan đến dòng tiền gửi và thanh toán liên ngân hàng.
  2. Bảng quy tắc ánh xạ góc nhìn CÓ/NỢ giữa ngân hàng và doanh nghiệp được kiểm chứng và đóng băng chính thức.
  3. Cấu trúc dữ liệu Rust không cho phép tạo tài khoản có mã ký tự sai quy chuẩn kế toán.

#### [WBS 1.2.2] Cài đặt Máy Trạng Thái Bút Toán Kép (Double-Entry State Machine)
- **Mô tả chi tiết phạm vi**:
  - Phát triển máy trạng thái kế toán kép trong crate `liva-ledger`.
  - Định nghĩa cấu trúc bút toán `JournalEntry`: mã bút toán duy nhất, ngày chứng từ, ngày ghi sổ, danh sách các định khoản Nợ/Có.
  - Cưỡng chế bất biến kế toán kép cốt lõi ngay tại thời điểm khởi tạo bút toán (Compile-time / Construction-time Invariant):
    $$\sum \text{DebitAmount} \equiv \sum \text{CreditAmount}$$
  - Ngăn chặn hoàn toàn hành vi tạo bút toán "một vế" (Unbalanced Entry); nếu tổng Nợ khác tổng Có, hàm tạo lập lập tức trả về `Err(LedgerError::UnbalancedEntry)`.
- **Vai trò phụ trách chính**: Lead Rust Engineer (RE1)
- **Vai trò phối hợp**: Kế toán trưởng (KTT / SME), Solution Architect (SA)
- **Ước lượng ngày công**: 5 Man-days
- **Task phụ thuộc**: WBS 1.2.1, WBS 1.1.2
- **Sản phẩm bàn giao**: Crate `crates/liva-ledger/src/journal.rs` & Bộ Test Kiểm Chứng Bất Biến Bút Toán (`crates/liva-ledger/tests/journal_invariants.rs`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Trình biên dịch và runtime từ chối dứt điểm mọi bút toán có chênh lệch $|\sum \text{Debit} - \sum \text{Credit}| > 0$.
  2. Hỗ trợ đầy đủ các dạng bút toán phức tạp: 1 Nợ - nhiều Có, nhiều Nợ - 1 Có, nhiều Nợ - nhiều Có.
  3. Đạt hiệu năng xử lý $\ge 100.000$ bút toán / giây trong bộ nhớ.

#### [WBS 1.2.3] Xây dựng Kho Lưu Trữ Sự Kiện Bất Biến (Append-Only Event Store)
- **Mô tả chi tiết phạm vi**:
  - Thiết kế kiến trúc Event Sourcing lưu trữ sự kiện tài chính vào SQLite WAL.
  - Danh mục sự kiện tài chính bất biến: `AccountCreated`, `StatementIngested`, `JournalEntryPosted`, `ReconciliationMatched`, `PeriodClosed`.
  - **Nguyên tắc Bất biến Tuyệt đối**: Cấm hoàn toàn câu lệnh SQL `UPDATE` hoặc `DELETE` trên bảng sự kiện kế toán.
  - Xây dựng cơ chế **Bút toán đảo (Storno Reversal)**: Mọi thao tác hủy bỏ hoặc điều chỉnh số liệu đều phải ghi nhận một sự kiện đảo ngược cân bằng trạng thái cũ, kèm lý do điều chỉnh và mã tham chiếu đến bút toán gốc.
- **Vai trò phụ trách chính**: Lead Rust Engineer (RE1)
- **Vai trò phối hợp**: Solution Architect (SA), Security Engineer (SE)
- **Ước lượng ngày công**: 5 Man-days
- **Task phụ thuộc**: WBS 1.2.2
- **Sản phẩm bàn giao**: Module Event Store & Giao Thức Bút Toán Đảo Storno (`crates/liva-ledger/src/event_store.rs`, `src/reversal.rs`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Kiểm tra mã nguồn khẳng định không tồn tại bất kỳ lệnh `UPDATE` hay `DELETE` nào trên kho sự kiện kế toán.
  2. Bút toán đảo tái lập chính xác trạng thái số dư ban đầu mà không để lại vết sai lệch số học.
  3. Khả năng tái tạo trạng thái số cái (State Reconstruction) từ chuỗi sự kiện với tốc độ $\ge 50.000$ sự kiện / giây.

#### [WBS 1.2.4] Cài đặt Cơ chế Xác Thực Chuỗi Băm HMAC & Bất biến Số Dư Kỳ
- **Mô tả chi tiết phạm vi**:
  - Tích hợp chuỗi băm mật mã HMAC-SHA256 liên kết tiếp nối giữa các sự kiện kế toán:
    $$\text{Hash}_k = \text{HMAC-SHA256}\Big(K_{\text{audit}}, \, \text{Hash}_{k-1} \,\|\, \text{Timestamp}_k \,\|\, \text{Payload}_k\Big)$$
  - Lập trình hàm xác thực bất biến vĩ mô `verify_balance_invariants`:
    $$\text{ClosingBalance} \equiv \text{OpeningBalance} + \sum \text{CreditTurnover} - \sum \text{DebitTurnover}$$
  - Cài đặt quy trình khóa sổ kế toán định kỳ (Period Close Lock): sau khi khóa sổ, cấm phát sinh thêm bất kỳ bút toán nào có ngày ghi sổ thuộc kỳ đã khóa.
- **Vai trò phụ trách chính**: Lead Rust Engineer (RE1)
- **Vai trò phối hợp**: Security Engineer (SE), Kế toán trưởng (KTT / SME)
- **Ước lượng ngày công**: 5 Man-days
- **Task phụ thuộc**: WBS 1.2.3
- **Sản phẩm bàn giao**: Module Xác Thực Toàn Vẹn & Bất Biến Kỳ Kế Toán (`crates/liva-ledger/src/integrity.rs`, `src/period_close.rs`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Phát hiện ngay lập tức hành vi sửa đổi trái phép dù chỉ 1 bit dữ liệu trong kho sự kiện (Tính toàn vẹn chuỗi băm đạt 100%).
  2. Hàm kiểm tra bất biến số dư thực thi hoàn tất trong thời gian $< 1\text{ ms}$ cho 10.000 giao dịch.
  3. Kế toán trưởng kiểm tra và ký xác nhận tính hợp lệ của quy trình khóa sổ kế toán.

---

### Luồng 1.3: Bộ Kiểm Thử Dựa Trên Thuộc Tính (Property-Based Test / Proptest)

#### [WBS 1.3.1] Xây dựng Bộ sinh Dữ liệu Kiểm thử Tài chính (`proptest` Generators)
- **Mô tả chi tiết phạm vi**:
  - Xây dựng module kiểm thử thuộc tính sử dụng crate `proptest`.
  - Hiện thực hóa các chiến lược sinh ngẫu nhiên dữ liệu giao dịch đa dạng (`Strategy`):
    * `arb_money()`: Sinh số tiền từ 1đ đến 500 nghìn tỷ đồng, bao phủ các trường hợp biên (`1đ`, `i64::MAX`).
    * `arb_transaction()`: Sinh giao dịch ngẫu nhiên với loại phát sinh Nợ/Có, ngày giá trị, mã chứng từ, diễn giải thanh toán tiếng Việt.
    * `arb_ledger_stream()`: Sinh chuỗi hàng nghìn giao dịch ngẫu nhiên liên tiếp có chèn các giao dịch đảo nợ và điều chỉnh.
  - Cấu hình khống chế độ sâu thu nhỏ `max_shrink_iters <= 10_000` trong `ProptestConfig` nhằm bảo vệ tài nguyên bộ nhớ khi phát hiện mẫu lỗi.
- **Vai trò phụ trách chính**: QA / SDET Engineer (QAE)
- **Vai trò phối hợp**: Senior Rust Systems Engineer (RE2), Kế toán trưởng (KTT / SME)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 1.1.2, WBS 1.2.1
- **Sản phẩm bàn giao**: Module Sinh Dữ Liệu Kiểm Thử Thuộc Tính (`tests/proptest_generators.rs`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Bộ sinh tạo ra dữ liệu hợp lệ, không vi phạm các ràng buộc kiểu dữ liệu cơ sở của `liva-money`.
  2. Tốc độ sinh dữ liệu trong RAM đạt $\ge 50.000$ giao dịch / giây.
  3. Bao phủ đầy đủ các ký tự đặc biệt, dấu tiếng Việt và các giá trị biên số học.

#### [WBS 1.3.2] Hiện thực Khung Thực thi Fuzzing Bất biến Sổ cái $\ge 1.000.000$ Test Cases
- **Mô tả chi tiết phạm vi**:
  - Thiết lập runner kiểm thử thuộc tính tự động hóa chạy liên tục tối thiểu **$1.000.000$ test cases** trên máy trạng thái `liva-ledger`.
  - Khẳng định 5 bất biến toán học tuyệt đối:
    1. *Ledger Balance Closure*: $\text{Closing} \equiv \text{Opening} + \sum \text{Credit} - \sum \text{Debit}$.
    2. *Double-Entry Conservation*: $\sum \text{Debit} \equiv \sum \text{Credit}$.
    3. *Non-Negative Balance*: Số dư không âm (ngoại trừ tài khoản thấu chi).
    4. *Idempotency*: Áp dụng trùng lặp cùng một sự kiện không làm thay đổi trạng thái sổ cái.
    5. *Reversal Symmetry*: Áp dụng giao dịch đảo đưa trạng thái sổ cái về nguyên bản trước khi phát sinh giao dịch.
  - Chia nhỏ quy trình fuzzing thành 100 lô (mỗi lô 10.000 transactions) có thu hồi bộ nhớ để bảo toàn ngân sách RAM máy trạm $\le 2\text{ GB}$.
  - Cấu hình `ProptestConfig { cases: 1_000_000, max_shrink_iters: 10_000, ..Default::default() }` chặn đứng nguy cơ treo tiến trình do đệ quy thu nhỏ vô hạn.
  - Đánh dấu thuộc tính `#[ignore]` trên test suite 1 triệu cases trong CI mặc định nhằm tránh kéo dài chu kỳ build thường nhật; kích hoạt chuyên biệt qua lệnh `cargo test --test ledger_invariant_fuzzing -- --ignored` trong nightly/stress pipelines.
- **Vai trò phụ trách chính**: QA / SDET Engineer (QAE)
- **Vai trò phối hợp**: Senior Rust Systems Engineer (RE2), Solution Architect (SA)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 1.3.1, WBS 1.2.2
- **Sản phẩm bàn giao**: Test Suite Fuzzing Bất Biến Sổ Cái & Báo Cáo Đo Kiểm Triệu Giao Dịch (`tests/ledger_invariant_fuzzing.rs`, `docs/qa/01_fuzzing_one_million_cases_report.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Chạy hoàn tất đủ $1.000.000$ test cases mà không có bất kỳ lỗi panic, abort hoặc crash nào (0 failure).
  2. Bất biến `Closing = Opening + ΣCredit − ΣDebit` đạt tỷ lệ chính xác $100.000\%$ trên toàn bộ 1 triệu trường hợp.
  3. Mức tiêu thụ bộ nhớ RAM tối đa của tiến trình test không vượt quá 2.0 GB trong suốt quá trình chạy.
  4. Thuộc tính `#[ignore]` được gắn đúng quy chuẩn, bảo đảm lệnh CI tiêu chuẩn hoàn tất nhanh chóng.

#### [WBS 1.3.3] Kiểm thử Chống Tràn số & Tiêm Lỗi Dữ liệu Cực hạn
- **Mô tả chi tiết phạm vi**:
  - Xây dựng bộ test chuyên biệt tiêm các kịch bản thù địch và dữ liệu dị dạng:
    * Tiêm giao dịch có giá trị làm tràn số nguyên `i64::MAX`.
    * Tiêm giao dịch ghi nợ vào tài khoản có số dư 0 đồng mà không có hạn mức thấu chi.
    * Tiêm chuỗi đảo nợ vô tận lặp đi lặp lại.
    * Tiêm tệp sao kê có số dư đóng/mở bị làm lệch số liệu cố tình.
  - Xác nhận hệ thống phản hồi `Err(LedgerError)` an toàn theo nguyên tắc Fail-Closed, bảo toàn tuyệt đối tính toàn vẹn ACID của cơ sở dữ liệu SQLite WAL.
- **Vai trò phụ trách chính**: QA / SDET Engineer (QAE)
- **Vai trò phối hợp**: Senior Rust Systems Engineer (RE2)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 1.3.2
- **Sản phẩm bàn giao**: Test Suite Tiêm Lỗi Đối Kháng (`tests/ledger_boundary_stress.rs`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. 100% các phép toán vượt ngưỡng `i64::MAX` bị chặn đứng và trả về lỗi `Overflow` xác định, không gây hoảng loạn tiến trình.
  2. Tỷ lệ phát hiện và từ chối các file sao kê có số dư bị làm giả đạt chính xác $100.0\%$ (Zero False Negative).
  3. CSDL giữ vững trạng thái toàn vẹn, không xảy ra tình trạng phân mảnh hoặc khóa chết database.

---

### Luồng 1.4: Xây Dựng `liva-netguard` Sandbox & Chặn Đứng 100% Egress

#### [WBS 1.4.1] Xây dựng Bộ Quy tắc Lọc Gói Tin Cấp Kernel (`nftables` Ruleset Engine)
- **Mô tả chi tiết phạm vi**:
  - Biên soạn kịch bản cấu hình kernel `nftables` dành riêng cho daemon `liva-native-core`.
  - Thiết lập bảng lọc `inet liva_netguard`, hook chuỗi `output` với chính sách mặc định `drop`:
    * Whitelist duy nhất cho card mạng loopback (`oif "lo"`) và địa chỉ IP `127.0.0.0/8`, `::1`.
    * Từ chối toàn bộ kết nối TCP/UDP ra card mạng ngoài (`eth0`, `wlan0`) với mã thông báo `reject with icmpx type admin-prohibited`.
    * Ghi log kernel cảnh báo tức thời `[LIVA-NETGUARD-EGRESS-BLOCKED]` khi phát hiện gói tin vi phạm.
  - Đóng gói script nạp và gỡ bỏ luật an toàn tự động `netguard-init.sh`.
  - Thiết lập chỉ thị phân định biên dịch hệ điều hành `#[cfg(target_os = "linux")]` cho module nftables và kernel hooks của `liva-netguard`.
  - Đối với môi trường máy trạm kế toán chạy Windows (Windows 10/11 Pro), định hướng kiến trúc sử dụng Windows Filtering Platform (WFP) / Windows Defender Firewall API kết hợp cưỡng chế socket binding độc quyền `127.0.0.1` để ngăn chặn rò rỉ dữ liệu ngoại vi.
  - **Lưu ý xếp lịch**: Do Kỹ sư An toàn Thông tin (SE) dành 100% công suất (20 MD) cho các nhiệm vụ GĐ0 trong Tuần 1–4, Luồng 1.4 chính thức khởi động từ đầu Tuần 5 và kéo dài đến Tuần 10.
- **Vai trò phụ trách chính**: Security Engineer (SE)
- **Vai trò phối hợp**: Solution Architect (SA), Senior Rust Systems Engineer (RE2)
- **Ước lượng ngày công**: 4 Man-days
- **Task phụ thuộc**: WBS 0.6.3, WBS 0.7.1
- **Sản phẩm bàn giao**: Tập Tin Cấu Hình nftables & Script Quản Trị (`crates/liva-netguard/rules/liva-netguard.nft`, `scripts/netguard-init.sh`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Luật `nftables` nạp thành công trên Linux mà không gây gián đoạn các dịch vụ mạng nội bộ khác của hệ điều hành.
  2. Bất kỳ gói tin TCP/UDP nào có đích đến ngoài dải `127.0.0.0/8` hoặc `::1` xuất phát từ LIVA đều bị chặn đứng ngay lập tức.
  3. Kernel log ghi nhận chính xác cảnh báo vi phạm kèm địa chỉ IP và cổng đích.
  4. Chỉ thị `#[cfg(target_os = "linux")]` bảo đảm mã nguồn không gây lỗi biên dịch trên các nền tảng khác.

#### [WBS 1.4.2] Thiết lập Bộ Lọc Lệnh Hệ thống (`seccomp-bpf`) & Cô lập Dịch vụ (`systemd.unit`)
- **Mô tả chi tiết phạm vi**:
  - Hiện thực hóa bộ lọc system call bằng `seccomp-bpf` trong Rust (được bảo vệ bởi cờ `#[cfg(target_os = "linux")]`) trước khi tiến trình bước vào vòng lặp xử lý giao dịch. Chặn lệnh `connect()` tới các địa chỉ không phải loopback.
  - Cấm khởi tạo socket nguy hiểm: `SOCK_RAW`, `AF_PACKET`, `AF_NETLINK`.
  - Biên soạn file cấu hình `systemd` service unit với các cờ cô lập an ninh nghiêm ngặt:
    `IPAddressAllow=127.0.0.1/8 ::1/128`, `IPAddressDeny=any`, `ProtectSystem=strict`, `ProtectHome=true`, `PrivateTmp=true`, `NoNewPrivileges=true`.
  - Vận hành tiến trình dưới quyền người dùng dịch vụ không đặc quyền (`User=liva-service`).
- **Vai trò phụ trách chính**: Security Engineer (SE)
- **Vai trò phối hợp**: Senior Rust Systems Engineer (RE2)
- **Ước lượng ngày công**: 4 Man-days
- **Task phụ thuộc**: WBS 1.4.1
- **Sản phẩm bàn giao**: Module Rust seccomp & File Cấu Hình Dịch Vụ Systemd (`crates/liva-netguard/src/seccomp.rs`, `crates/liva-netguard/systemd/liva-banking.service`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Bộ lọc `seccomp-bpf` trả về mã lỗi `EPERM` ngay khi một luồng cố gắng mở kết nối tới IP công cộng.
  2. Lệnh kiểm tra an ninh `systemd-analyze security liva-banking.service` đạt điểm đánh giá an toàn tối ưu (màu xanh lá cây).
  3. Tiến trình không thể leo quyền root hoặc truy cập trái phép các thư mục hệ thống ngoài danh mục cho phép.

#### [WBS 1.4.3] Xây dựng Bộ Kiểm thử Tự động Xác minh Chặn 100% Egress Ngoài Loopback
- **Mô tả chi tiết phạm vi**:
  - Xây dựng test suite tự động hóa chạy trong CI/CD và môi trường kiểm định độc lập, thực hiện 5 kịch bản xâm nhập mạng chủ động:
    1. Thử mở kết nối `TcpStream::connect("8.8.8.8:53")` và `TcpStream::connect("20.205.243.166:443")` (GitHub IP).
    2. Thử bắn gói tin `UdpSocket` ra ngoài external DNS.
    3. Thử phân giải tên miền công cộng `api.openai.com`.
    4. Thử gửi yêu cầu HTTP POST chứa dữ liệu giả lập ra Internet.
    5. Kiểm tra kết nối loopback nội bộ tới cổng Web Gateway (`127.0.0.1:8002`).
  - Sử dụng công cụ bắt gói tin pcap độc lập để xác minh lưu lượng card mạng vật lý.
- **Vai trò phụ trách chính**: Security Engineer (SE)
- **Vai trò phối hợp**: QA / SDET Engineer (QAE)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 1.4.2
- **Sản phẩm bàn giao**: Test Suite Đối Kháng Zero-Egress & Báo Cáo Giám Sát Gói Tin (`tests/zero_egress_adversarial_suite.rs`, `docs/security/04_zero_egress_verification_report.md`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. $100\%$ các yêu cầu kết nối TCP/UDP ra ngoài loopback bị chặn hoàn toàn (nhận lỗi `PermissionDenied` hoặc `ConnectionRefused`).
  2. Bắt gói tin qua pcap ghi nhận chính xác **0 byte** dữ liệu truyền ra ngoài card mạng ngoại vi trong suốt quá trình chạy test.
  3. Bộ đếm vi phạm `blocked_attempts` trong hệ thống tăng chính xác theo số kịch bản tấn công thử nghiệm.

---

### Luồng 1.5: Canonical Transaction Schema & Trait `BankParser` Architecture

#### [WBS 1.5.1] Đặc tả & Hiện thực Hóa Canonical Transaction Schema Chuẩn Hóa
- **Mô tả chi tiết phạm vi**:
  - Định nghĩa cấu trúc dữ liệu chuẩn tắc `CanonicalTransaction`, `CanonicalStatement`, `CounterpartyMetadata` trong Rust.
  - Hỗ trợ chuẩn ISO 8601 (thời gian phát sinh UTC, ngày giá trị hạch toán), ISO 4217 (tiền tệ).
  - Khởi tạo mã định danh duy nhất tất định `transaction_id` bằng UUIDv5 dựa trên namespace và chuỗi băm các thuộc tính cốt lõi của giao dịch (`bank:acc:date:amount:ref:row`).
  - Trường số tiền sử dụng độc quyền kiểu `Money` (`i64`, atomic VND, 0 float).
  - Tích hợp trường `vendor_metadata: HashMap<String, serde_json::Value>` bảo toàn nguyên vẹn 100% các cột đặc thù riêng biệt của từng ngân hàng mà không làm biến dạng schema chuẩn.
- **Vai trò phụ trách chính**: Senior Rust Systems Engineer (RE2)
- **Vai trò phối hợp**: Solution Architect (SA), Kế toán trưởng (KTT / SME)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 1.1.2, WBS 0.2.3
- **Sản phẩm bàn giao**: Module Schema Chuẩn Hóa (`crates/liva-core-banking/src/schema/canonical.rs`) & JSON Schema Export (`schemas/canonical_transaction.json`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Lược đồ hỗ trợ ánh xạ 100% các trường dữ liệu từ các ngân hàng VCB, TCB, BIDV, CTG mà không làm thất thoát thông tin.
  2. Toàn bộ các trường tiền tệ sử dụng kiểu `Money`, vượt qua kiểm tra cấm float của clippy.
  3. UUIDv5 sinh ra bảo đảm tính tất định: cùng 1 dòng sao kê parse nhiều lần luôn cho ra cùng một `transaction_id`.

#### [WBS 1.5.2] Thiết kế Kiến trúc Trait `BankParser` & Engine Điều phối Nhận diện (Sniffer)
- **Mô tả chi tiết phạm vi**:
  - Thiết kế trait `BankParser` gồm 3 hàm cốt lõi:
    * `sniff(&self, sample: &[u8], filename: &str) -> SniffConfidence`: Nhận diện nhanh loại ngân hàng dựa trên magic bytes, header keywords và cấu trúc tệp.
    * `parse(&self, raw_bytes: &[u8], filename: &str) -> Result<CanonicalStatement, ParserError>`: Bóc tách nhị phân thành sao kê chuẩn tắc.
    * `validate_statement_invariants(&self, stmt: &CanonicalStatement) -> Result<(), InvariantViolationError>`: Xác thực phương trình kế toán bất biến mở/đóng kỳ của file sao kê.
  - Hiện thực hóa `BankParserRegistry` thread-safe (`Arc<RwLock<Vec<Box<dyn BankParser>>>>`) cho phép đăng ký parser động.
  - Xây dựng bộ Sniffer 4 pha: Container Detection $\rightarrow$ Header Keyword Scoring $\rightarrow$ Date/Amount Pattern Match $\rightarrow$ Template Confidence Match.
- **Vai trò phụ trách chính**: Senior Rust Systems Engineer (RE2)
- **Vai trò phối hợp**: Solution Architect (SA), QA / SDET Engineer (QAE)
- **Ước lượng ngày công**: 4 Man-days
- **Task phụ thuộc**: WBS 1.5.1
- **Sản phẩm bàn giao**: Module Trait & Bộ Sniffer Plugin Registry (`crates/liva-core-banking/src/parser/trait_def.rs`, `src/parser/registry.rs`, `src/parser/sniffer.rs`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Bộ sniffer tự động nhận diện chính xác định dạng file sao kê với độ chính xác $\ge 99.5\%$.
  2. Cho phép đăng ký thêm parser ngân hàng mới mà không cần chỉnh sửa hoặc biên dịch lại mã nguồn lõi.
  3. Cung cấp bộ kiểm tra bất biến số dư mở/đóng mặc định áp dụng tự động cho toàn bộ các parser plugin.

#### [WBS 1.5.3] Hiện thực Reference Parsers cho VCB, TCB, BIDV & Chuẩn ISO 20022 CAMT.053 XML
- **Mô tả chi tiết phạm vi**:
  - Phát triển 4 parser cụ thể cài đặt trait `BankParser`:
    * `VcbExcelParser`: Xử lý bảng tính Excel đa tầng, nhiều ô gộp, định dạng số chấm phẩy Việt Nam (`crates/calamine`).
    * `TcbCsvParser`: Xử lý CSV streaming tốc độ cao, bóc tách mã Napas trace trong chuỗi diễn giải (`crates/csv`).
    * `BidvPdfParser`: Bóc tách bảng biểu dữ liệu cấu trúc từ PDF sao kê ngân hàng (`crates/lopdf`).
    * `Iso20022XmlParser`: Bóc tách thông điệp ngân hàng mở quốc tế `camt.053.001.08` XML (`crates/quick-xml`).
  - Kiểm thử đối chiếu trực tiếp với tập dữ liệu sạch `data/golden_clean/` đã thu thập từ Giai đoạn 0.
- **Vai trò phụ trách chính**: Senior Rust Systems Engineer (RE2)
- **Vai trò phối hợp**: QA / SDET Engineer (QAE), Kế toán trưởng (KTT / SME)
- **Ước lượng ngày công**: 5 Man-days
- **Task phụ thuộc**: WBS 1.5.2, WBS 0.2.3
- **Sản phẩm bàn giao**: 4 Plugin Parser Tham Chiếu (`crates/liva-core-banking/src/parser/plugins/`) & Bộ Test Suite Khớp Golden Files.
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Parse thành công 100% các file trong tập Golden Files của VCB, TCB, BIDV và ISO 20022 CAMT.053 XML.
  2. Bóc tách và chuyển đổi chuẩn xác 100% số tiền và số dư đầu/cuối kỳ, không lệch 1 đồng.
  3. Tốc độ parse trên dòng dữ liệu đạt $< 0.5\text{ ms}$ / dòng.

---

### Luồng 1.6: Local Web UI Framework (127.0.0.1, Local Auth, RBAC)

#### [WBS 1.6.1] Xây dựng Máy chủ Web Cục bộ Nhúng Sâu & Ràng buộc Loopback 127.0.0.1
- **Mô tả chi tiết phạm vi**:
  - Thiết lập máy chủ HTTP cục bộ nhúng sâu trong lõi Rust sử dụng `hyper` và `tokio`.
  - Khóa cứng địa chỉ socket lắng nghe tại `127.0.0.1:<port>` (hoặc cổng ngẫu nhiên do hệ điều hành cấp phát). Cấm tuyệt đối việc lắng nghe trên `0.0.0.0` hoặc địa chỉ IP LAN.
  - Nhúng toàn bộ tài nguyên frontend tĩnh (Vue 3 / TypeScript SPA, CSS Tailwind, Web Fonts nội bộ) trực tiếp vào file nhị phân Rust thông qua macro `include_dir!`, bảo đảm hệ thống vận hành trơn tru ở chế độ Air-Gapped (0 CDN ngoại vi).
  - Cấu hình tiêu chuẩn HTTP Security Headers: Content Security Policy (CSP khóa cứng), X-Frame-Options: DENY, X-Content-Type-Options: nosniff, Referrer-Policy: no-referrer.
- **Vai trò phụ trách chính**: Full-stack Engineer (FSE)
- **Vai trò phối hợp**: Security Engineer (SE), Senior Rust Systems Engineer (RE2)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 1.4.1
- **Sản phẩm bàn giao**: Crate Web Gateway & Module Nhúng Tài Nguyên Tĩnh (`crates/liva-web-gateway/src/server.rs`, `src/assets.rs`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Máy chủ chỉ phản hồi các yêu cầu từ `127.0.0.1` hoặc `::1`, từ chối dứt điểm mọi kết nối từ card mạng ngoài.
  2. Giao diện Web hiển thị đầy đủ, không thiếu font hay icon khi ngắt toàn bộ kết nối Internet.
  3. Kiểm tra bảo mật CSP đạt chuẩn, không nạp bất kỳ tài nguyên ngoại vi nào.

#### [WBS 1.6.2] Xây dựng Cơ chế Xác thực Cục bộ (Local Auth) & Quản lý Phiên Bảo mật
- **Mô tả chi tiết phạm vi**:
  - Phát triển module quản lý tài khoản và xác thực người dùng cục bộ trên máy trạm:
    * Băm mật khẩu người dùng bằng thuật toán **Argon2id** (`argon2::Variant::Argon2id`, tham số an toàn bộ nhớ $m=65536, t=3, p=4$) với muối ngẫu nhiên cấp hệ điều hành.
    * Quản lý phiên làm việc thông qua Cookie an toàn: `Set-Cookie: __Host-LIVA-Session=<token>; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=28800`.
  - Tích hợp cơ chế chống tấn công CSRF hai lớp (Double Submit Cookie kết hợp Header `X-CSRF-Token`).
  - Xây dựng middleware Rate Limiter: khóa tạm thời tài khoản sau 5 lần thử mật khẩu sai liên tiếp trong 15 phút.
- **Vai trò phụ trách chính**: Full-stack Engineer (FSE)
- **Vai trò phối hợp**: Security Engineer (SE)
- **Ước lượng ngày công**: 3 Man-days
- **Task phụ thuộc**: WBS 1.6.1
- **Sản phẩm bàn giao**: Module Xác Thực Cục Bộ & Quản Lý Phiên (`crates/liva-web-gateway/src/auth/`, `src/middleware/csrf.rs`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Mật khẩu lưu trong SQLite được băm bằng Argon2id, không thể giải mã ngược.
  2. Toàn bộ các yêu cầu thay đổi trạng thái (POST/PUT/DELETE) bị chặn với lỗi `403 Forbidden` nếu thiếu CSRF token hợp lệ.
  3. Cookie phiên tuân thủ nghiêm ngặt cờ `__Host-`, không thể bị đánh cắp bởi mã script trình duyệt.

#### [WBS 1.6.3] Hiện thực Cơ chế Phân quyền RBAC Đa vai trò & Kiểm soát Kép (Circular 09 4-Eyes)
- **Mô tả chi tiết phạm vi**:
  - Triển khai hệ thống phân quyền theo vai trò (Role-Based Access Control - RBAC) trên cả giao diện Web và tầng kiểm tra API:
    * **Cán bộ Vận hành (Maker)**: Tải sao kê, chạy engine đối soát, tạo lệnh chi, lập đề xuất xử lý ngoại lệ HITL.
    * **Kiểm soát viên (Checker)**: Thẩm định hồ sơ, phê duyệt/từ chối lệnh chi, ký duyệt chênh lệch đối soát, đóng dấu băm Merkle Tree.
    * **Quản trị viên (Admin)**: Cấu hình hệ thống, quản lý tài khoản, xem nhật ký kiểm toán.
  - Cưỡng chế nguyên tắc bất biến Kiểm soát kép theo Thông tư 09/2020/TT-NHNN:
    $$\text{maker.employee\_id} \ne \text{checker.employee\_id} \quad \land \quad \text{maker.citizen\_id\_hash} \ne \text{checker.citizen\_id\_hash}$$
  - Ngăn chặn hoàn toàn tình trạng một cá nhân lập hai tài khoản để tự tạo và tự duyệt lệnh tài chính của chính mình.
- **Vai trò phụ trách chính**: Full-stack Engineer (FSE)
- **Vai trò phối hợp**: Lead Rust Engineer (RE1), Tư vấn Pháp lý (TVPL / LC), Kế toán trưởng (KTT / SME)
- **Ước lượng ngày công**: 4 Man-days
- **Task phụ thuộc**: WBS 1.6.2, WBS 1.2.3
- **Sản phẩm bàn giao**: Module Phân Quyền RBAC & Giao Diện 3 Cổng Tác Nghiệp (`crates/liva-web-gateway/src/rbac.rs`, `liva-ui/src/views/portals/`).
- **Tiêu chí nghiệm thu kỹ thuật**:
  1. Người dùng mang vai trò Maker nhận lỗi `403 Forbidden` khi cố tình gọi API phê duyệt lệnh chi.
  2. Kiểm soát viên Checker không thể phê duyệt lệnh do chính mình tạo ra (hệ thống từ chối và ghi log cảnh báo kiểm toán).
  3. Mọi hành động duyệt chi đều sinh bản ghi kiểm toán bất biến chứa dấu băm SHA-256 và định danh cán bộ.

---

## 4. BẢNG TỔNG HỢP NGUỒN LỰC (MAN-DAYS ROLLUP) & CÂN ĐỐI CÔNG SUẤT

### 4.1. Ma Trận Phân Bổ Ngày Công Theo Vai Trò (8–9 FTE Baseline)

**Phương pháp luận tính toán ngày công phối hợp (Coordinating Days Methodology)**:
- Cột **Lead (Phụ trách chính)**: Tổng hợp chính xác $100\%$ từ dưới lên (Bottom-up Rollup) từ 49 nhiệm vụ chi tiết tại Mục 2 và Mục 3, phản ánh thời lượng trực tiếp lập trình, thiết kế kiến trúc hoặc xây dựng bộ quy tắc.
- Cột **Phối (Phối hợp & Soát xét)**: Được chuẩn hóa theo định mức từ $0.5$ đến $1.0\text{ Man-day}$ cho mỗi vai trò tham gia phản biện chéo, rà soát mã nguồn (code review), kiểm thử độc lập, hoặc họp thẩm định nghiệp vụ trên từng task được chỉ định.
- **Triệt tiêu ngày công ảo**: Tại Giai đoạn 1, AI & SLM Engineer (AIE) có $0\text{ MD Lead}$ và $0\text{ MD Phối}$ do toàn bộ 20 tasks GĐ1 tập trung vào Rust Core, Kernel Isolation và Local Web UI, loại bỏ triệt để 28 MD phối hợp ma không có căn cứ thực tế.

```
+---------------------------------------------------------------------------------------------------------+
|                                    BẢNG TỔNG HỢP PHÂN BỔ NGÀY CÔNG THEO VAI TRÒ                         |
+--------------------+----------+--------------------+--------------------+-------------------------------+
| Vai Trò Nhân Lực   | Định Mức | GIAI ĐOẠN 0 (4T)   | GIAI ĐOẠN 1 (8T)   | TỔNG CỘNG 2 GIAI ĐOẠN         |
| (Role Designation) | FTE      | Lead / Phối / Tổng | Lead / Phối / Tổng | Lead MD  / Phối MD / Tổng MD  |
+--------------------+----------+--------------------+--------------------+-------------------------------+
| Solution Arch (SA) | 1.0 FTE  | 14 MD / 06 MD / 20 | 04 MD / 18 MD / 22 | 18 Man-days / 24 MD / 42 MD   |
| Lead Rust (RE1)    | 1.0 FTE  | 11 MD / 06 MD / 17 | 27 MD / 05 MD / 32 | 38 Man-days / 11 MD / 49 MD   |
| Senior Rust (RE2)  | 1.0 FTE  | 12 MD / 06 MD / 18 | 12 MD / 12 MD / 24 | 24 Man-days / 18 MD / 42 MD   |
| AI Engineer (AIE)  | 1.0 FTE  | 11 MD / 04 MD / 15 | 00 MD / 00 MD / 00 | 11 Man-days / 04 MD / 15 MD   |
| Full-stack (FSE)   | 1.0 FTE  | 10 MD / 05 MD / 15 | 10 MD / 16 MD / 26 | 20 Man-days / 21 MD / 41 MD   |
| Security Eng (SE)  | 1.0 FTE  | 20 MD / 00 MD / 20 | 11 MD / 16 MD / 27 | 31 Man-days / 16 MD / 47 MD   |
| QA / SDET (QAE)    | 1.0 FTE  | 06 MD / 10 MD / 16 | 09 MD / 24 MD / 33 | 15 Man-days / 34 MD / 49 MD   |
| Kế toán trg (KTT)  | 1.0 FTE  | 07 MD / 07 MD / 14 | 00 MD / 25 MD / 25 | 07 Man-days / 32 MD / 39 MD   |
| Pháp lý (TVPL)     | 0.5 FTE  | 03 MD / 02 MD / 05 | 00 MD / 10 MD / 10 | 03 Man-days / 12 MD / 15 MD   |
+--------------------+----------+--------------------+--------------------+-------------------------------+
| TỔNG CỘNG TOÀN ĐỘI | 8.5 FTE  | 94 MD / 46 MD / 140| 73 MD /126 MD / 199| 167 MD  / 172 MD / 339 MD     |
+--------------------+----------+--------------------+--------------------+-------------------------------+
```
*(Ghi chú: Toàn bộ số liệu Lead MD khớp 100% từng dòng với 29 tasks GĐ0 = 94 MD và 20 tasks GĐ1 = 73 MD; Tổng Lead cả 2 giai đoạn là 167 MD).*

---

### 4.2. Đối Chiếu Công Suất Lịch Biểu Thực Tế (Calendar Schedule Capacity Reconciliation)

**Phân biệt giữa Ngân Sách Giai Đoạn và Công Suất Lịch Biểu Thời Gian Thực**:
- **Khung thời gian lịch biểu thực tế**: Toàn dự án diễn ra trong **10 tuần lịch liên tục (Tuần 1 đến Tuần 10)**, tương đương **50 ngày làm việc**.
- **Loại bỏ ngụy biện tính trùng (Double-Counting)**:
  * GĐ0 kéo dài 4 tuần (Tuần 1–4 = 20 ngày làm việc $\times$ 8.5 FTE = 170 MD).
  * GĐ1 kéo dài 8 tuần (Tuần 3–10 = 40 ngày làm việc $\times$ 8.5 FTE = 340 MD).
  * Hai giai đoạn gối sóng tại Tuần 3–4 (2 tuần = 10 ngày làm việc $\times$ 8.5 FTE = **85 Man-days**).
  * Việc cộng gộp cơ học $170 + 340 = 510\text{ MD}$ là sai lệch do đã tính hai lần 85 MD của Tuần 3–4.
  * **Tổng công suất khả dụng thực tế của đội ngũ 8.5 FTE trong 10 tuần lịch**:
    $$\text{Tổng công suất khả dụng} = 50\text{ ngày làm việc} \times 8.5\text{ FTE} = \mathbf{425\text{ Man-days}}$$

```
+-------------------------------------------------------------------------------------------------------------------+
|                                    BẢNG ĐỐI CHIẾU CÂN ĐỐI CÔNG SUẤT VÀ DỰ PHÒNG THỰC TẾ (425 MD BASELINE)        |
+------------------------------+----------+-----------+-----------+-----------+-------------+-------------+---------+
| Vai Trò Nhân Lực (Role)      | FTE Định | GĐ0 Thực  | GĐ1 Thực  | Tổng Thực | Tổng Khả    | Tỷ Lệ Sử    | Quỹ Dự  |
|                              | Mức      | Tế Phân Bổ| Tế Phân Bổ| Tế Phân Bổ| Dụng (50N)  | Dụng (Cap %)| Phòng MD|
+------------------------------+----------+-----------+-----------+-----------+-------------+-------------+---------+
| 1. Solution Architect (SA)   | 1.0 FTE  | 20 MD     | 22 MD     | 42 MD     | 50 MD       | 84.0 %      | 08 MD   |
| 2. Lead Rust Engineer (RE1)  | 1.0 FTE  | 17 MD     | 32 MD     | 49 MD     | 50 MD       | 98.0 %      | 01 MD   |
| 3. Senior Rust Engineer (RE2)| 1.0 FTE  | 18 MD     | 24 MD     | 42 MD     | 50 MD       | 84.0 %      | 08 MD   |
| 4. AI & SLM Engineer (AIE)   | 1.0 FTE  | 15 MD     | 00 MD     | 15 MD     | 50 MD       | 30.0 %      | 35 MD   |
| 5. Full-stack Engineer (FSE) | 1.0 FTE  | 15 MD     | 26 MD     | 41 MD     | 50 MD       | 82.0 %      | 09 MD   |
| 6. Security Engineer (SE)    | 1.0 FTE  | 20 MD     | 27 MD     | 47 MD     | 50 MD       | 94.0 %      | 03 MD   |
| 7. QA / SDET Engineer (QAE)  | 1.0 FTE  | 16 MD     | 33 MD     | 49 MD     | 50 MD       | 98.0 %      | 01 MD   |
| 8. Kế toán trưởng (KTT / SME)| 1.0 FTE  | 14 MD     | 25 MD     | 39 MD     | 50 MD       | 78.0 %      | 11 MD   |
| 9. Tư vấn Pháp lý (TVPL / LC)| 0.5 FTE  | 05 MD     | 10 MD     | 15 MD     | 25 MD       | 60.0 %      | 10 MD   |
+------------------------------+----------+-----------+-----------+-----------+-------------+-------------+---------+
| **TỔNG CỘNG PHÂN BỔ**        | 8.5 FTE  | **140 MD**| **199 MD**| **339 MD**| **425 MD**  | **79.8 %**  | **86 MD**|
| **QUỸ DỰ PHÒNG (BUFFER)**    | —        | —         | —         | **86 MD** | —           | **20.2 %**  | —       |
+------------------------------+----------+-----------+-----------+-----------+-------------+-------------+---------+
```

---

### 4.3. Đánh Giá Cân Đối Tải, Điểm Nghẽn Tiến Độ & Quỹ Dự Phòng Thực Tế

1. **Giải Tỏa Điểm Nghẽn Tiến Độ của Kỹ Sư An Toàn Thông Tin (SE)**:
   - Trong 4 tuần đầu (GĐ0), Kỹ sư An toàn Thông tin (SE) phụ trách chính 20 Man-days Lead trên 20 ngày làm việc (WBS 0.7.1, 0.7.2, 0.2.2, 0.6.1, 0.6.2, 0.6.3), đạt đúng $100\%$ công suất, không bị gánh thêm bất kỳ ngày công phối hợp nào.
   - Để tránh xung đột tiến độ, **Luồng 1.4 (`liva-netguard`, 11 MD Lead) không khởi động trong Tuần 3–4 mà chính thức bắt đầu từ Tuần 5 đến Tuần 10**. Trong 6 tuần này (30 ngày làm việc), SE được phân bổ 11 Lead + 16 Phối = 27 MD ($90.0\%$ tải), hoàn toàn khả thi và bảo đảm tiến độ chất lượng cao.
2. **Triệt Tiêu Quá Tải Trong Giai Đoạn 0**:
   - Tư vấn Pháp lý (TVPL, 0.5 FTE = 10 MD công suất GĐ0) được chuẩn hóa về mức 3 Lead + 2 Phối = **5 MD ($50.0\%$ tải)**, loại trừ triệt để tình trạng quá tải $140\%$ trước đây.
   - Tổng phân bổ toàn đội trong GĐ0 là **140 MD / 170 MD khả dụng ($82.4\%$)**, dự trữ 30 MD đệm an toàn ngay trong 4 tuần đầu tiên.
3. **Quỹ Dự Phòng Rủi Ro Thực Tế (Contingency Reserve: 86 Man-days ~ 20.2%)**:
   - Quỹ dự phòng được tính toán trên cơ sở chuẩn **425 Man-days thực tế** ($425 - 339 = \mathbf{86\text{ Man-days}}$, tương đương **$20.2\%$**).
   - Nguồn dự phòng linh hoạt này gồm:
     * 35 MD từ vị trí AI Engineer sau khi hoàn tất các nhiệm vụ SLM/OCR GĐ0, sẵn sàng hỗ trợ kiểm thử đối soát hoặc tối ưu hóa hiệu năng.
     * 51 MD dự phòng rải đều trên các vị trí KTT (11 MD), TVPL (10 MD), FSE (9 MD), SA (8 MD), RE2 (8 MD), SE (3 MD), RE1 (1 MD), QAE (1 MD).
4. **Minh Định Ranh Giới Gối Sóng (Overlapping Waves Tuần 3–4)**:
   - *Các luồng gối sóng được phép khởi động trong Tuần 3–4*: Luồng 1.1 (`liva-money`), Luồng 1.2 (`liva-ledger`), Luồng 1.3 (`proptest`), và Luồng 1.5 (`BankParser` Schema) sau khi hoàn tất Gate 0A.
   - *Các luồng khởi động từ Tuần 5*: Luồng 1.4 (`liva-netguard`) và Luồng 1.6 (`liva-web-gateway`) sau khi nghiệm thu toàn diện Gate 0B.

---

## 5. MA TRẬN TRACEABILITY ĐỐI SOÁT EXIT CRITERIA (1:1 MAPPING)

Để bảo đảm tính chặt chẽ và không bỏ sót bất kỳ tiêu chuẩn đầu ra nào, bảng dưới đây thực hiện ánh xạ **1:1 toàn diện** giữa toàn bộ 4 Exit Criteria của GĐ0 và 3 Exit Criteria của GĐ1 với các task WBS cụ thể, phương pháp đo kiểm và vai trò chịu trách nhiệm nghiệm thu:

```
+-------------------------------------------------------------------------------------------------------------------+
|                                    MA TRẬN ĐỐI SOÁT 1:1 TIÊU CHÍ ĐẦU RA (EXIT CRITERIA)                           |
+-------------+------------------------------------+------------------+-----------------------+---------------------+
| Mã Exit     | Tên & Nội Dung Tiêu Chí Đầu Ra     | Mã Task WBS Chịu | Phương Pháp Đo Kiểm & | Vai Trò Ký Biên Bản |
| Criteria    | (Acceptance Target Criteria)       | Trách Nhiệm Đóng | Bằng Chứng Nghiệm Thu | Nghiệm Thu Cuối Cùng|
+-------------+------------------------------------+------------------+-----------------------+---------------------+
| **GĐ0-EC1** | **SRS & HỒ SƠ THIẾT KẾ ĐƯỢC DUYỆT**| • WBS 0.1.4 (BPMN| • Biên bản họp thẩm   | • Kế toán trưởng    |
|             | Toàn bộ yêu cầu phần mềm (SRS), user| • WBS 0.3.1 (NĐ13|   định đa bên.        | • Solution Architect|
|             | journeys kế toán, ma trận quy tắc  | • WBS 0.3.2 (TT09| • Hồ sơ tài liệu kỹ   | • Tư vấn Pháp lý    |
|             | đối soát và tài liệu kiến trúc kỹ  | • WBS 0.4.3 (Vouc|   thuật hoàn chỉnh    |                     |
|             | thuật được các bên ký duyệt.       | • WBS 0.4.4 (Integ   đạt chuẩn docs-check|                     |
+-------------+------------------------------------+------------------+-----------------------+---------------------+
| **GĐ0-EC2** | **GOLDEN FILES >= 3 NGÂN HÀNG**    | • WBS 0.2.1 (Coll| • Chạy script PII     | • Security Engineer |
|             | Thu thập tối thiểu 3 ngân hàng VN  | • WBS 0.2.2 (PII |   scanner độc lập.    | • QA / SDET Engineer|
|             | (VCB, TCB, BIDV) + CAMT.053 XML;   | • WBS 0.2.3 (Anon| • Bảng `manifest.json`| • Kế toán trưởng    |
|             | 100% dữ liệu PII được che mờ và    |                  |   kèm mã băm SHA-256  |                     |
|             | gắn nhãn ground-truth đầy đủ.      |                  |   cho >= 5.000 dòng.  |                     |
+-------------+------------------------------------+------------------+-----------------------+---------------------+
| **GĐ0-EC3** | **THREAT MODEL STRIDE & DFD DUYỆT**| • WBS 0.6.1 (DFD)| • Báo cáo STRIDE hoàn | • Security Engineer |
|             | Mô hình đe dọa STRIDE hoàn chỉnh,  | • WBS 0.6.2 (STRI|   tất 18 kịch bản.    | • Solution Architect|
|             | xác lập ranh giới Zero-Egress và sơ| • WBS 0.6.3 (Fail| • DFD L0/L1/L2 đạt    |                     |
|             | đồ luồng dữ liệu DFD L0/L1/L2 được |                  |   chuẩn bảo mật và    |                     |
|             | phê duyệt không còn rủi ro hở.     |                  |   xác lập Fail-Closed.|                     |
+-------------+------------------------------------+------------------+-----------------------+---------------------+
| **GĐ0-EC4** | **HẠ TẦNG DEV NỘI BỘ & SYNTHETIC** | • WBS 0.7.1 (GitL| • GitLab CE LAN uptime| • Security Engineer |
|             | Hạ tầng self-hosted (GitLab CE, CI | • WBS 0.7.2 (Runn|   99.9%, runner offline| • Senior Rust (RE2) |
|             | Runner giới hạn RAM, chat nội bộ)  | • WBS 0.7.3 (Chat|   chạy pass pipeline. | • QA / SDET Engineer|
|             | và bộ sinh dữ liệu tổng hợp sẵn    | • WBS 0.8.4 (CLI)| • Sinh 50.000 dòng    |                     |
|             | sàng vận hành.                     |                  |   giao dịch trong 10s.|                     |
+-------------+------------------------------------+------------------+-----------------------+---------------------+
| **GĐ1-EC1** | **DEMO LEDGER CRUD + INVARIANTS**  | • WBS 1.2.2 (Stat| • Demo trực quan hạch | • Lead Rust (RE1)   |
|             | Tạo lập, truy vấn, ghi sổ kế toán  | • WBS 1.2.3 (Even|   toán sổ cái kép.    | • Kế toán trưởng    |
|             | kép CRUD; vượt qua >= 1.000.000 test| • WBS 1.2.4 (HMAC| • Lệnh `cargo test    | • QA / SDET Engineer|
|             | cases proptest chứng minh bảo toàn | • WBS 1.3.2 (Prop|   -p liva-ledger`     |                     |
|             | bất biến số dư, 0 panic, 0 tràn số.| • WBS 1.3.3 (Strs|   vượt 1 triệu cases. |                     |
+-------------+------------------------------------+------------------+-----------------------+---------------------+
| **GĐ1-EC2** | **EGRESS TEST 100% BLOCK THỰC CHỨNG| • WBS 1.4.1 (Nfta| • Chạy test suite đối | • Security Engineer |
|             | Hệ thống liva-netguard (nftables,  | • WBS 1.4.2 (Secc|   kháng xâm nhập.     | • QA / SDET Engineer|
|             | seccomp-bpf, systemd) chặn 100% kết| • WBS 1.4.3 (Adve| • Log kernel ghi nhận | • Solution Architect|
|             | nối TCP/UDP ra ngoài loopback      |                  |   chặn kết nối; pcap  |                     |
|             | 127.0.0.1; 0 byte lọt ra Internet. |                  |   ghi nhận 0 byte lọt.|                     |
+-------------+------------------------------------+------------------+-----------------------+---------------------+
| **GĐ1-EC3** | **CI/CD XANH SẠCH & ZERO FLOAT**   | • WBS 1.1.1 (Deny| • Pipeline CI xanh:   | • Lead Rust (RE1)   |
|             | Pipeline CI hoàn toàn tự động, đạt | • WBS 1.1.3 (VAS |   `cargo clippy` 0    | • Full-stack (FSE)  |
|             | exit code 0: clippy zero warning,  | • WBS 1.5.3 (Pars|   warning, deny float | • QA / SDET Engineer|
|             | clippy deny float arithmetic trên  | • WBS 1.6.3 (RBAC|   đạt chuẩn, 100% unit|                     |
|             | module tiền tệ, 100% tests pass.   |                  |   và e2e tests pass.  |                     |
+-------------+------------------------------------+------------------+-----------------------+---------------------+
```

---

## 6. QUẢN TRỊ RỦI RO & QUY CHUẨN NGHIỆM THU

### 6.1. Ma Trận Rủi Ro & Kế Hoạch Ứng Phó (Technical, Legal & Business)

```
+-------------------------------------------------------------------------------------------------------------------+
|                                    MA TRẬN QUẢN TRỊ RỦI RO DỰ ÁN (RISK MATRIX)                                    |
+----+--------------------+----------+------------+-----------------------------------------------------------------+
| STT| Rủi Ro Nhận Diện   | Phân Loại| Mức Độ     | Phương Án Phòng Ngừa & Kiểm Soát Kỹ Thuật (Mitigation Plan)     |
+----+--------------------+----------+------------+-----------------------------------------------------------------+
| 1  | **Sai lệch Dấu     | Kỹ thuật | NGHIÊM     | CẤM TIỆT `f32`/`f64`. Cưỡng chế chỉ thị biên dịch               |
|    | phẩy động (Float)  |          | TRỌNG      | `#![deny(clippy::float_arithmetic)]` trên toàn bộ crate         |
|    | trong tính toán**  |          |            | `liva-money` và `liva-ledger`. Sử dụng số nguyên `i64` VND.    |
+----+--------------------+----------+------------+-----------------------------------------------------------------+
| 2  | **Lộ lọt dữ liệu   | Pháp lý  | RẤT CAO    | Áp dụng cơ chế lọc PII 2 tầng (Regex + NER từ điển tiếng Việt); |
|    | cá nhân PII theo   |          |            | Sandbox `liva-netguard` khóa 100% cổng mạng ngoại vi;           |
|    | Nghị định 13**     |          |            | Nguyên tắc Fail-Closed: nghi ngờ là tự động che mờ.             |
+----+--------------------+----------+------------+-----------------------------------------------------------------+
| 3  | **Cạnh tranh khóa  | Kỹ thuật | TRUNG BÌNH | Thiết kế mô hình Single-Writer Actor (MPSC Channel trong Rust); |
|    | ghi trong SQLite   |          |            | Mọi thao tác ghi sổ cái đẩy vào hàng đợi thực thi tuần tự batch;|
|    | WAL (SQLITE_BUSY)**|          |            | Vô hạn luồng đọc đồng thời không khóa qua WAL.                  |
+----+--------------------+----------+------------+-----------------------------------------------------------------+
| 4  | **Ngân hàng thay   | Nghiệp vụ| TRUNG BÌNH | Kiến trúc `BankParser` trait plugin cho phép nạp parser mới     |
|    | đổi mẫu sao kê đột |          |            | độc lập mà không cần sửa code lõi; Local SLM bóc tách bảng      |
|    | ngột (Format Drift)|          |            | biểu tự động (Zero-shot Table Ingestion) hỗ trợ nhận diện.     |
+----+--------------------+----------+------------+-----------------------------------------------------------------+
| 5  | **Tràn bộ nhớ RAM  | Kỹ thuật | CAO        | Luôn truyền cờ `-j 2 -- --test-threads 2` khi chạy build/test;  |
|    | khi chạy Fuzzing   |          |            | Chia nhỏ 1.000.000 cases thành 100 batch có cơ chế giải phóng   |
|    | 1.000.000 cases**  |          |            | bộ đệm giữa các batch; kiểm soát RAM kiểm thử $\le 2.0\text{ GB}$.|
+----+--------------------+----------+------------+-----------------------------------------------------------------+
| 6  | **ERP từ chối nạp  | Nghiệp vụ| TRUNG BÌNH | Bộ Schema Validator kiểm tra dữ liệu trước khi xuất file;       |
|    | tệp chứng từ xuất  |          |            | Đọc log phản hồi từ ERP theo cơ chế Closed-Loop Confirmation để |
|    | bản (Rejection)**  |          |            | tự động rollback trạng thái hạch toán nếu phát sinh lỗi.        |
+----+--------------------+----------+------------+-----------------------------------------------------------------+
| 7  | **Rủi ro Bản quyền | Pháp lý  | TRUNG BÌNH | Đóng băng lựa chọn mô hình Qwen 2.5 với giấy phép Apache 2.0    |
|    | SLM thương mại**   |          |            | nguyên bản, không dùng các bản fork không rõ nguồn gốc bản quyền|
+----+--------------------+----------+------------+-----------------------------------------------------------------+
```

---

### 6.2. Tiêu Chuẩn Chất Lượng Kỹ Thuật (Quality Gates)

Mọi đóng góp mã nguồn và sản phẩm bàn giao bắt buộc phải vượt qua 5 cổng kiểm định chất lượng nghiêm ngặt trước khi được chấp thuận:

1. **Cổng Kiểm Định Mã Nguồn (Compiler & Linter Gate)**:
   - `cargo fmt --all -- --check` đạt chuẩn format (Exit code 0).
   - `cargo clippy --workspace --all-targets -j 2 -- -D warnings` đạt 0 warning, 0 error.
   - Directive `#![deny(clippy::float_arithmetic)]` bắt buộc hiện diện và xanh sạch trên toàn bộ các crate nghiệp vụ tài chính.
2. **Cổng Kiểm Thử Đơn Vị & Tích Hợp (Unit & Integration Test Gate)**:
   - `cargo test --workspace -j 2 -- --test-threads 2` vượt qua $100\%$ các bài kiểm tra (0 failed).
   - Độ bao phủ kiểm thử (Code Coverage) đạt tối thiểu $90\%$ trên crate `liva-money` và `liva-ledger`.
3. **Cổng Kiểm Thử Bất Biến & Fuzzing (Invariant & Fuzzing Gate)**:
   - Bộ kiểm thử `proptest` thực thi vượt qua tối thiểu $1.000.000$ test cases mà không có bất kỳ vi phạm bất biến nào ($\Delta = 0$).
   - Mức tiêu thụ bộ nhớ RAM tối đa của tiến trình kiểm thử luôn duy trì $\le 2.0\text{ GB}$.
4. **Cổng An Toàn Mạng Zero-Egress (Network Isolation Gate)**:
   - Bộ kiểm thử đối kháng thâm nhập mạng khẳng định $100\%$ các gói tin gửi ra ngoài loopback `127.0.0.1` đều bị kernel nftables hoặc seccomp chặn đứng.
   - Bắt gói tin độc lập (pcap audit) ghi nhận chính xác 0 byte truyền ra card mạng ngoài.
5. **Cổng Tài Liệu & Pháp Lý (Documentation & Compliance Gate)**:
   - Lệnh kiểm tra liên kết tài liệu `node scripts/docs-check.mjs` đạt 0 lỗi, 0 cảnh báo liên kết hỏng.
   - Hồ sơ DPIA Mẫu 04 và các Thư tư vấn pháp lý có chữ ký xác nhận của luật sư chuyên trách.

---

### 6.3. Quy Trình Chuyển Giao Giai Đoạn (GĐ0 $\rightarrow$ GĐ1 Staged Handshake Protocol)

Để bảo đảm tính linh hoạt cho mô hình gối sóng Tuần 3–4 mà vẫn duy trì kỷ luật kiểm soát chất lượng tuyệt đối, quy trình bàn giao giữa Giai đoạn 0 và Giai đoạn 1 được tổ chức theo **Giao Thức Chuyển Giao Phân Đoạn (Staged Handshake Protocol)** gồm 2 cổng thẩm định:

```
                  GIAO THỨC CHUYỂN GIAO PHÂN ĐOẠN (STAGED HANDSHAKE PROTOCOL)

┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ CỔNG 0A: ĐÓNG BĂNG KIẾN TRÚC LÕI & KÍCH HOẠT GỐI SÓNG (CUỐI TUẦN 2)                              │
│ - Điều kiện tiên quyết: Nghiệm thu Scaffolding Rust (WBS 0.5.1) và Bảng danh mục COA (WBS 0.1.1).│
│ - Phê duyệt: Solution Architect (SA) và Kế toán trưởng (KTT) ký biên bản Gate 0A.                │
│ - Kích hoạt sớm (Early Mobilization Tuần 3–4):                                                   │
│   * Luồng 1.1 (liva-money): Xây dựng kiểu dữ liệu Money, VAS Rounding và Currency Parser.        │
│   * Luồng 1.2 (liva-ledger): Triển khai Double-Entry State Machine và Event Store bất biến.      │
│   * Luồng 1.3 (proptest): Xây dựng bộ sinh ngẫu nhiên giao dịch tài chính (arb_money).          │
│   * Luồng 1.5 (WBS 1.5.1): Soạn thảo Canonical Transaction Schema trong Rust.                     │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
                                                │
                                                ▼
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│ CỔNG 0B: NGHIỆM THU TOÀN DIỆN 4 EXIT CRITERIA GĐ0 & TRIỂN KHAI TOÀN DIỆN (CUỐI TUẦN 4)          │
│ - Điều kiện tiên quyết: Hoàn tất 100% 4 Exit Criteria GĐ0 (SRS ký duyệt, Golden Files PII-free, │
│   STRIDE Threat Model & DFD L0/L1/L2, Hạ tầng phát triển GitLab CE nội bộ).                      │
│ - Phê duyệt: Hội đồng đa bên (SA, KTT, TVPL, SE) ký Biên bản Nghiệm thu Tổng thể GĐ0.            │
│ - Kích hoạt triển khai đầy đủ (Full Mobilization từ đầu Tuần 5):                                 │
│   * Luồng 1.4 (liva-netguard): Kỹ sư SE hoàn tất GĐ0, chính thức khởi động nftables & seccomp.   │
│   * Luồng 1.5 (WBS 1.5.2 & 1.5.3): Triển khai BankParser Sniffer và 4 Reference Parsers.         │
│   * Luồng 1.6 (liva-web-gateway): Triển khai Web Gateway 127.0.0.1, Local Auth và RBAC 4-Eyes.  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 7. KẾT LUẬN & PHÊ DUYỆT

Tài liệu **Kế hoạch Phân rã Cấu trúc Công việc (WBS Chi tiết Giai đoạn 0 & 1)** này là bản đặc tả kỹ thuật và kế hoạch thực thi sản xuất chính thức của dự án **LIVA Banking Harness**. 

Toàn bộ **49 nhiệm vụ chi tiết** được thiết kế với tính khả thi cao, phân bổ nguồn lực cân đối trên đội ngũ **8.5 FTE**, duy trì quỹ dự phòng an toàn thực tế **20.2% (86 Man-days trên tổng công suất 425 Man-days)**, và thiết lập cơ chế kiểm soát kỹ thuật nghiêm ngặt nhằm hiện thực hóa cam kết: **Nền tảng Tác tử Tài chính Cục bộ An toàn, Chuẩn xác Tuyệt đối và Độc lập Hàng đầu Việt Nam**.

---

### HỘI ĐỒNG PHÊ DUYỆT BẢN ĐẶC TẢ KỸ THUẬT

| Đại Diện Khối Kiến Trúc & Kỹ Thuật | Đại Diện Khối Nghiệp Vụ Tài Chính | Đại Diện Khối Pháp Lý & Tuân Thủ |
| :---: | :---: | :---: |
| *(Đã ký duyệt)* | *(Đã ký duyệt)* | *(Đã ký duyệt)* |
| **Solution Architect (SA)** | **Kế toán trưởng (KTT / SME)** | **Tư vấn Pháp lý (TVPL / LC)** |
| Ngày: 2026-09-15 | Ngày: 2026-09-15 | Ngày: 2026-09-15 |

*(Bản quyền thuộc về Dự án LIVA Banking Harness — Lưu hành Nội bộ & Đối tác Thẩm định)*

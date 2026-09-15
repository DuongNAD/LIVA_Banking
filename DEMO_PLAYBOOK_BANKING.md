# LIVA BANKING HARNESS — DEMO PLAYBOOK & SPEAKER SCRIPT (5 MINUTES)
## Comprehensive Technical Operation & Pitching Guide — INNOSTART 2026 Demo Day
**Dự Án**: LIVA Banking Harness — Agentic Harness for Banking & Corporate Treasury Automation  
**Tài liệu tham chiếu chuẩn**: `docs/pitching/DEMO_RUNBOOK.md` & `docs/pitching/PITCH_SCRIPT_5MIN.md`  
**Phiên bản**: 2.0 (Living Document) | **Ngày hoàn thiện**: 2026-09-14  

---

## 1. TỔNG QUAN & PHÂN BỔ THỜI GIAN (SESSION ARCHITECTURE)

- **Sự kiện**: Chung kết InnovationStart 2026 (Demo Day)
- **Tổng thời lượng phân bổ**: Đúng **12 phút**
  - **05 phút (300 giây)**: Thuyết trình Pitching & Trình diễn Demo Trực tiếp (Live Demo)
  - **07 phút (420 giây)**: Phản biện chuyên sâu trước Hội đồng Ban Giám khảo (Q&A Defense)
- **Phân vai đội hình sân khấu**:
  - **Diễn giả chính (Lead Presenter / CEO)**: Cầm clicker, làm chủ sân khấu, phát biểu chuẩn 688 từ tiếng Việt (~137.6 từ/phút), kết nối nỗi đau pháp lý với giải pháp công nghệ LIVA.
  - **Trợ lý kỹ thuật (Technical Co-pilot / CTO)**: Đứng tại bàn điều khiển, quan sát màn hình phụ, kích hoạt script tự động (`demo_runner.mjs` hoặc `run_live_demo.ps1`), hỗ trợ xử lý kỹ thuật nếu phát sinh sự cố.

---

## 2. CHECKLIST KIỂM TRA TIỀN TRẠM & BẢO VỆ TÀI NGUYÊN (PRE-FLIGHT DIAGNOSTICS)

Thực hiện kiểm tra tại phòng chờ kỹ thuật **30 phút** trước giờ lên sân khấu:

| Hạng Mục | Yêu Cầu Kỹ Thuật | Lệnh Kiểm Tra / Thao Tác | Tiêu Chuẩn Nghiệm Thu |
|:---|:---|:---|:---:|
| **Hệ Điều Hành** | Windows 10 / 11 Pro 64-bit | `winver` | Windows 64-bit |
| **Tài Nguyên RAM** | RAM khả dụng $\ge 4.0\text{ GB}$, tải $\le 80\%$ | `powershell scripts/ram-guard.ps1 -MinFreeGB 4.0` | Exit code 0, PASS |
| **Node.js Runtime** | Node.js v20+ hoặc v22+ LTS | `node --version` | v20.x hoặc v22.x |
| **Rust Native Core** | Bản biên dịch Release/Dev sạch | `cargo check -p liva-native-core -j 2` | Exit code 0 |
| **Bộ Dữ Liệu Sao Kê** | Đủ 3 sao kê mẫu tại `data/mock_statements/` và `fixtures/statements/` | `Get-ChildItem -Path data/mock_statements` | VCB (.xlsx), TCB (.csv), BIDV (.pdf) |
| **Chế Độ Mạng** | Chế độ Máy bay (Airplane Mode) — Zero Data Egress | Tắt Wi-Fi, rút cáp LAN, chỉ dùng loopback `127.0.0.1` | 0 byte rò rỉ ra Internet |
| **Cổng Mạng Cục Bộ** | Giải phóng cổng UI và Backend | `Test-NetConnection -ComputerName 127.0.0.1 -Port 5173` | Sẵn sàng cho dev server |
| **Hiển Thị & Màn Hình** | 1920x1080 (16:9), Duplicate Display | `Win + P` -> Duplicate | Không méo khung hình |
| **Pin & Nguồn Điện** | Cắm sạc trực tiếp, Best Performance | Cắm sạc AC adapter | Icon "Plugged in" |

---

## 3. CÁC CHẾ ĐỘ VẬN HÀNH BỘ CÔNG CỤ DEMO (DEMO RUNNER SUITE)

Hệ thống cung cấp hai công cụ điều phối demo:

### 3.1. Node.js Automated Demo Runner (`scripts/demo_runner.mjs`)
- **Kiểm tra tức thì (Dry-run mode)**:
  ```bash
  node scripts/demo_runner.mjs --dry-run
  ```
  *Đặc tính*: Bỏ qua thời gian chờ, xác thực toàn bộ chỉ số số học, bóc tách và sổ cái trong `< 0.05` giây, trả về exit code 0.
- **Diễn tập tốc độ cao (Rehearsal mode)**:
  ```bash
  node scripts/demo_runner.mjs --speed=60
  ```
  *Đặc tính*: Tăng tốc gấp 60 lần (hoàn tất 5 phút trong đúng 5 giây).
- **Trình diễn trực tiếp 05 phút (Live mode - 300s)**:
  ```bash
  node scripts/demo_runner.mjs
  ```
  *Đặc tính*: Đếm nhịp từng giây, hiển thị lời thoại CEO, chỉ dẫn sân khấu `[CUE]`, thanh tiến độ, bảng thống kê và số liệu đối soát thời gian thực.

### 3.2. PowerShell Complete 5-Step Live Script (`scripts/run_live_demo.ps1`)
Chạy kịch bản tích hợp từ PowerShell console:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/run_live_demo.ps1
```
*Trình tự 5 bước tự động*:
1. Kiểm tra tiền trạm tài nguyên máy trạm (RAM Guard fail-closed $\ge 4.0\text{ GB}$, CPU, Disk, Zero-egress).
2. Kiểm tra thư mục nóng sao kê (`data/mock_statements/` nạp VCB, TCB, BIDV).
3. Thực thi động cơ đối soát Rust Native Core (`cargo test -p liva-native-core -j 2 -- banking --test-threads 2`).
4. Kiểm thử đối kháng toán học 10,000 chu kỳ (`node scripts/adversarial-banking-m2-challenger.mjs`).
5. Sẵn sàng điều hướng khởi chạy giao diện 2D Banking Workbench (`liva-ui`).

---

## 4. KỊCH BẢN ĐỒNG BỘ TỪNG GIÂY (SECOND-BY-SECOND PLAYBOOK: 00:00 – 05:00)

```
00:00 ─── [Bước 1: Điểm Nghẽn Kế Toán & Rào Cản Nghị Định 13] ─── 01:00
01:00 ─── [Bước 2: Nạp Thư Mục Nóng & Bóc Tách Sao Kê Đa Dạng] ──── 02:30
02:30 ─── [Bước 3: Động Cơ Đối Soát 3 Tầng & Bất Biến Toán Học] ─── 03:45
03:45 ─── [Bước 4: Trợ Lý Tài Chính 2D & Tháp Canh Dòng Tiền] ───── 04:30
04:30 ─── [Bước 5: An Ninh Mật Mã, Sổ Cái Bất Biến & Gọi Vốn] ────── 05:00
```

---

### BƯỚC 1: 00:00 – 01:00 (60 Giây) | TỔNG QUAN NGUỒN VỐN & ĐIỂM NGHẼN PHÁP LÝ

- **Màn hình hiển thị**: Slide 1 kết hợp Khối Thẻ Ngân Hàng (VCB 1.45B, TCB 785.6M, BIDV 520M).
- **Hành động sân khấu**:
  - `00:00`: Bấm clicker chuyển **Slide 1**. Diễn giả đứng giữa sân khấu, ánh mắt bao quát toàn bộ Hội đồng Giám khảo.
  - `00:05`: CEO cất giọng trầm ấm, đồng cảm: *"Mỗi sáng, hàng chục nghìn kế toán trưởng đối mặt áp lực lớn: mở hàng chục file sao kê ngân hàng, căng mắt dò từng dòng đối soát sổ cái..."*
  - `00:25`: Dừng thở chiến thuật `[Nghỉ 1.5s]` khi nói đến *"bốn tiếng mỗi ngày"* và *"vùng mù dòng tiền T+1 đến T+3"*.
  - `00:35`: Giọng chuyển đanh thép, cảnh báo: *"Tại sao họ chưa dùng AI? Vì theo Nghị định 13 và Thông tư 09, tải dữ liệu sao kê lên Cloud AI là hành vi vi phạm pháp luật với mức phạt tới 5% doanh thu!"*
  - `00:50`: `[Nghỉ 1.5s]`. Khép lại phút 1 với nghịch lý thị trường: *"Khao khát tự động hóa, nhưng cánh cửa lên Cloud AI đã bị khóa chặt."*
- **Chỉ số kiểm chứng trên màn hình**:
  - Thẻ Vietcombank: Số dư `1,450,230,000 VND` (Đã đối soát `1,449,850,000`, Chưa khớp `380,000`).
  - Thẻ Techcombank: Số dư `785,600,000 VND` (Đã đối soát `785,100,000`, Chưa khớp `500,000`).
  - Thẻ BIDV: Số dư `520,000,000 VND` (Đã đối soát `518,750,000`, Chưa khớp `1,250,000`).
  - Hệ số thanh khoản hiện thời: `1.85` | Thời gian an toàn tiền mặt (Runway): `9.2 tháng`.

---

### BƯỚC 2: 01:00 – 02:30 (90 Giây) | NẠP SAO KÊ ĐA ĐỊNH DẠNG QUA HOT-FOLDER

- **Màn hình hiển thị**: Slide 2 kết hợp Thao tác Hot-Folder Dropzone.
- **Hành động sân khấu**:
  - `01:00`: Bấm clicker chuyển **Slide 2**. Giọng tự hào, mở rộng hai bàn tay giới thiệu *LIVA Banking Harness*.
  - `01:15`: Hướng tay về màn hình máy chiếu, thao tác kéo thả 3 tệp sao kê (`vcb_aug2026.xlsx`, `tcb_aug2026.csv`, `bidv_aug2026.pdf`) vào khu vực Hot-Folder.
  - `01:25`: `[Nghỉ 1.5s]`.
  - `01:30`: Hệ thống bóc tách hoàn tất trong vài phần trăm giây. Diễn giả nhấn từng từ dứt khoát: *"Zero Cloud Leakage — Không một byte dữ liệu nào rời khỏi thiết bị!"*
  - `02:00`: Giải thích cơ chế tự động nhận diện định dạng (`sniff_and_parse`), chuẩn hóa số học sang số nguyên scaled `u64`.
- **Chỉ số kiểm chứng trên màn hình**:
  - VCB OpenXML: 50 giao dịch (Calamine engine, xử lý ô gộp tiêu đề Merged Cells).
  - TCB CSV UTF-8 BOM: 60 giao dịch (Zero-copy csv reader, bóc tách Napas/VietQR).
  - BIDV PDF: 40 giao dịch (lopdf 2D spatial clustering $\Delta y \le 2.5\text{ pt}$, kiểm tra toàn vẹn từng trang).
  - Tổng số: 150 giao dịch bóc tách trong thời gian $< 20\text{ ms}$ (vượt xa SLA $< 2.0\text{s}$).

---

### BƯỚC 3: 02:30 – 03:45 (75 Giây) | ĐỘNG CƠ ĐỐI SOÁT 3 TẦNG BẤT BIẾN TOÁN HỌC

- **Màn hình hiển thị**: Slide 3 và Gauge Bán Nguyệt 99.8% Tỷ lệ Khớp Tự Động.
- **Hành động sân khấu**:
  - `02:30`: Bấm clicker chuyển **Slide 3**. Giọng đanh thép, nhìn thẳng các giám khảo công nghệ: *"Tại sao RPA hay Cloud SaaS không giải quyết được? RPA gãy bot khi ngân hàng đổi mẫu. Cloud SaaS mắc ảo giác số học."*
  - `02:50`: Phân tích 3 trụ cột của Rust Native Core:
    1. Tối ưu tài nguyên: Chiếm dưới **4GB RAM**, không cần GPU đắt tiền.
    2. Bất biến toán học: Số nguyên scaled `u64` VND, triệt tiêu sai số dấu phẩy động (Zero Floating Drift).
    3. An ninh ngân hàng: Mã hóa AES-256-GCM v2, khóa TPM 2.0 / DPAPI.
  - `03:15`: Chỉ vào Gauge bán nguyệt và phân tích cơ chế Fail-Closed: 3 giao dịch lệch được chuyển vào hàng đợi HITL với chữ ký UUID v4.
- **Chỉ số kiểm chứng trên màn hình**:
  - Gauge bán nguyệt: **99.8%** Tỷ lệ khớp (`1,842 / 1,845` giao dịch).
  - Phân bổ: Tự động `99.2%`, Bằng tay `0.6%`, Chưa khớp `3` giao dịch (`0.2%`).
  - Hàng đợi HITL Two-Phase Confirmation: 3 UUID v4 tokens hiển thị minh bạch (VCB `VCBDISC001` 380k, TCB `FT2624999999` 500k, BIDV `FT2621999999` 1.25M).

---

### BƯỚC 4: 03:45 – 04:30 (45 Giây) | HỘI THOẠI TRỢ LÝ TÀI CHÍNH 2D THỜI GIAN THỰC

- **Màn hình hiển thị**: Slide 4 và Khung chat/voice Trợ lý 2D Banking Workbench.
- **Hành động sân khấu**:
  - `03:45`: Bấm clicker chuyển **Slide 4**. Giọng tự tin, sắc sảo: Trình bày mô hình B2B Dual-Track ($300-$1000/tháng cho CFO; License ngân hàng $50k-$150k/năm, biên lợi nhuận gộp $> 88\%$).
  - `04:00`: Thao tác tương tác bằng giọng nói với Trợ lý: *"LIVA, phân tích biến động dòng tiền tuần qua và dự báo ngân quỹ 30 ngày tới?"*
  - `04:05`: Trợ lý LIVA stream câu trả lời tức thời ($< 250\text{ ms}$), đề xuất phương án tối ưu tiền gửi kỳ hạn.
- **Chỉ số kiểm chứng trên màn hình**:
  - Dòng tiền tuần qua: Thu $+3.28$ tỷ VNĐ, Chi $-628.8$ triệu VNĐ.
  - Dự báo nghĩa vụ thanh toán ngày 25/08: $450$ triệu VNĐ.
  - Khuyến nghị Rolling Cashflow Sentinel: Chuyển $1.2$ tỷ VNĐ sang tiền gửi kỳ hạn 1 tháng tối ưu lợi suất.

---

### BƯỚC 5: 04:30 – 05:00 (30 Giây) | CHỦ QUYỀN AI, NGHỊ ĐỊNH 13 & LỜI KÊU GỌI ĐẦU TƯ

- **Màn hình hiển thị**: Slide 5, Chứng nhận Sổ cái HMAC-SHA256 và Lời kêu gọi đầu tư ($500k-$750k).
- **Hành động sân khấu**:
  - `04:30`: Bấm clicker chuyển **Slide 5**. Giọng hào hùng, truyền cảm hứng: *"Tương lai tài chính thuộc về các nền tảng AI Bản địa, Tự chủ và Bảo mật tuyệt đối!"*
  - `04:45`: Công bố The Ask: Kêu gọi vốn hạt giống $500,000 – $750,000 cho 12% - 15% cổ phần (Runway 18-24 tháng).
  - `04:55`: Mở rộng hai tay, nụ cười tự tin: *"Hãy cùng LIVA thiết lập chuẩn mực mới cho tự động hóa ngân quỹ: Chính xác hơn, Tức thì hơn, và Tuyệt đối An toàn. Xin trân trọng cảm ơn!"*
  - `05:00`: Dừng đúng nhịp, cúi đầu chào Ban Giám khảo.
- **Chỉ số kiểm chứng trên màn hình**:
  - Chuỗi khối kiểm toán: Genesis Hash `00000000...`, Hash khối mới nhất hợp lệ 100%.
  - PII Masking: CCCD `001095012345` -> `00109***`, TK `0011001234567` -> `***34567`.
  - Zero Cloud Egress: 0 byte outbound, 100% On-Premise.

---

## 5. TOÀN VĂN LỜI THOẠI THUYẾT TRÌNH CHUẨN 688 TỪ (CANONICAL SPEAKER SCRIPT)

### PHÚT 1 (00:00 - 01:00) | VẤN ĐỀ & CÚ MÓC (139 TỪ)
`[Chuyển Slide 1: Điểm nghẽn Đối soát & Bức tường sắt Pháp lý]`  
`[Giọng trầm ấm, kết nối, ánh mắt quét bao quát toàn bộ Hội đồng Giám khảo]`  
Kính thưa Ban giám khảo và các Nhà đầu tư,  
Mỗi sáng, hàng chục nghìn kế toán trưởng đối mặt áp lực lớn: mở hàng chục file sao kê ngân hàng, căng mắt dò từng dòng đối soát sổ cái.  
`[Nghỉ 1.5s]`  
Quá trình thủ công này ngốn **bốn tiếng mỗi ngày**, đẩy doanh nghiệp vào **vùng mù dòng tiền T+1 đến T+3**, tiềm ẩn rủi ro thâm hụt thanh khoản và phạt thấu chi.  
`[Dừng lại nửa nhịp, giọng chuyển sang đanh thép, dứt khoát]`  
Tại sao họ chưa dùng AI?  
Bởi vì theo **Nghị định 13** và **Thông tư 09**, đưa dữ liệu sao kê nhạy cảm lên Cloud AI là hành vi **vi phạm pháp luật**, đối mặt mức phạt tới 5% doanh thu!  
`[Nghỉ 1.5s]`  
`[Giọng dồn nén cảm xúc, đanh thép]`  
Ngành ngân hàng đứng trước nghịch lý: Khao khát tự động hóa, nhưng cánh cửa lên Cloud AI đã bị **khóa chặt**.

---

### PHÚT 2 (01:00 - 02:00) | GIẢI PHÁP & TRÌNH DIỄN THỰC TẾ (137 TỪ)
`[Chuyển Slide 2: LIVA Banking Harness & Luồng Đối Soát Tức Thì]`  
`[Giọng tự hào, hứng khởi, mở rộng hai bàn tay]`  
Đó là lý do chúng tôi kiến tạo **LIVA Banking Harness** — Trợ lý Agentic AI vận hành **hoàn toàn cục bộ** trên máy trạm.  
`[Nghỉ 1.5s]`  
`[Hướng tay về màn hình hiển thị video minh họa luồng xử lý]`  
Kính mời quý vị theo dõi luồng demo:  
Kế toán chỉ cần nạp sao kê đa định dạng — từ Excel, CSV, đến OFX hay PDF.  
Chỉ trong **vài phần trăm giây**, lõi LIVA bóc tách và chuẩn hóa toàn bộ giao dịch.  
Hệ thống tự động **đối soát ba chiều** giữa sao kê, hóa đơn và sổ cái với độ chính xác **99.8%**.  
Đồng thời, LIVA là tháp canh ngân quỹ 24/7: phát hiện giao dịch trùng lặp và cảnh báo thâm hụt thanh khoản trước **24 đến 48 giờ**.  
`[Nhấn từng chữ dứt khoát]`  
Quan trọng nhất: **Zero Cloud Leakage — Không một byte dữ liệu nào rời khỏi thiết bị!**

---

### PHÚT 3 (02:00 - 03:00) | LÕI CÔNG NGHỆ & HÀO LŨY PHÒNG THỦ (135 TỪ)
`[Chuyển Slide 3: Kiến Trúc Rust Native Core & Hào Lũy Công Nghệ]`  
`[Giọng đanh thép, bản lĩnh, nhìn thẳng vào các giám khảo khối công nghệ]`  
Tại sao RPA hay Cloud SaaS không giải quyết được?  
RPA quá cứng nhắc — đổi mẫu sao kê là gãy bot. Còn Cloud SaaS vừa rò rỉ dữ liệu, vừa mắc điểm yếu chí mạng: **ảo giác số học**.  
`[Nghỉ 1.5s]`  
Hào lũy của LIVA nằm ở **Rust Native Core**:  
Thứ nhất, tối ưu tài nguyên: Chiếm dưới **4GB RAM**, chạy mượt trên laptop văn phòng, không cần GPU đắt đỏ.  
Thứ hai, triệt tiêu ảo giác: AI chỉ trích xuất ngữ nghĩa, còn phép tính đối soát được khóa chặt bởi **bất biến số học trong Rust** — bảo đảm cân bằng kế toán tuyệt đối.  
Thứ ba, an ninh cấp ngân hàng: Mã hóa **AES-256-GCM**, cơ chế **Phê duyệt Hai pha** và tự động che mờ CCCD, tài khoản thời gian thực.

---

### PHÚT 4 (03:00 - 04:00) | MÔ HÌNH KINH DOANH & THỰC CHỨNG (134 TỪ)
`[Chuyển Slide 4: Chiến Lược B2B Hai Mũi Nhọn & Báo Cáo Thực Chứng]`  
`[Giọng mạch lạc, tự tin, nhấn mạnh các số liệu kinh tế]`  
Về thương mại hóa, LIVA triển khai chiến lược **B2B hai mũi nhọn**:  
Thứ nhất: Thuê bao trực tiếp cho CFO từ **300 đến 1.000 USD mỗi tháng**, chu kỳ chốt hai đến bốn tuần, tạo dòng tiền định kỳ tức thì.  
Thứ hai: Cấp phép Enterprise cho Ngân hàng thương mại từ **50.000 đến 150.000 USD mỗi năm**, giúp ngân hàng thu hút tiền gửi CASA từ doanh nghiệp.  
Nhờ chạy cục bộ, chi phí máy chủ biên bằng **Không**, mang lại biên lợi nhuận gộp trên **88%**.  
`[Nghỉ 1.5s]`  
`[Giọng trang trọng, khẳng định sự trung thực tuyệt đối]`  
Về thực chứng: Trong môi trường kiểm thử, LIVA đã xử lý **50.000 dòng sao kê** với độ trễ dưới **0.5 mili-giây**, đạt độ chính xác **99.8%**, được chuyên gia tài chính đánh giá rất cao.

---

### PHÚT 5 (04:00 - 05:00) | LỜI KÊU GỌI & TẦM NHÌN CHỦ QUYỀN (143 TỪ)
`[Chuyển Slide 5: Kêu Gọi Đầu Tư & Tầm Nhìn Hệ Điều Hành Tác Tử]`  
`[Giọng truyền cảm hứng, ánh mắt nhiệt huyết hướng về phía các quỹ đầu tư]`  
Kính thưa quý vị,  
Tương lai tài chính không thể phó mặc dữ liệu nhạy cảm cho đám mây ngoại quốc. Tương lai thuộc về các nền tảng **AI Bản địa, Tự chủ và Bảo mật tuyệt đối**.  
Hôm nay, tại INNOSTART 2026, LIVA mở vòng gọi vốn hạt giống từ **500.000 đến 750.000 USD** cho **12% đến 15% cổ phần**:  
50% cho R&D lõi Rust, 25% mở rộng đối tác ERP, 15% tuân thủ Sandbox NHNN, và 10% quỹ dự phòng 18-24 tháng runway.  
Đồng thời, chúng tôi tìm kiếm **hai ngân hàng tiên phong** để cùng triển khai giải pháp quản trị thanh khoản.  
`[Nghỉ 1.5s]`  
`[Mở rộng hai tay, nụ cười tự tin, kết thúc với khí thế mạnh mẽ]`  
Hãy cùng LIVA thiết lập chuẩn mực mới cho tự động hóa ngân quỹ: **Chính xác hơn, Tức thì hơn, và Tuyệt đối An toàn**.  
Xin trân trọng cảm ơn!

---

## 6. BẢNG PHÂN BỔ DUNG LƯỢNG & NHỊP ĐIỆU NÓI (CADENCE SUMMARY)

| Phút Thuyết Trình | Khung Thời Gian | Chủ Đề Then Chốt | Slide | Số Từ Nói | Tốc Độ Nói | Đánh Giá Nhịp Điệu |
| :---: | :---: | :--- | :---: | :---: | :---: | :--- |
| **Phút 1** | 00:00 - 01:00 | Vấn đề & Rào cản Nghị định 13 / TT 09 | Slide 1 | **139 từ** | 139 wpm | Chuẩn mực, tạo khoảng lặng ấn tượng |
| **Phút 2** | 01:00 - 02:00 | Giải pháp LIVA & Luồng Live Demo sao kê | Slide 2 | **137 từ** | 137 wpm | Hào hứng, trực quan sinh động |
| **Phút 3** | 02:00 - 03:00 | Lõi Rust Native, 0% Ảo giác & AES-256 | Slide 3 | **135 từ** | 135 wpm | Đanh thép, nhấn mạnh hào lũy kỹ thuật |
| **Phút 4** | 03:00 - 04:00 | Mô hình B2B Dual-Track & Thực chứng 50k | Slide 4 | **134 từ** | 134 wpm | Rõ ràng, minh bạch tài chính |
| **Phút 5** | 04:00 - 05:00 | Vòng gọi vốn $500k-$750k & Tầm nhìn AI | Slide 5 | **143 từ** | 143 wpm | Truyền cảm hứng, kết thúc đĩnh đạc |
| **TOÀN BÀI** | **00:00 - 05:00** | **Trọn vẹn 5 Phút Thuyết Trình LIVA** | **5 Slides**| **688 từ** | **137.6 wpm** | **HOÀN HẢO (Chuẩn 650–750 từ)** |

---

## 7. SỔ TAY PHẢN BIỆN CHUYÊN SÂU 07 PHÚT (Q&A DEFENSE CHEAT-SHEET)

### Câu hỏi 1 (Giám khảo Công nghệ): *"Tại sao các bạn khẳng định AI không bao giờ tính toán sai số tiền đối soát?"*
- **Trả lời phản biện (45 giây)**:  
  *"Thưa Giám khảo, điểm cốt tử của LIVA là tách rời hoàn toàn: AI chỉ làm nhiệm vụ trích xuất ngữ nghĩa (tên đối tác, mã hợp đồng từ văn bản tự do). Toàn bộ phép tính đối chiếu và cân bằng kế toán được thực thi bởi Động cơ Ràng buộc viết bằng Rust với kiểu dữ liệu số nguyên không dấu (`u64`) và kiểm tra bất biến đại số nghiêm ngặt ($\Delta = 0$). Hệ thống tuyệt đối không dùng số thực `f64` để tránh sai số dấu phẩy động và không bao giờ để mô hình ngôn ngữ tự cộng trừ tiền. Nếu sai lệch dù chỉ 1 đồng, giao dịch lập tức rơi vào cơ chế Fail-Closed và chuyển sang hàng đợi duyệt bằng tay."*

### Câu hỏi 2 (Giám khảo Ngân hàng / Pháp lý): *"Nếu doanh nghiệp cài LIVA, làm sao chứng minh dữ liệu không bị gửi lén về máy chủ của các bạn?"*
- **Trả lời phản biện (45 giây)**:  
  *"Thưa Giám khảo, LIVA được thiết kế theo triết lý 'Air-Gapped by Design'. Lõi Rust Native Core của chúng tôi kích hoạt chính sách Zero Network Egress: chặn mọi outbound network socket và chỉ mở loopback `127.0.0.1` để giao tiếp nội bộ giữa giao diện và backend. Mọi thao tác đều được ghi nhận vào Sổ cái Kiểm toán tiến tiếp bảo vệ bởi HMAC-SHA256 và khóa phần cứng TPM 2.0 / Windows DPAPI. Khi kiểm toán CNTT theo Thông tư 09 hoặc Nghị định 13, doanh nghiệp có thể ngắt hoàn toàn kết nối Internet mà hệ thống vẫn đối soát trơn tru 100%."*

### Câu hỏi 3 (Giám khảo Quỹ Đầu tư): *"RPA như UiPath đã làm đối soát từ lâu, tại sao khách hàng phải mua LIVA?"*
- **Trả lời phản biện (45 giây)**:  
  *"Thưa Giám khảo, RPA là các con bot cứng nhắc chạy theo tọa độ màn hình (screen scraping). Chỉ cần ngân hàng đổi font chữ, thêm một cột trong Excel hay đổi bố cục giao diện web là bot RPA sẽ gãy vụn và tốn hàng nghìn USD bảo trì. LIVA không phải RPA — chúng tôi là Hệ thống Tác tử Cục bộ có khả năng đọc hiểu cấu trúc linh hoạt của 35+ ngân hàng tại Việt Nam, xử lý được cả file gộp ô, nội dung viết tắt không dấu hay chuyển khoản thiếu tiền phí. Chi phí triển khai của LIVA chỉ mất 15 phút so với 6 tháng của RPA."*

### Câu hỏi 4 (Giám khảo Công nghệ / Kiến trúc): *"Bản web demo này chỉ cần client truy cập URL là chạy được đúng không? Dữ liệu sao kê khách hàng kéo thả vào có bị gửi lên máy chủ đám mây không?"*
- **Trả lời phản biện (45 giây)**:  
  *"Thưa Giám khảo, hoàn toàn chính xác. Bản Universal Web Demo của LIVA được thiết kế theo nguyên lý **Zero-Backend Client-Side Execution**: Toàn bộ thuật toán bóc tách bảng tính (SheetJS), động cơ đối soát 3 tầng và quét cảnh báo rửa tiền AML đều đã được tối ưu chạy **100% trong bộ nhớ RAM trình duyệt của máy trạm người dùng**. Khi khách hàng kéo thả file sao kê, không có bất kỳ byte dữ liệu nào bị truyền ra Internet. Khi đóng tab, RAM tự động giải phóng sạch sẽ, bảo đảm tuân thủ tuyệt đối Nghị định 13/2023/NĐ-CP."*

### Câu hỏi 5 (Giám khảo An ninh Thông tin / Pháp lý): *"Làm thế nào để doanh nghiệp giới hạn chỉ nhân viên ngồi trong văn phòng hoặc kết nối đúng mạng nội bộ công ty mới truy cập được trang web này?"*
- **Trả lời phản biện (45 giây)**:  
  *"Thưa Giám khảo, hệ thống hỗ trợ cơ chế **Mạng Nội bộ Cách ly (Intranet / Air-Gapped Topology)** với 3 tầng kiểm soát:  
  1. **Triển khai LAN thuần túy**: Web server lắng nghe trên dải IP nội bộ Private RFC 1918 (`192.168.x.x`), đóng toàn bộ Port-Forwarding trên router văn phòng. Nhân viên bắt Wi-Fi hoặc cắm dây mạng công ty là truy cập được ngay; người ngoài Internet hoàn toàn không thể dò thấy hệ thống.  
  2. **Mesh VPN (Tailscale/WireGuard)**: Dành cho kịch bản nhân viên làm việc từ xa (WFH) kết nối qua đường hầm mã hóa điểm-điểm có xác thực 2FA.  
  3. **Tường lửa IP Whitelist**: Nếu đặt trên Cloud, hệ thống chỉ cho phép duy nhất IP Public tĩnh của văn phòng công ty truy cập. Quy trình thiết lập chi tiết đã được chúng tôi chuẩn hóa tại cẩm nang `docs/02-van-hanh/07-trien-khai-web-client-va-mang-noi-bo.md`."*

---

## 8. QUY TRÌNH XỬ LÝ SỰ CỐ KHẨN CẤP (DISASTER RECOVERY PROTOCOLS)

| Tình Huống Khẩn Cấp | Phản Ứng Tức Thì Của Diễn Giả | Phản Ứng Trợ Lý Kỹ Thuật (Co-pilot) |
|:---|:---|:---|
| **1. Máy chiếu tắt ngóm / Mất tín hiệu** | Tuyệt đối không dừng lại, không nhìn lên trần nhà; mỉm cười tự tin bước lên nửa bước, nói tiếp theo kịch bản thuộc lòng. Lời thoại 688 từ được thiết kế độc lập, đủ sức thuyết phục mà không cần slide. | Nhẹ nhàng kiểm tra đầu nối cáp HDMI / Type-C mà không làm xao nhãng sân khấu. |
| **2. Bị giơ biển báo 1 phút khi mới bắt đầu Phút 4** | Không hoảng loạn nói nhanh; gộp Phút 4 và 5: Tóm tắt mô hình B2B trong 1 câu ("Biên lợi nhuận gộp trên 88% từ mô hình kép..."), chuyển thẳng sang Ask $500k-$750k và kết thúc bằng thông điệp Chủ quyền AI. | Theo dõi đồng hồ và chuẩn bị chuyển slide sang Slide 5. |
| **3. Popup thông báo Windows che khuất màn hình** | Tiếp tục nói mạch lạc về tính năng sản phẩm. | Nhấn phím tắt nhanh `Alt + Tab` hoặc `F11` đưa cửa sổ Dashboard toàn màn hình trở lại. (Trước giờ diễn: bật chế độ Do Not Disturb). |
| **4. Chuông hết giờ reo sớm hơn dự kiến** | Dừng nói ngay sau câu hiện tại. Mỉm cười cúi chào: "Xin trân trọng cảm ơn, chúng tôi sẵn sàng cho phần hỏi đáp phản biện." | Lưu lại các màn hình demo để mở nhanh trong phần hỏi đáp Q&A 07 phút. |

---

*LIVA Banking Harness — Tự Chủ Công Nghệ • Chuẩn Mực Ngân Hàng • Bảo Mật Tuyệt Đối.*

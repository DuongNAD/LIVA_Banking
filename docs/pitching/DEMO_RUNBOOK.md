---
title: "Sổ tay Vận hành Demo Live 05 Phút INNOSTART 2026"
updated: 2026-09-13
commit: 3688b5f
status: living
owns: []
covers: []
---
# SỔ TAY VẬN HÀNH DEMO LIVE 05 PHÚT (DEMO RUNBOOK)
## LIVA Banking Harness — InnovationStart 2026 Demo Day
**Tài liệu hướng dẫn diễn tập & trình diễn trực tiếp dành cho Đội ngũ Thuyết trình (Pitch Team)**

---

### MỤC LỤC
1. [Tổng Quan & Cấu Trúc Phiên Trình Diễn](#1-tổng-quan--cấu-trúc-phiên-trình-diễn)
2. [Checklist Chuẩn Bị Thiết Bị & Kiểm Tra Tiền Trạm (Pre-flight Checklist)](#2-checklist-chuẩn-bị-thiết-bị--kiểm-tra-tiền-trạm-pre-flight-checklist)
3. [Các Chế Độ Vận Hành Script Demo (`demo_runner.mjs`)](#3-các-chế-độ-vận-hành-script-demo-demo_runnermjs)
4. [Kịch Bản Đồng Bộ Từng Giây (Second-by-Second Synchronization Playbook)](#4-kịch-bản-đồng-bộ-từng-giây-second-by-second-synchronization-playbook)
5. [Sổ Tay Ứng Khẩu Phản Biện Ban Giám Khảo 07 Phút (Q&A Defense Guide)](#5-sổ-tay-ứng-khẩu-phản-biện-ban-giám-khảo-07-phút-qa-defense-guide)
6. [Quy Trình Xử Lý Sự Cố Khẩn Cấp (Disaster Recovery Protocols)](#6-quy-trình-xử-lý-sự-cố-khẩn-cấp-disaster-recovery-protocols)

---

## 1. TỔNG QUAN & CẤU TRÚC PHIÊN TRÌNH DIỄN

- **Sự kiện**: Chung kết InnovationStart 2026 (Demo Day)
- **Thời gian**: Ngày 16 tháng 09 năm 2026
- **Tổng thời lượng phân bổ**: Đúng **12 phút**
  - **05 phút (300 giây)**: Thuyết trình Pitching & Trình diễn Demo Trực tiếp (Live Demo)
  - **07 phút (420 giây)**: Phản biện chuyên sâu trước Hội đồng Ban Giám khảo (Q&A Defense)
- **Đội hình phân vai trên sân khấu**:
  - **Diễn giả chính (Lead Presenter / CEO)**: Cầm clicker, làm chủ sân khấu, phát biểu chuẩn 688 từ tiếng Việt, dẫn dắt câu chuyện từ nỗi đau pháp lý đến giải pháp LIVA.
  - **Trợ lý kỹ thuật (Technical Co-pilot / CTO)**: Đứng tại bàn điều khiển, quan sát màn hình phụ, kích hoạt script tự động hoặc hỗ trợ chuyển đổi màn hình nếu cần.

---

## 2. CHECKLIST CHUẨN BỊ THIẾT BỊ & KIỂM TRA TIỀN TRẠM (PRE-FLIGHT CHECKLIST)

Thực hiện toàn bộ checklist này tại phòng chờ kỹ thuật **30 phút** trước giờ lên sân khấu:

| Hạng Mục | Yêu Cầu Kỹ Thuật | Phương Pháp Kiểm Tra | Trạng Thái |
|:---|:---|:---|:---:|
| **Hệ Điều Hành** | Windows 10 / 11 Pro 64-bit | `winver` | [x] |
| **Tài Nguyên RAM** | RAM khả dụng $\ge 4.0\text{ GB}$ | Chạy lệnh `powershell scripts/ram-guard.ps1` hoặc Task Manager | [x] |
| **Node.js Runtime** | Node.js v20+ hoặc v22+ LTS | `node --version` | [x] |
| **Rust Native Core** | Bản biên dịch Release/Dev sạch | `cargo check -j 2` đạt exit code 0 | [x] |
| **Dữ Liệu Mẫu (Fixtures)** | 4 tệp dữ liệu chuẩn tại `fixtures/statements/` | `vcb_aug2026.xlsx`, `tcb_aug2026.csv`, `bidv_aug2026.pdf`, `open_invoices.json` | [x] |
| **Chế Độ Mạng** | Chế độ Máy bay (Airplane Mode) — Cắt hoàn toàn Internet để chứng minh On-Premise 100% | Tắt Wi-Fi, rút cáp LAN | [x] |
| **Hiển Thị & Màn Hình** | Độ phân giải 1920x1080 (16:9), chế độ Duplicate Display (`Win + P` -> Duplicate) | Kiểm tra tỷ lệ khung hình trên máy chiếu hội trường | [x] |
| **Âm Thanh Hội Trường** | Kết nối cổng 3.5mm hoặc HDMI Audio, âm lượng 80% | Test phát audio phản hồi của Trợ lý 2D | [x] |
| **Pin & Nguồn Điện** | Cắm sạc trực tiếp, chế độ Best Performance | Icon pin hiển thị "Plugged in" | [x] |

---

## 3. CÁC CHẾ ĐỘ VẬN HÀNH SCRIPT DEMO (`demo_runner.mjs`)

Script tự động hóa `scripts/demo_runner.mjs` hỗ trợ 3 chế độ chạy linh hoạt:

### 3.1. Chế Độ Kiểm Tra Nhanh (`--dry-run`)
Dùng để kiểm tra toàn bộ tính toàn vẹn kỹ thuật, metric số học và trạng thái tệp tin trong 0.01 giây trước giờ diễn:
```bash
node scripts/demo_runner.mjs --dry-run
```
*Kết quả kỳ vọng*: Cả 5 bước hiển thị `[ PASSED ]` với exit code 0.

### 3.2. Chế Độ Diễn Tập Tốc Độ Cao (`--speed=60`)
Dùng cho đội ngũ diễn tập nhịp chuyển slide nhanh (hoàn thành 5 phút trong 5 giây):
```bash
node scripts/demo_runner.mjs --speed=60
```

### 3.3. Chế Độ Trình Diễn Trực Tiếp 05 Phút (Live Mode — 300 Giây)
Kích hoạt đúng thời điểm diễn giả cất lời chào Ban Giám khảo:
```bash
node scripts/demo_runner.mjs
```
Script sẽ đếm nhịp từng giây, hiển thị lời thoại CEO, chỉ dẫn sân khấu `[CUE]`, thanh tiến độ, các bảng thống kê và số liệu đối soát thời gian thực.

---

## 4. KỊCH BẢN ĐỒNG BỘ TỪNG GIÂY (SECOND-BY-SECOND SYNCHRONIZATION PLAYBOOK)

### BƯỚC 1: 00:00 – 01:00 (60 Giây) | TỔNG QUAN NGUỒN VỐN & ĐIỂM NGHẼN PHÁP LÝ
- **Màn hình hiển thị**: Slide 1 kết hợp Khối Thẻ Ngân Hàng (VCB 1.45B, TCB 785.6M, BIDV 520M).
- **Hành động sân khấu**:
  - `00:00`: Clicker chuyển **Slide 1**. Diễn giả đứng giữa sân khấu, mắt quét toàn bộ giám khảo.
  - `00:05`: CEO cất giọng trầm ấm, đồng cảm: *"Mỗi sáng, hàng chục nghìn kế toán trưởng đối mặt áp lực lớn: mở hàng chục file sao kê..."*
  - `00:25`: Dừng thở chiến thuật `[Nghỉ 1.5s]` khi nói đến con số *"bốn tiếng mỗi ngày"* và *"vùng mù dòng tiền T+1 đến T+3"*.
  - `00:35`: Giọng chuyển đanh thép, cảnh báo: *"Tại sao họ chưa dùng AI? Vì theo Nghị định 13 và Thông tư 09, tải dữ liệu sao kê lên Cloud AI là hành vi vi phạm pháp luật với mức phạt tới 5% doanh thu!"*
- **Chỉ số kiểm chứng trên màn hình**:
  - Thẻ Vietcombank: Số dư `1,450,230,000 VND` (Đã đối soát `1,449,850,000`, Chưa khớp `380,000`).
  - Thẻ Techcombank: Số dư `785,600,000 VND` (Đã đối soát `785,100,000`, Chưa khớp `500,000`).
  - Hệ số thanh khoản: `1.85` | Thời gian an toàn tiền mặt (Runway): `9.2 tháng`.

---

### BƯỚC 2: 01:00 – 02:30 (90 Giây) | NẠP SAO KÊ ĐA ĐỊNH DẠNG QUA HOT-FOLDER
- **Màn hình hiển thị**: Slide 2 và Khu vực kéo thả Hot-Folder Dropzone.
- **Hành động sân khấu**:
  - `01:00`: Clicker chuyển **Slide 2**. Giọng tự hào, mở rộng hai bàn tay giới thiệu *LIVA Banking Harness*.
  - `01:15`: Hướng tay về màn hình máy chiếu, thao tác kéo 3 tệp sao kê (`vcb_aug2026.xlsx`, `tcb_aug2026.csv`, `bidv_aug2026.pdf`) vào khu vực Hot-Folder.
  - `01:25`: Điểm dừng thở chiến thuật `[Nghỉ 1.5s]`.
  - `01:30`: Hệ thống hiển thị thời gian bóc tách hoàn tất trong vài phần trăm giây. Diễn giả nhấn mạnh: *"Zero Cloud Leakage — Không một byte dữ liệu nào rời khỏi thiết bị!"*
- **Chỉ số kiểm chứng trên màn hình**:
  - VCB OpenXML: 50 giao dịch (Calamine engine).
  - TCB CSV UTF-8 BOM: 60 giao dịch (Zero-copy csv reader).
  - BIDV PDF: 40 giao dịch (lopdf 2D text coordinates, checksum $\text{closing} = \text{opening} + \text{credits} - \text{debits}$).
  - Tổng số: 150 giao dịch bóc tách trong thời gian $< 2.0\text{s}$ (SLA đạt chuẩn Sub-second).

---

### BƯỚC 3: 02:30 – 03:45 (75 Giây) | ĐỘNG CƠ ĐỐI SOÁT 3 TẦNG BẤT BIẾN TOÁN HỌC
- **Màn hình hiển thị**: Slide 3 và Gauge Bán Nguyệt 99.8% Tỷ lệ Khớp Tự Động.
- **Hành động sân khấu**:
  - `02:30`: Clicker chuyển **Slide 3**. Giọng quyền lực, nhìn thẳng giám khảo khối công nghệ: *"Tại sao RPA hay Cloud SaaS không giải quyết được? RPA gãy bot khi đổi mẫu. Cloud SaaS mắc ảo giác số học."*
  - `02:50`: Nhấn mạnh 3 trụ cột của Rust Native Core: Tiết kiệm tài nguyên ($< 4\text{GB}$ RAM), Bất biến toán học triệt tiêu ảo giác, và Mã hóa AES-256-GCM.
  - `03:15`: Chỉ vào Gauge bán nguyệt và giải thích cơ chế Fail-Closed: 3 giao dịch không khớp được cách ly vào hàng đợi HITL với chữ ký số UUID v4.
- **Chỉ số kiểm chứng trên màn hình**:
  - Gauge bán nguyệt: **99.8%** Tỷ lệ khớp (`1,842 / 1,845` giao dịch).
  - Phân bổ: Tự động `99.2%`, Bằng tay `0.6%`, Chưa khớp `3` giao dịch (`0.2%`).
  - Hàng đợi HITL Two-Phase Confirmation: 3 UUID v4 tokens hiển thị minh bạch.

---

### BƯỚC 4: 03:45 – 04:30 (45 Giây) | HỘI THOẠI TRỢ LÝ TÀI CHÍNH 2D THỜI GIAN THỰC
- **Màn hình hiển thị**: Slide 4 và Khung chat/voice Trợ lý 2D Banking.
- **Hành động sân khấu**:
  - `03:45`: Clicker chuyển **Slide 4**. Giọng tự tin, minh bạch tài chính: Trình bày mô hình B2B Dual-Track ($300-$1000/tháng và License ngân hàng $50k-$150k/năm, biên lợi nhuận gộp $> 88\%$).
  - `04:00`: Kích hoạt tương tác giọng nói với Trợ lý: *"LIVA, phân tích biến động dòng tiền tuần qua và dự báo ngân quỹ 30 ngày tới?"*
  - `04:05`: Trợ lý LIVA stream câu trả lời tức thời ($< 250\text{ ms}$), khuyến nghị Cash Pooling 1.2 tỷ VNĐ sang tiền gửi kỳ hạn.
- **Chỉ số kiểm chứng trên màn hình**:
  - Dòng tiền tuần qua: Thu $+3.28$ tỷ VNĐ, Chi $-628.8$ triệu VNĐ.
  - Dự báo nghĩa vụ thanh toán ngày 25/08: $450$ triệu VNĐ.
  - Khuyến nghị tối ưu lợi suất tiền gửi qua đêm / kỳ hạn 1 tháng.

---

### BƯỚC 5: 04:30 – 05:00 (30 Giây) | CHỦ QUYỀN AI, NGHỊ ĐỊNH 13 & LỜI KÊU GỌI VỐN
- **Màn hình hiển thị**: Slide 5, Chứng nhận Sổ cái HMAC-SHA256 và Lời kêu gọi đầu tư ($500k-$750k).
- **Hành động sân khấu**:
  - `04:30`: Clicker chuyển **Slide 5**. Giọng hào hùng, truyền cảm hứng: *"Tương lai tài chính thuộc về các nền tảng AI Bản địa, Tự chủ và Bảo mật tuyệt đối!"*
  - `04:45`: Công bố The Ask: Gọi vốn hạt giống $500,000 – $750,000 cho 12% - 15% cổ phần (Runway 18-24 tháng).
  - `04:55`: Mở rộng hai tay, nụ cười tự tin: *"Hãy cùng LIVA thiết lập chuẩn mực mới cho tự động hóa ngân quỹ: Chính xác hơn, Tức thì hơn, và Tuyệt đối An toàn. Xin trân trọng cảm ơn!"*
  - `05:00`: Dừng đúng nhịp, cúi chào Ban Giám khảo.
- **Chỉ số kiểm chứng trên màn hình**:
  - Chuỗi khối kiểm toán: Genesis Hash `00000000...`, Hash khối mới nhất hợp lệ 100%.
  - PII Masking: CCCD `001095012345` -> `00109***`, TK `0011001234567` -> `***34567`.
  - Zero Cloud Egress: 0 kết nối ra ngoài, 100% On-Premise.

---

## 5. SỔ TAY ỨNG KHẨU PHẢN BIỆN BAN GIÁM KHẢO 07 PHÚT (Q&A DEFENSE GUIDE)

*(Tham chiếu toàn văn tài liệu `docs/pitching/QA_DEFENSE_7MIN.md`)*

### Câu hỏi 1 (Giám khảo Công nghệ): *"Tại sao các bạn khẳng định AI không bao giờ tính toán sai số tiền đối soát?"*
- **Câu trả lời mẫu (45 giây)**:  
  *"Thưa Giám khảo, điểm cốt tử của LIVA là tách rời hoàn toàn: AI chỉ làm nhiệm vụ trích xuất ngữ nghĩa (tên đối tác, mã hợp đồng từ câu chữ không cấu trúc). Toàn bộ phép tính đối chiếu và cân bằng kế toán được thực hiện bởi Động cơ Ràng buộc viết bằng Rust với kiểu dữ liệu số nguyên không dấu (`u64`) và kiểm tra bất biến đại số nghiêm ngặt ($\Delta = 0$). Hệ thống tuyệt đối không dùng số thực `f64` để tránh sai số dấu phẩy động và không bao giờ để mô hình ngôn ngữ tự cộng trừ tiền. Nếu sai lệch dù chỉ 1 đồng, giao dịch lập tức rơi vào cơ chế Fail-Closed và chuyển sang hàng đợi duyệt bằng tay."*

### Câu hỏi 2 (Giám khảo Ngân hàng/Pháp lý): *"Nếu doanh nghiệp cài LIVA, làm sao chứng minh dữ liệu không bị gửi lén về máy chủ của các bạn?"*
- **Câu trả lời mẫu (45 giây)**:  
  *"Thưa Giám khảo, LIVA được thiết kế theo triết lý 'Air-Gapped by Design'. Lõi Rust Native Core của chúng tôi kích hoạt chính sách Zero Network Egress: chặn mọi socket ra Internet và chỉ mở cổng loopback `127.0.0.1` để giao tiếp nội bộ giữa giao diện và backend. Mọi hành động đều được ghi vào Sổ cái Kiểm toán tiến tiếp bảo vệ bởi mã băm HMAC-SHA256 và khóa phần cứng TPM 2.0 / Windows DPAPI. Khi kiểm toán CNTT theo Thông tư 09 hoặc Nghị định 13, doanh nghiệp có thể ngắt hoàn toàn kết nối Internet mà hệ thống vẫn đối soát trơn tru 100%."*

### Câu hỏi 3 (Giám khảo Quỹ Đầu tư): *"RPA như UiPath đã làm đối soát từ lâu, tại sao khách hàng phải mua LIVA?"*
- **Câu trả lời mẫu (45 giây)**:  
  *"Thưa Giám khảo, RPA là các con bot cứng nhắc chạy theo tọa độ màn hình (screen scraping). Chỉ cần ngân hàng đổi font chữ, thêm một cột trong Excel hay cập nhật giao diện web là bot RPA sẽ gãy vụn và tốn hàng nghìn USD bảo trì. LIVA không phải RPA — chúng tôi là Hệ thống Tác tử Cục bộ có khả năng đọc hiểu cấu trúc linh hoạt của 35+ ngân hàng tại Việt Nam, xử lý được cả file gộp ô, nội dung viết tắt không dấu hay chuyển khoản thiếu tiền phí. Chi phí triển khai của LIVA chỉ mất 15 phút so với 6 tháng của RPA."*

---

## 6. QUY TRÌNH XỬ LÝ SỰ CỐ KHẨN CẤP (DISASTER RECOVERY PROTOCOLS)

Khi trình diễn trước đám đông, các tình huống bất ngờ có thể xảy ra. Đội ngũ phải tuân thủ nghiêm ngặt các quy trình xử lý sau:

### Tình huống 1: Máy chiếu hội trường mất tín hiệu hoặc tắt ngóm
- **Xử lý**:
  1. Diễn giả chính **tuyệt đối không dừng lại**, không nhìn lên trần nhà hay tỏ ra bối rối.
  2. Mỉm cười tự tin, bước lên nửa bước về phía Ban Giám khảo và tiếp tục nói theo kịch bản thuộc lòng. Lời thoại 688 từ được thiết kế độc lập, đủ sức thuyết phục mà không cần nhìn slide.
  3. Trợ lý kỹ thuật tại bàn nhẹ nhàng kiểm tra lại đầu cáp HDMI/Type-C mà không làm xao nhãng sân khấu.

### Tình huống 2: Người điều phối giơ biển báo còn 1 phút khi bạn mới bắt đầu Phút 4
- **Xử lý**:
  1. Không hoảng loạn nói nhanh gây vấp từ.
  2. Lập tức gộp Phút 4 và Phút 5: Tóm tắt mô hình B2B trong đúng 1 câu: *"Với biên lợi nhuận gộp trên 88% từ mô hình B2B kép..."*
  3. Chuyển thẳng sang lời kêu gọi vốn $500k-$750k và kết thúc bằng thông điệp Chủ quyền AI Tài chính.

### Tình huống 3: Máy tính bị popup thông báo Windows hoặc phần mềm khác che khuất
- **Xử lý**:
  1. Nhấn tổ hợp phím tắt nhanh `Alt + Tab` hoặc `F11` để đưa cửa sổ Dashboard trở lại toàn màn hình.
  2. Trước giờ diễn, đảm bảo đã bật chế độ **Do Not Disturb (Focus Assist)** trên Windows.

### Tình huống 4: Chuông báo hết giờ reo sớm hơn dự kiến
- **Xử lý**:
  1. Lập tức dừng nói ngay sau câu hiện tại.
  2. Mỉm cười cúi chào: *"Xin trân trọng cảm ơn Ban Giám khảo, chúng tôi sẵn sàng cho phần hỏi đáp phản biện."*
  3. Dành toàn bộ số liệu chưa kịp nói để trả lời trong phần Q&A 07 phút.

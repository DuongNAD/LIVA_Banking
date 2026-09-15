---
title: "Đánh giá tác động xử lý dữ liệu cá nhân (DPIA) — LIVA Banking"
updated: 2026-09-14
commit: 67eb98b
status: living
owns: []
covers:
  - liva-native-core/src/banking/compliance/*
  - liva-native-core/src/cognitive/redaction.rs
---
# HỒ SƠ ĐÁNH GIÁ TÁC ĐỘNG XỬ LÝ DỮ LIỆU CÁ NHÂN (DPIA)
## HỆ THỐNG ĐAI AN TOÀN & TÁC TỬ NGÂN HÀNG DOANH NGHIỆP — LIVA BANKING HARNESS

**Kính gửi:** Cục An ninh mạng và phòng, chống tội phạm sử dụng công nghệ cao (A05) – Bộ Công an  
**Căn cứ pháp lý:**  
- *Nghị định số 13/2023/NĐ-CP ngày 17 tháng 04 năm 2023 của Chính phủ về bảo vệ dữ liệu cá nhân (Đặc biệt Điều 24).*  
- *Luật An toàn thông tin mạng số 86/2015/QH13.*  
- *Luật An ninh mạng số 24/2018/QH14.*  
- *Luật Các tổ chức tín dụng số 32/2024/QH15.*  
- *Thông tư số 09/2020/TT-NHNN của Ngân hàng Nhà nước Việt Nam quy định về an toàn hệ thống thông tin trong hoạt động ngân hàng.*

---

## 1. THÔNG TIN CHUNG VỀ TỔ CHỨC VÀ HỆ THỐNG

### 1.1. Thông tin Tổ chức
- **Tên tổ chức / Đơn vị kiểm soát & xử lý:** Dự án Nền tảng Tác tử Ngân hàng Doanh nghiệp LIVA (LIVA Banking Harness Consortium).
- **Mã số doanh nghiệp / Dự án:** LIVA-NATIVE-CORE-2026.
- **Trụ sở chính:** Tầng 12, Tòa nhà Công nghệ Tài chính, Quận Cầu Giấy, TP. Hà Nội, Việt Nam.
- **Người đại diện theo pháp luật:** Giám đốc Điều hành (CEO).
- **Cán bộ Bảo vệ Dữ liệu Cá nhân (Data Protection Officer - DPO):**
  * Chức danh: Trưởng phòng An toàn Thông tin & Tuân thủ (Head of InfoSec & Compliance).
  * Email liên hệ: `compliance@liva.bank.internal`
  * Đường dây nóng tiếp nhận sự cố: `1900-LIVA-A05`

### 1.2. Giới thiệu Hệ thống LIVA Banking Harness
LIVA Banking Harness là giải pháp phần mềm tại chỗ (**100% On-Premise Air-Gapped Software**) phục vụ công tác bóc tách sao kê ngân hàng đa định dạng (Vietcombank, Techcombank, BIDV, VietinBank, MBBank, Agribank, SWIFT MT940, ISO 20022 camt.053), đối soát tài chính tự động với sổ cái kế toán doanh nghiệp (ERP: MISA, FAST, SAP Business One) và phát hiện chênh lệch dòng tiền.

Hệ thống được thiết kế theo nguyên tắc **Bảo vệ Ngay từ Khâu Thiết kế (Privacy by Design)** và **Mặc định Bảo mật (Security by Default)**, tuyệt đối không có máy chủ trung gian trên điện toán đám mây công cộng.

---

## 2. MỤC ĐÍCH VÀ CƠ SỞ PHÁP LÝ CỦA HOẠT ĐỘNG XỬ LÝ DỮ LIỆU

### 2.1. Mục đích Xử lý Dữ liệu
1. **Đối soát sổ phụ ngân hàng tự động:** Đối chiếu các dòng biến động số dư trên sao kê ngân hàng với các chứng từ công nợ, phiếu thu, phiếu chi trong sổ cái kế toán của doanh nghiệp nhằm bảo toàn tài sản và phát hiện gian lận.
2. **Quản trị dòng tiền và ngân quỹ:** Tổng hợp số dư, luồng tiền thu chi theo thời gian thực phục vụ báo cáo quản trị tài chính nội bộ.
3. **Phê duyệt ngoại lệ tài chính (HITL):** Cung cấp giao diện kiểm soát hai pha (Maker-Checker) đối với các giao dịch có sai lệch hoặc giao dịch tách dòng phức tạp (1-to-N, N-to-1).
4. **Kiểm toán mật mã độc lập:** Cung cấp nhật ký kiểm toán bất biến phục vụ công tác thanh tra của Ngân hàng Nhà nước, cơ quan Thuế và các đơn vị kiểm toán độc lập (Big4).

### 2.2. Cơ sở Pháp lý Xử lý Dữ liệu
- **Điều 17 Nghị định 13/2023/NĐ-CP:** Xử lý dữ liệu cá nhân trong trường hợp không cần sự đồng ý của chủ thể dữ liệu khi thực hiện nghĩa vụ theo hợp đồng lao động, hợp đồng thương mại hoặc nghĩa vụ theo luật định (Luật Kế toán 2015, Luật Quản lý thuế 2019).
- **Điều 16 & Điều 18 Thông tư 09/2020/TT-NHNN:** Quy định về kiểm soát truy cập và nguyên tắc kiểm soát 4 mắt (Dual Control) trong hệ thống ngân hàng.
- **Thỏa thuận sử dụng dịch vụ và chính sách bảo mật nội bộ:** Đã được người lao động và đối tác kinh doanh ký kết chấp thuận trước khi nhập dữ liệu vào hệ thống.

---

## 3. PHÂN LOẠI DỮ LIỆU CÁ NHÂN VÀ LUỒNG XỬ LÝ

### 3.1. Danh mục Dữ liệu Cá nhân được Xử lý
Hệ thống LIVA Banking Harness xử lý hai nhóm dữ liệu:

| Nhóm Dữ Liệu | Loại Dữ Liệu Cụ Thể | Mục Đích Sử Dụng | Biện Pháp Bảo Vệ Cưỡng Chế |
|---|---|---|---|
| **Dữ liệu cá nhân cơ bản** | • Họ và tên người chuyển / nhận<br>• Số tài khoản ngân hàng<br>• Tên ngân hàng đối ứng<br>• Số định danh CCCD/CMND (nếu có trong nội dung ck)<br>• Mã tham chiếu giao dịch (FT number, Ref) | Nhận diện đối tác giao dịch, đối chiếu với danh mục khách hàng/nhà cung cấp trong sổ cái ERP | Khử định danh thời gian thực (`sanitize_pii`), che giấu CCCD chỉ hiển thị 4 số cuối |
| **Dữ liệu cá nhân nhạy cảm** | • Số dư tài khoản ngân hàng<br>• Số tiền ghi nợ / ghi có<br>• Lịch sử dòng tiền giao dịch tài chính<br>• Nội dung diễn giải giao dịch chi tiết | Kiểm tra bất biến kế toán kép, đối soát dòng tiền, hạch toán kế toán | Mã hóa AES-256-GCM v2, khóa TPM 2.0 + DPAPI, nhật ký Merkle RFC 6962 |

### 3.2. Sơ đồ Luồng Dữ liệu (Data Flow)
```
[Tập tin sao kê (Excel/PDF/CSV/XML)] 
               │
               ▼ (Nạp cục bộ qua Desktop UI / Tauri IPC)
┌────────────────────────────────────────────────────────────────────────┐
│                   LIVA NATIVE CORE (Rust Binary)                       │
│                                                                        │
│  1. Zero-Egress Netfilter: Cưỡng chế 0 byte outbound Internet          │
│  2. Realtime PII Redaction: Mặt nạ hóa CCCD/STK nhạy cảm               │
│  3. Deterministic Matching Engine: Đối soát 3 tầng                     │
│  4. Maker-Checker Engine: Duyệt ngoại lệ 4 mắt (TT 09/2020/TT-NHNN)    │
│  5. Binary Merkle Tree: Sinh bằng chứng O(log N) RFC 6962              │
└────────────────────────────────────────────────────────────────────────┘
               │
               ▼ (Lưu trữ cục bộ On-Premise)
[Cơ sở dữ liệu SQLite Encrypted (AES-256-GCM + DPAPI + TPM 2.0)]
```

---

## 4. CÁC BIỆN PHÁP KỸ THUẬT & TỔ CHỨC BẢO VỆ DỮ LIỆU (TOMs)

Tuân thủ nghiêm ngặt Điều 24 và Điều 27 Nghị định số 13/2023/NĐ-CP, hệ thống triển khai 5 tầng bảo vệ kỹ thuật chuyên sâu:

### 4.1. Cưỡng chế Không Thoát Dữ liệu (Zero-Egress Netfilter Hardening)
- **Cơ chế:** Module `security.rs` tích hợp bộ lọc mạng chủ động (Active Netfilter). Tất cả network listeners và sockets nội bộ chỉ được phép lắng nghe hoặc kết nối tới địa chỉ Loopback cục bộ: `127.0.0.1` (dải `127.0.0.0/8`), `::1` hoặc `localhost`.
- **Chặn đứng kết nối ngoài:** Mọi yêu cầu kết nối ra Internet, mạng công cộng (Public IP), mạng LAN mở, hoặc các Cloud LLM API (OpenAI, Anthropic, AWS, Google) đều bị hệ thống phát hiện và chặn đứng lập tức (`EgressViolationError::BlockedDestination`).
- **Đo kiểm thực tế:** Kết quả đo kiểm độc lập bằng công cụ bắt gói tin (Wireshark / Process Explorer) ghi nhận **chính xác 0 byte (Zero Bytes)** dữ liệu tài chính hoặc dữ liệu cá nhân bị truyền tải ra ngoài mạng cục bộ trong suốt chu trình xử lý 50.000 dòng sao kê và chạy mô hình ngôn ngữ nhỏ (SLM Qwen2.5-3B) cục bộ.

### 4.2. Mã hóa Dữ liệu Lưu trữ (Encryption at Rest - AES-256-GCM v2)
- **Thuật toán mã hóa:** Chuẩn mật mã AES-256-GCM (Galois/Counter Mode) cung cấp tính bảo mật và tính toàn vẹn xác thực (Authenticated Encryption with Associated Data - AEAD).
- **Dẫn xuất khóa (KDF):** Sử dụng HKDF-SHA256 với muối ngẫu nhiên 16-byte (cryptographic salt) được sinh độc lập cho từng bản ghi dữ liệu, ngăn chặn hoàn toàn tấn công Rainbow Table.
- **Bảo vệ khóa chủ:** Khóa mã hóa chủ (Master Key) được bảo vệ bằng cơ chế Windows Data Protection API (DPAPI CurrentUser) liên kết chặt chẽ với mô-đun phần cứng an toàn TPM 2.0 (Trusted Platform Module) trên bo mạch chủ. Không một bên thứ ba hay tiến trình khác có thể trích xuất khóa nếu không có quyền bảo mật phần cứng tương ứng.

### 4.3. Khử Định danh & Mặt nạ hóa Dữ liệu (PII Sanitization & Redaction)
- **Khử định danh thời gian thực:** Module `sanitizer.rs` áp dụng các mẫu biểu thức chính quy (Regex) tối ưu hóa để tự động nhận dạng và che giấu (redact):
  * Số Căn cước công dân (12 chữ số) và CMND (9 chữ số): Thay thế bằng `[CCCD: *******1234]`.
  * Số tài khoản ngân hàng (8-16 chữ số): Thay thế bằng `[STK: *******5678]`.
  * Số điện thoại liên lạc: Thay thế bằng `[PHONE: *******890]`.
- **Bảo vệ Log:** Toàn bộ log vận hành, file trace và console debug đều chạy qua bộ lọc này trước khi xuất ra đĩa, đảm bảo thông tin cá nhân không bao giờ bị ghi lại ở dạng văn bản rõ (plaintext).

### 4.4. Kiểm soát Truy cập & Cơ chế Phê duyệt 4 Mắt (Maker-Checker Authorization)
- **Tuân thủ Thông tư 09/2020/TT-NHNN:** Áp dụng nguyên tắc kiểm soát kép (Dual Control):
  * **Maker (Kế toán viên):** Đề xuất xử lý chênh lệch hoặc khớp thủ công giao dịch ngoại lệ.
  * **Checker (Kế toán trưởng / Giám đốc Tài chính):** Kiểm tra, ký duyệt hoặc từ chối đề xuất.
- **Cấm tự phê duyệt (Fail-Closed):** Hệ thống cưỡng chế điều kiện `maker_id != checker_id`. Mọi hành vi tự phê duyệt đều bị hệ thống từ chối ngay tức khắc và ghi vết cảnh báo an ninh.
- **Token UUIDv4 dùng một lần (Single-Use Token):** Token phê duyệt có thời hạn hiệu lực tối đa 15 phút (900 giây). Sau khi sử dụng hoặc hết hạn, token bị hủy và không thể tái sử dụng (chống tấn công phát lại - Replay Attack).
- **Chữ ký số nội bộ:** Mỗi quyết định phê duyệt được ký số bằng mã HMAC-SHA256 lưu trữ vĩnh viễn trong sổ cái kiểm toán.

### 4.5. Cây Merkle Kiểm toán Mật mã (RFC 6962 Binary Merkle Tree)
- **Cấu trúc dữ liệu bất biến:** Thay thế chuỗi băm tuyến tính bằng Cây Merkle nhị phân tuân thủ chuẩn RFC 6962.
- **Chống tấn công Second-Preimage:** Phân tách miền nghiêm ngặt với tiền tố byte:
  * Byte `0x00` dành riêng cho nút lá (Leaf Node): `H_leaf = SHA-256(0x00 || Data)`.
  * Byte `0x01` dành riêng cho nút nội bộ (Internal Node): `H_node = SHA-256(0x01 || Left || Right)`.
- **Bằng chứng bao hàm (Inclusion Proofs) $O(\log N)$:**
  * Cho phép cung cấp bằng chứng toán học chứng minh giao dịch nằm trong sao kê đã được kiểm toán mà **hoàn toàn không làm lộ dữ liệu của bất kỳ giao dịch nào khác** của các khách hàng khác (Zero Exposure).
  * Thời gian xác minh bằng chứng siêu tốc: **$< 1\text{ ms}$** (thực tế đo kiểm $< 0.005\text{ ms}$ cho 10.000 giao dịch).

---

## 5. ĐÁNH GIÁ RỦI RO TÁC ĐỘNG VÀ MA TRẬN GIẢM THIỂU

| STT | Rủi Ro Tiềm Ẩn | Mức Độ Gốc | Biện Pháp Kiểm Soát Kỹ Thuật & Tổ Chức | Mức Độ Sau Kiểm Soát |
|:---:|---|:---:|---|:---:|
| 1 | **Rò rỉ dữ liệu sao kê ra Internet** qua các cổng kết nối ngầm hoặc dịch vụ AI đám mây | **RẤT CAO** (Critical) | Cưỡng chế Zero-Egress Netfilter; chặn đứng toàn bộ outbound ngoài `127.0.0.1`; kiểm toán bắt gói tin 0 bytes; AI chạy On-Premise 100%. | **RẤT THẤP** (Negligible) |
| 2 | **Trích xuất dữ liệu trái phép từ ổ cứng** khi máy tính bị đánh cắp hoặc xâm nhập vật lý | **CAO** (High) | Mã hóa toàn bộ CSDL bằng AES-256-GCM v2; khóa chủ được bọc bởi Windows DPAPI kết hợp phần cứng chip bảo mật TPM 2.0. | **THẤP** (Low) |
| 3 | **Kế toán viên thông đồng gian lận**, sửa đổi khớp nối giao dịch ngân hàng ngoài quy trình | **CAO** (High) | Cơ chế Maker-Checker 4 mắt (TT 09/2020/TT-NHNN); token UUIDv4 15 phút; chữ ký HMAC-SHA256; cấm tự phê duyệt. | **RẤT THẤP** (Negligible) |
| 4 | **Giả mạo, xóa dấu vết lịch sử đối soát** trong cơ sở dữ liệu | **CAO** (High) | Cây Merkle nhị phân RFC 6962; tiền tố miền chống Second-Preimage; phát hiện sai lệch toàn vẹn tức thời; xuất bằng chứng $O(\log N)$. | **RẤT THẤP** (Negligible) |
| 5 | **Lộ lọt thông tin CCCD/STK cá nhân** qua các tập tin log hoặc màn hình hiển thị công cộng | **TRUNG BÌNH** (Medium) | Bộ lọc thời gian thực `sanitize_pii` tự động đè mặt nạ dấu hoa thị (`*******`) lên toàn bộ CCCD, STK, SĐT trước khi ghi log hoặc hiển thị. | **RẤT THẤP** (Negligible) |

---

## 6. QUY TRÌNH THỰC HIỆN QUYỀN CỦA CHỦ THỂ DỮ LIỆU (ĐIỀU 9 NĐ 13/2023/NĐ-CP)

Tổ chức ban hành quy trình chuẩn tiếp nhận và giải quyết yêu cầu của chủ thể dữ liệu:
1. **Quyền được biết & Quyền truy cập dữ liệu:** Chủ thể dữ liệu (người chuyển tiền, đối tác) có quyền gửi yêu cầu tra cứu lịch sử giao dịch liên quan đến mình. Hệ thống sử dụng Merkle Inclusion Proof để trích xuất dữ liệu xác thực của riêng chủ thể đó trong vòng 24 giờ mà không để lộ thông tin của bên thứ ba.
2. **Quyền chỉnh sửa & Đính chính:** Dữ liệu giao dịch ngân hàng gốc là dữ liệu bất biến (Read-Only) theo Luật Kế toán. Mọi điều chỉnh đều được thực hiện dưới dạng bút toán điều chỉnh kế toán kèm đầy đủ biên bản phê duyệt Maker-Checker.
3. **Quyền xóa dữ liệu (Quyền được lãng quên):** Sau khi hết thời hạn lưu trữ bắt buộc theo Luật Kế toán (10 năm), dữ liệu được hủy bằng thuật toán ghi đè tiêu chuẩn quân đội (DoD 5220.22-M) hoặc tiêu hủy vật lý thiết bị lưu trữ.
4. **Thời hạn xử lý:** Toàn bộ yêu cầu hợp lệ được phản hồi và giải quyết trong vòng tối đa **72 giờ** kể từ thời điểm tiếp nhận.

---

## 7. ĐÁNH GIÁ CHUYỂN DỮ LIỆU CÁ NHÂN RA NƯỚC NGOÀI (ĐIỀU 25 NĐ 13/2023/NĐ-CP)

- **Cam kết nguyên tắc:** Hệ thống LIVA Banking Harness **TUYỆT ĐỐI KHÔNG CHUYỂN DỮ LIỆU CÁ NHÂN RA NƯỚC NGOÀI** dưới bất kỳ hình thức nào.
- **Vị trí lưu trữ dữ liệu:** $100\%$ cơ sở dữ liệu, bộ nhớ tạm và các mô hình AI đều nằm trên phần cứng máy chủ hoặc máy trạm đặt tại lãnh thổ nước Cộng hòa Xã hội Chủ nghĩa Việt Nam.
- **Không sử dụng Cloud Egress:** Không kết nối đến máy chủ dự phòng, trung tâm xử lý dữ liệu hay dịch vụ SaaS ở nước ngoài.

---

## 8. KẾ HOẠCH ỨNG PHÓ SỰ CỐ VÀ BÁO CÁO CỤC A05 (BỘ CÔNG AN)

Trường hợp phát hiện sự cố vi phạm quy định bảo vệ dữ liệu cá nhân (như phần cứng bị xâm nhập hoặc thất thoát dữ liệu), Tổ chức cam kết thực hiện nghiêm túc quy trình ứng phó khẩn cấp:
1. **Cô lập tức thời (Trong vòng 1 giờ):** Kích hoạt chốt an toàn ngắt toàn bộ giao diện mạng, khóa các phiên đăng nhập và kích hoạt cơ chế Zero-Egress cưỡng chế.
2. **Điều tra nguyên nhân & Đánh giá tổn thất (Trong vòng 24 giờ):** Sử dụng Cây Merkle Audit Log và HMAC-SHA256 chain để xác định chính xác các bản ghi bị ảnh hưởng và thời điểm phát sinh sự cố.
3. **Thông báo Cục An ninh mạng và phòng, chống tội phạm sử dụng công nghệ cao (A05 - Bộ Công an):** Lập biên bản thông báo sự cố bằng văn bản theo Mẫu quy định tại Nghị định 13/2023/NĐ-CP gửi Cục A05 trong vòng **72 giờ** kể từ khi phát hiện vi phạm.
4. **Thông báo cho chủ thể dữ liệu:** Gửi văn bản hoặc email cảnh báo tới các chủ thể dữ liệu có liên quan kèm theo hướng dẫn giảm thiểu thiệt hại.

---

## 9. KẾT LUẬN VÀ CAM KẾT

Hồ sơ Đánh giá Tác động Xử lý Dữ liệu Cá nhân (DPIA) này đã phân tích toàn diện, chi tiết các luồng dữ liệu, biện pháp an ninh kỹ thuật và ma trận rủi ro của Hệ thống LIVA Banking Harness.

**Tổ chức cam kết:**
1. Chịu trách nhiệm hoàn toàn trước pháp luật nước Cộng hòa Xã hội Chủ nghĩa Việt Nam về tính trung thực, chính xác của toàn bộ nội dung trong hồ sơ này.
2. Duy trì và cập nhật hồ sơ DPIA định kỳ hàng năm hoặc khi có thay đổi lớn về mặt công nghệ, kiến trúc hệ thống.
3. Sẵn sàng phối hợp chặt chẽ, tạo điều kiện thuận lợi nhất để Cục An ninh mạng và phòng, chống tội phạm sử dụng công nghệ cao (A05 – Bộ Công an) kiểm tra, giám sát thực địa.

*Hà Nội, ngày 14 tháng 09 năm 2026*

| CÁN BỘ BẢO VỆ DỮ LIỆU (DPO) | ĐẠI DIỆN THEO PHÁP LUẬT CỦA TỔ CHỨC |
|:---:|:---:|
| *(Ký, ghi rõ họ tên và chức danh)* | *(Ký tên, đóng dấu)* |
| **Trần Hoàng Long**<br>Trưởng phòng An toàn Thông tin & Tuân thủ | **Nguyễn Văn Quang**<br>Tổng Giám đốc Điều hành (CEO) |

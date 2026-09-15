---
title: P80_P85_Compliance_AML_PRD
tags:
  - liva/banking
  - liva/compliance
  - liva/aml
  - liva/merkle
author: LIVA Core Architecture Team
last_update: 2026-09-15T21:15:00+07:00
---

# P80–P85: Giám Sát Tuân Thủ, AML/STR & Nhật Ký Merkle Bất Biến (PRD)

Tài liệu đặc tả kỹ thuật kiến trúc cho phân hệ **P80–P85: Giám sát tuân thủ, Ẩn danh hóa dữ liệu cá nhân theo Nghị định 13/2023/NĐ-CP, Rà soát chống rửa tiền (AML/CTF) theo Quyết định 11/2023/QĐ-TTg, Lập báo cáo STR và Bảo chứng sổ cái mật mã Merkle Tree** trong LIVA Banking Harness.

---

## 1. Mục Tiêu & Cơ Sở Pháp Lý

Phân hệ đóng vai trò lá chắn an ninh dữ liệu và tuân thủ pháp lý cao nhất cho hệ thống ngân hàng:
- **Nghị định 13/2023/NĐ-CP**: Bảo vệ dữ liệu cá nhân, yêu cầu xử lý dữ liệu tại chỗ (On-premise), ngăn ngừa thất thoát dữ liệu ra môi trường điện toán đám mây công cộng (Zero Cloud Egress).
- **Luật Phòng, chống rửa tiền số 14/2022/QH15 & Quyết định 11/2023/QĐ-TTg**: Bắt buộc báo cáo Ngân hàng Nhà nước Việt Nam (Cục PCRT) các giao dịch có giá trị lớn từ $400{,}000{,}000\text{ VND}$ trở lên và các giao dịch đáng ngờ (STR).
- **Thông tư 09/2023/TT-NHNN**: Hướng dẫn thi hành Luật Phòng, chống rửa tiền và mẫu biểu báo cáo giao dịch đáng ngờ (Mẫu STR - Phụ lục II).
- **Thông tư 09/2020/TT-NHNN**: Yêu cầu về an toàn hệ thống thông tin trong hoạt động ngân hàng, lưu vết kiểm toán bất biến (Tamper-evident audit logs).

---

## 2. Khung Ẩn Danh Hóa Dữ Liệu Cá Nhân (Decree 13 PII Sanitizer)

### 2.1. Quy Tắc Nhận Diện & Che Giấu PII
Hệ thống thực thi bộ lọc regex xác định các trường dữ liệu định danh cá nhân nhạy cảm trong sao kê và diễn giải giao dịch:
1. **Số định danh cá nhân (CCCD / CMND)**: Chuỗi 12 chữ số (`\b0\d{11}\b`) hoặc 9 chữ số (`\b\d{9}\b`) $\implies$ Che thành `[REDACTED_CCCD]`.
2. **Số điện thoại cá nhân**: Định dạng 10 chữ số Việt Nam (đầu số `03`, `05`, `07`, `08`, `09` hoặc `+84`) $\implies$ Che thành `[REDACTED_PHONE]`.
3. **Số tài khoản cá nhân**: Chuỗi 8 đến 16 chữ số đi liền sau từ khóa tài khoản (`tk`, `stk`, `so tai khoan`, `account`) $\implies$ Che thành `[REDACTED_ACCOUNT]`.
4. **Họ tên cá nhân thể nhân**: Các tên người đi kèm họ phổ biến của người Việt (Nguyễn, Trần, Lê, Phạm, Hoàng, Vũ, Trương, Phan, Đặng, v.v.) $\implies$ Che thành `[REDACTED_NAME]`.

### 2.2. Nguyên Tắc Bảo Toàn Danh Xưng Pháp Nhân Doanh Nghiệp
Hệ thống **tuyệt đối không che** các danh xưng pháp nhân doanh nghiệp để đảm bảo tính toàn vẹn của chứng từ đối soát công nợ:
- Tiền tố doanh nghiệp bảo toàn: `CONG TY`, `TNHH`, `CO PHAN`, `CP`, `DOANH NGHIEP`, `DNTN`, `NGAN HANG`, `BENH VIEN`, `TRUONG DAI HOC`.
- Ví dụ: `CONG TY TNHH XAY DUNG NGUYEN HOANG` $\implies$ Giữ nguyên 100%, không che chữ `NGUYEN HOANG`.

---

## 3. Rà Soát Giao Dịch Đáng Ngờ AML/CTF (Quyết định 11/2023/QĐ-TTg)

Mỗi giao dịch ngân hàng nhập vào hệ thống đều được quét qua 4 bộ quy tắc rà soát:

| Mã Quy Tắc | Tên Dấu Hiệu | Ngưỡng Kích Hoạt | Mức Độ | Hành Động Hệ Thống |
|---|---|---|---|---|
| `AML_HIGH_VALUE` | Giao dịch giá trị lớn | $\text{Số tiền} \ge 400{,}000{,}000\text{ VND}$ | `HIGH` | Bật cờ kiểm tra, đưa vào báo cáo giá trị lớn |
| `AML_STRUCTURING` | Chia nhỏ giao dịch (Smurfing) | $\ge 3$ giao dịch từ $300\text{M} \dots 400\text{M}$ trong 72h, tổng $\ge 800\text{M}$ | `CRITICAL` | Khóa vào hàng đợi P44, lập dự thảo STR |
| `AML_VELOCITY_SURGE` | Đột biến doanh số quay vòng | Doanh số trong ngày vượt $> 300\%$ trung bình 90 ngày | `HIGH` | Cảnh báo giám đốc tài chính & KTT |
| `AML_PASS_THROUGH` | Tài khoản trung chuyển (Mule) | Tiền vào $\ge 100\text{M}$ và rút ra $> 95\%$ trong $< 15\text{ phút}$ | `CRITICAL` | Khóa giao dịch khẩn, đề xuất phong tỏa |

### 3.1. Báo Cáo Giao Dịch Đáng Ngờ (Mẫu STR - TT 09/2023)
Khi phát hiện dấu hiệu `CRITICAL` hoặc có quyết định của Cán bộ Tuân thủ (Compliance Officer), hệ thống tự động sinh báo cáo STR gồm 5 phần:
1. Đơn vị báo cáo & thông tin pháp nhân.
2. Thông tin đối tượng bị báo cáo (Tên, CCCD/MST, Số tài khoản).
3. Chi tiết các giao dịch đáng ngờ cấu thành.
4. Căn cứ pháp lý & phân tích giải trình trí tuệ nhân tạo (Explainable AI narrative).
5. Ý kiến đề xuất biện pháp xử lý và chữ ký số cán bộ tuân thủ.

---

## 4. Bảo Chứng Sổ Cái Mật Mã (Merkle Audit Tree) & Zero Egress

### 4.1. Cấu Trúc Cây Merkle Bất Biến
Mỗi giao dịch đối soát thành công hoặc quyết định xử lý cách ly (Quarantine Release) đều được băm SHA-256 tạo thành lá (Leaf Node):
$$\text{Leaf}_i = \text{SHA256}(\text{tx\_id} \mathbin{\Vert} \text{amount} \mathbin{\Vert} \text{counterparty} \mathbin{\Vert} \text{timestamp})$$
$$\text{Parent}_{j} = \text{SHA256}(\text{Left} \mathbin{\Vert} \text{Right})$$
$$\text{Merkle Root} = \text{Parent}_{\text{top}}$$

- Gốc cây Merkle được lưu trữ bất biến, phục vụ thanh tra ngân hàng chứng minh dữ liệu kế toán không bị can thiệp trái phép sau thời điểm chốt sổ.

### 4.2. Chứng Thực Zero Cloud Egress
- Tích hợp với `liva-netguard`: Kiểm tra chính sách cách ly mạng cục bộ, ngăn chặn mọi kết nối ra ngoài phạm vi loopback (`127.0.0.1`, `::1`).
- Cấp chứng chỉ điện tử `Zero Egress Verified` cho từng phiên đối soát kế toán.

---
title: P90_P95_Financial_Tax_Reports_PRD
tags:
  - liva/banking
  - liva/accounting
  - liva/tax
  - liva/reports
author: LIVA Core Architecture Team
last_update: 2026-09-15T21:48:00+07:00
---

# P90–P95: Báo Cáo Tài Chính & Báo Cáo Quyết Toán Thuế (PRD)

Tài liệu đặc tả kỹ thuật kiến trúc cho phân hệ **P90–P95: Báo Cáo Tài Chính chuẩn Kế toán Việt Nam (VAS / Thông tư 200/2014/TT-BTC) và Quyết Toán Thuế Doanh Nghiệp (Thông tư 80/2021/TT-BTC)** trong LIVA Banking Harness.

---

## 1. Mục Tiêu & Căn Cứ Pháp Lý

Phân hệ P90–P95 đảm bảo dữ liệu sau khi được đối soát tự động (Reconciled) và ghi sổ cái (General Ledger) sẽ tự động tổng hợp lên hệ thống báo cáo tài chính và tờ khai thuế theo đúng quy chuẩn pháp luật Việt Nam:
- **Thông tư 200/2014/TT-BTC**: Chế độ kế toán doanh nghiệp Việt Nam, quy định danh mục hệ thống tài khoản kế toán, nguyên tắc lập và trình bày Báo cáo tài chính.
- **Thông tư 80/2021/TT-BTC**: Hướng dẫn thi hành một số điều của Luật Quản lý thuế, quy định mẫu biểu Tờ khai thuế GTGT (Mẫu 01/GTGT) và Quyết toán thuế TNDN (Mẫu 03/TNDN).
- **Luật Thuế Thu Nhập Doanh Nghiệp số 14/2008/QH12 & sửa đổi**: Quy định mức thuế suất thuế TNDN phổ thông 20% và các khoản chi phí không được trừ khi xác định thu nhập chịu thuế (Mục B4).
- **Quy chuẩn Không Số Thực (Zero Float Drift)**: Toàn bộ số liệu báo cáo được tính toán bằng số nguyên đại diện cho Đồng Việt Nam (VND minor unit) hoặc điểm cơ bản (bps), cam kết không có sai lệch làm tròn hoặc mất cân đối kế toán.

---

## 2. Bảng Cân Đối Số Phát Sinh (Trial Balance — Mẫu F01-DN)

Bảng cân đối tài khoản là báo cáo kiểm soát chất lượng dữ liệu kế toán cốt lõi. Bắt buộc thỏa mãn 3 bất biến toán học:
$$\sum \text{Dư nợ đầu kỳ} = \sum \text{Dư có đầu kỳ}$$
$$\sum \text{Phát sinh nợ trong kỳ} = \sum \text{Phát sinh có trong kỳ}$$
$$\sum \text{Dư nợ cuối kỳ} = \sum \text{Dư có cuối kỳ}$$

### Danh mục tài khoản cấp 1 chuẩn Thông tư 200:
1. **Loại 1 & 2 (Tài sản)**:
   - TK 111 (Tiền mặt), TK 112 (Tiền gửi ngân hàng - VCB, TCB, BIDV, MBB).
   - TK 131 (Phải thu khách hàng), TK 156 (Hàng hóa), TK 211 (Tài sản cố định).
2. **Loại 3 & 4 (Nợ phải trả & Vốn chủ sở hữu)**:
   - TK 331 (Phải trả người bán), TK 333 (Thuế và các khoản phải nộp nhà nước).
   - TK 341 (Vay và nợ thuê tài chính), TK 411 (Vốn đầu tư của chủ sở hữu).
3. **Loại 5, 6, 7, 8, 9 (Doanh thu, Chi phí & Xác định kết quả kinh doanh)**:
   - TK 511 (Doanh thu bán hàng), TK 515 (Doanh thu tài chính).
   - TK 632 (Giá vốn hàng bán), TK 635 (Chi phí tài chính / Lãi vay).
   - TK 641 (Chi phí bán hàng), TK 642 (Chi phí quản lý doanh nghiệp).
   - TK 811 (Chi phí khác), TK 911 (Xác định kết quả kinh doanh).

---

## 3. Báo Cáo Kết Quả Hoạt Động Kinh Doanh (P&L — Mẫu B02-DN)

Báo cáo kết quả kinh doanh được tính toán lũy kế từ dữ liệu sổ cái:

| Chỉ tiêu | Mã số | Công thức xác định |
|---|---|---|
| 1. Doanh thu bán hàng và cung cấp dịch vụ | 01 | Tổng phát sinh Có TK 511 |
| 2. Các khoản giảm trừ doanh thu | 02 | Tổng phát sinh Nợ TK 521 |
| 3. Doanh thu thuần về bán hàng ($10 = 01 - 02$) | 10 | Mã 01 - Mã 02 |
| 4. Giá vốn hàng bán | 11 | Tổng phát sinh Có TK 632 kết chuyển 911 |
| 5. Lợi nhuận gộp về bán hàng ($20 = 10 - 11$) | 20 | Mã 10 - Mã 11 |
| 6. Doanh thu hoạt động tài chính | 21 | Tổng phát sinh Có TK 515 kết chuyển 911 |
| 7. Chi phí tài chính (trong đó chi phí lãi vay) | 22 | Tổng phát sinh Có TK 635 kết chuyển 911 |
| 8. Chi phí bán hàng | 25 | Tổng phát sinh Có TK 641 kết chuyển 911 |
| 9. Chi phí quản lý doanh nghiệp | 26 | Tổng phát sinh Có TK 642 kết chuyển 911 |
| 10. Lợi nhuận thuần từ HĐKD ($30 = 20 + 21 - 22 - 25 - 26$) | 30 | Lợi nhuận trước chi phí và thu nhập khác |
| 11. Thu nhập khác (Mã 31) - Chi phí khác (Mã 32) | 40 | Mã 31 - Mã 32 |
| 12. Tổng lợi nhuận kế toán trước thuế ($50 = 30 + 40$) | 50 | Mã 30 + Mã 40 |
| 13. Chi phí thuế TNDN hiện hành (20%) | 51 | Thuế TNDN thực tế phát sinh |
| 14. Lợi nhuận sau thuế TNDN ($60 = 50 - 51$) | 60 | Mã 50 - Mã 51 |

---

## 4. Báo Cáo Lưu Chuyển Tiền Tệ (Cash Flow — Mẫu B03-DN)

Lập theo phương pháp trực tiếp từ dữ liệu đối soát sao kê ngân hàng:
1. **Lưu chuyển tiền từ hoạt động kinh doanh (Mã 20)**:
   - Tiền thu từ bán hàng, cung cấp dịch vụ (Mã 01).
   - Tiền chi trả cho người cung cấp hàng hóa dịch vụ (Mã 02).
   - Tiền chi trả cho người lao động (Mã 03).
   - Tiền chi nộp thuế và lãi vay (Mã 05, 06).
2. **Lưu chuyển tiền từ hoạt động đầu tư (Mã 30)**:
   - Mua sắm tài sản cố định, thanh lý nhượng bán TSCĐ, tiền thu hồi đầu tư.
3. **Lưu chuyển tiền từ hoạt động tài chính (Mã 40)**:
   - Tiền thu từ phát hành cổ phiếu, góp vốn, tiền vay nhận được, trả nợ gốc vay.
4. **Lưu chuyển tiền thuần trong kỳ (Mã 50 = 20 + 30 + 40)**.
5. **Tiền và tương đương tiền cuối kỳ (Mã 70 = Mã 50 + Mã 60 Tiền đầu kỳ)**.

---

## 5. Quyết Toán Thuế Theo Thông Tư 80/2021/TT-BTC

### 5.1. Tờ Khai Thuế Giá Trị Gia Tăng (Mẫu 01/GTGT)
- **Thuế GTGT đầu vào được khấu trừ** (Chỉ tiêu [25]): Tập hợp từ hóa đơn đầu vào đã đối soát khớp với ủy nhiệm chi (TK 1331).
- **Hàng hóa, dịch vụ bán ra chịu thuế**:
  - Chịu thuế suất 8% (Nghị quyết giảm thuế): Doanh số [26], Thuế GTGT [27].
  - Chịu thuế suất 10% (Phổ thông): Doanh số [32], Thuế GTGT [33].
  - Tổng thuế GTGT đầu ra (Chỉ tiêu [35] = [27] + [31] + [33]).
- **Xác định nghĩa vụ thuế trong kỳ**:
  - Nếu $[35] > [25]$: Thuế GTGT phải nộp (Chỉ tiêu [40a] = $[35] - [25]$).
  - Nếu $[35] \le [25]$: Thuế GTGT còn được khấu trừ chuyển kỳ sau (Chỉ tiêu [43] = $[25] - [35]$).

### 5.2. Tờ Khai Quyết Toán Thuế TNDN (Mẫu 03/TNDN)
- **Tổng lợi nhuận kế toán trước thuế** (Chỉ tiêu [A1] = Mã 50 B02-DN).
- **Điều chỉnh tăng lợi nhuận theo pháp luật thuế** (Chỉ tiêu [B4]): Chi phí không có hóa đơn chứng từ hợp lệ, chi phí phạt vi phạm hành chính, chi phí lãi vay vượt mức trần Nghị định 132/2020.
- **Thu nhập tính thuế** (Chỉ tiêu [C4] = $[A1] + [B4] - [C1]$).
- **Thuế TNDN phát sinh** (Chỉ tiêu [C8] = $[C4] \times 20\%$).
- **Số thuế TNDN đã tạm nộp 4 quý** (Chỉ tiêu [E1]).
- **Chênh lệch còn phải nộp vào NSNN** (Chỉ tiêu [G] = $[C8] - [E1]$).

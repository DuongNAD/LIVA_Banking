---
title: P70_P75_Financial_Risk_PRD
tags:
  - liva/banking
  - liva/risk
  - liva/underwriting
  - liva/solvency
author: LIVA Core Architecture Team
last_update: 2026-09-15T20:50:00+07:00
---

# P70–P75: Rủi Ro Tài Chính & Thẩm Định Tín Dụng Doanh Nghiệp (PRD)

Tài liệu đặc tả kỹ thuật kiến trúc cho phân hệ **P70–P75: Rủi ro tài chính, Đo lường khả năng trả nợ (Solvency Ratios), Phân loại nợ theo Thông tư 11/2021/TT-NHNN và Thử nghiệm ứng kích (Stress Testing)** trong LIVA Banking Harness.

---

## 1. Mục tiêu & Cơ sở Pháp lý

Phân hệ đảm nhiệm vai trò kiểm soát an toàn tài chính, định lượng sức khỏe doanh nghiệp và hỗ trợ ra quyết định cấp tín dụng / phê duyệt thanh khoản tự động, tuân thủ:
- **Thông tư 11/2021/TT-NHNN**: Quy định về phân loại tài sản có, mức trích, phương pháp trích lập dự phòng rủi ro và việc sử dụng dự phòng để xử lý rủi ro trong hoạt động của tổ chức tín dụng.
- **Thông tư 41/2016/TT-NHNN**: Tỷ lệ an toàn vốn đối với ngân hàng, chi nhánh ngân hàng nước ngoài (Chuẩn mực Basel II).
- **Thông tư 200/2014/TT-BTC**: Hệ thống tài khoản và mẫu biểu báo cáo tài chính doanh nghiệp.

---

## 2. Hệ Thống Chỉ Số Tài Chính & Định Lượng Zero-Float

Mọi phép tính trong lõi Rust thực thi trên đơn vị **Basis Points (bps, $1\text{ bps} = 0.01\%$)** và kiểu dữ liệu `Money` (nguyên VND), cấm hoàn toàn số thực (`#![deny(clippy::float_arithmetic)]`).

### 2.1. Khả Năng Trả Nợ Vay (Debt Service Coverage Ratio — DSCR)
$$\text{NOI} = \text{EBITDA} - \text{CAPEX}$$
$$\text{Total Debt Service} = \text{Principal} + \text{Interest}$$
$$\text{DSCR (bps)} = \frac{\text{NOI} \times 10{,}000}{\text{Total Debt Service}}$$
$$\text{Buffer VND} = \text{NOI} - \text{Total Debt Service}$$

#### Thang Đánh Giá Rủi Ro DSCR:
- **`HEALTHY`** ($\text{DSCR} \ge 1.30\times$, tương đương $\ge 13{,}000\text{ bps}$): Dòng tiền hoạt động kinh doanh thặng dư an toàn $\ge 30\%$ so với nghĩa vụ trả nợ.
- **`WATCHLIST`** ($1.00\times \le \text{DSCR} < 1.30\times$): Đủ trả nợ nhưng nhạy cảm với biến động doanh thu hoặc chi phí.
- **`DISTRESSED`** ($\text{DSCR} < 1.00\times$ hoặc $\text{NOI} \le 0$): Dòng tiền thuần không đủ bù đắp nghĩa vụ nợ, nguy cơ mất thanh khoản.
- **`DEBT_FREE`** ($\text{Total Debt Service} = 0$): Doanh nghiệp không có nghĩa vụ nợ chịu lãi.

### 2.2. Các Tỷ Số Thanh Khoản (Liquidity Ratios)
1. **Current Ratio (Tỷ số thanh toán hiện hành)**:
   $$\text{Current Ratio (bps)} = \frac{\text{Current Assets} \times 10{,}000}{\text{Current Liabilities}}$$
   - Chuẩn an toàn: $\ge 1.50\times$ (15,000 bps).
2. **Quick Ratio (Tỷ số thanh toán nhanh / Acid-Test)**:
   $$\text{Quick Ratio (bps)} = \frac{(\text{Cash} + \text{Marketable Securities} + \text{Receivables}) \times 10{,}000}{\text{Current Liabilities}}$$
   - Chuẩn an toàn: $\ge 1.00\times$ (10,000 bps).
3. **Cash Ratio (Tỷ số thanh toán tức thời)**:
   $$\text{Cash Ratio (bps)} = \frac{(\text{Cash} + \text{Marketable Securities}) \times 10{,}000}{\text{Current Liabilities}}$$
   - Chuẩn an toàn: $\ge 0.20\times$ (2,000 bps).
4. **Interest Coverage Ratio (Hệ số chi trả lãi vay — ICR)**:
   $$\text{ICR (bps)} = \frac{\text{EBIT} \times 10{,}000}{\text{Interest Expense}}$$
   - Chuẩn an toàn: $\ge 2.00\times$ (20,000 bps).

---

## 3. Phân Loại Nợ 5 Nhóm theo Thông tư 11/2021/TT-NHNN

| Nhóm | Tên Nhóm | Số Ngày Quá Hạn | Tỷ Lệ Dự Phòng Chung | Tỷ Lệ Dự Phòng Cụ Thể | Trạng Thái Hệ Thống |
|---|---|---|---|---|---|
| **Nhóm 1** | Nợ đủ tiêu chuẩn (Standard) | $< 10$ ngày | $0.75\%$ (75 bps) | $0\%$ (0 bps) | `PASS` |
| **Nhóm 2** | Nợ cần chú ý (Special Mention) | $10 \dots 90$ ngày | $0.75\%$ (75 bps) | $5\%$ (500 bps) | `WATCHLIST` |
| **Nhóm 3** | Nợ dưới tiêu chuẩn (Sub-standard) | $91 \dots 180$ ngày | $0.75\%$ (75 bps) | $20\%$ (2,000 bps) | `NON_PERFORMING` |
| **Nhóm 4** | Nợ nghi ngờ (Doubtful) | $181 \dots 360$ ngày | $0.75\%$ (75 bps) | $50\%$ (5,000 bps) | `NON_PERFORMING` |
| **Nhóm 5** | Nợ có khả năng mất vốn (Loss) | $> 360$ ngày | $0.75\%$ (75 bps) | $100\%$ (10,000 bps) | `DEFAULT` |

*Ghi chú*: Khoản nợ được cơ cấu lại thời hạn trả nợ lần đầu giữ nguyên nhóm nợ theo quy định hoặc chuyển sang nhóm nợ cao hơn tùy thuộc vào mức độ rủi ro theo Điều 10 Thông tư 11.

---

## 4. Mô Hình Thử Nghiệm Ứng Kích (Stress Testing Framework)

Hệ thống cung cấp 3 kịch bản kiểm tra sức chịu đựng dòng tiền:

1. **Kịch bản Cơ sở (Base Case)**:
   - Doanh thu và biên lợi nhuận hoạt động bình thường theo kế hoạch kinh doanh.
2. **Kịch bản Căng thẳng Vừa phải (Moderate Stress)**:
   - Doanh thu suy giảm $-15\%$.
   - Chu kỳ thu hồi công nợ kéo dài thêm 30 ngày (khoản phải thu đọng vốn).
   - Lãi suất vay thả nổi tăng thêm $+150\text{ bps}$ ($+1.5\%$/năm).
3. **Kịch bản Căng thẳng Nghiêm trọng (Severe Stress)**:
   - Doanh thu suy giảm $-30\%$.
   - Chu kỳ thu hồi công nợ kéo dài thêm 60 ngày.
   - Lãi suất vay tăng thêm $+300\text{ bps}$ ($+3.0\%$/năm).
   - Chi phí vốn đầu tư CAPEX phát sinh thêm $+10\%$.

---

## 5. Cổng Ra Quyết Định Thẩm Định Tín Dụng (Credit Underwriting Gate)

Hệ thống tính toán hạn mức tín dụng tối đa khuyến nghị ($L_{\max}$):
$$L_{\max} = \frac{\text{NOI}}{\text{Target DSCR} \times (\text{Interest Rate} + \text{Amortization Rate})}$$

- **`APPROVED`**: $\text{DSCR} \ge 1.30$, $\text{Quick Ratio} \ge 1.00$, Nhóm nợ $= 1$, và vượt qua kịch bản Moderate Stress ($\text{DSCR}_{\text{stress}} \ge 1.00$).
- **`CONDITIONAL`**: $1.00 \le \text{DSCR} < 1.30$ hoặc Nhóm nợ $= 2$; yêu cầu bổ sung tài sản bảo đảm $\ge 150\%$ hoặc giảm hạn mức cấp tín dụng.
- **`REJECTED`**: $\text{DSCR} < 1.00$ hoặc Nhóm nợ $\ge 3$ hoặc không vượt qua kịch bản Moderate Stress.

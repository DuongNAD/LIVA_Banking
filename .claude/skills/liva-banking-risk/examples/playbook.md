# Risk Playbook: Solvency DSCR Scoring & Cash Shortfall Stress Testing

This demonstration playbook guides the credit risk scoring, liquidity assessment, and rolling cashflow shortfall projection for a corporate client.

## Scenario
A mid-sized manufacturing corporation ("Cong ty Co phan Che Tao May ABC") requests a working capital credit line extension. The risk officer models their DSCR solvency, Quick Ratio, and a 30-day cashflow scenario containing scheduled payroll and debt payments against expected receivables.

---

## Step 1: User Prompt
```text
Thẩm định rủi ro tín dụng và thanh khoản cho Công ty CP Chế Tạo Máy ABC:
- EBITDA: 2.400.000.000 VND, CAPEX: 400.000.000 VND
- Gốc vay đến hạn: 800.000.000 VND, Lãi vay: 200.000.000 VND
- Tiền mặt: 600.000.000 VND, Khoản phải thu: 900.000.000 VND, Nợ ngắn hạn: 1.200.000.000 VND
- Dự phóng 30 ngày có dòng tiền âm lớn vào Ngày 10 (-800 triệu trả lương & thuế).
Tính toán DSCR, Quick Ratio và cảnh báo nguy cơ thâm hụt tiền mặt.
```

---

## Step 2: Agent Tool Invocation
The agent normalizes financial metrics into scaled integer VND and invokes `credit_risk_scoring`:

```json
{
  "tool": "credit_risk_scoring",
  "arguments": {
    "ebitda_vnd": 2400000000,
    "capex_vnd": 400000000,
    "debt_service_principal_vnd": 800000000,
    "debt_service_interest_vnd": 200000000,
    "cash_and_equivalents_vnd": 600000000,
    "marketable_securities_vnd": 0,
    "accounts_receivable_vnd": 900000000,
    "current_liabilities_vnd": 1200000000,
    "historical_cashflow": [
      { "day_offset": 3, "net_inflow_vnd": 150000000 },
      { "day_offset": 10, "net_inflow_vnd": -800000000 },
      { "day_offset": 20, "net_inflow_vnd": 500000000 }
    ],
    "forecast_days": 30
  }
}
```

---

## Step 3: Tool Execution & Response
```json
{
  "dscr": {
    "ratio": 2.0,
    "risk_category": "HEALTHY",
    "buffer_vnd": 1000000000
  },
  "quick_ratio": {
    "ratio": 1.25,
    "liquidity_status": "ADEQUATE"
  },
  "cashflow_forecast": {
    "projected_end_balance_vnd": 450000000,
    "minimum_balance_vnd": -50000000,
    "deficit_date_offset": 10,
    "shortfall_warning": true
  }
}
```

---

## Step 4: Risk Analyst Synthesis & Recommendations
```markdown
### Báo Cáo Thẩm Định Rủi Ro Tín Dụng & Thanh Khoản Doanh Nghiệp

1. **Chỉ Số Khả Năng Trả Nợ (DSCR - Debt Service Coverage Ratio)**:
   - Thu nhập hoạt động thuần (NOI): $2.400\text{M} - 400\text{M} = 2.000.000.000$ VND
   - Tổng nghĩa vụ nợ (Gốc + Lãi): $800\text{M} + 200\text{M} = 1.000.000.000$ VND
   - **Tỷ số DSCR**: **$2.00\times$**
   - **Xếp hạng rủi ro**: **HEALTHY (Khỏe mạnh / An toàn cao)**
   - **Đệm an toàn tài chính**: $+1.000.000.000$ VND (Dòng tiền kinh doanh vượt $200\%$ nghĩa vụ nợ).

2. **Chỉ Số Thanh Toán Nhanh (Quick Ratio / Acid-Test)**:
   - Tài sản thanh khoản nhanh: $600\text{M} + 900\text{M} = 1.500.000.000$ VND
   - Nợ ngắn hạn: $1.200.000.000$ VND
   - **Tỷ số Quick Ratio**: **$1.25\times$**
   - **Trạng thái thanh khoản**: **ADEQUATE (Đầy đủ)**. Doanh nghiệp sở hữu 1,25 đồng tài sản nhanh cho mỗi 1,00 đồng nợ ngắn hạn.

3. **Cảnh Báo Thâm Hụt Ngân Quỹ Cục Bộ (T+10 Cash Shortfall Alert)**:
   - **Nguy cơ**: Dù khả năng trả nợ cả năm tốt (DSCR = 2.0), dòng tiền thực tế sẽ bị **âm 50.000.000 VND vào Ngày 10** do tập trung chi trả lương và nghĩa vụ thuế trước khi tiền hàng về.
   - **Điểm đáy dòng tiền (Minimum Balance)**: $-50.000.000$ VND.
   - **Khuyến nghị xử lý**:
     - Cấp hạn mức thấu chi ngắn hạn (Overdraft facility) tối thiểu 200.000.000 VND từ ngày T+8 đến T+15.
     - Đẩy sớm tiến độ thu hồi công nợ 150M ở ngày T+3 để bù đắp điểm trũng dòng tiền.
```

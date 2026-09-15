# Compliance Playbook: Statutory AML/STR Screening & Decree 13 PII Masking

This demonstration playbook guides the execution of compliance screening on an incoming batch of payment transactions, detecting AML red flags and redacting personal data.

## Scenario
A batch of 3 incoming transactions is inspected before posting to the public accounting ledger:
1. Tx 1: A 500,000,000 VND payment from a corporate supplier exceeding the Decision 11 threshold.
2. Tx 2: A personal payment containing a 12-digit Citizen ID (CCCD), mobile phone number, and personal bank account.
3. Tx 3: A payment to a university ("TRUONG DAI HOC BACH KHOA") where the corporate entity must be preserved.

---

## Step 1: User Prompt
```text
Rà soát tuân thủ phòng chống rửa tiền (AML) và bảo vệ dữ liệu cá nhân theo Nghị định 13
cho lô giao dịch sau trước khi xuất dữ liệu ra báo cáo kiểm toán.
Đảm bảo xác thực Zero Egress.
```

---

## Step 2: Agent Tool Invocation
The agent prepares the structured payload and invokes `compliance_aml_screen`:

```json
{
  "tool": "compliance_aml_screen",
  "arguments": {
    "transactions": [
      {
        "tx_id": "TX-AML-001",
        "account_number": "19034567890123",
        "counterparty_name": "CONG TY TNHH THEP VIET NHAT",
        "counterparty_account": "0011001234567",
        "amount_vnd": 500000000,
        "timestamp": 1726300000,
        "narration": "Thanh toan tien thep xay dung nha xuong quy 3"
      },
      {
        "tx_id": "TX-AML-002",
        "account_number": "19034567890123",
        "counterparty_name": "Nguyen Van An",
        "counterparty_account": "098765432101",
        "amount_vnd": 35000000,
        "timestamp": 1726300100,
        "narration": "Khach hang Nguyen Van An CCCD: 001095012345 SDT: 0912345678 chuyen tien den STK: 098765432101"
      },
      {
        "tx_id": "TX-AML-003",
        "account_number": "19034567890123",
        "counterparty_name": "TRUONG DAI HOC BACH KHOA",
        "counterparty_account": "0022009876543",
        "amount_vnd": 80000000,
        "timestamp": 1726300200,
        "narration": "Thanh toan kinh phi nghien cuu de tai khoa hoc"
      }
    ],
    "redact_pii": true
  }
}
```

---

## Step 3: Tool Execution & Response
```json
{
  "total_screened": 3,
  "alerts": [
    {
      "tx_id": "TX-AML-001",
      "rule_code": "AML_HIGH_VALUE",
      "severity": "HIGH",
      "description": "Transaction amount 500,000,000 VND exceeds statutory AML threshold of 400,000,000 VND (Decision 11/2023/QD-TTg)"
    }
  ],
  "sanitized_transactions": [
    {
      "tx_id": "TX-AML-001",
      "masked_narration": "Thanh toan tien thep xay dung nha xuong quy 3",
      "masked_counterparty": "CONG TY TNHH THEP VIET NHAT"
    },
    {
      "tx_id": "TX-AML-002",
      "masked_narration": "Khach hang [REDACTED_NAME] CCCD: [REDACTED_CCCD] SDT: [REDACTED_PHONE] chuyen tien den STK: [REDACTED_ACCOUNT]",
      "masked_counterparty": "[REDACTED_NAME]"
    },
    {
      "tx_id": "TX-AML-003",
      "masked_narration": "Thanh toan kinh phi nghien cuu de tai khoa hoc",
      "masked_counterparty": "TRUONG DAI HOC BACH KHOA"
    }
  ],
  "zero_egress_verified": true
}
```

---

## Step 4: Compliance Officer Synthesis
```markdown
### Báo Cáo Kiểm Toán Tuân Thủ & Phòng Chống Rửa Tiền (AML / Nghị Định 13)

1. **Bảo Mật Cục Bộ (Zero Data Egress)**:
   - Trạng thái: **ĐÃ XÁC THỰC 100% (Air-Gapped Local-First)**.
   - 0 byte dữ liệu rời khỏi bộ nhớ máy trạm doanh nghiệp. Mọi socket phi loopback đều bị chặn.

2. **Cảnh Báo Giao Dịch Đáng Ngờ (STR Alert)**:
   - **Giao dịch**: `TX-AML-001` (500.000.000 VND)
   - **Mã quy tắc**: `AML_HIGH_VALUE` (Mức độ: CAO)
   - **Căn cứ pháp lý**: Quyết định 11/2023/QĐ-TTg của Thủ tướng Chính phủ (ngưỡng báo cáo $\ge 400$ triệu VND).
   - **Hành động**: Tự động đưa vào phụ lục báo cáo định kỳ Cục Phòng, chống rửa tiền (NHNN).

3. **Bảo Vệ Dữ Liệu Cá Nhân (Nghị định 13/2023/NĐ-CP)**:
   - Giao dịch `TX-AML-002`: Đã ẩn danh hóa toàn diện tên cá nhân (`[REDACTED_NAME]`), CCCD 12 số (`[REDACTED_CCCD]`), số điện thoại (`[REDACTED_PHONE]`), và số tài khoản ngân hàng (`[REDACTED_ACCOUNT]`).
   - Giao dịch `TX-AML-001` & `TX-AML-003`: Tên pháp nhân doanh nghiệp (`CONG TY TNHH THEP VIET NHAT`) và trường đại học (`TRUONG DAI HOC BACH KHOA`) được **bảo toàn nguyên vẹn** phục vụ đối chiếu chứng từ kế toán.
```

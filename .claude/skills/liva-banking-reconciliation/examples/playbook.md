# Reconciliation Playbook: Multi-Bank Statement Ingestion & Ledger Matching

This demonstration playbook guides the execution of an end-to-end bank statement reconciliation cycle against an enterprise ERP ledger.

## Scenario
The finance department receives monthly statement files from Vietcombank and Techcombank. The operator needs to reconcile 110 open ERP invoices against 110 bank transactions, identify exact matches, accommodate wire fees, resolve batch payments, and quarantine unmatched discrepancies.

---

## Step 1: User Prompt
```text
Đối soát sao kê tài khoản Vietcombank tháng 8/2026 tại file 'fixtures/statements/vcb_aug2026.xlsx'
với sổ cái ERP tại 'fixtures/erp_ledger/open_invoices.json'.
Dung sai phí chuyển khoản tối đa 11.000 VND.
```

---

## Step 2: Agent Tool Invocation
The agent analyzes the prompt, identifies the target statement and ledger paths, and invokes `banking_reconcile`:

```json
{
  "tool": "banking_reconcile",
  "arguments": {
    "statement_file_path": "fixtures/statements/vcb_aug2026.xlsx",
    "statement_format": "vcb_excel",
    "erp_ledger_path": "fixtures/erp_ledger/open_invoices.json",
    "fee_tolerance_vnd": 11000
  }
}
```

---

## Step 3: Tool Execution & Response
```json
{
  "statement_summary": {
    "bank": "Vietcombank",
    "account": "0011001234567",
    "opening_balance": 1400000000,
    "closing_balance": 1525000000,
    "total_credit": 200000000,
    "total_debit": 75000000,
    "tx_count": 50
  },
  "balance_invariant_valid": true,
  "matched_exact_count": 35,
  "matched_fuzzy_count": 10,
  "matched_split_count": 4,
  "discrepancies_count": 1,
  "hitl_quarantined": [
    {
      "tx_id": "VCB-TX-20260830-049",
      "amount": 25000000,
      "reason": "Unallocated credit transfer: Narration 'Chuyen tien mua hang' does not match any open invoice reference",
      "hitl_token": "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d"
    }
  ]
}
```

---

## Step 4: Agent Synthesis & Output to User
```markdown
### Báo Cáo Đối Soát Sao Kê Vietcombank (Tháng 8/2026)

1. **Kiểm Tra Tính Toàn Vẹn Số Học**:
   - Số dư đầu kỳ: 1.400.000.000 VND
   - Tổng phát sinh Có (+): 200.000.000 VND
   - Tổng phát sinh Nợ (-): 75.000.000 VND
   - Số dư cuối kỳ: 1.525.000.000 VND
   - **Trạng thái**: KHỚP TUYỆT ĐỐI ($\Delta = 0$ VND, 0 floating-point drift).

2. **Kết Quả Phân Loại 3 Tầng**:
   - **Tầng 1 (Khớp chính xác 1:1)**: 35 giao dịch (100% tự động, đúng mã chứng từ và số tiền).
   - **Tầng 2 (Khớp mờ Heuristic & Phí Napas)**: 10 giao dịch (Chênh lệch phí chuyển khoản 1.100 - 11.000 VND được ghi nhận đúng chuẩn).
   - **Tầng 3 (Khớp gộp Split 1:N / N:1)**: 4 giao dịch (Đã giải bài toán subset-sum cho các khoản thanh toán nhiều hóa đơn gộp).
   - **Tỷ lệ khớp tự động**: 98% (49/50 giao dịch).

3. **Giao Dịch Cách Ly Cần Phê Duyệt Thủ Công (HITL Quarantine)**:
   - **Mã giao dịch**: `VCB-TX-20260830-049`
   - **Số tiền**: 25.000.000 VND
   - **Lý do**: Nội dung chuyển khoản chung chung ("Chuyen tien mua hang"), không khớp mã HĐ mở.
   - **HITL Token**: `a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d` (Hiệu lực: 15 phút).
   - **Đề xuất**: Chuyển sang vai trò Maker-Checker (`liva-banking-treasury`) để kế toán trưởng xác minh đối tác và chỉ định hóa đơn đối ứng.
```

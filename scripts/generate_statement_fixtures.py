#!/usr/bin/env python3
"""
generate_statement_fixtures.py - Generates realistic Vietnamese banking statement fixtures:
1. fixtures/statements/vcb_aug2026.xlsx (50 VCB transactions, calamine-compatible)
2. fixtures/statements/tcb_aug2026.csv (60 TCB transactions, UTF-8 BOM, Napas/VietQR)
3. fixtures/statements/open_invoices.json (120 internal ERP invoices)
4. fixtures/erp_ledger/open_invoices.json (identical copy for ERP ledger)

Designed for Milestone 4 of LIVA Banking Harness.
"""

import json
import os
import random
from datetime import datetime, timedelta
import xlsxwriter

# Ensure output directories exist
STATEMENTS_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "fixtures", "statements"))
ERP_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "fixtures", "erp_ledger"))
os.makedirs(STATEMENTS_DIR, exist_ok=True)
os.makedirs(ERP_DIR, exist_ok=True)

def fmt_vnd(amount: int, include_decimals: bool = True) -> str:
    """Formats amount with dot-thousands e.g. 15.000.000,00 or 15.000.000"""
    s = f"{amount:,}".replace(",", ".")
    if include_decimals:
        return f"{s},00"
    return s

# Base dates: August 2026 (Unix timestamps in seconds)
# 2026-08-01 00:00:00 UTC = 1785542400
BASE_TIMESTAMP = 1785542400

# ---------------------------------------------------------------------------
# 1. GENERATE OPEN INVOICES (120 items)
# ---------------------------------------------------------------------------
# Breakdown:
# - 82 exact 1:1 match invoices (HD101 to HD182)
# - 20 fuzzy heuristic invoices (HD183 to HD202 with Napas fee diffs and name variants)
# - 15 composite split invoices:
#     Split A (VCB): HD203 (40M), HD204 (50M), HD205 (30M) -> Total 120M
#     Split B (VCB): HD206 (35M), HD207 (50M) -> Total 85M
#     Split C (TCB): HD208 (70M), HD209 (80M) -> Total 150M
#     Split D (TCB): HD210 (25M), HD211 (30M), HD212 (40M) -> Total 95M
#     Split E (BIDV): HD213 (30M), HD214 (30M), HD215 (30M) -> Total 90M
#     Split F (BIDV): HD216 (35M), HD217 (30M) -> Total 65M
# - 3 unresolved discrepancy items (INV-2026-118, INV-2026-119, INV-2026-120)
#   failing closed to 0.2% HITL review queue.
# Total = 82 + 20 + 15 + 3 = 120 invoices.

invoices = []

# List of realistic Vietnamese corporate partners
PARTNERS = [
    ("CUST001", "Công ty TNHH Thương mại Dịch vụ An Phát", "CONG TY TNHH TM DV AN PHAT"),
    ("CUST002", "Công ty Cổ phần Xây dựng và Địa ốc Hòa Bình", "CONG TY CP XD VA DIA OC HOA BINH"),
    ("CUST003", "Công ty TNHH Công nghệ Thông tin FPT Smart Cloud", "CONG TY TNHH FPT SMART CLOUD"),
    ("CUST004", "Công ty Cổ phần Tập đoàn Masan", "CONG TY CP TAP DOAN MASAN"),
    ("CUST005", "Công ty TNHH Thép Việt Nhật", "CONG TY TNHH THEP VIET NHAT"),
    ("CUST006", "Công ty Cổ phần Sữa Việt Nam (Vinamilk)", "CONG TY CP SUA VIET NAM"),
    ("CUST007", "Công ty TNHH Logistics Vận tải Biển Đông", "CONG TY TNHH LOGISTICS BIEN DONG"),
    ("CUST008", "Công ty Cổ phần Hóa chất Đức Giang", "CONG TY CP HOA CHAT DUC GIANG"),
    ("CUST009", "Công ty TNHH Xuất nhập khẩu Hoàng Gia", "CONG TY TNHH XNK HOANG GIA"),
    ("CUST010", "Công ty Cổ phần Thiết bị Y tế Phương Đông", "CONG TY CP THIET BI Y TE PHUONG DONG"),
    ("CUST011", "Công ty TNHH Dệt may Phong Phú", "CONG TY TNHH DET MAY PHONG PHU"),
    ("CUST012", "Công ty Cổ phần Nhựa Tiền Phong", "CONG TY CP NHUA TIEN PHONG"),
    ("CUST013", "Công ty TNHH Nông sản Trung An", "CONG TY TNHH NONG SAN TRUNG AN"),
    ("CUST014", "Công ty Cổ phần Tập đoàn Đèo Cả", "CONG TY CP TAP DOAN DEO CA"),
    ("CUST015", "Công ty TNHH Cơ điện lạnh Đại Việt", "CONG TY TNHH CO DIEN LANH DAI VIET"),
]

# A. Exact Invoices (1..82)
exact_amounts = [
    12500000, 18000000, 24000000, 32000000, 45000000, 16500000, 28000000, 37500000, 52000000, 19000000,
    14000000, 22500000, 31000000, 41000000, 55000000, 13500000, 26000000, 34500000, 48000000, 17000000,
    15500000, 21000000, 29500000, 38000000, 49000000, 18500000, 27000000, 36000000, 51000000, 20000000,
    11500000, 23000000, 33000000, 42000000, 58000000, 14500000, 25500000, 35000000, 47000000, 17500000,
    13000000, 22000000, 30500000, 39500000, 53000000, 16000000, 26500000, 36500000, 46000000, 19500000,
    12000000, 24500000, 31500000, 43500000, 56000000, 15000000, 28500000, 37000000, 50000000, 21500000,
    10500000, 23500000, 32500000, 44000000, 57000000, 14000000, 27500000, 38500000, 49500000, 18000000,
    13500000, 21500000, 33500000, 41500000, 54000000, 16500000, 29000000, 39000000, 51500000, 22500000,
    17000000, 26000000
]
assert len(exact_amounts) == 82

for i in range(82):
    inv_num = 101 + i
    partner = PARTNERS[i % len(PARTNERS)]
    # account mapping: first 30 to VCB, next 34 to TCB, remaining 18 to BIDV
    if i < 30:
        acc = "acc_vcb"
    elif i < 64:
        acc = "acc_tcb"
    else:
        acc = "acc_bidv"
    
    day = 1 + (i % 28)
    ts = BASE_TIMESTAMP + (day - 1) * 86400 + random.randint(300, 3600)
    invoices.append({
        "id": f"INV-2026-{inv_num:04d}",
        "account_id": acc,
        "doc_no": f"HD{inv_num}",
        "entry_date": ts,
        "entry_type": "CREDIT",
        "amount": exact_amounts[i],
        "partner_code": partner[0],
        "partner_name": partner[1],
        "description": f"Ban hang theo hop dong HD{inv_num}",
        "reconciled_status": "UNMATCHED",
        "created_at": ts
    })

# B. Fuzzy Invoices (83..102 -> HD183 to HD202)
# Amounts with fee differences (1,100 to 11,000 VND)
fuzzy_base_amounts = [
    50000000, 40000000, 35000000, 60000000, 45000000, 55000000, 30000000, 65000000, 70000000, 38000000,
    42000000, 48000000, 52000000, 36000000, 58000000, 62000000, 44000000, 46000000, 54000000, 68000000
]
assert len(fuzzy_base_amounts) == 20

for i in range(20):
    inv_num = 183 + i
    partner = PARTNERS[(i + 3) % len(PARTNERS)]
    if i < 7:
        acc = "acc_vcb"
    elif i < 15:
        acc = "acc_tcb"
    else:
        acc = "acc_bidv"
    
    day = 2 + (i % 26)
    ts = BASE_TIMESTAMP + (day - 1) * 86400 + random.randint(600, 4000)
    invoices.append({
        "id": f"INV-2026-{inv_num:04d}",
        "account_id": acc,
        "doc_no": f"HD{inv_num}",
        "entry_date": ts,
        "entry_type": "CREDIT",
        "amount": fuzzy_base_amounts[i],
        "partner_code": partner[0],
        "partner_name": partner[1],
        "description": f"Cung cap dich vu theo HD{inv_num}",
        "reconciled_status": "UNMATCHED",
        "created_at": ts
    })

# C. Composite Split Invoices (103..117 -> HD203 to HD217)
# Split A (VCB): HD203 (40M), HD204 (50M), HD205 (30M) -> Total 120M
# Split B (VCB): HD206 (35M), HD207 (50M) -> Total 85M
# Split C (TCB): HD208 (70M), HD209 (80M) -> Total 150M
# Split D (TCB): HD210 (25M), HD211 (30M), HD212 (40M) -> Total 95M
# Split E (BIDV): HD213 (30M), HD214 (30M), HD215 (30M) -> Total 90M
# Split F (BIDV): HD216 (35M), HD217 (30M) -> Total 65M
splits_spec = [
    # (inv_num, amount, acc, partner_idx, day)
    (203, 40000000, "acc_vcb", 0, 10),
    (204, 50000000, "acc_vcb", 0, 10),
    (205, 30000000, "acc_vcb", 0, 10),

    (206, 35000000, "acc_vcb", 4, 15),
    (207, 50000000, "acc_vcb", 4, 15),

    (208, 70000000, "acc_tcb", 1, 12),
    (209, 80000000, "acc_tcb", 1, 12),

    (210, 25000000, "acc_tcb", 4, 18),
    (211, 30000000, "acc_tcb", 4, 18),
    (212, 40000000, "acc_tcb", 4, 18),

    (213, 30000000, "acc_bidv", 14, 14),
    (214, 30000000, "acc_bidv", 14, 14),
    (215, 30000000, "acc_bidv", 14, 14),

    (216, 35000000, "acc_bidv", 8, 20),
    (217, 30000000, "acc_bidv", 8, 20),
]
assert len(splits_spec) == 15

for item in splits_spec:
    inv_num, amt, acc, p_idx, day = item
    partner = PARTNERS[p_idx]
    ts = BASE_TIMESTAMP + (day - 1) * 86400 + 7200
    invoices.append({
        "id": f"INV-2026-{inv_num:04d}",
        "account_id": acc,
        "doc_no": f"HD{inv_num}",
        "entry_date": ts,
        "entry_type": "CREDIT",
        "amount": amt,
        "partner_code": partner[0],
        "partner_name": partner[1],
        "description": f"Ban hang thanh toan gop dot thang 8 HD{inv_num}",
        "reconciled_status": "UNMATCHED",
        "created_at": ts
    })

# D. Exactly 3 Unresolved Discrepancy Invoices (118..120)
# These represent genuine open receivables that failed match or remain uncollected,
# failing closed to the 0.2% HITL review queue.
invoices.append({
    "id": "INV-2026-0991",
    "account_id": "acc_vcb",
    "doc_no": "HD991",
    "entry_date": BASE_TIMESTAMP + 12 * 86400,
    "entry_type": "CREDIT",
    "amount": 4500000,
    "partner_code": "CUST099",
    "partner_name": "Công ty TNHH Quảng cáo Sao Mai",
    "description": "Khoan phat sinh chua nhan thanh toan tu doi tac Sao Mai",
    "reconciled_status": "UNMATCHED",
    "created_at": BASE_TIMESTAMP + 12 * 86400
})
invoices.append({
    "id": "INV-2026-0992",
    "account_id": "acc_tcb",
    "doc_no": "HD992",
    "entry_date": BASE_TIMESTAMP + 17 * 86400,
    "entry_type": "CREDIT",
    "amount": 12000000,
    "partner_code": "CUST098",
    "partner_name": "Công ty Cổ phần Đầu tư Bất động sản Thiên An",
    "description": "Hoa don cho xac nhan doi chieu Cong no thang 8",
    "reconciled_status": "UNMATCHED",
    "created_at": BASE_TIMESTAMP + 17 * 86400
})
invoices.append({
    "id": "INV-2026-0993",
    "account_id": "acc_bidv",
    "doc_no": "HD993",
    "entry_date": BASE_TIMESTAMP + 24 * 86400,
    "entry_type": "CREDIT",
    "amount": 28000000,
    "partner_code": "CUST097",
    "partner_name": "Công ty TNHH Vận tải Du lịch Biển Xanh",
    "description": "Hoa don dich vu chua chuyen tien tai khoan BIDV",
    "reconciled_status": "UNMATCHED",
    "created_at": BASE_TIMESTAMP + 24 * 86400
})

assert len(invoices) == 120, f"Expected exactly 120 invoices, got {len(invoices)}"

# Save open_invoices.json
open_invoices_path = os.path.join(STATEMENTS_DIR, "open_invoices.json")
with open(open_invoices_path, "w", encoding="utf-8") as f:
    json.dump(invoices, f, ensure_ascii=False, indent=2)

erp_open_invoices_path = os.path.join(ERP_DIR, "open_invoices.json")
with open(erp_open_invoices_path, "w", encoding="utf-8") as f:
    json.dump(invoices, f, ensure_ascii=False, indent=2)

print(f"[OK] Generated {len(invoices)} open invoices at:")
print(f"     -> {open_invoices_path}")
print(f"     -> {erp_open_invoices_path}")

# ---------------------------------------------------------------------------
# 2. GENERATE VCB EXCEL FIXTURE (vcb_aug2026.xlsx - 50 transactions)
# ---------------------------------------------------------------------------
# Initial Balance: 1,450,230,000 VND (Pitch CEO Step 1: VCB 1.45B)
# Structure:
# - Rows 0..8: Vietcombank header metadata
# - Row 10: Column headers
# - Rows 11..60: 50 transactions:
#     - 30 exact 1:1 match credits (HD101..HD130)
#     - 7 fuzzy heuristic credits (HD183..HD189, with fee deductions 1,100..11,000)
#     - 2 composite split credits (Split A 120M, Split B 85M)
#     - 1 unallocated discrepancy credit (380,000 VND - HITL quarantine)
#     - 10 operational expense debits
#   Total = 30 + 7 + 2 + 1 + 10 = 50 transactions!
# - Row 61: Summary footer with totals & closing balance

vcb_path = os.path.join(STATEMENTS_DIR, "vcb_aug2026.xlsx")
wb = xlsxwriter.Workbook(vcb_path)
ws = wb.add_worksheet("SoPhuGiaoDich")

# Formatting
fmt_title = wb.add_format({"bold": True, "font_size": 14, "font_name": "Arial", "color": "#005522"})
fmt_meta = wb.add_format({"font_name": "Arial", "font_size": 10})
fmt_meta_bold = wb.add_format({"bold": True, "font_name": "Arial", "font_size": 10})
fmt_header = wb.add_format({"bold": True, "font_name": "Arial", "font_size": 10, "bg_color": "#DDEEDD", "border": 1, "align": "center"})
fmt_data = wb.add_format({"font_name": "Arial", "font_size": 9, "border": 1})
fmt_data_num = wb.add_format({"font_name": "Arial", "font_size": 9, "border": 1, "align": "right"})
fmt_total = wb.add_format({"bold": True, "font_name": "Arial", "font_size": 10, "bg_color": "#EFEFEF", "border": 1})
fmt_total_num = wb.add_format({"bold": True, "font_name": "Arial", "font_size": 10, "bg_color": "#EFEFEF", "border": 1, "align": "right"})

# Header Metadata
ws.write(0, 0, "NGÂN HÀNG THƯƠNG MẠI CỔ PHẦN NGOẠI THƯƠNG VIỆT NAM (VIETCOMBANK)", fmt_title)
ws.write(1, 0, "SỔ PHỤ CHI TIẾT TÀI KHOẢN TIỀN GỬI THANH TOÁN", fmt_meta_bold)
ws.write(2, 0, "Số tài khoản: 0011001234567", fmt_meta_bold)
ws.write(3, 0, "Tên tài khoản: CONG TY TNHH LIVA SOLUTIONS", fmt_meta_bold)
ws.write(4, 0, "Loại tiền: VND", fmt_meta)
ws.write(5, 0, "Kỳ sao kê: Từ ngày 01/08/2026 đến ngày 31/08/2026", fmt_meta)
ws.write(6, 0, "Số dư đầu kỳ: 1.450.230.000,00", fmt_meta_bold)

# Column Headers (Row 9, 0-indexed)
headers = [
    "Ngày giao dịch",
    "Ngày giá trị",
    "Số chứng từ",
    "Số tiền ghi nợ",
    "Số tiền ghi có",
    "Số dư",
    "Nội dung giao dịch",
    "Tên đối tác"
]
for col_idx, h in enumerate(headers):
    ws.write(9, col_idx, h, fmt_header)
    ws.set_column(col_idx, col_idx, 18)

ws.set_column(6, 6, 45) # Narration column wider
ws.set_column(7, 7, 35) # Partner column wider

vcb_running_balance = 1450230000
vcb_sum_credit = 0
vcb_sum_debit = 0
vcb_tx_count = 0

vcb_tx_rows = []

# 1. 30 Exact match credits
for i in range(30):
    inv = invoices[i]
    day = 1 + i
    date_str = f"{day:02d}/08/2026"
    doc_ref = inv["doc_no"]
    amt = inv["amount"]
    vcb_sum_credit += amt
    vcb_running_balance += amt
    vcb_tx_rows.append({
        "date": date_str,
        "val_date": date_str,
        "ref": f"VCB{doc_ref}",
        "debit": "",
        "credit": fmt_vnd(amt),
        "balance": fmt_vnd(vcb_running_balance),
        "narration": f"THANH TOAN TIEN HANG {doc_ref}",
        "partner": inv["partner_name"]
    })

# 2. 7 Fuzzy heuristic credits (HD183..HD189) with Napas fee deductions (1,100..11,000 VND)
fee_schedule = [1100, 2200, 3300, 5500, 7700, 8800, 11000]
for i in range(7):
    inv = invoices[82 + i]
    fee = fee_schedule[i]
    amt = inv["amount"] - fee # Net amount deposited after fee
    day = 3 + i * 3
    date_str = f"{day:02d}/08/2026"
    doc_ref = inv["doc_no"]
    vcb_sum_credit += amt
    vcb_running_balance += amt
    vcb_tx_rows.append({
        "date": date_str,
        "val_date": date_str,
        "ref": f"VCB{doc_ref}",
        "debit": "",
        "credit": fmt_vnd(amt),
        "balance": fmt_vnd(vcb_running_balance),
        "narration": f"CK TIEN DICH VU {doc_ref} DA TRU PHI GD",
        "partner": inv["partner_name"].upper().replace("CÔNG TY", "CTY").replace("TNHH", "")
    })

# 3. 2 Composite split credits
# Split A: HD203 (40M) + HD204 (50M) + HD205 (30M) = 120M
split_a_amt = 120000000
vcb_sum_credit += split_a_amt
vcb_running_balance += split_a_amt
vcb_tx_rows.append({
    "date": "10/08/2026",
    "val_date": "10/08/2026",
    "ref": "VCBSPLIT203",
    "debit": "",
    "credit": fmt_vnd(split_a_amt),
    "balance": fmt_vnd(vcb_running_balance),
    "narration": "CONG TY AN PHAT CK THANH TOAN GOP HD203 HD204 HD205",
    "partner": "CONG TY TNHH TM DV AN PHAT"
})

# Split B: HD206 (35M) + HD207 (50M) = 85M
split_b_amt = 85000000
vcb_sum_credit += split_b_amt
vcb_running_balance += split_b_amt
vcb_tx_rows.append({
    "date": "15/08/2026",
    "val_date": "15/08/2026",
    "ref": "VCBSPLIT206",
    "debit": "",
    "credit": fmt_vnd(split_b_amt),
    "balance": fmt_vnd(vcb_running_balance),
    "narration": "CONG TY THEP VIET NHAT CK HD206 VA HD207",
    "partner": "CONG TY TNHH THEP VIET NHAT"
})

# 4. 1 Unallocated discrepancy credit: 380,000 VND (Matches UI Mockup VCB Chưa khớp 380,000 VND)
disc_vcb_amt = 380000
vcb_sum_credit += disc_vcb_amt
vcb_running_balance += disc_vcb_amt
vcb_tx_rows.append({
    "date": "20/08/2026",
    "val_date": "20/08/2026",
    "ref": "VCBDISC001",
    "debit": "",
    "credit": fmt_vnd(disc_vcb_amt),
    "balance": fmt_vnd(vcb_running_balance),
    "narration": "Chuyen tien phi duy tri tai khoan khong ro nguon goc",
    "partner": "KHACH HANG VAN LA"
})

# 5. 10 Operational expense debits
debits_spec = [
    (15000000, "THANH TOAN TIEN DIEN VAN PHONG THANG 8", "CONG TY DIEN LUC HA NOI"),
    (8500000, "THANH TOAN TIEN NUOC VA DICH VU TOA NHA", "BAN QUAN LY TOA NHA LIVA TOWER"),
    (45000000, "NOP THUE GTGT VA TNDN THANG 7", "KHO BAC NHA NUOC TP HA NOI"),
    (55000000, "CHI TRA LUONG CAN BO NHAN VIEN THANG 7 DOT 2", "CONG TY TNHH LIVA SOLUTIONS"),
    (3500000, "PHI DICH VU NGAN HANG VCB DIGITAL THANG 8", "NGAN HANG VIETCOMBANK"),
    (18000000, "THANH TOAN DICH VU CLOUD HA TANG LOCAL ON PREM", "CONG TY TNHH FPT SMART CLOUD"),
    (9200000, "CHI PHI TIEP KHACH HOI NGHỊ INNOSTART 2026", "NHA HANG PHUONG DONG"),
    (12500000, "CHI PHI VAN PHONG PHAM VA MUC IN", "CONG TY CP THIET BI VAN PHONG ANH DUONG"),
    (24000000, "CHI PHI THUE SERVER VA AN NINH MANG", "TAP DOAN BUU CHINH VIEN THONG VNPT"),
    (14300000, "PHI BAO HIEM XA HOI VA Y TE THANG 8", "BAO HIEM XA HOI TP HA NOI")
]

for idx, (amt, narr, part) in enumerate(debits_spec):
    day = 2 + idx * 2
    date_str = f"{day:02d}/08/2026"
    vcb_sum_debit += amt
    vcb_running_balance -= amt
    vcb_tx_rows.append({
        "date": date_str,
        "val_date": date_str,
        "ref": f"VCBDEB{idx+1:03d}",
        "debit": fmt_vnd(amt),
        "credit": "",
        "balance": fmt_vnd(vcb_running_balance),
        "narration": narr,
        "partner": part
    })

assert len(vcb_tx_rows) == 50, f"Expected 50 VCB transactions, got {len(vcb_tx_rows)}"

# Write 50 rows to worksheet
start_row = 10
for r_offset, tx in enumerate(vcb_tx_rows):
    r = start_row + r_offset
    ws.write(r, 0, tx["date"], fmt_data)
    ws.write(r, 1, tx["val_date"], fmt_data)
    ws.write(r, 2, tx["ref"], fmt_data)
    ws.write(r, 3, tx["debit"], fmt_data_num)
    ws.write(r, 4, tx["credit"], fmt_data_num)
    ws.write(r, 5, tx["balance"], fmt_data_num)
    ws.write(r, 6, tx["narration"], fmt_data)
    ws.write(r, 7, tx["partner"], fmt_data)

# Summary row at row 60 (61st row)
ws.write(60, 0, "Tổng cộng phát sinh:", fmt_total)
ws.write(60, 1, "", fmt_total)
ws.write(60, 2, "", fmt_total)
ws.write(60, 3, fmt_vnd(vcb_sum_debit), fmt_total_num)
ws.write(60, 4, fmt_vnd(vcb_sum_credit), fmt_total_num)
ws.write(60, 5, fmt_vnd(vcb_running_balance), fmt_total_num)
ws.write(60, 6, "Số dư cuối kỳ: " + fmt_vnd(vcb_running_balance), fmt_total)
ws.write(60, 7, "", fmt_total)

wb.close()
print(f"[OK] Generated {len(vcb_tx_rows)} VCB transactions in {vcb_path}")
print(f"     Opening: 1,450,230,000 | Credits: {vcb_sum_credit:,} | Debits: {vcb_sum_debit:,} | Closing: {vcb_running_balance:,}")

# ---------------------------------------------------------------------------
# 3. GENERATE TCB CSV FIXTURE (tcb_aug2026.csv - 60 transactions)
# ---------------------------------------------------------------------------
# Initial Balance: 785,600,000 VND (Pitch CEO Step 1: TCB 785.6M)
# Must include:
# - UTF-8 BOM (\xEF\xBB\xBF)
# - Semicolon delimiters
# - 60 transactions:
#     - 34 exact matches (HD131..HD164)
#     - 8 fuzzy heuristic matches (HD190..HD197 with Napas fee diffs)
#     - 2 composite split matches (Split C 150M, Split D 95M)
#     - 1 unallocated discrepancy transaction: 500,000 VND (Matches UI Mockup TCB Chưa khớp 500,000 VND)
#     - 15 operational expense debits
#   Total = 34 + 8 + 2 + 1 + 15 = 60 transactions!
# - Napas/VietQR references in narration (FT..., NPS..., QRIBFT...)

tcb_path = os.path.join(STATEMENTS_DIR, "tcb_aug2026.csv")
tcb_running_balance = 785600000
tcb_sum_credit = 0
tcb_sum_debit = 0

tcb_rows = []

# Header rows
tcb_lines = [
    "Số tài khoản:;19034567890123;;;;;;\n",
    "Tên tài khoản:;CONG TY TNHH LIVA SOLUTIONS;;;;;;\n",
    "Số dư đầu kỳ:;785.600.000;;;;;;\n",
    "Kỳ sao kê:;01/08/2026 - 31/08/2026;;;;;;\n",
    "Ngày giao dịch;Mã giao dịch;Số tiền ghi nợ;Số tiền ghi có;Số dư;Nội dung chi tiết;Tên đối tác\n"
]

# 1. 34 Exact match credits (invoices 30..63)
for i in range(34):
    inv = invoices[30 + i]
    day = 1 + (i % 28)
    hour = 8 + (i % 10)
    minute = (i * 7) % 60
    date_str = f"{day:02d}/08/2026 {hour:02d}:{minute:02d}:00"
    doc_ref = inv["doc_no"]
    amt = inv["amount"]
    tcb_sum_credit += amt
    tcb_running_balance += amt
    ft_code = f"FT2624{i+1:06d}"
    tcb_rows.append({
        "date": date_str,
        "code": ft_code,
        "debit": "",
        "credit": fmt_vnd(amt, False),
        "balance": fmt_vnd(tcb_running_balance, False),
        "narration": f"Napas VietQR TT {doc_ref} Tu {inv['partner_name']}",
        "partner": inv["partner_name"]
    })

# 2. 8 Fuzzy heuristic credits (HD190..HD197)
tcb_fees = [1100, 2200, 3300, 4400, 5500, 7700, 9900, 11000]
for i in range(8):
    inv = invoices[89 + i]
    fee = tcb_fees[i]
    amt = inv["amount"] - fee
    day = 2 + i * 3
    date_str = f"{day:02d}/08/2026 10:15:30"
    doc_ref = inv["doc_no"]
    tcb_sum_credit += amt
    tcb_running_balance += amt
    nps_code = f"NPS2608{i+1:04d}"
    tcb_rows.append({
        "date": date_str,
        "code": nps_code,
        "debit": "",
        "credit": fmt_vnd(amt, False),
        "balance": fmt_vnd(tcb_running_balance, False),
        "narration": f"QRIBFT TT {doc_ref} DA TRU PHI CHUYEN KHOAN NAPAS",
        "partner": inv["partner_name"].replace("Công ty Cổ phần", "CTY CP").replace("Công ty TNHH", "CTY TNHH")
    })

# 3. 2 Composite split matches
# Split C: HD208 (70M) + HD209 (80M) = 150M
split_c_amt = 150000000
tcb_sum_credit += split_c_amt
tcb_running_balance += split_c_amt
tcb_rows.append({
    "date": "12/08/2026 14:20:00",
    "code": "FT2624991001",
    "debit": "",
    "credit": fmt_vnd(split_c_amt, False),
    "balance": fmt_vnd(tcb_running_balance, False),
    "narration": "Napas VietQR TT HD208 VA HD209 TU CONG TY CP XD VA DIA OC HOA BINH",
    "partner": "CONG TY CP XD VA DIA OC HOA BINH"
})

# Split D: HD210 (25M) + HD211 (30M) + HD212 (40M) = 95M
split_d_amt = 95000000
tcb_sum_credit += split_d_amt
tcb_running_balance += split_d_amt
tcb_rows.append({
    "date": "18/08/2026 16:45:10",
    "code": "NPS99881122",
    "debit": "",
    "credit": fmt_vnd(split_d_amt, False),
    "balance": fmt_vnd(tcb_running_balance, False),
    "narration": "Napas VietQR chuyen khoan thanh toan hoa don HD210 HD211 HD212 THEP VIET NHAT",
    "partner": "CONG TY TNHH THEP VIET NHAT"
})

# 4. 1 Unallocated discrepancy credit: 500,000 VND (Matches UI Mockup TCB Chưa khớp 500,000 VND)
disc_tcb_amt = 500000
tcb_sum_credit += disc_tcb_amt
tcb_running_balance += disc_tcb_amt
tcb_rows.append({
    "date": "22/08/2026 11:30:00",
    "code": "FT2624999999",
    "debit": "",
    "credit": fmt_vnd(disc_tcb_amt, False),
    "balance": fmt_vnd(tcb_running_balance, False),
    "narration": "Phi thuong nien the Visa Corporate chua doi chieu TCB",
    "partner": "TECHCOMBANK CARDS DIVISION"
})

# 5. 15 Operational expense debits
tcb_debits_spec = [
    (12000000, "CHI PHI TIEP KHACH DU AN NGAN HANG TCB", "NHA HANG SEN TAY HO"),
    (6200000, "MUA VAN PHONG PHAM THANG 8", "NHA SACH TIEN PHONG"),
    (22000000, "THANH TOAN TIEN INTERNET VA HA TANG CLOUD", "CONG TY FPT TELECOM"),
    (18000000, "CHI PHI XANG XE VA DICH VU XE CONG TAC", "CONG TY CP MAI LINH"),
    (35000000, "TAM UNG CONG TAC PHI DU AN INNOSTART", "NGUYEN MINH TRI"),
    (14000000, "CHI PHI VE MAY BAY CONG TAC TP HCM", "VIETNAM AIRLINES"),
    (9800000, "CHI PHI KHACH SAN HOI THAO TECHFEST", "KHACH SAN REX SAI GON"),
    (4500000, "CUOC DIEN THOAI DOANH NGHIEP THANG 8", "VIETTEL TELECOM"),
    (2200000, "PHI SMS BANKING VA INTERNET BANKING TCB", "NGAN HANG TECHCOMBANK"),
    (50000000, "THANH TOAN NHA CUNG CAP BAO TRI PHAN MEM", "CONG TY TNHH CMC SOFTWARE"),
    (1200000, "PHI CHUYEN TIEN LIEN NGAN HANG NAPAS 247", "NGAN HANG TECHCOMBANK"),
    (8900000, "CHI PHI DANG KY THUONG HIEU VA SO HUU TRI TUE", "CUC SO HUU TRI TUE"),
    (16500000, "CHI PHI THIET KE AN PHAM MARKETING 2D", "CONG TY CP CREATIVE HUB"),
    (7500000, "CHI PHI CHUYEN PHAT NHANH HO SO THAU", "CONG TY CP VIETTEL POST"),
    (25000000, "THANH TOAN TIEN THUE THIET BI MAY CHU TEST", "CONG TY TNHH HANOI COMPUTER")
]

for idx, (amt, narr, part) in enumerate(tcb_debits_spec):
    day = 1 + idx * 2
    date_str = f"{day:02d}/08/2026 15:00:00"
    tcb_sum_debit += amt
    tcb_running_balance -= amt
    tcb_rows.append({
        "date": date_str,
        "code": f"VN2624DEB{idx+1:02d}",
        "debit": fmt_vnd(amt, False),
        "credit": "",
        "balance": fmt_vnd(tcb_running_balance, False),
        "narration": narr,
        "partner": part
    })

assert len(tcb_rows) == 60, f"Expected 60 TCB transactions, got {len(tcb_rows)}"

for row in tcb_rows:
    line = f"{row['date']};{row['code']};{row['debit']};{row['credit']};{row['balance']};{row['narration']};{row['partner']}\n"
    tcb_lines.append(line)

# Summary row
tcb_lines.append(f"Tổng phát sinh;;{fmt_vnd(tcb_sum_debit, False)};{fmt_vnd(tcb_sum_credit, False)};{fmt_vnd(tcb_running_balance, False)};Số dư cuối kỳ: {fmt_vnd(tcb_running_balance, False)};\n")

# Write CSV with UTF-8 BOM
with open(tcb_path, "wb") as f:
    f.write(b"\xEF\xBB\xBF") # UTF-8 BOM
    for line in tcb_lines:
        f.write(line.encode("utf-8"))

print(f"[OK] Generated {len(tcb_rows)} TCB transactions in {tcb_path}")
print(f"     Opening: 785,600,000 | Credits: {tcb_sum_credit:,} | Debits: {tcb_sum_debit:,} | Closing: {tcb_running_balance:,}")


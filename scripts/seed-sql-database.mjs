/**
 * LIVA Banking — Database Seeder & SQL Exporter
 * Generates 200+ rich, diverse Vietnamese banking transactions, invoices, and vouchers.
 * Uses Node 22 built-in `node:sqlite` (DatabaseSync).
 * Outputs:
 * 1. SQLite Database: `data/liva_banking.db`
 * 2. Standard ANSI SQL Seed File: `data/liva_banking_seed.sql`
 */

import { DatabaseSync } from 'node:sqlite';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT_DIR = path.resolve(__dirname, '..');
const DB_PATH = path.join(ROOT_DIR, 'data', 'liva_banking.db');
const SCHEMA_PATH = path.join(ROOT_DIR, 'data', 'schema.sql');
const SEED_SQL_PATH = path.join(ROOT_DIR, 'data', 'liva_banking_seed.sql');

console.log('========================================================');
console.log(' LIVA Banking — Synthetic SQL Database Generator');
console.log('========================================================');

// 1. Vietnamese Corporate Registry (50 Real-World Style Enterprises)
const CORPORATE_REGISTRY = [
  { code: 'MSN', name: 'Tập đoàn Masan', full: 'CÔNG TY CỔ PHẦN TẬP ĐOÀN MASAN', tax: '0303576603', sector: 'Hàng Tiêu Dùng' },
  { code: 'VNM', name: 'Vinamilk', full: 'CÔNG TY CỔ PHẦN SỮA VIỆT NAM', tax: '0300588569', sector: 'Thực Phẩm & Sữa' },
  { code: 'HPG', name: 'Tập đoàn Hòa Phát', full: 'CÔNG TY CỔ PHẦN TẬP ĐOÀN HÒA PHÁT', tax: '0900189284', sector: 'Thép & Công Nghiệp' },
  { code: 'FPT', name: 'Tập đoàn FPT', full: 'CÔNG TY CỔ PHẦN FPT', tax: '0101248141', sector: 'Công Nghệ Thông Tin' },
  { code: 'VTL', name: 'Tập đoàn Viettel', full: 'TẬP ĐOÀN CÔNG NGHIỆP - VIỄN THÔNG QUÂN ĐỘI', tax: '0100109106', sector: 'Viễn Thông' },
  { code: 'MWG', name: 'Thế Giới Di Động', full: 'CÔNG TY CỔ PHẦN ĐẦU TƯ THẾ GIỚI DI ĐỘNG', tax: '0306734134', sector: 'Bán Lẻ Kỹ Thuật Số' },
  { code: 'GAS', name: 'PV GAS', full: 'TỔNG CÔNG TY KHÍ VIỆT NAM - CTCP', tax: '0300138669', sector: 'Dầu Khí & Năng Lượng' },
  { code: 'PLX', name: 'Petrolimex', full: 'TẬP ĐOÀN XĂNG DẦU VIỆT NAM', tax: '0100107266', sector: 'Xăng Dầu' },
  { code: 'THM', name: 'TH True Milk', full: 'CÔNG TY CỔ PHẦN THỰC PHẨM SỮA TH', tax: '2901111956', sector: 'Nông Nghiệp Sạch' },
  { code: 'VJC', name: 'Vietjet Air', full: 'CÔNG TY CỔ PHẦN HÀNG KHÔNG VIETJET', tax: '0102325379', sector: 'Hàng Không' },
  { code: 'HVN', name: 'Vietnam Airlines', full: 'TỔNG CÔNG TY HÀNG KHÔNG VIỆT NAM - CTCP', tax: '0100107518', sector: 'Hàng Không Quốc Gia' },
  { code: 'SAB', name: 'Sabeco', full: 'TỔNG CÔNG TY CỔ PHẦN BIA - RƯỢU - NƯỚC GIẢI KHÁT SÀI GÒN', tax: '0300583046', sector: 'Đồ Uống' },
  { code: 'PNJ', name: 'Vàng Bạc PNJ', full: 'CÔNG TY CỔ PHẦN VÀNG BẠC ĐÁ QUÝ PHÚ NHUẬN', tax: '0300521758', sector: 'Trang Sức' },
  { code: 'REE', name: 'Cơ Điện Lạnh REE', full: 'CÔNG TY CỔ PHẦN CƠ ĐIỆN LẠNH', tax: '0300741143', sector: 'Cơ Điện & BĐS' },
  { code: 'DGC', name: 'Hóa Chất Đức Giang', full: 'CÔNG TY CỔ PHẦN TẬP ĐOÀN HÓA CHẤT ĐỨC GIANG', tax: '0101452588', sector: 'Hóa Chất' },
  { code: 'GMD', name: 'Gemadept', full: 'CÔNG TY CỔ PHẦN GEMADEPT', tax: '0301156708', sector: 'Cảng Biển & Logistics' },
  { code: 'HAH', name: 'Hải An Logistics', full: 'CÔNG TY CỔ PHẦN VẬN TẢI VÀ XẾP DỠ HẢI AN', tax: '0200871778', sector: 'Vận Tải Biển' },
  { code: 'CTR', name: 'Viettel Construction', full: 'TỔNG CÔNG TY CỔ PHẦN CÔNG TRÌNH VIETTEL', tax: '0100109106-038', sector: 'Hạ Tầng Xây Lắp' },
  { code: 'FOX', name: 'FPT Telecom', full: 'CÔNG TY CỔ PHẦN VIỄN THÔNG FPT', tax: '0101778163', sector: 'Internet Băng Rộng' },
  { code: 'KDC', name: 'Tập đoàn KIDO', full: 'CÔNG TY CỔ PHẦN TẬP ĐOÀN KIDO', tax: '0303272658', sector: 'Dầu Ăn & Bánh Kẹo' },
  { code: 'DBC', name: 'Dabaco Việt Nam', full: 'CÔNG TY CỔ PHẦN TẬP ĐOÀN DABACO VIỆT NAM', tax: '2300104683', sector: 'Thức Ăn Chăn Nuôi' },
  { code: 'PC1', name: 'Tập đoàn PC1', full: 'CÔNG TY CỔ PHẦN TẬP ĐOÀN PC1', tax: '0100100740', sector: 'Xây Lắp Điện' },
  { code: 'NT2', name: 'Điện lực Dầu khí Nhơn Trạch', full: 'CÔNG TY CỔ PHẦN ĐIỆN LỰC DẦU KHÍ NHƠN TRẠCH 2', tax: '3600897607', sector: 'Nhiệt Điện Khí' },
  { code: 'VSH', name: 'Thủy điện Vĩnh Sơn Sông Hinh', full: 'CÔNG TY CỔ PHẦN THỦY ĐIỆN VĨNH SƠN - SÔNG HINH', tax: '4100570745', sector: 'Thủy Điện' },
  { code: 'BCM', name: 'Becamex IDC', full: 'TỔNG CÔNG TY ĐẦU TƯ VÀ PHÁT TRIỂN CÔNG NGHIỆP - CTCP', tax: '3700145020', sector: 'Khu Công Nghiệp' },
  { code: 'SZC', name: 'Sonadezi Châu Đức', full: 'CÔNG TY CỔ PHẦN SONADEZI CHÂU ĐỨC', tax: '3500806492', sector: 'BĐS Công Nghiệp' },
  { code: 'VGC', name: 'Viglacera', full: 'TỔNG CÔNG TY VIGLACERA - CTCP', tax: '0100108173', sector: 'Vật Liệu Xây Dựng' },
  { code: 'BMP', name: 'Nhựa Bình Minh', full: 'CÔNG TY CỔ PHẦN NHỰA BÌNH MINH', tax: '0302035520', sector: 'Nhựa Kỹ Thuật' },
  { code: 'NTP', name: 'Nhựa Tiền Phong', full: 'CÔNG TY CỔ PHẦN NHỰA THIẾU NIÊN TIỀN PHONG', tax: '0200167782', sector: 'Ống Nhựa Xây Dựng' },
  { code: 'SCS', name: 'Dịch vụ Hàng hóa Sài Gòn', full: 'CÔNG TY CỔ PHẦN DỊCH VỤ HÀNG HÓA SÀI GÒN', tax: '0305658099', sector: 'Logistics Hàng Không' }
];

// 2. Settlement Accounts across 5 Banks & 4 Payment Channels
const ACCOUNTS = [
  {
    id: 'acc_vcb_citad',
    bank_code: 'VCB',
    channel_code: 'CITAD',
    bank_name: 'Ngân hàng TMCP Ngoại thương Việt Nam (Vietcombank)',
    channel_name: 'Hệ thống Thanh toán Điện tử Liên ngân hàng CITAD (NHNN)',
    clearing_authority: 'Sở Giao dịch Ngân hàng Nhà nước Việt Nam',
    account_number: '0071001234567',
    account_name: 'CONG TY TNHH LIVA BANKING SOLUTIONS',
    opening_balance_vnd: 25000000000,
    minimum_reserve_vnd: 5000000000,
  },
  {
    id: 'acc_tcb_napas',
    bank_code: 'TCB',
    channel_code: 'NAPAS',
    bank_name: 'Ngân hàng TMCP Kỹ thương Việt Nam (Techcombank)',
    channel_name: 'Hệ thống Chuyển mạch Tài chính & Bù trừ Điện tử NAPAS 24/7',
    clearing_authority: 'Công ty Cổ phần Thanh toán Quốc gia Việt Nam',
    account_number: '19033889900012',
    account_name: 'CONG TY TNHH LIVA BANKING SOLUTIONS',
    opening_balance_vnd: 18500000000,
    minimum_reserve_vnd: 3000000000,
  },
  {
    id: 'acc_bidv_bilateral',
    bank_code: 'BIDV',
    channel_code: 'BILATERAL',
    bank_name: 'Ngân hàng TMCP Đầu tư và Phát triển Việt Nam (BIDV)',
    channel_name: 'Kênh Quyết toán Song phương & Nostro Đại lý',
    clearing_authority: 'Trung tâm Thanh toán BIDV Hội sở',
    account_number: '12010000998877',
    account_name: 'CONG TY TNHH LIVA BANKING SOLUTIONS',
    opening_balance_vnd: 12400000000,
    minimum_reserve_vnd: 2000000000,
  },
  {
    id: 'acc_ctg_citad',
    bank_code: 'CTG',
    channel_code: 'CITAD',
    bank_name: 'Ngân hàng TMCP Công Thương Việt Nam (VietinBank)',
    channel_name: 'Hệ thống Thanh toán Liên ngân hàng Giá trị Cao CITAD',
    clearing_authority: 'Sở Giao dịch Ngân hàng Nhà nước Việt Nam',
    account_number: '110600123999',
    account_name: 'CONG TY TNHH LIVA BANKING SOLUTIONS',
    opening_balance_vnd: 15200000000,
    minimum_reserve_vnd: 2500000000,
  },
  {
    id: 'acc_mbb_swift',
    bank_code: 'MBB',
    channel_code: 'SWIFT',
    bank_name: 'Ngân hàng TMCP Quân đội (MBBank)',
    channel_name: 'Hệ thống Viễn thông Tài chính Quốc tế SWIFT (MT103/MT940)',
    clearing_authority: 'Society for Worldwide Interbank Financial Telecommunication',
    account_number: '0511100223344',
    account_name: 'CONG TY TNHH LIVA BANKING SOLUTIONS',
    opening_balance_vnd: 8900000000,
    minimum_reserve_vnd: 1500000000,
  }
];

// Initialize SQLite DB
if (fs.existsSync(DB_PATH)) {
  fs.unlinkSync(DB_PATH);
}
const db = new DatabaseSync(DB_PATH);

// Execute Schema
const schemaSql = fs.readFileSync(SCHEMA_PATH, 'utf-8');
db.exec(schemaSql);
console.log('[✓] Đã tạo bảng lược đồ SQLite thành công.');

// Prepared Statements
const insertAccount = db.prepare(`
  INSERT INTO accounts (id, bank_code, channel_code, bank_name, channel_name, clearing_authority, account_number, account_name, balance_vnd, opening_balance_vnd, minimum_reserve_vnd)
  VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
`);

const insertInvoice = db.prepare(`
  INSERT INTO invoices (id, invoice_no, partner_code, partner_name, tax_code, contract_no, expected_amount, direction, due_date, issue_date, status)
  VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
`);

const insertStatement = db.prepare(`
  INSERT INTO bank_statements (id, account_id, bank_code, channel_code, txn_ref, booking_date, value_date, debit_amount, credit_amount, balance_after, fee_amount, partner_name, counterpart_account, description, invoice_ref, match_status, reconciliation_tier)
  VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
`);

const insertVoucher = db.prepare(`
  INSERT INTO payment_vouchers (id, voucher_no, source_account_id, target_bank, target_account, beneficiary_name, amount_vnd, channel, purpose, maker_id, maker_name, checker_id, checker_name, status, otp_code, otp_verified_at, approved_at)
  VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
`);

const insertAmlAlert = db.prepare(`
  INSERT INTO aml_alerts (id, statement_id, rule_code, severity, title, description, trigger_amount, partner_name, is_str_reported, str_report_id)
  VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
`);

const insertMerkleLog = db.prepare(`
  INSERT INTO merkle_ledger (event_type, entity_id, payload_hash, prev_root, current_root, actor_id)
  VALUES (?, ?, ?, ?, ?, ?)
`);

// 3. Populate Accounts
for (const acc of ACCOUNTS) {
  insertAccount.run(
    acc.id, acc.bank_code, acc.channel_code, acc.bank_name, acc.channel_name,
    acc.clearing_authority, acc.account_number, acc.account_name,
    acc.opening_balance_vnd, acc.opening_balance_vnd, acc.minimum_reserve_vnd
  );
}
console.log(`[✓] Đã khởi tạo ${ACCOUNTS.length} tài khoản thanh khoản liên ngân hàng.`);

// 4. Generate 60 Invoices
const INVOICES = [];
for (let i = 1; i <= 60; i++) {
  const corp = CORPORATE_REGISTRY[(i - 1) % CORPORATE_REGISTRY.length];
  const invNo = `HD-2026-${String(1000 + i).slice(1)}`;
  const baseAmount = ((i * 17) % 80 + 10) * 10_000_000; // 100M - 900M
  // Ensure Invoices 31..50 are RECEIVABLE so Tier 2 and Tier 3 composite splits align on CREDIT direction
  const direction = (i > 30 && i <= 50) ? 'RECEIVABLE' : (i % 4 === 0 ? 'PAYABLE' : 'RECEIVABLE');
  const issueDate = `2026-08-${String((i % 28) + 1).padStart(2, '0')}`;
  const dueDate = `2026-09-${String((i % 28) + 1).padStart(2, '0')}`;

  const inv = {
    id: `inv_${invNo}`,
    invoice_no: invNo,
    partner_code: corp.code,
    partner_name: corp.name,
    tax_code: corp.tax,
    contract_no: `HD-${corp.code}/2026-KT`,
    expected_amount: baseAmount,
    direction,
    due_date: dueDate,
    issue_date: issueDate,
    status: 'PENDING'
  };
  INVOICES.push(inv);
  insertInvoice.run(
    inv.id, inv.invoice_no, inv.partner_code, inv.partner_name, inv.tax_code,
    inv.contract_no, inv.expected_amount, inv.direction, inv.due_date, inv.issue_date, inv.status
  );
}
console.log(`[✓] Đã sinh ${INVOICES.length} hóa đơn công nợ Core Banking.`);

// 5. Generate 160 Diverse Bank Statements
const STATEMENTS = [];
let accountBalances = {};
for (const acc of ACCOUNTS) {
  accountBalances[acc.id] = acc.opening_balance_vnd;
}

let runningMerkleRoot = '0000000000000000000000000000000000000000000000000000000000000000';

for (let i = 1; i <= 160; i++) {
  const acc = ACCOUNTS[(i - 1) % ACCOUNTS.length];
  const corp = CORPORATE_REGISTRY[(i - 1) % CORPORATE_REGISTRY.length];
  const txnRef = `${acc.bank_code}2608${String(10000 + i).slice(1)}`;
  let bookingDate = `2026-08-${String((i % 28) + 1).padStart(2, '0')} ${String(8 + (i % 9)).padStart(2, '0')}:${String((i * 7) % 60).padStart(2, '0')}:00`;
  
  let debit = 0;
  let credit = 0;
  let fee = 0;
  let matchStatus = 'UNMATCHED';
  let tier = 0;
  let invRef = null;
  let desc = '';

  if (i <= 25) {
    // TIER 1: Exact 1:1 Match with Invoices 0..24
    const targetInv = INVOICES[i - 1];
    if (targetInv.direction === 'PAYABLE') {
      debit = targetInv.expected_amount;
    } else {
      credit = targetInv.expected_amount;
    }
    bookingDate = `${targetInv.issue_date} 09:15:00`;
    invRef = targetInv.invoice_no;
    desc = `THANH TOAN TIEN HANG HOP DONG SO ${targetInv.invoice_no} THEO CHUNG TU GOC ${corp.name}`;
    matchStatus = 'EXACT_MATCH';
    tier = 1;
  } else if (i <= 35) {
    // TIER 2: Fuzzy Heuristic & Interbank Fee Unbundling on Invoices 25..34
    const targetInv = INVOICES[i - 1];
    fee = [1100, 2200, 3300, 5500, 11000][(i - 26) % 5];
    credit = targetInv.expected_amount - fee;
    bookingDate = `${targetInv.issue_date} 10:20:00`;
    // Do not set invRef and do not put invoice_no in narration, so Tier 1 skips it
    invRef = null;
    desc = `UNC ${targetInv.partner_name} chuyen tien thanh toan don hang da tru phi chuyen khoan`;
    matchStatus = 'FUZZY_MATCH';
    tier = 2;
  } else if (i <= 38) {
    // TIER 3 (Phase A): Composite Split Settlement (1:N: 1 Bank Transaction pays 2 Invoices)
    // 3 txs (i=36, 37, 38) pay 6 invoices: (34,35), (36,37), (38,39)
    const invA = INVOICES[34 + (i - 36) * 2];
    const invB = INVOICES[34 + (i - 36) * 2 + 1];
    credit = invA.expected_amount + invB.expected_amount;
    bookingDate = `${invA.issue_date} 11:30:00`;
    desc = `GOM LENH THANH TOAN GIAI DOAN 1 CHO DOI TAC ${corp.name}`;
    invRef = null;
    matchStatus = 'SPLIT_MATCH';
    tier = 3;
  } else if (i <= 46) {
    // TIER 3 (Phase B): Split Installments (N:1: 2 Bank Transactions pay 1 Invoice)
    // 8 txs (i=39..46) pay 4 invoices (40, 41, 42, 43)
    const invIdx = 40 + Math.floor((i - 39) / 2);
    const targetInv = INVOICES[invIdx];
    const isFirstInstallment = (i - 39) % 2 === 0;
    const half = Math.floor(targetInv.expected_amount * 0.5);
    credit = isFirstInstallment ? half : (targetInv.expected_amount - half);
    bookingDate = `${targetInv.issue_date} ${isFirstInstallment ? '13:10:00' : '15:45:00'}`;
    desc = `DOT ${isFirstInstallment ? 1 : 2} QUYET TOAN TIEN HANG DOI TAC ${corp.name}`;
    invRef = null;
    matchStatus = 'SPLIT_MATCH';
    tier = 3;
  } else if (i <= 60) {
    // HITL EXCEPTION QUEUE (Variances > fee tolerance, Missing documentation)
    debit = ((i * 13) % 40 + 5) * 5_000_000;
    desc = `TAM UNG CONG TAC PHI VA CHI PHI NOI BO CHUA CO CHUNG TU DOI SOAT - CHO KSV PHUC TRA`;
    matchStatus = 'EXCEPTION';
    tier = 0;
  } else if (i <= 75) {
    // AML RED FLAGS (Smurfing, Rapid Pass-Through, Night Velocity, High-Value)
    if (i === 61 || i === 62 || i === 63) {
      credit = 390_000_000;
      desc = `NOP TIEN MAT VAO TAI KHOAN TAI QUAY CHI NHANH ${i - 60} NGUOI NOP NGUYEN VAN A`;
    } else if (i === 64) {
      debit = 499_500_000;
      desc = `CHUYEN TIEN NHANH SANG VI DIEN TU TAI KHOAN VI-998822 TRONG VONG 5 PHUT SAU KHI NHAN`;
    } else if (i === 65) {
      debit = 85_000_000;
      desc = `RUT TIEN NGOAI GIO GIAO DICH LUC 02:25 SANG QUA CONG THANH TOAN QUOC TE`;
    } else {
      credit = 1_500_000_000;
      desc = `QUYET TOAN HOP DONG XUAT NHAP KHAU MAY MOC CONG NGHIEP NANG GIA TRI CAO`;
    }
    matchStatus = 'UNMATCHED';
    tier = 0;
  } else {
    // General Interbank Liquidity, Clearing & Settlement
    const isCredit = i % 3 !== 0;
    const amount = ((i * 23) % 60 + 5) * 12_000_000;
    if (isCredit) {
      credit = amount;
    } else {
      debit = amount;
    }
    desc = `GIAO DICH THANH TOAN BIEU PHI DICH VU VA QUYET TOAN LIEN NGAN HANG ${acc.channel_code}`;
    matchStatus = 'UNMATCHED';
    tier = 0;
  }

  // Update Invariant Balance
  accountBalances[acc.id] = accountBalances[acc.id] + credit - debit;
  const balanceAfter = accountBalances[acc.id];

  const stmt = {
    id: `stmt_${txnRef}`,
    account_id: acc.id,
    bank_code: acc.bank_code,
    channel_code: acc.channel_code,
    txn_ref: txnRef,
    booking_date: bookingDate,
    value_date: bookingDate.split(' ')[0],
    debit_amount: debit,
    credit_amount: credit,
    balance_after: balanceAfter,
    fee_amount: fee,
    partner_name: corp.name,
    counterpart_account: `00${(i * 987) % 9000000000 + 1000000000}`,
    description: desc,
    invoice_ref: invRef,
    match_status: matchStatus,
    reconciliation_tier: tier
  };
  STATEMENTS.push(stmt);

  insertStatement.run(
    stmt.id, stmt.account_id, stmt.bank_code, stmt.channel_code, stmt.txn_ref,
    stmt.booking_date, stmt.value_date, stmt.debit_amount, stmt.credit_amount,
    stmt.balance_after, stmt.fee_amount, stmt.partner_name, stmt.counterpart_account,
    stmt.description, stmt.invoice_ref, stmt.match_status, stmt.reconciliation_tier
  );

  // Update Merkle Tree Log for this transaction event
  const payloadHash = crypto.createHash('sha256').update(JSON.stringify(stmt)).digest('hex');
  const newRoot = crypto.createHash('sha256').update(runningMerkleRoot + payloadHash).digest('hex');
  insertMerkleLog.run('STATEMENT_INGESTED', stmt.id, payloadHash, runningMerkleRoot, newRoot, 'SYSTEM_INGEST');
  runningMerkleRoot = newRoot;
}
console.log(`[✓] Đã sinh và nạp ${STATEMENTS.length} giao dịch sao kê vào SQLite.`);

// Update Final Balances into Accounts Table
const updateAccountBal = db.prepare('UPDATE accounts SET balance_vnd = ? WHERE id = ?');
for (const acc of ACCOUNTS) {
  updateAccountBal.run(accountBalances[acc.id], acc.id);
}
console.log('[✓] Đã bảo chứng bất biến số dư (Closing = Opening + Credit - Debit).');

// 6. Generate 20 Payment Vouchers (Maker-Checker TT09)
for (let v = 1; v <= 20; v++) {
  const corp = CORPORATE_REGISTRY[(v * 3) % CORPORATE_REGISTRY.length];
  const acc = ACCOUNTS[(v - 1) % ACCOUNTS.length];
  const vNo = `VCH-2026-${String(500 + v).padStart(4, '0')}`;
  const amount = ((v * 29) % 70 + 10) * 15_000_000;
  const status = v <= 10 ? 'APPROVED' : v <= 16 ? 'PENDING_APPROVAL' : 'DRAFT';
  const checkerId = status === 'APPROVED' ? 'SUP-88214' : null;
  const checkerName = status === 'APPROVED' ? 'Trần Thị Giám Đốc' : null;
  const approvedAt = status === 'APPROVED' ? `2026-08-28 15:${String(10 + v).padStart(2, '0')}:00` : null;

  insertVoucher.run(
    `vch_${vNo}`, vNo, acc.id, 'VCB', `007100${String(990000 + v)}`, corp.name,
    amount, acc.channel_code, `Thanh toán hợp đồng cung ứng dịch vụ cho ${corp.name}`,
    'OPR-77092', 'Nguyễn Văn Kế Toán', checkerId, checkerName,
    status, status === 'APPROVED' ? '889922' : null,
    status === 'APPROVED' ? approvedAt : null, approvedAt
  );
}
console.log('[✓] Đã nạp 20 lệnh chi Maker-Checker.');

// 7. Generate 10 AML Alerts
const AML_RULES = [
  { code: 'AML-SMURF', severity: 'CRITICAL', title: 'Phát hiện cấu trúc gom/chia nhỏ (Smurfing)', desc: 'Nhiều giao dịch dưới 400M VND trong 24 giờ né báo cáo theo Quyết định 11/2023/QĐ-TTg.' },
  { code: 'AML-PASSTHROUGH', severity: 'CRITICAL', title: 'Dòng tiền chuyển tiếp siêu tốc (Rapid Pass-Through)', desc: 'Tiền vào tài khoản và rút ra ví điện tử 99.9% trong vòng dưới 10 phút.' },
  { code: 'AML-NIGHT-VELOCITY', severity: 'HIGH', title: 'Tần suất giao dịch bất thường ngoài giờ (Night Velocity)', desc: 'Giao dịch phát sinh liên tiếp từ 01:00 - 04:00 sáng với giá trị lớn hơn 50M VND.' },
  { code: 'AML-HIGH-VALUE', severity: 'HIGH', title: 'Giao dịch giá trị lớn vượt ngưỡng (High-Value Inflow)', desc: 'Giao dịch vượt 1.000.000.000 VND yêu cầu lập hồ sơ thẩm tra nguồn gốc tiền gửi.' }
];

for (let a = 1; a <= 10; a++) {
  const rule = AML_RULES[(a - 1) % AML_RULES.length];
  const corp = CORPORATE_REGISTRY[(a * 5) % CORPORATE_REGISTRY.length];
  const triggerAmount = ((a * 43) % 90 + 20) * 25_000_000;
  const isReported = a <= 4 ? 1 : 0;
  const strId = isReported ? `STR-NHNN-2026-${String(900 + a)}` : null;

  insertAmlAlert.run(
    `aml_alert_${a}`, null, rule.code, rule.severity, rule.title,
    `${rule.desc} Đối tác liên quan: ${corp.name}`, triggerAmount, corp.name,
    isReported, strId
  );
}
console.log('[✓] Đã nạp 10 sự kiện cảnh báo rủi ro AML/STR.');

// 8. Generate ANSI SQL Dump (data/liva_banking_seed.sql)
let sqlDump = `-- ==============================================================================
-- LIVA Banking Core — ANSI SQL Export Dump
-- Generated: ${new Date().toISOString()}
-- Total Records: ${ACCOUNTS.length + INVOICES.length + STATEMENTS.length + 20 + 10} records
-- ==============================================================================

`;

sqlDump += schemaSql + '\n\n';

function toSqlVal(val) {
  if (val === null || val === undefined) return 'NULL';
  if (typeof val === 'number') return val.toString();
  return `'${String(val).replace(/'/g, "''")}'`;
}

// Dump Accounts
sqlDump += '-- ACCOUNTS\n';
for (const row of db.prepare('SELECT * FROM accounts').all()) {
  sqlDump += `INSERT INTO accounts (id, bank_code, channel_code, bank_name, channel_name, clearing_authority, account_number, account_name, balance_vnd, opening_balance_vnd, minimum_reserve_vnd) VALUES (${toSqlVal(row.id)}, ${toSqlVal(row.bank_code)}, ${toSqlVal(row.channel_code)}, ${toSqlVal(row.bank_name)}, ${toSqlVal(row.channel_name)}, ${toSqlVal(row.clearing_authority)}, ${toSqlVal(row.account_number)}, ${toSqlVal(row.account_name)}, ${row.balance_vnd}, ${row.opening_balance_vnd}, ${row.minimum_reserve_vnd});\n`;
}

// Dump Invoices
sqlDump += '\n-- INVOICES\n';
for (const row of db.prepare('SELECT * FROM invoices').all()) {
  sqlDump += `INSERT INTO invoices (id, invoice_no, partner_code, partner_name, tax_code, contract_no, expected_amount, direction, due_date, issue_date, status) VALUES (${toSqlVal(row.id)}, ${toSqlVal(row.invoice_no)}, ${toSqlVal(row.partner_code)}, ${toSqlVal(row.partner_name)}, ${toSqlVal(row.tax_code)}, ${toSqlVal(row.contract_no)}, ${row.expected_amount}, ${toSqlVal(row.direction)}, ${toSqlVal(row.due_date)}, ${toSqlVal(row.issue_date)}, ${toSqlVal(row.status)});\n`;
}

// Dump Bank Statements
sqlDump += '\n-- BANK STATEMENTS\n';
for (const row of db.prepare('SELECT * FROM bank_statements').all()) {
  sqlDump += `INSERT INTO bank_statements (id, account_id, bank_code, channel_code, txn_ref, booking_date, value_date, debit_amount, credit_amount, balance_after, fee_amount, partner_name, counterpart_account, description, invoice_ref, match_status, reconciliation_tier) VALUES (${toSqlVal(row.id)}, ${toSqlVal(row.account_id)}, ${toSqlVal(row.bank_code)}, ${toSqlVal(row.channel_code)}, ${toSqlVal(row.txn_ref)}, ${toSqlVal(row.booking_date)}, ${toSqlVal(row.value_date)}, ${row.debit_amount}, ${row.credit_amount}, ${row.balance_after}, ${row.fee_amount}, ${toSqlVal(row.partner_name)}, ${toSqlVal(row.counterpart_account)}, ${toSqlVal(row.description)}, ${toSqlVal(row.invoice_ref)}, ${toSqlVal(row.match_status)}, ${row.reconciliation_tier});\n`;
}

// Dump Payment Vouchers
sqlDump += '\n-- PAYMENT VOUCHERS\n';
for (const row of db.prepare('SELECT * FROM payment_vouchers').all()) {
  sqlDump += `INSERT INTO payment_vouchers (id, voucher_no, source_account_id, target_bank, target_account, beneficiary_name, amount_vnd, channel, purpose, maker_id, maker_name, checker_id, checker_name, status, otp_code, otp_verified_at, approved_at) VALUES (${toSqlVal(row.id)}, ${toSqlVal(row.voucher_no)}, ${toSqlVal(row.source_account_id)}, ${toSqlVal(row.target_bank)}, ${toSqlVal(row.target_account)}, ${toSqlVal(row.beneficiary_name)}, ${row.amount_vnd}, ${toSqlVal(row.channel)}, ${toSqlVal(row.purpose)}, ${toSqlVal(row.maker_id)}, ${toSqlVal(row.maker_name)}, ${toSqlVal(row.checker_id)}, ${toSqlVal(row.checker_name)}, ${toSqlVal(row.status)}, ${toSqlVal(row.otp_code)}, ${toSqlVal(row.otp_verified_at)}, ${toSqlVal(row.approved_at)});\n`;
}

// Dump AML Alerts
sqlDump += '\n-- AML ALERTS\n';
for (const row of db.prepare('SELECT * FROM aml_alerts').all()) {
  sqlDump += `INSERT INTO aml_alerts (id, statement_id, rule_code, severity, title, description, trigger_amount, partner_name, is_str_reported, str_report_id) VALUES (${toSqlVal(row.id)}, ${toSqlVal(row.statement_id)}, ${toSqlVal(row.rule_code)}, ${toSqlVal(row.severity)}, ${toSqlVal(row.title)}, ${toSqlVal(row.description)}, ${row.trigger_amount}, ${toSqlVal(row.partner_name)}, ${row.is_str_reported}, ${toSqlVal(row.str_report_id)});\n`;
}

fs.writeFileSync(SEED_SQL_PATH, sqlDump, 'utf-8');
console.log(`[✓] Đã xuất tệp SQL dump tại: ${SEED_SQL_PATH} (${(fs.statSync(SEED_SQL_PATH).size / 1024).toFixed(1)} KB)`);

// 9. Export Frontend Client Synced JSON
const FRONTEND_JSON_PATH = path.join(ROOT_DIR, 'teamwork_projects', 'liva_banking_universal', 'src', 'data', 'sqlDatabaseSeed.json');
const frontendDataDir = path.dirname(FRONTEND_JSON_PATH);
if (!fs.existsSync(frontendDataDir)) {
  fs.mkdirSync(frontendDataDir, { recursive: true });
}
const exportJson = {
  generatedAt: new Date().toISOString(),
  accounts: db.prepare('SELECT * FROM accounts').all(),
  invoices: db.prepare('SELECT * FROM invoices').all(),
  bankStatements: db.prepare('SELECT * FROM bank_statements').all(),
  paymentVouchers: db.prepare('SELECT * FROM payment_vouchers').all(),
  amlAlerts: db.prepare('SELECT * FROM aml_alerts').all(),
  merkleLedger: db.prepare('SELECT * FROM merkle_ledger').all(),
  stats: {
    accountsCount: ACCOUNTS.length,
    invoicesCount: db.prepare('SELECT count(*) as count FROM invoices').get().count,
    statementsCount: db.prepare('SELECT count(*) as count FROM bank_statements').get().count,
    vouchersCount: db.prepare('SELECT count(*) as count FROM payment_vouchers').get().count,
    amlAlertsCount: db.prepare('SELECT count(*) as count FROM aml_alerts').get().count,
    totalLiquidityVnd: db.prepare('SELECT sum(balance_vnd) as total FROM accounts').get().total,
  }
};
fs.writeFileSync(FRONTEND_JSON_PATH, JSON.stringify(exportJson, null, 2), 'utf-8');
console.log(`[✓] Đã xuất JSON đồng bộ cho Frontend tại: ${FRONTEND_JSON_PATH} (${(fs.statSync(FRONTEND_JSON_PATH).size / 1024).toFixed(1)} KB)`);

// 10. Quick Verification Query
const totalStmts = exportJson.stats.statementsCount;
const totalInvs = exportJson.stats.invoicesCount;
const totalVouchers = exportJson.stats.vouchersCount;
const totalAlerts = exportJson.stats.amlAlertsCount;
const totalLiquidity = exportJson.stats.totalLiquidityVnd;

console.log('\n================ BÁO CÁO THỐNG KÊ SQL DATABASE ================');
console.log(`- Tổng số tài khoản thanh khoản: ${ACCOUNTS.length}`);
console.log(`- Tổng thanh khoản hệ thống:    ${new Intl.NumberFormat('vi-VN').format(totalLiquidity)} VND`);
console.log(`- Tổng số hóa đơn Core Banking:  ${totalInvs}`);
console.log(`- Tổng số giao dịch sao kê:     ${totalStmts}`);
console.log(`- Tổng lệnh chi Maker-Checker:   ${totalVouchers}`);
console.log(`- Tổng cảnh báo giám sát AML:    ${totalAlerts}`);
console.log('================================================================\n');

db.close();


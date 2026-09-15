const fs = require('fs');
const path = require('path');
const XLSX = require('xlsx');

const demoDir = path.resolve(__dirname, '../data/demo_ready');
if (!fs.existsSync(demoDir)) fs.mkdirSync(demoDir, { recursive: true });

// 1. Vietcombank Excel (.xlsx) with AML Structuring & Exact Matches
const vcbRows = [
  ['NGÂN HÀNG THƯƠNG MẠI CỔ PHẦN NGOẠI THƯƠNG VIỆT NAM - VIETCOMBANK'],
  ['Số tài khoản: 0071001234567', '', 'Tên tài khoản: CONG TY TNHH LIVA SOLUTIONS'],
  ['Kỳ sao kê: 01/08/2026 - 31/08/2026', '', 'Loại tiền tệ: VND'],
  ['Ngày giao dịch', 'Mã GD', 'Số tiền ghi nợ', 'Số tiền ghi có', 'Số dư', 'Nội dung chi tiết', 'Tên người gửi/thụ hưởng'],
  ['01/08/2026 09:15', 'VCB26214001', '', 145000000, 1595230000, 'CT TT TIEN HANG HOP DONG SO 882026 CONG TY AN PHAT', 'CONG TY TNHH TM DV AN PHAT'],
  ['02/08/2026 10:30', 'VCB26214002', '', 280000000, 1875230000, 'THANH TOAN DOT 1 CUNG UNG THIET BI Y TE HOA BINH', 'CTCP XAY DUNG HOA BINH'],
  ['05/08/2026 14:20', 'VCB26214003', 15500000, '', 1859730000, 'PHI DICH VU BAO TRI MAY PHAT DIEN T8', 'CONG TY DIEN LUC HA NOI'],
  ['10/08/2026 11:00', 'VCB26214004', '', 390000000, 2249730000, 'CHUYEN TIEN KHOAN GIAI NGAN HOP DONG DU AN LIVA-01', 'NGUYEN HOANG PHUC'],
  ['10/08/2026 11:45', 'VCB26214005', '', 385000000, 2634730000, 'CHUYEN TIEN GIAI NGAN GOI 2 DU AN LIVA-01', 'TRAN THI BICH NGOC'],
  ['10/08/2026 14:10', 'VCB26214006', '', 395000000, 3029730000, 'GIAI NGAN DOT 3 DU AN LIVA-01 HOAN TAT', 'LE VAN THANG'],
  ['15/08/2026 16:30', 'VCB26214007', '', 500000000, 3529730000, 'TT TIEN MUA NGUYEN VAT LIEU THEP THANG 8', 'CONG TY TNHH THEP VIET NHAT'],
  ['20/08/2026 08:30', 'VCB26214008', 499000000, '', 3030730000, 'CHUYEN TRA TIEN HANG XUAT KHAU TRONG NGAY', 'CONG TY LOGISTICS BIEN DONG']
];
const wb = XLSX.utils.book_new();
const ws = XLSX.utils.aoa_to_sheet(vcbRows);
XLSX.utils.book_append_sheet(wb, ws, 'SaoKe_VCB_T8');
XLSX.writeFile(wb, path.join(demoDir, '01_VCB_SaoKe_Thang8_Chuan.xlsx'));

// 2. Techcombank CSV with High Value & Fuzzy Matches
const tcbCsvContent = 
`Số tài khoản:;19034567890123;;;;;;
Tên tài khoản:;CONG TY TNHH LIVA SOLUTIONS;;;;;;
Số dư đầu kỳ:;785.600.000;;;;;;
Kỳ sao kê:;01/08/2026 - 31/08/2026;;;;;;
Ngày giao dịch;Mã giao dịch;Số tiền ghi nợ;Số tiền ghi có;Số dư;Nội dung chi tiết;Tên đối tác
01/08/2026 08:30:00;TCB26240001;;45.000.000;830.600.000;Napas VietQR TT HD131 An Phat;Công ty TNHH TM DV An Phát
03/08/2026 11:20:00;TCB26240002;;550.000.000;1.380.600.000;Napas VietQR TT MUA XE CHUYEN DUNG DU AN;Công ty CP Tập đoàn Masan
05/08/2026 15:40:00;TCB26240003;2.200.000;;1.378.400.000;Phi duy tri quan ly tai khoan doanh nghiep thang 8;Techcombank
08/08/2026 16:10:00;TCB26240004;;120.000.000;1.498.400.000;Napas VietQR ck tien hang may bien ap hd 99;Công ty TNHH Cơ điện lạnh Đại Việt
12/08/2026 17:00:00;TCB26240005;;85.000.000;1.583.400.000;Napas VietQR Thanh toan dot 2 hop dong FPT;Công ty TNHH FPT Smart Cloud
`;
fs.writeFileSync(path.join(demoDir, '02_TCB_SaoKe_Thang8_Chuan.csv'), tcbCsvContent, 'utf-8');

// 3. BIDV CSV with Night Anomaly & Pass-through velocity
const bidvCsvContent =
`Số tài khoản:;12010001234567;;;;;;
Tên tài khoản:;CONG TY TNHH LIVA SOLUTIONS;;;;;;
Kỳ sao kê:;01/08/2026 - 31/08/2026;;;;;;
Ngày giao dịch;Mã giao dịch;Số tiền ghi nợ;Số tiền ghi có;Số dư;Nội dung chi tiết;Tên đối tác
02/08/2026 02:15:20;BIDV2621001;;420.000.000;932.000.000;CHUYEN TIEN LIEN NGAN HANG GIAO DICH NGOAI GIO;VU TRONG PHUONG
02/08/2026 02:22:45;BIDV2621002;419.500.000;;512.500.000;RUT TOAN BO SO TIEN VAO VI DIEN TU TRONG VONG 7 PHUT;CONG TY TNHH TRUNG GIAN THANH TOAN X
05/08/2026 10:00:00;BIDV2621003;;60.000.000;572.500.000;TT TIEN THIET KE WEBSITE VA PHAN MEM T8;CTCP GIAI PHAP CONG NGHE NOVA
15/08/2026 14:30:00;BIDV2621004;500.000;;572.000.000;Phi thuong nien dich vu ngan hang dien tu;BIDV
`;
fs.writeFileSync(path.join(demoDir, '03_BIDV_SaoKe_NgoaiGio_BatThuong.csv'), bidvCsvContent, 'utf-8');

// 4. Ledger Invoices (Sổ cái kế toán đối ứng để chạy đối soát 3 tầng)
const ledgerCsvContent =
`Mã hóa đơn;Ngày hạch toán;Đối tác;Số tiền phải thu;Số tiền phải trả;Diễn giải kế toán;Trạng thái
HD-2026-88;01/08/2026;CONG TY TNHH TM DV AN PHAT;145.000.000;;Bán thiết bị tự động hóa cho An Phát;CHƯA_THANH_TOÁN
HD-2026-89;02/08/2026;CTCP XAY DUNG HOA BINH;280.000.000;;Thi công lắp đặt trạm kiểm soát;CHƯA_THANH_TOÁN
HD-2026-90;01/08/2026;Công ty TNHH TM DV An Phát;45.000.000;;Hóa đơn dịch vụ tư vấn Q3;CHƯA_THANH_TOÁN
HD-2026-91;03/08/2026;Công ty CP Tập đoàn Masan;550.000.000;;Cung ứng lô xe nâng chuyên dụng Masan;CHƯA_THANH_TOÁN
HD-2026-92;08/08/2026;Công ty TNHH Cơ điện lạnh Đại Việt;120.000.000;;Cung cấp máy biến áp cao thế;CHƯA_THANH_TOÁN
HD-2026-93;12/08/2026;Công ty TNHH FPT Smart Cloud;85.000.000;;Gia hạn dịch vụ máy chủ đám mây;CHƯA_THANH_TOÁN
HD-2026-94;05/08/2026;CTCP GIAI PHAP CONG NGHE NOVA;60.000.000;;Nghiệm thu phần mềm giai đoạn 1;CHƯA_THANH_TOÁN
`;
fs.writeFileSync(path.join(demoDir, '04_SoCai_KeToan_HoaDon.csv'), ledgerCsvContent, 'utf-8');

// 5. Google Sheets / Excel Direct Copy-Paste Text (Tab-separated)
const pasteableContent = 
`Ngày giao dịch	Mã giao dịch	Số tiền ghi nợ	Số tiền ghi có	Số dư	Nội dung chi tiết	Tên đối tác
01/08/2026 09:15	VCB26214001		145000000	1595230000	CT TT TIEN HANG HOP DONG SO 882026 CONG TY AN PHAT	CONG TY TNHH TM DV AN PHAT
02/08/2026 10:30	VCB26214002		280000000	1875230000	THANH TOAN DOT 1 CUNG UNG THIET BI Y TE HOA BINH	CTCP XAY DUNG HOA BINH
03/08/2026 11:20	TCB26240002		550000000	1380600000	Napas VietQR TT MUA XE CHUYEN DUNG DU AN	Công ty CP Tập đoàn Masan
02/08/2026 02:15	BIDV2621001		420000000	932000000	CHUYEN TIEN LIEN NGAN HANG GIAO DICH NGOAI GIO	VU TRONG PHUONG
02/08/2026 02:22	BIDV2621002	419500000		512500000	RUT TOAN BO SO TIEN VAO VI DIEN TU TRONG VONG 7 PHUT	CONG TY TNHH TRUNG GIAN THANH TOAN X
10/08/2026 11:00	VCB26214004		390000000	2249730000	CHUYEN TIEN KHOAN GIAI NGAN HOP DONG DU AN LIVA-01	NGUYEN HOANG PHUC
10/08/2026 11:45	VCB26214005		385000000	2634730000	CHUYEN TIEN GIAI NGAN GOI 2 DU AN LIVA-01	TRAN THI BICH NGOC
10/08/2026 14:10	VCB26214006		395000000	3029730000	GIAI NGAN DOT 3 DU AN LIVA-01 HOAN TAT	LE VAN THANG
`;
fs.writeFileSync(path.join(demoDir, '05_DuLieu_Dan_GoogleSheets.txt'), pasteableContent, 'utf-8');

console.log('SUCCESS: Created 5 production-grade demo test files in ' + demoDir);

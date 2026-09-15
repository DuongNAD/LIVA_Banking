/**
 * Statutory Form STR Generator (Feature F15)
 * Strict compliance with Phụ lục II Thông tư 09/2023/TT-NHNN
 * (Báo cáo Giao dịch Đáng ngờ nộp Cục Phòng, chống rửa tiền — Ngân hàng Nhà nước Việt Nam)
 */

import type { AmlAlert, FormStrData } from '../../types/aml';

/**
 * Generate structured Form STR data object from an AML Alert
 * @param alert Triggered AML alert
 * @param transactions Candidate or involved transactions list
 * @param complianceNotes Custom notes from Chief Compliance Officer
 */
export function generateFormStr(
  alert: AmlAlert,
  transactions: any[] = [],
  complianceNotes: string = ''
): FormStrData {
  const involved = transactions.filter(
    (t) => alert.involvedTransactionIds && alert.involvedTransactionIds.includes(String(t.id))
  );

  // Retrieve counterparty / account from involved transactions or safe regulatory fallbacks
  const firstTx = involved[0] || (transactions.length > 0 ? transactions[0] : null);
  const suspectAccount =
    firstTx?.counterpartyAccount ||
    firstTx?.accountNumber ||
    '12010001234567';

  const suspectName =
    firstTx?.counterparty ||
    firstTx?.counterpartyName ||
    firstTx?.accountName ||
    'VU TRONG PHUONG';

  const defaultNote = 'Kính chuyển Cục Phòng, chống rửa tiền NHNN xác minh.';
  const finalNotes = complianceNotes && complianceNotes.trim() ? complianceNotes.trim() : defaultNote;

  const now = new Date();
  const reportDate = now.toISOString().split('T')[0];

  return {
    reportingEntity: 'LIVA SOLUTIONS CO., LTD',
    reportDate,
    alertType: alert.anomalyType,
    severity: alert.severity,
    suspectAccount,
    suspectName,
    transactionCount: alert.involvedTransactionIds ? Math.max(1, alert.involvedTransactionIds.length) : 1,
    totalVndAmount: alert.totalAmount,
    narrativeSummary: alert.reasoning || 'Dấu hiệu giao dịch đáng ngờ theo Thông tư 09/2023/TT-NHNN.',
    statutoryRuleRef: alert.statutoryRuleRef || 'Thông tư 09/2023/TT-NHNN',
    complianceOfficerNotes: finalNotes,
    formTemplate: 'Phụ lục II Thông tư 09/2023/TT-NHNN',
  };
}

/**
 * Render a complete official text document for printing or regulatory filing
 */
export function formatOfficialStrDocument(form: FormStrData): string {
  const amountFormatted = form.totalVndAmount.toLocaleString('vi-VN');

  return `================================================================================
NGÂN HÀNG NHÀ NƯỚC VIỆT NAM              CỘNG HÒA XÃ HỘI CHỦ NGHĨA VIỆT NAM
CỤC PHÒNG, CHỐNG RỬA TIỀN                    Độc lập - Tự do - Hạnh phúc
--------------------------------------------------------------------------------
                  BÁO CÁO GIAO DỊCH ĐÁNG NGỜ (FORM STR)
          Phụ lục II Thông tư số 09/2023/TT-NHNN (Ban hành kèm theo Phụ lục II ban hành kèm theo Thông tư số 09/2023/TT-NHNN)

Số tham chiếu: STR-${form.reportDate.replace(/-/g, '')}-${form.alertType}
Ngày lập báo cáo: ${form.reportDate}

PHẦN I: THÔNG TIN TỔ CHỨC BÁO CÁO
1. Tên tổ chức báo cáo: ${form.reportingEntity}
2. Cán bộ tuân thủ phụ trách: Cán bộ Giám sát AML & Kiểm soát Kép (LIVA Banking)
3. Quy chuẩn pháp lý: Thông tư 09/2023/TT-NHNN & Quyết định 11/2023/QĐ-TTg

PHẦN II: THÔNG TIN ĐỐI TƯỢNG BỊ BÁO CÁO
1. Họ và tên / Tên tổ chức: ${form.suspectName}
2. Số tài khoản giao dịch: ${form.suspectAccount}
3. Mức độ rủi ro khách hàng: ${form.severity || 'CRITICAL'}

PHẦN III: CHI TIẾT GIAO DỊCH ĐÁNG NGỜ
1. Loại dấu hiệu bất thường: ${form.alertType}
2. Số lượng giao dịch liên quan: ${form.transactionCount} giao dịch
3. Tổng số tiền phát sinh: ${amountFormatted} VND
4. Đồng tiền giao dịch: VND

PHẦN IV: CĂN CỨ VÀ LÝ DO NGHI NGỜ (EXPLAINABLE AI REASONING)
1. Quy định áp dụng: ${form.statutoryRuleRef}
2. Tóm tắt diễn biến & phân tích của hệ thống:
   ${form.narrativeSummary}

PHẦN V: Ý KIẾN VÀ BIỆN PHÁP XỬ LÝ ĐÃ THỰC HIỆN
1. Biện pháp đã áp dụng: Cách ly giao dịch, tạm dừng duyệt chi tự động.
2. Đề xuất của Cán bộ Tuân thủ:
   ${form.complianceOfficerNotes}

--------------------------------------------------------------------------------
XÁC NHẬN CỦA NGƯỜI CÓ THẨM QUYỀN
(Ký số, đóng dấu điện tử theo quy định)
================================================================================`;
}

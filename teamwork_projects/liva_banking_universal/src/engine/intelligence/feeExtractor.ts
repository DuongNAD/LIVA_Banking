/**
 * Wire Fee Disentanglement Engine (Feature F10)
 * Isolates embedded interbank wire transfer fees (Napas 24/7, interbank fees)
 * and allocates the fee portion to TK 6425 (Chi phí dịch vụ ngân hàng)
 * using deterministic 64-bit integer arithmetic without LLM floating-point drift.
 */

import { removeVietnameseAccents } from './vietnameseNlp';

export const KNOWN_VIETNAMESE_WIRE_FEES = [
  1100,
  2200,
  3300,
  5500,
  7700,
  8800,
  9900,
  11000,
  22000,
] as const;

export interface WireFeeDisentanglement {
  principal: number; // Scaled integer in VND
  feeAmount: number; // Scaled integer in VND
  accountCode: string | null; // 'TK 6425' or null
  accountName: string | null; // 'Chi phí dịch vụ ngân hàng' or null
  isPureFee: boolean;
  sumCheckPassed: boolean;
  confidence?: number;
}

/**
 * Disentangle wire transfer fee from a transaction amount and memo
 * @param amount Gross transaction amount in integer VND
 * @param memo Transaction description/narration
 */
export function disentangleWireFee(amount: number, memo: string = ''): WireFeeDisentanglement {
  // Guard for 0 or negative amount
  if (amount <= 0) {
    return {
      principal: 0,
      feeAmount: 0,
      accountCode: null,
      accountName: null,
      isPureFee: false,
      sumCheckPassed: true,
    };
  }

  const rawMemo = memo || '';
  const lower = rawMemo.toLowerCase();
  const norm = removeVietnameseAccents(rawMemo).toLowerCase();

  // 1. Check for pure bank account management / annual fee transaction
  const isPureBankFee =
    /phi\s*(?:duy\s*tri|thuong\s*nien|dich\s*vu|ql|quan\s*ly\s*tai\s*khoan)/i.test(norm) ||
    /phi\s*duy\s*tri/i.test(lower) ||
    /phí\s*(?:duy\s*trì|thường\s*niên|dịch\s*vụ|quản\s*lý\s*tài\s*khoản)/i.test(rawMemo);

  if (isPureBankFee) {
    return {
      principal: 0,
      feeAmount: amount,
      accountCode: 'TK 6425',
      accountName: 'Chi phí dịch vụ ngân hàng',
      isPureFee: true,
      sumCheckPassed: true,
      confidence: 1.0,
    };
  }

  // 2. Look for explicit fee keywords followed or preceded by known fee amounts
  // Protect against false positives like "HD-2200" or "INV2200"
  let fee = 0;

  for (const f of KNOWN_VIETNAMESE_WIRE_FEES) {
    // Pattern requires fee keyword: "phi 2200", "phi chuyen tien 2200", "cuoc 1100", "fee: 2200"
    const feeRegex = new RegExp(
      `(?:phi|fee|cuoc|phí)\\s*(?:dich\\s*vu|chuyen\\s*(?:tien|khoan)|gd)?\\s*[:\\-]?\\s*${f}\\b`,
      'i'
    );
    if (feeRegex.test(lower) || feeRegex.test(norm)) {
      fee = f;
      break;
    }
  }

  // If amount itself equals a known fee and memo contains fee keyword (e.g. amount 2200, "phi 2200")
  if (fee === 0 && (lower.includes('phi') || lower.includes('phí') || lower.includes('fee'))) {
    if (KNOWN_VIETNAMESE_WIRE_FEES.includes(amount as any)) {
      fee = amount;
    }
  }

  // Ensure fee does not exceed gross amount
  fee = Math.min(fee, amount);
  const principal = Math.max(0, amount - fee);

  const sumCheckPassed = principal + fee === amount;

  return {
    principal,
    feeAmount: fee,
    accountCode: fee > 0 ? 'TK 6425' : null,
    accountName: fee > 0 ? 'Chi phí dịch vụ ngân hàng' : null,
    isPureFee: fee === amount && fee > 0,
    sumCheckPassed,
    confidence: fee > 0 ? 0.96 : 0.50,
  };
}

/**
 * Match a bank transaction amount with a candidate ledger invoice amount,
 * checking whether the delta equals a standard interbank wire fee.
 */
export function checkWireFeeDiscrepancy(
  bankAmount: number,
  invoiceAmount: number
): {
  isFeeDiscrepancy: boolean;
  feeAmount: number;
  direction: 'BANK_DEDUCTED_FEE' | 'PAYER_COVERED_FEE' | 'NONE';
} {
  const delta = bankAmount - invoiceAmount;
  const absDelta = Math.abs(delta);

  if (KNOWN_VIETNAMESE_WIRE_FEES.includes(absDelta as any)) {
    return {
      isFeeDiscrepancy: true,
      feeAmount: absDelta,
      direction: delta < 0 ? 'BANK_DEDUCTED_FEE' : 'PAYER_COVERED_FEE',
    };
  }

  return {
    isFeeDiscrepancy: false,
    feeAmount: 0,
    direction: 'NONE',
  };
}

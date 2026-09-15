/**
 * Vietnamese Semantic Intent Normalizer & Banking Remarks NLP (Feature F09)
 * Interprets colloquial Vietnamese banking abbreviations, extracts invoice/tax/trace entities,
 * and matches transaction descriptions to ledger invoices with confidence scoring.
 */

import type { LedgerEntry } from '../../types/banking';

export type VietnameseRemarksIntent =
  | 'PAYMENT'
  | 'ADVANCE'
  | 'BANK_FEE'
  | 'PAYROLL'
  | 'WITHDRAWAL'
  | 'INTERNAL_TRANSFER'
  | 'OTHER';

export interface ExtractedRemarksResult {
  original: string;
  normalized: string;
  invoiceNumbers: string[];
  taxCodes: string[];
  phoneNumbers: string[];
  traceCodes: string[];
  intent: VietnameseRemarksIntent;
  confidence: number;
}

export const VIETNAMESE_BANKING_ABBREVIATIONS: Record<string, string> = {
  // Transfer terms
  ck: 'chuyển khoản',
  'c/k': 'chuyển khoản',
  'ch/khoan': 'chuyển khoản',
  ib: 'internet banking',
  mb: 'mobile banking',
  nps: 'chuyển mạch tài chính napas',
  napas: 'chuyển mạch tài chính napas',
  vietqr: 'mã thanh toán vietqr',

  // Payment terms
  tt: 'thanh toán',
  ttoan: 'thanh toán',
  'th/toan': 'thanh toán',
  't.toan': 'thanh toán',

  // Advance / Contract terms
  ung: 'tạm ứng',
  't/ung': 'tạm ứng',
  'tam ung': 'tạm ứng',
  dc: 'đặt cọc',
  hd: 'hóa đơn',
  hđ: 'hóa đơn',
  hdon: 'hóa đơn',
  hopdong: 'hợp đồng',
  'hop dong': 'hợp đồng',

  // Installment terms
  d1: 'đợt 1',
  d2: 'đợt 2',
  d3: 'đợt 3',
  gd1: 'giai đoạn 1',
  gd2: 'giai đoạn 2',

  // Tax / Fee terms
  mst: 'mã số thuế',
  vat: 'thuế giá trị gia tăng',
  gtgt: 'giá trị gia tăng',
  phi: 'phí dịch vụ',
  'phi ck': 'phí chuyển khoản',
  'phi gd': 'phí giao dịch',
  'phi dv': 'phí dịch vụ',
};

/**
 * Remove Vietnamese accents/diacritics for normalization
 */
export function removeVietnameseAccents(str: string): string {
  return str
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .replace(/đ/g, 'd')
    .replace(/Đ/g, 'D');
}

/**
 * Normalize colloquial Vietnamese banking abbreviations and extract structured entities
 */
export function normalizeVietnameseIntent(memo: string): ExtractedRemarksResult {
  if (!memo || !memo.trim()) {
    return {
      original: '',
      normalized: '',
      invoiceNumbers: [],
      taxCodes: [],
      phoneNumbers: [],
      traceCodes: [],
      intent: 'OTHER',
      confidence: 0,
    };
  }

  const raw = memo.trim();
  const lower = raw.toLowerCase();

  // 1. Extract Invoice / Contract numbers
  // Matches: "hd 88", "hd-2026-88", "hdon 131", "SO 4910", "HD::88", "INV-001"
  const invoiceRegex = /\b(?:hd|hđ|hdon|hoa\s*don|hop\s*dong|so|inv|pkt)[\s\-_:#]*([a-zA-Z0-9\/\-_]+)\b/gi;
  const invoiceNumbers: string[] = [];
  let match: RegExpExecArray | null;
  while ((match = invoiceRegex.exec(lower)) !== null) {
    const inv = match[1].toUpperCase();
    if (!invoiceNumbers.includes(inv)) {
      invoiceNumbers.push(inv);
    }
  }

  // Also handle dense punctuation: e.g. "CK...TIEN,,MAY--BOM//HD::88"
  if (invoiceNumbers.length === 0) {
    const denseMatch = /(?:hd|hđ|inv)[^\w]+([a-zA-Z0-9\-]+)/i.exec(lower);
    if (denseMatch && denseMatch[1]) {
      invoiceNumbers.push(denseMatch[1].toUpperCase());
    }
  }

  // 2. Extract Vietnamese Tax Codes (MST: 10 or 13 digits)
  const mstRegex = /\b(\d{10}(?:-\d{3})?)\b/g;
  const taxCodes: string[] = [];
  while ((match = mstRegex.exec(raw)) !== null) {
    if (!taxCodes.includes(match[1])) {
      taxCodes.push(match[1]);
    }
  }

  // 3. Extract Mobile Phone Numbers (10 digits starting with 03, 05, 07, 08, 09 or +84)
  const phoneRegex = /(?:\+84|0)(?:3|5|7|8|9)\d{8}\b/g;
  const phoneNumbers: string[] = [];
  while ((match = phoneRegex.exec(raw)) !== null) {
    if (!phoneNumbers.includes(match[0])) {
      phoneNumbers.push(match[0]);
    }
  }

  // 4. Extract Napas / VietQR Trace Codes
  const traceRegex = /\b(?:NPS|FT|MBB|TCB|VCB|BIDV|TX|VN)\d{6,}\b/gi;
  const traceCodes: string[] = [];
  while ((match = traceRegex.exec(raw)) !== null) {
    const trace = match[0].toUpperCase();
    if (!traceCodes.includes(trace)) {
      traceCodes.push(trace);
    }
  }

  // 5. Token expansion using abbreviation dictionary
  // Split on non-alphanumeric punctuation (except Vietnamese letters)
  const words = lower.split(/[^a-zA-Z0-9_đàáảãạăằắẳẵặâầấẩẫậèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵ]+/);
  const expandedTokens = words
    .filter(Boolean)
    .map((w) => VIETNAMESE_BANKING_ABBREVIATIONS[w] || w);
  const normalized = expandedTokens.join(' ');

  // 6. Intent classification
  let intent: VietnameseRemarksIntent = 'OTHER';
  let confidence = 0.5;

  const normNoAccent = removeVietnameseAccents(normalized).toLowerCase();

  if (/tam ung|ung|dat coc/i.test(normalized) || /tam ung/i.test(normNoAccent)) {
    intent = 'ADVANCE';
    confidence = 0.95;
  } else if (/phi duy tri|phi ql|phi dv|phi dich vu|quan ly tai khoan|phi thuong nien|phí/i.test(normalized)) {
    intent = 'BANK_FEE';
    confidence = 0.95;
  } else if (/luong|thuong|tien luong/i.test(normNoAccent)) {
    intent = 'PAYROLL';
    confidence = 0.92;
  } else if (/rut tien|rut vi/i.test(normNoAccent)) {
    intent = 'WITHDRAWAL';
    confidence = 0.90;
  } else if (/noi bo|dieu chuyen/i.test(normNoAccent)) {
    intent = 'INTERNAL_TRANSFER';
    confidence = 0.88;
  } else if (
    /chuyen khoan|thanh toan|ck|tt|napas|vietqr|tien hang|hoa don|hop dong|hd/i.test(normalized) ||
    /thanh toan|chuyen khoan|hoa don/i.test(normNoAccent)
  ) {
    intent = 'PAYMENT';
    confidence = 0.95;
  }

  return {
    original: raw,
    normalized,
    invoiceNumbers,
    taxCodes,
    phoneNumbers,
    traceCodes,
    intent,
    confidence,
  };
}

/**
 * Link transaction description to candidate ledger invoices using entity extraction and semantic similarity
 */
export function matchRemarksToLedger(
  narration: string,
  ledgers: LedgerEntry[]
): {
  matchedLedger?: LedgerEntry;
  confidence: number;
  explanation: string;
} {
  if (!narration || ledgers.length === 0) {
    return { confidence: 0, explanation: 'Không có thông tin diễn giải hoặc sổ cái rỗng' };
  }

  const extracted = normalizeVietnameseIntent(narration);
  const normNarration = removeVietnameseAccents(extracted.normalized).toLowerCase();

  let bestMatch: LedgerEntry | undefined;
  let highestScore = 0;
  let reason = '';

  for (const ledger of ledgers) {
    let score = 0;
    const docNoNorm = removeVietnameseAccents(ledger.docNo).toUpperCase();
    const partnerNorm = removeVietnameseAccents(ledger.partnerName).toLowerCase();

    // 1. Direct invoice number match
    for (const inv of extracted.invoiceNumbers) {
      if (docNoNorm.includes(inv) || inv.includes(docNoNorm)) {
        score += 0.85;
        reason = `Khớp số hóa đơn: ${inv} với chứng từ ${ledger.docNo}`;
        break;
      }
    }

    // 2. Partner name match
    if (partnerNorm && normNarration.includes(partnerNorm)) {
      score += 0.40;
      reason += (reason ? ' & ' : '') + `Khớp tên đối tác: ${ledger.partnerName}`;
    } else if (partnerNorm) {
      // Check partial tokens
      const partnerTokens = partnerNorm.split(/\s+/).filter((t) => t.length > 2 && t !== 'cong' && t !== 'ty');
      const matchedTokens = partnerTokens.filter((t) => normNarration.includes(t));
      if (matchedTokens.length >= 2) {
        score += 0.25;
        reason += (reason ? ' & ' : '') + `Khớp từ khóa đối tác: ${matchedTokens.join(', ')}`;
      }
    }

    // Cap at 0.99
    score = Math.min(0.99, score);

    if (score > highestScore && score >= 0.50) {
      highestScore = score;
      bestMatch = ledger;
    }
  }

  return {
    matchedLedger: bestMatch,
    confidence: highestScore,
    explanation: reason || 'Không tìm thấy chứng từ phù hợp',
  };
}

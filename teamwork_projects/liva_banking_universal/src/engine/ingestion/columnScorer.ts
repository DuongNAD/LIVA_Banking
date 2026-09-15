/**
 * Dynamic Column Role Inference & Candidate Header Scorer
 * Scores columns and candidate header rows for multi-bank statement ingestion.
 */

export type ColumnRole =
  | 'date'
  | 'valueDate'
  | 'code'
  | 'debit'
  | 'credit'
  | 'amount'
  | 'balance'
  | 'narration'
  | 'counterparty'
  | 'counterpartyAccount'
  | 'unknown';

export interface ColumnMapping {
  date: number;
  valueDate: number;
  code: number;
  debit: number;
  credit: number;
  amount: number;
  balance: number;
  narration: number;
  counterparty: number;
  counterpartyAccount: number;
}

export interface HeaderScoreResult {
  headerRowIndex: number;
  score: number;
  mapping: ColumnMapping;
  headers: string[];
}

export interface StatementMetadata {
  accountNumber?: string;
  accountName?: string;
  openingBalance?: number;
  currency?: string;
  period?: string;
}

const ROLE_PATTERNS: Record<Exclude<ColumnRole, 'unknown'>, RegExp[]> = {
  date: [
    /^(ngày\s*giao\s*dịch|ngày\s*gd|ngày\s*ct|ngày\s*hạch\s*toán|trans(?:action)?\s*date|posting\s*date|booking\s*date|date)$/i,
    /ngày.*(gd|giao\s*dịch|hạch\s*toán)/i,
    /^ngày$/i,
  ],
  valueDate: [
    /^(ngày\s*giá\s*trị|ngày\s*hiệu\s*lực|value\s*date|val\s*date)$/i,
    /giá\s*trị/i,
  ],
  code: [
    /^(mã\s*giao\s*dịch|mã\s*gd|số\s*giao\s*dịch|số\s*gd|số\s*chứng\s*từ|mã\s*chứng\s*từ|số\s*ct|mã\s*hóa\s*đơn|số\s*hóa\s*đơn|mã\s*hd|số\s*hd|ref(?:\s*no)?|trace(?:\s*no)?|doc(?:\s*no)?|trans(?:action)?\s*id|tx\s*code|mã\s*lệnh)$/i,
    /(mã\s*gd|số\s*gd|số\s*ct|chứng\s*từ|tham\s*chiếu|ref|ft)/i,
  ],
  debit: [
    /^(số\s*tiền\s*ghi\s*nợ|ghi\s*nợ|tiền\s*ghi\s*nợ|phát\s*sinh\s*nợ|tiền\s*chi|rút\s*tiền|nợ|debit(?:\s*amount)?)$/i,
    /ghi\s*nợ|phát\s*sinh\s*nợ/i,
    /^nợ$/i,
  ],
  credit: [
    /^(số\s*tiền\s*ghi\s*có|ghi\s*có|tiền\s*ghi\s*có|phát\s*sinh\s*có|tiền\s*gửi|nạp\s*tiền|thu\s*tiền|có|credit(?:\s*amount)?)$/i,
    /ghi\s*có|phát\s*sinh\s*có/i,
    /^có$/i,
  ],
  amount: [
    /^(số\s*tiền|số\s*tiền\s*giao\s*dịch|số\s*tiền\s*phải\s*thu|số\s*tiền\s*phải\s*trả|phát\s*sinh|amount|trans(?:action)?\s*amount|net\s*amount)$/i,
    /số\s*tiền/i,
  ],
  balance: [
    /^(số\s*dư|số\s*dư\s*sau\s*gd|số\s*dư\s*cuối|số\s*dư\s*lũy\s*kế|số\s*dư\s*khả\s*dụng|balance|running\s*balance|current\s*balance)$/i,
    /số\s*dư/i,
  ],
  narration: [
    /^(nội\s*dung\s*chi\s*tiết|nội\s*dung|diễn\s*giải(?:\s*kế\s*toán)?|chi\s*tiết\s*giao\s*dịch|chi\s*tiết|narration|description|details|memo|lý\s*do)$/i,
    /nội\s*dung|diễn\s*giải/i,
  ],
  counterparty: [
    /^(tên\s*người\s*gửi\/?thụ\s*hưởng|tên\s*người\s*thụ\s*hưởng|người\s*thụ\s*hưởng|người\s*gửi|tên\s*đối\s*tác|đối\s*tác|khách\s*hàng|beneficiary(?:\s*name)?|counterparty(?:\s*name)?|partner(?:\s*name)?)$/i,
    /đối\s*tác|thụ\s*hưởng|khách\s*hàng/i,
  ],
  counterpartyAccount: [
    /^(tài\s*khoản\s*đối\s*ứng|số\s*tk\s*đối\s*ứng|tk\s*thụ\s*hưởng|số\s*tk\s*thụ\s*hưởng|tài\s*khoản\s*thụ\s*hưởng|beneficiary\s*acc(?:ount)?|counterparty\s*acc(?:ount)?)$/i,
    /tk\s*đối\s*ứng|tk\s*thụ\s*hưởng/i,
  ],
};

/**
 * Classifies the role of a single column header text.
 */
export function classifyHeader(cellText: string): ColumnRole {
  const clean = cellText.trim().toLowerCase();
  if (!clean) return 'unknown';

  for (const [role, patterns] of Object.entries(ROLE_PATTERNS) as [Exclude<ColumnRole, 'unknown'>, RegExp[]][]) {
    for (const pat of patterns) {
      if (pat.test(clean)) {
        return role;
      }
    }
  }

  return 'unknown';
}

/**
 * Evaluates candidate header rows and chooses the one with the highest confidence score.
 */
export function scoreCandidateHeaderRows(rows: string[][], maxSearchRows = 15): HeaderScoreResult {
  let bestRowIndex = -1;
  let bestScore = -1;
  let bestMapping: ColumnMapping = {
    date: -1,
    valueDate: -1,
    code: -1,
    debit: -1,
    credit: -1,
    amount: -1,
    balance: -1,
    narration: -1,
    counterparty: -1,
    counterpartyAccount: -1,
  };
  let bestHeaders: string[] = [];

  const limit = Math.min(rows.length, maxSearchRows);

  for (let r = 0; r < limit; r++) {
    const row = rows[r];
    if (!row || row.length < 2) continue;

    const mapping: ColumnMapping = {
      date: -1,
      valueDate: -1,
      code: -1,
      debit: -1,
      credit: -1,
      amount: -1,
      balance: -1,
      narration: -1,
      counterparty: -1,
      counterpartyAccount: -1,
    };

    let score = 0;
    const recognizedRoles = new Set<ColumnRole>();

    for (let c = 0; c < row.length; c++) {
      const cell = row[c];
      const role = classifyHeader(cell);

      if (role !== 'unknown' && !recognizedRoles.has(role)) {
        recognizedRoles.add(role);
        mapping[role] = c;

        switch (role) {
          case 'date':
            score += 30;
            break;
          case 'debit':
          case 'credit':
            score += 25;
            break;
          case 'amount':
            score += 20;
            break;
          case 'balance':
            score += 15;
            break;
          case 'code':
            score += 15;
            break;
          case 'narration':
            score += 20;
            break;
          case 'counterparty':
            score += 10;
            break;
          default:
            score += 5;
            break;
        }
      }
    }

    // Must have at least a date column AND an amount/debit/credit column to be a valid header
    const hasDate = mapping.date >= 0;
    const hasAmount = mapping.amount >= 0 || (mapping.debit >= 0 && mapping.credit >= 0) || mapping.debit >= 0 || mapping.credit >= 0;

    if (hasDate && hasAmount && score > bestScore) {
      bestScore = score;
      bestRowIndex = r;
      bestMapping = mapping;
      bestHeaders = row;
    }
  }

  // Fallback if no clean header recognized
  if (bestRowIndex === -1 && rows.length > 0) {
    bestRowIndex = 0;
    bestHeaders = rows[0] || [];
  }

  return {
    headerRowIndex: bestRowIndex,
    score: bestScore,
    mapping: bestMapping,
    headers: bestHeaders,
  };
}

/**
 * Extracts bank metadata (account number, account name, opening balance, currency)
 * from header rows preceding the main transaction table.
 */
export function extractStatementMetadata(
  rows: string[][],
  stopRowIndex: number,
  parseAmountFn: (raw: string) => number
): StatementMetadata {
  const metadata: StatementMetadata = {
    currency: 'VND',
  };

  const limit = Math.min(rows.length, stopRowIndex >= 0 ? stopRowIndex : rows.length);

  for (let r = 0; r < limit; r++) {
    const row = rows[r];
    const fullRowText = row.join(' ');
    const lowerText = fullRowText.toLowerCase();

    // Account Number
    if (lowerText.includes('số tài khoản') || lowerText.includes('số tk') || lowerText.includes('account no')) {
      for (const cell of row) {
        const cLower = cell.toLowerCase();
        if (cLower.includes('số tài khoản') || cLower.includes('số tk') || cLower.includes('account no')) {
          const m = cell.match(/\b\d{8,16}\b/);
          if (m) {
            metadata.accountNumber = m[0];
            break;
          }
        } else {
          const m = cell.match(/\b\d{8,16}\b/);
          if (m && !metadata.accountNumber) {
            metadata.accountNumber = m[0];
          }
        }
      }
    }

    // Account Name
    if (lowerText.includes('tên tài khoản') || lowerText.includes('account name')) {
      for (const cell of row) {
        const cLower = cell.toLowerCase();
        if (cLower.includes('tên tài khoản') || cLower.includes('account name')) {
          const cleanName = cell.replace(/^.*(?:tên\s*tài\s*khoản|account\s*name)\s*[:\-]?\s*/i, '').trim();
          if (cleanName.length > 2) {
            metadata.accountName = cleanName;
            break;
          }
        }
      }
    }

    // Opening Balance
    if (lowerText.includes('số dư đầu kỳ') || lowerText.includes('opening balance') || lowerText.includes('số dư đầu')) {
      for (const cell of row) {
        const amt = parseAmountFn(cell);
        if (amt > 0) {
          metadata.openingBalance = amt;
          break;
        }
      }
    }

    // Period
    if (lowerText.includes('kỳ sao kê') || lowerText.includes('từ ngày') || lowerText.includes('statement period')) {
      metadata.period = fullRowText.trim();
    }
  }

  return metadata;
}

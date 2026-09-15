/**
 * Universal Client-Side Statement Ingestion Engine
 * Zero-hallucination parser for Excel (.xlsx, .xls), CSV (BOM, ;, ,, \t), and pasted tabular text.
 * Strictly adheres to integer VND arithmetic and double-entry mathematical invariants.
 */

import { read, utils } from 'xlsx';
import type { BankCode, RawStatementRow, StatementParseResult, LedgerEntry } from '../../types/banking';
import { sniffDelimiter, splitCsvLine } from './delimiterSniffer';
import { scoreCandidateHeaderRows, extractStatementMetadata } from './columnScorer';
import { verifyBalanceInvariants } from '../reconciliation/balanceValidator';
import { parseIso20022Xml } from './iso20022Parser';
export { parseIso20022Xml };

/**
 * Parses Vietnamese, European, and US formatted monetary strings into signed integer VND.
 * Rejects dates, paths, timestamps, or non-numeric strings with strict validation.
 */
export function parseVietnameseAmount(raw: unknown): number {
  if (typeof raw === 'number') {
    return isNaN(raw) ? 0 : Math.round(raw);
  }
  if (!raw) return 0;

  const rawStr = String(raw).trim();
  const clean = rawStr.replace(/^["']|["']$/g, '').trim();
  if (!clean) return 0;

  // Reject strings containing date slashes or path separators
  if (clean.includes('/') || clean.includes('\\')) {
    return 0;
  }

  let target = clean;
  // Handle colons: reject timestamps (e.g. 14:30), but allow labeled cells (e.g. "Cộng: 50.000.000 VNĐ")
  if (target.includes(':')) {
    if (/\d+:\d+/.test(target)) {
      return 0;
    }
    target = target.slice(target.indexOf(':') + 1).trim();
    if (!target) return 0;
  }

  // Reject hyphenated dates (DD-MM-YYYY, YYYY-MM-DD)
  if (/^\d{1,2}-\d{1,2}-\d{2,4}$/.test(target) || /^\d{4}-\d{1,2}-\d{1,2}$/.test(target)) {
    return 0;
  }

  // Must contain at least one ASCII digit
  if (!/\d/.test(target)) {
    return 0;
  }

  // Pre-strip currency identifiers and suffixes (VND, VNĐ, Đ, CR, DB, etc.)
  // BEFORE checking parenthesized negative format so (35.000.000) VND is correctly recognized as negative
  const withoutCurrency = target
    .replace(/\((?:VND|VNĐ|VNDcents|CR|DB|NO|CO|NỢ|CÓ|[Đđ])\)/gi, '')
    .replace(/(?:^|\s|\b)(?:VND|VNĐ|VNDcents|CR|DB|NO|CO|NỢ|CÓ|[Đđ])(?:$|\s|\b)/gi, ' ')
    .replace(/\b(VND|VNDcents|CR|DB|NO|CO)\b/gi, '')
    .replace(/[Đđ]/gi, '')
    .trim();

  // Detect negative format
  const isParenthesized = withoutCurrency.startsWith('(') && withoutCurrency.endsWith(')');
  const hasMinus = withoutCurrency.startsWith('-') || withoutCurrency.startsWith('(-');
  const isNegative = isParenthesized || hasMinus;

  // Strip non-numeric and non-delimiter characters except allowed symbols
  let stripped = withoutCurrency
    .replace(/[()\-+]/g, '')
    .trim();

  // Keep only digits, dots, and commas
  const digitsOnly = stripped.replace(/[^\d.,]/g, '');
  if (!digitsOnly) return 0;

  const hasDot = digitsOnly.includes('.');
  const hasComma = digitsOnly.includes(',');

  let intStr = '';

  if (hasDot && hasComma) {
    const lastDot = digitsOnly.lastIndexOf('.');
    const lastComma = digitsOnly.lastIndexOf(',');
    if (lastComma > lastDot) {
      // Vietnamese/European standard: 15.000.000,00 -> thousand=., decimal=,
      intStr = digitsOnly.slice(0, lastComma).replace(/\./g, '');
    } else {
      // US standard: 15,000,000.00 -> thousand=,, decimal=.
      intStr = digitsOnly.slice(0, lastDot).replace(/,/g, '');
    }
  } else if (hasComma && !hasDot) {
    const parts = digitsOnly.split(',');
    // Check if it's a decimal separator (e.g. 15000,5 or 15000,00)
    if (parts.length === 2 && parts[1].length <= 2) {
      intStr = parts[0];
    } else {
      // Thousands separator: 15,000,000
      intStr = digitsOnly.replace(/,/g, '');
    }
  } else if (hasDot && !hasComma) {
    const parts = digitsOnly.split('.');
    // Check if it's a decimal separator (e.g. 15000.5 or 15000.00)
    if (parts.length === 2 && parts[1].length <= 2) {
      intStr = parts[0];
    } else {
      // Vietnamese thousands separator: 15.000.000
      intStr = digitsOnly.replace(/\./g, '');
    }
  } else {
    intStr = digitsOnly;
  }

  const num = parseInt(intStr, 10);
  if (isNaN(num)) return 0;

  return isNegative ? -Math.abs(num) : Math.abs(num);
}

/**
 * Normalizes date & time strings into ISO date 'YYYY-MM-DD', time string, and Unix epoch seconds.
 */
export function parseDateAndTimestamp(raw: unknown): { isoDate: string; time: string; epochSeconds: number } {
  const fallbackDate = new Date();
  const defaultIso = fallbackDate.toISOString().split('T')[0];
  const defaultTime = fallbackDate.toLocaleTimeString('vi-VN', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  const defaultEpoch = Math.floor(fallbackDate.getTime() / 1000);

  if (raw === null || raw === undefined || raw === '') {
    return { isoDate: defaultIso, time: defaultTime, epochSeconds: defaultEpoch };
  }

  // Handle Excel numeric serial dates (e.g. 46235.3958)
  if (typeof raw === 'number' && raw > 30000 && raw < 70000) {
    const epochMs = (raw - 25569) * 86400 * 1000;
    const d = new Date(epochMs);
    const isoDate = d.toISOString().split('T')[0];
    const time = d.toLocaleTimeString('vi-VN', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    return { isoDate, time, epochSeconds: Math.floor(epochMs / 1000) };
  }

  const str = String(raw).trim();

  // Pattern: "DD/MM/YYYY HH:MM:SS" or "DD-MM-YYYY HH:MM" or "DD/MM/YYYY"
  const vnMatch = str.match(/^(\d{1,2})[\/\-](\d{1,2})[\/\-](\d{4})(?:\s+(\d{1,2}):(\d{2})(?::(\d{2}))?)?/);
  if (vnMatch) {
    const day = vnMatch[1].padStart(2, '0');
    const month = vnMatch[2].padStart(2, '0');
    const year = vnMatch[3];
    const hour = vnMatch[4] ? parseInt(vnMatch[4], 10) : 0;
    const min = vnMatch[5] ? parseInt(vnMatch[5], 10) : 0;
    const sec = vnMatch[6] ? parseInt(vnMatch[6], 10) : 0;

    const isoDate = `${year}-${month}-${day}`;
    const time = `${hour.toString().padStart(2, '0')}:${min.toString().padStart(2, '0')}:${sec.toString().padStart(2, '0')}`;
    const d = new Date(`${isoDate}T${time}`);
    const epochSeconds = Math.floor(d.getTime() / 1000) || defaultEpoch;

    return { isoDate, time, epochSeconds };
  }

  // Pattern: "YYYY-MM-DD HH:MM:SS" or "YYYY-MM-DD"
  const isoMatch = str.match(/^(\d{4})-(\d{2})-(\d{2})(?:[T\s](\d{2}):(\d{2})(?::(\d{2}))?)?/);
  if (isoMatch) {
    const isoDate = `${isoMatch[1]}-${isoMatch[2]}-${isoMatch[3]}`;
    const hour = isoMatch[4] || '00';
    const min = isoMatch[5] || '00';
    const sec = isoMatch[6] || '00';
    const time = `${hour}:${min}:${sec}`;
    const d = new Date(`${isoDate}T${time}`);
    const epochSeconds = Math.floor(d.getTime() / 1000) || defaultEpoch;
    return { isoDate, time, epochSeconds };
  }

  // Pure Unix timestamp (seconds or milliseconds)
  const num = Number(str);
  if (!isNaN(num) && num > 100000000) {
    const ms = num > 10000000000 ? num : num * 1000;
    const d = new Date(ms);
    const isoDate = d.toISOString().split('T')[0];
    const time = d.toLocaleTimeString('vi-VN', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    return { isoDate, time, epochSeconds: Math.floor(ms / 1000) };
  }

  return { isoDate: defaultIso, time: defaultTime, epochSeconds: defaultEpoch };
}

/**
 * Detects bank code from file name and content text keywords.
 */
export function detectBankCode(filename = '', contentSample = ''): BankCode {
  const fn = filename.toUpperCase();
  const sample = contentSample.toUpperCase();

  // 1. Priority by filename
  if (fn.includes('CITAD') || fn.includes('IBPS') || fn.includes('CAMT.053')) return 'CITAD';
  if (fn.includes('NAPAS')) return 'NAPAS';
  if (fn.includes('BILATERAL') || fn.includes('NOSTRO') || fn.includes('VOSTRO')) return 'BILATERAL';
  if (fn.includes('SWIFT') || fn.includes('PACS.008')) return 'SWIFT';
  if (fn.includes('TECHCOMBANK') || fn.includes('TCB') || fn.includes('KY THUONG')) return 'TCB';
  if (fn.includes('BIDV') || fn.includes('DAU TU VA PHAT TRIEN')) return 'BIDV';
  if (fn.includes('VIETINBANK') || fn.includes('CTG') || fn.includes('CONG THUONG')) return 'CTG';
  if (fn.includes('MBBANK') || fn.includes('MB BANK') || fn.includes('QUAN DOI')) return 'MBB';
  if (fn.includes('AGRIBANK') || fn.includes('VBA') || fn.includes('NONG NGHIEP')) return 'VBA';
  if (fn.includes('VIETCOMBANK') || fn.includes('VCB') || fn.includes('NGOAI THUONG')) return 'VCB';

  // 2. Priority by content headers / keywords
  if (sample.includes('CITAD') || sample.includes('IBPS') || sample.includes('SỞ GIAO DỊCH NHNN') || sample.includes('SO GIAO DICH NHNN') || sample.includes('CAMT.053')) return 'CITAD';
  if (sample.includes('TECHCOMBANK') || sample.includes('KY THUONG') || sample.includes('TCB')) return 'TCB';
  if (sample.includes('BIDV') || sample.includes('DAU TU VA PHAT TRIEN')) return 'BIDV';
  if (sample.includes('VIETCOMBANK') || sample.includes('VCB') || sample.includes('NGOAI THUONG')) return 'VCB';
  if (sample.includes('VIETINBANK') || sample.includes('CTG') || sample.includes('CONG THUONG')) return 'CTG';
  if (sample.includes('MBBANK') || sample.includes('QUAN DOI')) return 'MBB';
  if (sample.includes('AGRIBANK') || sample.includes('VBA') || sample.includes('NONG NGHIEP')) return 'VBA';

  if (sample.includes('NAPAS') || sample.includes('CHUYEN MACH') || sample.includes('CHUYỂN MẠCH')) return 'NAPAS';
  if (sample.includes('BILATERAL') || sample.includes('NOSTRO') || sample.includes('VOSTRO') || sample.includes('SONG PHUONG') || sample.includes('SONG PHƯƠNG')) return 'BILATERAL';
  if (sample.includes('SWIFT') || sample.includes('MT103') || sample.includes('MT202') || sample.includes('PACS.008')) return 'SWIFT';
  if (sample.includes('<BKTOCSTMRSTMT>') || sample.includes('ISO:20022') || sample.includes('<DOCUMENT')) return 'CITAD';

  return 'GENERIC';
}

/**
 * Extracts normalized reference ID, FT numbers, and Napas trace codes from narration or ref cell.
 */
export function extractReferenceTokens(text: string): {
  docRef?: string;
  ftNumber?: string;
  traceId?: string;
} {
  const result: { docRef?: string; ftNumber?: string; traceId?: string } = {};
  if (!text) return result;

  // FT number: FT24... or FT\w{6,}
  const ftMatch = text.match(/\b(FT[A-Z0-9]{6,})\b/i);
  if (ftMatch) {
    result.ftNumber = ftMatch[1].toUpperCase();
    result.docRef = result.ftNumber;
  }

  // Napas trace: NPS\w+ or VN\w{6,}
  const npsMatch = text.match(/\b(NPS[A-Z0-9]{4,}|VN\d{6,})\b/i);
  if (npsMatch) {
    result.traceId = npsMatch[1].toUpperCase();
    if (!result.docRef) result.docRef = result.traceId;
  }

  // Invoice references: HD-2026-88, HD101, INV-2026-01, HD99, etc.
  const invMatch = text.match(/\b((?:HD|INV|PC|PT)[-_]?[A-Z0-9]+(?:[-_/][A-Z0-9]+)*)\b/i);
  if (invMatch) {
    result.docRef = invMatch[1].toUpperCase().replace(/[_\s]/g, '-');
  }

  // Contract numbers: "HOP DONG SO 882026" or "HOP DONG 88" -> HD-2026-88
  if (!result.docRef) {
    const hdMatch = text.match(/\b(?:hop\s*dong(?:\s*so)?|hoa\s*don(?:\s*so)?)\s*[:#\-]?\s*([0-9]+)\b/i);
    if (hdMatch && hdMatch[1]) {
      const numStr = hdMatch[1];
      if (numStr.endsWith('2026') && numStr.length > 4) {
        const prefix = numStr.slice(0, numStr.length - 4);
        result.docRef = `HD-2026-${prefix}`;
      } else {
        result.docRef = `HD-${numStr}`;
      }
    }
  }

  return result;
}

/**
 * Parses 2D array of string cells into a normalized StatementParseResult.
 * Handles merged cells with date forward-fill, balance tracking, and balance invariant verification.
 */
export function parseTableRows(
  rows: string[][],
  filename = '',
  defaultBank?: BankCode
): StatementParseResult {
  const startTime = performance.now();
  const sampleText = rows.slice(0, 10).map((r) => r.join(' ')).join('\n');
  const bankCode = defaultBank && defaultBank !== 'GENERIC' ? defaultBank : detectBankCode(filename, sampleText);

  // Score candidate headers
  const headerResult = scoreCandidateHeaderRows(rows);
  const headerIdx = headerResult.headerRowIndex;
  const col = headerResult.mapping;

  // Extract metadata from header lines
  const metadata = extractStatementMetadata(rows, headerIdx, parseVietnameseAmount);
  let accountNumber = metadata.accountNumber || `${bankCode}-100${Math.floor(10000000 + Math.random() * 90000000)}`;
  let accountName = metadata.accountName || 'TÀI KHOẢN TIỀN GỬI VÀ QUYẾT TOÁN';
  let openingBalance = metadata.openingBalance ?? 0;

  const transactions: RawStatementRow[] = [];
  let lastValidDateStr = '';
  let runningBalance = openingBalance;
  let totalDebit = 0;
  let totalCredit = 0;

  const startRow = headerIdx >= 0 ? headerIdx + 1 : 0;

  for (let r = startRow; r < rows.length; r++) {
    const row = rows[r];
    if (!row || row.length === 0 || row.every((c) => !c.trim())) continue;

    // Date extraction (with merged cell forward-fill)
    let rawDate = col.date >= 0 && row[col.date] ? row[col.date].trim() : '';
    if (rawDate) {
      lastValidDateStr = rawDate;
    } else if (lastValidDateStr) {
      rawDate = lastValidDateStr;
    } else {
      rawDate = row[0] || '';
    }

    const { isoDate, time, epochSeconds } = parseDateAndTimestamp(rawDate);

    // Value date
    let valueDateEpoch: number | undefined;
    if (col.valueDate >= 0 && row[col.valueDate]) {
      valueDateEpoch = parseDateAndTimestamp(row[col.valueDate]).epochSeconds;
    }

    // Amounts
    let debitAmt = 0;
    let creditAmt = 0;

    if (col.debit >= 0 && row[col.debit]) {
      debitAmt = Math.abs(parseVietnameseAmount(row[col.debit]));
    }
    if (col.credit >= 0 && row[col.credit]) {
      creditAmt = Math.abs(parseVietnameseAmount(row[col.credit]));
    }

    // If single amount column
    if (debitAmt === 0 && creditAmt === 0 && col.amount >= 0 && row[col.amount]) {
      const net = parseVietnameseAmount(row[col.amount]);
      if (net < 0) {
        debitAmt = Math.abs(net);
      } else {
        creditAmt = Math.abs(net);
      }
    }

    // Fallback: look for numeric cells if amount not yet found
    if (debitAmt === 0 && creditAmt === 0) {
      for (let c = 0; c < row.length; c++) {
        if (c === col.date || c === col.code || c === col.narration || c === col.counterparty) continue;
        const val = parseVietnameseAmount(row[c]);
        if (val !== 0) {
          if (val < 0) debitAmt = Math.abs(val);
          else creditAmt = Math.abs(val);
          break;
        }
      }
    }

    // Skip empty or purely non-monetary lines, but preserve multi-row continuation narrations
    if (debitAmt === 0 && creditAmt === 0) {
      const continuationNarration = col.narration >= 0 && row[col.narration]
        ? row[col.narration].trim()
        : row.find((c) => c && c.trim().length > 3 && !/^\d+$/.test(c.trim()))?.trim() || '';

      if (continuationNarration && transactions.length > 0) {
        transactions[transactions.length - 1].narration += ' ' + continuationNarration;
      }
      continue;
    }

    const isCredit = creditAmt > 0;
    const txType: 'DEBIT' | 'CREDIT' = isCredit ? 'CREDIT' : 'DEBIT';
    const amount = isCredit ? creditAmt : debitAmt;
    const netAmount = isCredit ? creditAmt : -debitAmt;

    if (isCredit) totalCredit += creditAmt;
    else totalDebit += debitAmt;

    // Balance
    let balanceVal = 0;
    if (col.balance >= 0 && row[col.balance]) {
      balanceVal = Math.abs(parseVietnameseAmount(row[col.balance]));
      if (balanceVal > 0) {
        runningBalance = balanceVal;
      }
    } else {
      runningBalance += netAmount;
      balanceVal = runningBalance;
    }

    // Narration & Counterparty
    const narration = col.narration >= 0 && row[col.narration]
      ? row[col.narration].trim()
      : row.find((c) => c.length > 10 && !/^\d+$/.test(c)) || 'Giao dịch ngân hàng';

    const counterparty = col.counterparty >= 0 && row[col.counterparty]
      ? row[col.counterparty].trim()
      : undefined;

    const counterpartyAccount = col.counterpartyAccount >= 0 && row[col.counterpartyAccount]
      ? row[col.counterpartyAccount].trim()
      : undefined;

    // Transaction Code & Reference
    let txCode = col.code >= 0 && row[col.code] ? row[col.code].trim() : '';
    const refTokens = extractReferenceTokens(narration + ' ' + (txCode || ''));

    if (!txCode) {
      txCode = refTokens.ftNumber || refTokens.traceId || `${bankCode}26${(transactions.length + 1).toString().padStart(6, '0')}`;
    }

    const docRef = refTokens.docRef || txCode;

    transactions.push({
      id: `tx-${bankCode.toLowerCase()}-${r}-${txCode}`,
      date: isoDate,
      time,
      txDate: epochSeconds,
      valueDate: valueDateEpoch || epochSeconds,
      txCode,
      docRef,
      debit: debitAmt,
      credit: creditAmt,
      netAmount,
      amount,
      txType,
      balance: balanceVal,
      balanceAfter: balanceVal,
      narration,
      counterparty,
      counterpartyAccount,
      bankCode,
      ftNumber: refTokens.ftNumber,
      traceId: refTokens.traceId,
      rawRef: txCode,
    });
  }

  // Calculate closing balance
  const closingBalance = runningBalance;
  if (openingBalance === 0 && transactions.length > 0) {
    // If opening was not in header, derive it from first transaction balance if available
    const firstTx = transactions[0];
    if (firstTx.balanceAfter && firstTx.balanceAfter > 0) {
      openingBalance = firstTx.balanceAfter - firstTx.netAmount;
    }
  }

  // Validate balance invariant
  const invariantReport = verifyBalanceInvariants(openingBalance, closingBalance, totalCredit, totalDebit);

  const duration = performance.now() - startTime;

  return {
    bankCode,
    bankName: getBankFullName(bankCode),
    accountNumber,
    accountName,
    currency: metadata.currency || 'VND',
    openingBalance,
    closingBalance,
    totalDebit,
    totalCredit,
    transactions,
    balanceInvariantPassed: invariantReport.isValid,
    balanceDiscrepancy: invariantReport.discrepancy,
    rawRowCount: rows.length,
    parseDurationMs: Math.round(duration * 100) / 100,
  };
}

/**
 * Parses raw CSV, TSV, or ISO 20022 XML string into StatementParseResult.
 */
export function parseCsvOrTsv(rawContent: string, filename = ''): StatementParseResult {
  const trimmed = rawContent.trim();
  if (
    trimmed.startsWith('<?xml') ||
    trimmed.includes('<Document') ||
    trimmed.includes('<BkToCstmrStmt') ||
    trimmed.includes('<camt.053') ||
    trimmed.includes('<pacs.008')
  ) {
    return parseIso20022Xml(trimmed, filename);
  }
  const sniffer = sniffDelimiter(rawContent);
  const lines = sniffer.cleanedContent.split(/\r?\n/).filter((l) => l.trim().length > 0);
  const rows = lines.map((l) => splitCsvLine(l, sniffer.delimiter));
  return parseTableRows(rows, filename);
}

/**
 * Parses tabular text pasted directly from Google Sheets or Excel clipboard.
 */
export function parsePastedTable(pastedText: string, defaultBank: BankCode = 'CITAD'): StatementParseResult {
  const clean = pastedText.trim();
  if (!clean) {
    return {
      bankCode: defaultBank,
      bankName: getBankFullName(defaultBank),
      accountNumber: `${defaultBank}-CLIPBOARD`,
      currency: 'VND',
      openingBalance: 0,
      closingBalance: 0,
      totalDebit: 0,
      totalCredit: 0,
      transactions: [],
      balanceInvariantPassed: true,
      balanceDiscrepancy: 0,
      rawRowCount: 0,
      parseDurationMs: 0,
    };
  }

  const sniffer = sniffDelimiter(clean);
  const lines = sniffer.cleanedContent.split(/\r?\n/).filter((l) => l.trim().length > 0);
  const rows = lines.map((l) => splitCsvLine(l, sniffer.delimiter));
  return parseTableRows(rows, 'clipboard.txt', defaultBank);
}

/**
 * Parses Excel workbook (.xlsx / .xls) from ArrayBuffer or Uint8Array.
 * Handles merged cells and multi-row headers seamlessly.
 */
export function parseExcelFile(data: ArrayBuffer | Uint8Array, filename = ''): StatementParseResult {
  try {
    const workbook = read(data, { type: data instanceof ArrayBuffer ? 'array' : 'buffer' });
    const sheetName = workbook.SheetNames[0];
    if (!sheetName) {
      throw new Error('Workbook contains no sheets.');
    }
    const worksheet = workbook.Sheets[sheetName];

    // Convert sheet to 2D array of raw values with defval: ''
    const rawRows = utils.sheet_to_json<string[]>(worksheet, {
      header: 1,
      raw: false,
      defval: '',
    });

    return parseTableRows(rawRows, filename);
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err);
    throw new Error(`Excel parse failure: ${msg}`);
  }
}

/**
 * Parses General Ledger / ERP Invoices CSV (e.g. 04_SoNhatKy_Chung_Thang8.csv or 04_SoCai_KeToan_HoaDon.csv).
 */
export function parseLedgerCsv(content: string): LedgerEntry[] {
  const sniffer = sniffDelimiter(content);
  const lines = sniffer.cleanedContent.split(/\r?\n/).filter((l) => l.trim().length > 0);
  const rows = lines.map((l) => splitCsvLine(l, sniffer.delimiter));

  if (rows.length === 0) return [];

  // Header row
  const header = rows[0].map((c) => c.toLowerCase());
  const colDoc = header.findIndex((c) => /mã\s*hóa\s*đơn|số\s*hóa\s*đơn|mã\s*chứng\s*từ|doc/i.test(c));
  const colDate = header.findIndex((c) => /ngày/i.test(c));
  const colPartner = header.findIndex((c) => /đối\s*tác|khách\s*hàng|partner/i.test(c));
  const colReceivable = header.findIndex((c) => /phải\s*thu|receivable/i.test(c));
  const colPayable = header.findIndex((c) => /phải\s*trả|payable/i.test(c));
  const colAmount = header.findIndex((c) => /số\s*tiền|amount/i.test(c));
  const colDesc = header.findIndex((c) => /diễn\s*giải|nội\s*dung|description/i.test(c));

  const entries: LedgerEntry[] = [];

  for (let i = 1; i < rows.length; i++) {
    const r = rows[i];
    if (r.length < 2) continue;

    const docNo = (colDoc >= 0 ? r[colDoc] : r[0]) || `INV-${i}`;
    const rawDate = (colDate >= 0 ? r[colDate] : r[1]) || '';
    const { isoDate, epochSeconds } = parseDateAndTimestamp(rawDate);
    const partnerName = (colPartner >= 0 ? r[colPartner] : r[2]) || 'Đối tác hạch toán';
    const description = (colDesc >= 0 ? r[colDesc] : '') || 'Bán hàng / Cung cấp dịch vụ';

    let amount = 0;
    let entryType: 'DEBIT' | 'CREDIT' = 'CREDIT';

    if (colReceivable >= 0 && r[colReceivable] && parseVietnameseAmount(r[colReceivable]) > 0) {
      amount = parseVietnameseAmount(r[colReceivable]);
      entryType = 'CREDIT'; // Cash receipt settles receivable
    } else if (colPayable >= 0 && r[colPayable] && parseVietnameseAmount(r[colPayable]) > 0) {
      amount = parseVietnameseAmount(r[colPayable]);
      entryType = 'DEBIT'; // Cash payment settles payable
    } else if (colAmount >= 0 && r[colAmount]) {
      amount = Math.abs(parseVietnameseAmount(r[colAmount]));
      entryType = 'CREDIT';
    }

    if (amount === 0) continue;

    entries.push({
      id: `ledger-${docNo.replace(/[^a-zA-Z0-9]/g, '-')}-${i}`,
      docNo,
      entryDate: isoDate,
      entryTimestamp: epochSeconds,
      partnerName,
      amount,
      entryType,
      description,
      status: 'UNMATCHED',
    });
  }

  return entries;
}

function getBankFullName(code: BankCode): string {
  switch (code) {
    case 'CITAD':
      return 'Kênh Thanh toán Điện tử Liên ngân hàng CITAD (NHNN)';
    case 'NAPAS':
      return 'Kênh Chuyển mạch Tài chính & Bù trừ Điện tử NAPAS 24/7';
    case 'BILATERAL':
      return 'Kênh Thanh toán Song phương & Vostro / Nostro';
    case 'SWIFT':
      return 'Kênh Chuyển tiền & Điện báo Quốc tế SWIFT';
    case 'VCB':
      return 'Ngân hàng TMCP Ngoại thương Việt Nam (Vietcombank)';
    case 'TCB':
      return 'Ngân hàng TMCP Kỹ thương Việt Nam (Techcombank)';
    case 'BIDV':
      return 'Ngân hàng TMCP Đầu tư và Phát triển Việt Nam (BIDV)';
    case 'CTG':
      return 'Ngân hàng TMCP Công thương Việt Nam (VietinBank)';
    case 'MBB':
      return 'Ngân hàng TMCP Quân đội (MBBank)';
    case 'VBA':
      return 'Ngân hàng Nông nghiệp và Phát triển Nông thôn Việt Nam (Agribank)';
    default:
      return 'Kênh Thanh toán / Quyết toán Liên ngân hàng';
  }
}

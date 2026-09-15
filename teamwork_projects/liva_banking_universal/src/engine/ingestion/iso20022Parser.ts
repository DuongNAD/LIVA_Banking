/**
 * ISO 20022 XML Statement Parser (camt.053.001.02 / 04 / 08 & pacs.008)
 * Standard interbank electronic statement ingestion compliant with SBV & international standards.
 * Zero-dependency, pure TypeScript implementation safe for both Browser and Node.js runtime.
 */

import type { BankCode, RawStatementRow, StatementParseResult } from '../../types/banking';
import { parseVietnameseAmount, parseDateAndTimestamp, extractReferenceTokens } from './universalParser';
import { verifyBalanceInvariants } from '../reconciliation/balanceValidator';

/**
 * Extracts inner text of an XML tag, ignoring attributes and namespaces.
 */
function getXmlTagContent(xml: string, tagName: string): string | null {
  const regex = new RegExp(`<(?=[^>]*\\b${tagName}\\b)[^>]*>([\\s\\S]*?)<\\/[^>]*${tagName}[^>]*>`, 'i');
  const match = xml.match(regex);
  return match ? match[1].trim() : null;
}

/**
 * Extracts all matching tag blocks from an XML string.
 */
function getAllXmlBlocks(xml: string, tagName: string): string[] {
  const regex = new RegExp(`<(?=[^>]*\\b${tagName}\\b)[^>]*>([\\s\\S]*?)<\\/[^>]*${tagName}[^>]*>`, 'gi');
  const blocks: string[] = [];
  let match: RegExpExecArray | null;
  while ((match = regex.exec(xml)) !== null) {
    blocks.push(match[1]);
  }
  return blocks;
}

/**
 * Parses an ISO 20022 XML (camt.053 or pacs.008) string into a StatementParseResult.
 */
export function parseIso20022Xml(xmlContent: string, filename = ''): StatementParseResult {
  const startTime = performance.now();
  const cleanXml = xmlContent.trim();

  // Detect bank/channel: CITAD, NAPAS, SWIFT, or BILATERAL
  let bankCode: BankCode = 'CITAD';
  const upperXml = cleanXml.toUpperCase();
  const upperFn = filename.toUpperCase();

  if (upperFn.includes('SWIFT') || upperXml.includes('SWIFT') || upperXml.includes('PACS.008')) {
    bankCode = 'SWIFT';
  } else if (upperFn.includes('NAPAS') || upperXml.includes('NAPAS')) {
    bankCode = 'NAPAS';
  } else if (upperFn.includes('BILATERAL') || upperXml.includes('NOSTRO') || upperXml.includes('VOSTRO')) {
    bankCode = 'BILATERAL';
  } else {
    bankCode = 'CITAD';
  }

  // Extract statement block
  const stmtBlock = getXmlTagContent(cleanXml, 'Stmt') || cleanXml;

  // Account details
  const acctBlock = getXmlTagContent(stmtBlock, 'Acct') || '';
  const iban = getXmlTagContent(acctBlock, 'IBAN');
  const othrBlock = getXmlTagContent(acctBlock, 'Othr');
  const othrId = othrBlock ? getXmlTagContent(othrBlock, 'Id') : null;
  const leafIdMatch = acctBlock.match(/<Id[^>]*>([^<]+)<\/Id>/i);
  const accountNumber =
    iban ||
    othrId ||
    (leafIdMatch ? leafIdMatch[1].trim() : null) ||
    `${bankCode}-ISO20022-ACCT`;

  const accountName = getXmlTagContent(acctBlock, 'Nm') || 'TÀI KHOẢN TIỀN GỬI VÀ QUYẾT TOÁN LIÊN NGÂN HÀNG';
  const currency = getXmlTagContent(acctBlock, 'Ccy') || 'VND';

  // Balances: OPBD (Opening Balance) and CLBD (Closing Balance)
  let openingBalance = 0;
  let closingBalance = 0;

  const balBlocks = getAllXmlBlocks(stmtBlock, 'Bal');
  for (const bal of balBlocks) {
    const tpCd = (getXmlTagContent(bal, 'Cd') || '').toUpperCase();
    const amtStr = getXmlTagContent(bal, 'Amt') || '0';
    const cdtDbt = (getXmlTagContent(bal, 'CdtDbtInd') || 'CRDT').toUpperCase();
    const rawAmt = Math.abs(parseVietnameseAmount(amtStr));
    const signedAmt = cdtDbt === 'DBIT' ? -rawAmt : rawAmt;

    if (tpCd === 'OPBD' || tpCd === 'PRCD') {
      openingBalance = signedAmt;
    } else if (tpCd === 'CLBD' || tpCd === 'CLAV') {
      closingBalance = signedAmt;
    }
  }

  // Transactions: <Ntry> blocks for camt.053 or <CdtTrfTxInf> blocks for pacs.008
  let ntryBlocks = getAllXmlBlocks(stmtBlock, 'Ntry');
  const isPacs = ntryBlocks.length === 0 && cleanXml.includes('CdtTrfTxInf');
  if (isPacs) {
    ntryBlocks = getAllXmlBlocks(cleanXml, 'CdtTrfTxInf');
  }

  const transactions: RawStatementRow[] = [];
  let runningBalance = openingBalance;
  let totalDebit = 0;
  let totalCredit = 0;

  for (let i = 0; i < ntryBlocks.length; i++) {
    const ntry = ntryBlocks[i];

    let rawAmt = 0;
    let isCredit = true;
    let debitAmt = 0;
    let creditAmt = 0;
    let netAmount = 0;
    let bookgRaw = '';
    let valRaw = '';
    let endToEndId = '';
    let txIdTag = '';
    let acctSvcrRef = '';
    let ustrd = '';
    let counterparty: string | undefined;
    let counterpartyAccount: string | undefined;

    if (isPacs) {
      const amtStr =
        getXmlTagContent(ntry, 'IntrBkSttlmAmt') ||
        getXmlTagContent(ntry, 'InstdAmt') ||
        getXmlTagContent(ntry, 'Amt') ||
        '0';
      rawAmt = Math.abs(parseVietnameseAmount(amtStr));
      isCredit = true;
      creditAmt = rawAmt;
      debitAmt = 0;
      netAmount = rawAmt;

      bookgRaw = getXmlTagContent(ntry, 'IntrBkSttlmDt') || getXmlTagContent(ntry, 'SttlmDt') || '';
      valRaw = bookgRaw;

      const pmtIdBlock = getXmlTagContent(ntry, 'PmtId') || ntry;
      endToEndId = getXmlTagContent(pmtIdBlock, 'EndToEndId') || '';
      txIdTag = getXmlTagContent(pmtIdBlock, 'TxId') || '';

      const rmtInfBlock = getXmlTagContent(ntry, 'RmtInf') || '';
      ustrd = getXmlTagContent(rmtInfBlock, 'Ustrd') || '';

      const dbtrBlock = getXmlTagContent(ntry, 'Dbtr') || '';
      counterparty = getXmlTagContent(dbtrBlock, 'Nm') || undefined;

      const dbtrAcct = getXmlTagContent(ntry, 'DbtrAcct') || '';
      const dbtrLeaf = dbtrAcct.match(/<Id[^>]*>([^<]+)<\/Id>/i);
      counterpartyAccount = dbtrLeaf ? dbtrLeaf[1].trim() : undefined;
    } else {
      // Amount & Direction
      const amtStr = getXmlTagContent(ntry, 'Amt') || '0';
      rawAmt = Math.abs(parseVietnameseAmount(amtStr));
      const cdtDbt = (getXmlTagContent(ntry, 'CdtDbtInd') || 'CRDT').toUpperCase();
      isCredit = cdtDbt === 'CRDT';

      debitAmt = isCredit ? 0 : rawAmt;
      creditAmt = isCredit ? rawAmt : 0;
      netAmount = isCredit ? rawAmt : -rawAmt;

      // Dates
      const bookgDtBlock = getXmlTagContent(ntry, 'BookgDt') || '';
      bookgRaw = getXmlTagContent(bookgDtBlock, 'DtTm') || getXmlTagContent(bookgDtBlock, 'Dt') || '';
      const valDtBlock = getXmlTagContent(ntry, 'ValDt') || '';
      valRaw = getXmlTagContent(valDtBlock, 'DtTm') || getXmlTagContent(valDtBlock, 'Dt') || bookgRaw;

      // References & Transaction IDs
      const txDtlsBlock = getXmlTagContent(ntry, 'TxDtls') || ntry;
      const refsBlock = getXmlTagContent(txDtlsBlock, 'Refs') || '';
      endToEndId = getXmlTagContent(refsBlock, 'EndToEndId') || '';
      txIdTag = getXmlTagContent(refsBlock, 'TxId') || '';
      acctSvcrRef = getXmlTagContent(ntry, 'AcctSvcrRef') || '';

      // Narration
      const rmtInfBlock = getXmlTagContent(txDtlsBlock, 'RmtInf') || '';
      ustrd = getXmlTagContent(rmtInfBlock, 'Ustrd') || getXmlTagContent(ntry, 'AddtlNtryInf') || '';

      // Counterparty
      const rltdPties = getXmlTagContent(txDtlsBlock, 'RltdPties') || '';
      if (isCredit) {
        const dbtr = getXmlTagContent(rltdPties, 'Dbtr') || '';
        counterparty = getXmlTagContent(dbtr, 'Nm') || undefined;
        const dbtrAcct = getXmlTagContent(rltdPties, 'DbtrAcct') || '';
        const dbtrId = getXmlTagContent(dbtrAcct, 'Id') || '';
        counterpartyAccount = getXmlTagContent(dbtrId, 'IBAN') || getXmlTagContent(dbtrId, 'Id') || undefined;
      } else {
        const cdtr = getXmlTagContent(rltdPties, 'Cdtr') || '';
        counterparty = getXmlTagContent(cdtr, 'Nm') || undefined;
        const cdtrAcct = getXmlTagContent(rltdPties, 'CdtrAcct') || '';
        const cdtrId = getXmlTagContent(cdtrAcct, 'Id') || '';
        counterpartyAccount = getXmlTagContent(cdtrId, 'IBAN') || getXmlTagContent(cdtrId, 'Id') || undefined;
      }
    }

    if (isCredit) totalCredit += creditAmt;
    else totalDebit += debitAmt;

    runningBalance += netAmount;

    const { isoDate, time, epochSeconds } = parseDateAndTimestamp(bookgRaw);
    const valTimeResult = valRaw ? parseDateAndTimestamp(valRaw) : null;
    const valueDateEpoch = valTimeResult ? valTimeResult.epochSeconds : epochSeconds;

    const combinedText = `${ustrd} ${endToEndId} ${txIdTag} ${acctSvcrRef}`.trim();
    const refTokens = extractReferenceTokens(combinedText);

    const txCode =
      acctSvcrRef ||
      txIdTag ||
      refTokens.ftNumber ||
      refTokens.traceId ||
      `${bankCode}ISO${(i + 1).toString().padStart(6, '0')}`;

    const docRef = endToEndId || refTokens.docRef || txCode;
    const narration = ustrd || `Giao dịch điện tử ISO 20022 ${txCode}`;

    transactions.push({
      id: `tx-iso-${bankCode.toLowerCase()}-${i}-${txCode}`,
      date: isoDate,
      time,
      txDate: epochSeconds,
      valueDate: valueDateEpoch,
      txCode,
      docRef,
      debit: debitAmt,
      credit: creditAmt,
      netAmount,
      amount: rawAmt,
      txType: isCredit ? 'CREDIT' : 'DEBIT',
      balance: runningBalance,
      balanceAfter: runningBalance,
      narration,
      counterparty,
      counterpartyAccount,
      bankCode,
      ftNumber: refTokens.ftNumber,
      traceId: refTokens.traceId,
      rawRef: txCode,
    });
  }

  // If closing balance was not in XML Bal blocks, use running balance
  if (closingBalance === 0 && (openingBalance !== 0 || transactions.length > 0)) {
    closingBalance = runningBalance;
  }

  const invariantReport = verifyBalanceInvariants(openingBalance, closingBalance, totalCredit, totalDebit);
  const duration = performance.now() - startTime;

  return {
    bankCode,
    bankName: getChannelFullName(bankCode),
    accountNumber,
    accountName,
    currency,
    openingBalance,
    closingBalance,
    totalDebit,
    totalCredit,
    transactions,
    balanceInvariantPassed: invariantReport.isValid,
    balanceDiscrepancy: invariantReport.discrepancy,
    rawRowCount: ntryBlocks.length,
    parseDurationMs: Math.round(duration * 100) / 100,
  };
}

function getChannelFullName(code: BankCode): string {
  switch (code) {
    case 'CITAD':
      return 'Kênh Thanh toán Điện tử Liên ngân hàng CITAD (NHNN) — ISO 20022';
    case 'NAPAS':
      return 'Kênh Chuyển mạch Tài chính & Bù trừ Điện tử NAPAS 24/7 — ISO 20022';
    case 'BILATERAL':
      return 'Kênh Thanh toán Song phương & Vostro / Nostro — ISO 20022';
    case 'SWIFT':
      return 'Kênh Điện báo & Thanh toán Quốc tế SWIFT (pacs.008 / camt.053)';
    default:
      return `${code} Interbank ISO 20022`;
  }
}

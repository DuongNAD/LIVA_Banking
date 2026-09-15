import { beforeEach, describe, expect, it } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useStatementStore } from '../src/stores/statementStore';
import { useReconciliationStore } from '../src/stores/reconciliationStore';

describe('Empirical Challenger: Hot-Folder Ingestion Edge Cases', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  describe('Edge Case Group 1: Unsupported File Extensions', () => {
    const unsupportedFiles = [
      'invoice.exe',
      'script.bat',
      'notes.txt',
      'document.docx',
      'payload.sh',
      'data.json',
      'image.png',
      'statement_without_extension',
    ];

    it.each(unsupportedFiles)('rejects unsupported extension: %s', (fileName) => {
      const store = useStatementStore();
      const result = store.detectBankAndFormat(fileName);

      // A rigorous bank ingestion pipeline rejects unsupported formats like .exe, .sh, .docx
      expect(result.format).toBe('unsupported');
      expect(store.isSupportedFormat(fileName)).toBe(false);
    });

    it('rejects malware.exe with REJECTED_UNSUPPORTED_FORMAT and skips transaction injection', async () => {
      const statementStore = useStatementStore();
      const reconcileStore = useReconciliationStore();
      const initialCount = reconcileStore.transactions.length;

      // Ingest an executable file
      const record = await statementStore.ingestFile({ name: 'malware.exe', size: 2048 });

      // Verifies remediation: status is REJECTED_UNSUPPORTED_FORMAT, 0 rows, 0 PII, 0 transactions injected
      expect(record.status).toBe('REJECTED_UNSUPPORTED_FORMAT');
      expect(record.format).toBe('unsupported');
      expect(record.parsedRowCount).toBe(0);
      expect(record.piiMaskedCount).toBe(0);
      expect(record.progress).toBe(0);
      expect(reconcileStore.transactions.length).toBe(initialCount); // Zero injected transactions!
    });
  });

  describe('Edge Case Group 2: Zero-Byte / Empty Files', () => {
    it('rejects a 0-byte file with REJECTED_EMPTY_FILE and does not fabricate PII or transactions', async () => {
      const statementStore = useStatementStore();
      const reconcileStore = useReconciliationStore();
      const initialCount = reconcileStore.transactions.length;

      // Ingest a 0-byte empty statement
      const record = await statementStore.ingestFile({ name: 'vcb_empty_statement.xlsx', size: 0 });

      // Verifies remediation: status is REJECTED_EMPTY_FILE, 0 rows, 0 PII, 0 transactions injected
      expect(record.fileSize).toBe(0);
      expect(record.status).toBe('REJECTED_EMPTY_FILE');
      expect(record.parsedRowCount).toBe(0);
      expect(record.piiMaskedCount).toBe(0);
      expect(record.progress).toBe(0);
      expect(reconcileStore.transactions.length).toBe(initialCount); // Zero injected transactions!
    });
  });

  describe('Edge Case Group 3: Bank Signature Sniffer Ambiguity', () => {
    it('correctly resolves bank using earliest keyword appearance without precedence collision', () => {
      const statementStore = useStatementStore();

      // Case A: A Vietcombank statement recording transfers to Techcombank
      // Filename: "VCB_to_Techcombank_settlement.xlsx" -> Issuing bank is VCB
      const resA = statementStore.detectBankAndFormat('VCB_to_Techcombank_settlement.xlsx');
      expect(resA.bank).toBe('VCB'); // Remediated: correctly identified as VCB
      expect(resA.format).toBe('xlsx');

      // Case B: BIDV statement transferring to Vietcombank
      const resB = statementStore.detectBankAndFormat('BIDV_to_Vietcombank_transfer.pdf');
      expect(resB.bank).toBe('BIDV');
      expect(resB.format).toBe('pdf');

      // Case C: Techcombank statement mentioning Vietcombank
      const resC = statementStore.detectBankAndFormat('TCB_payroll_to_VCB_accounts.csv');
      expect(resC.bank).toBe('TCB');
      expect(resC.format).toBe('csv');
    });

    it('demonstrates fallback heuristic without bank keyword', () => {
      const statementStore = useStatementStore();

      // Generic statement filenames with no bank names in filename
      const genericCsv = statementStore.detectBankAndFormat('statement_2026_09.csv');
      const genericXlsx = statementStore.detectBankAndFormat('statement_2026_09.xlsx');
      const genericPdf = statementStore.detectBankAndFormat('statement_2026_09.pdf');

      expect(genericCsv.bank).toBe('TCB');
      expect(genericXlsx.bank).toBe('VCB');
      expect(genericPdf.bank).toBe('BIDV');
    });
  });
});

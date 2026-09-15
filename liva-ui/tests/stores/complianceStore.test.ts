import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useComplianceStore } from '../../src/stores/complianceStore';

describe('complianceStore Pinia Unit Tests (P80–P85)', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  describe('Initial State & Computed Indicators', () => {
    it('initializes with statutory zero egress active and PII redaction enabled', () => {
      const store = useComplianceStore();
      expect(store.isZeroEgressActive).toBe(true);
      expect(store.isPiiRedactionEnabled).toBe(true);
      expect(store.merkleRootHash.startsWith('0x')).toBe(true);
      expect(store.merkleRootHash.length).toBe(66);
    });

    it('calculates accurate counts for critical and high AML alerts', () => {
      const store = useComplianceStore();
      expect(store.alerts.length).toBe(3);
      expect(store.criticalCount).toBe(2);
      expect(store.highCount).toBe(1);
    });
  });

  describe('Decree 13 PII Redaction & Corporate Entity Preservation', () => {
    it('sanitizes CCCD, phone numbers, and bank accounts while preserving corporate names', () => {
      const store = useComplianceStore();
      const sanitized = store.sanitizedPiiPreview;

      // Sensitive personal PII must be redacted
      expect(sanitized).toContain('[REDACTED_CCCD]');
      expect(sanitized).toContain('[REDACTED_PHONE]');
      expect(sanitized).toContain('[REDACTED_ACCOUNT]');
      expect(sanitized).not.toContain('031092008451');
      expect(sanitized).not.toContain('0908123456');
      expect(sanitized).not.toContain('0071000982123');

      // Corporate name MUST be strictly preserved for invoice matching
      expect(sanitized).toContain('CONG TY TNHH XAY DUNG NGUYEN HOANG');
    });

    it('toggles PII redaction on and off', () => {
      const store = useComplianceStore();
      store.togglePiiRedaction(false);
      expect(store.isPiiRedactionEnabled).toBe(false);
      expect(store.sanitizedPiiPreview).toBe(store.piiSampleInput);

      store.togglePiiRedaction(true);
      expect(store.isPiiRedactionEnabled).toBe(true);
      expect(store.sanitizedPiiPreview).toContain('[REDACTED_CCCD]');

      store.togglePiiRedaction();
      expect(store.isPiiRedactionEnabled).toBe(false);
    });
  });

  describe('Form STR Generation & Officer Sign-off (Circular 09/2023)', () => {
    it('generates an official STR draft from an AML alert', () => {
      const store = useComplianceStore();
      const alert = store.alerts[0]; // alt-01 (AML_HIGH_VALUE)
      expect(alert.hasStrReport).toBe(false);

      const report = store.generateStrReport('alt-01');
      expect(report).not.toBeNull();
      expect(report?.suspectName).toBe(alert.counterpartyName);
      expect(report?.suspectAccount).toBe(alert.counterpartyAccount);
      expect(report?.totalVndAmount).toBe(550000000);
      expect(report?.statutoryRuleRef).toContain('Quyết định 11/2023/QĐ-TTg');
      expect(report?.transactionCount).toBe(1);
      expect(report?.merkleProofHash).toBe(store.merkleRootHash);

      expect(alert.hasStrReport).toBe(true);
      expect(store.activeStrModal).toEqual(report);
    });

    it('sets transaction count = 3 for AML_STRUCTURING alerts', () => {
      const store = useComplianceStore();
      const report = store.generateStrReport('alt-02');
      expect(report?.transactionCount).toBe(3);
    });

    it('returns null if alert ID is not found', () => {
      const store = useComplianceStore();
      const report = store.generateStrReport('non-existent-id');
      expect(report).toBeNull();
    });

    it('signs and submits the STR report with compliance officer credentials', () => {
      const store = useComplianceStore();
      const report = store.generateStrReport('alt-03');
      expect(report).not.toBeNull();

      const officerId = 'OFFICER-AML-991';
      const notes = 'Đã đối soát đối chiếu, chuyển hồ sơ qua Cơ quan Thanh tra Giám sát NHNN';

      store.signAndSubmitStr(report!.strId, officerId, notes);

      expect(store.activeStrModal?.officerId).toBe(officerId);
      expect(store.activeStrModal?.complianceOfficerNotes).toBe(notes);
      expect(store.activeStrModal?.signedAt).toBeDefined();

      store.closeStrModal();
      expect(store.activeStrModal).toBeNull();
    });
  });

  describe('Immutable Merkle Root Calculation', () => {
    it('recalculates a valid 256-bit hexadecimal Merkle root hash', () => {
      const store = useComplianceStore();

      const newRoot = store.recalculateMerkleRoot();
      expect(newRoot.startsWith('0x')).toBe(true);
      expect(newRoot.length).toBe(66);
      expect(store.merkleRootHash).toBe(newRoot);
    });
  });
});

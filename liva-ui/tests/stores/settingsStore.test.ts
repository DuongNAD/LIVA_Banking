import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useSettingsStore } from '../../src/stores/settingsStore';

describe('settingsStore Pinia Unit Tests (P100–P106)', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  describe('P101: RBAC Users & Segregation of Duties (SoD)', () => {
    it('initializes with 5 distinct users and zero SoD violations', () => {
      const store = useSettingsStore();

      expect(store.users).toHaveLength(5);
      expect(store.hasSodViolation).toBe(false);

      const roles = store.users.map((u) => u.role);
      expect(roles).toContain('ADMIN');
      expect(roles).toContain('MAKER');
      expect(roles).toContain('CHECKER');
      expect(roles).toContain('COMPLIANCE');
      expect(roles).toContain('AUDITOR');
    });

    it('detects an SoD violation if duplicate usernames with conflicting roles exist', () => {
      const store = useSettingsStore();

      store.users.push({
        id: 'USR-CONFLICT',
        username: 'nam.tv', // Duplicate username holding conflicting role
        fullName: 'Trịnh Văn Nam Duplicate',
        role: 'CHECKER',
        isActive: true,
        lastLogin: '2026-09-15 17:00:00',
      });

      expect(store.hasSodViolation).toBe(true);
    });
  });

  describe('P102: Open Banking Connectors', () => {
    it('tracks bank connections and allows toggling connection & sandbox environments', () => {
      const store = useSettingsStore();

      expect(store.connectedBankCount).toBe(3); // VCB, TCB, BIDV active; MBB inactive

      // Connect MBB
      store.toggleBankConnection('MBB');
      expect(store.connectedBankCount).toBe(4);

      const vcb = store.bankConfigs.find((b) => b.bankCode === 'VCB');
      expect(vcb?.isSandbox).toBe(true);

      // Switch to Live
      store.toggleBankSandbox('VCB');
      expect(vcb?.isSandbox).toBe(false);
    });
  });

  describe('P103: ERP Bridges', () => {
    it('manages ERP bridge states and active connectors', () => {
      const store = useSettingsStore();

      expect(store.activeErpCount).toBe(2); // MISA, FAST active; SAP inactive

      // Activate SAP B1
      store.toggleErpActive('SAP Business One');
      expect(store.activeErpCount).toBe(3);

      // Deactivate MISA
      store.toggleErpActive('MISA AMIS');
      expect(store.activeErpCount).toBe(2);
    });
  });

  describe('P104: Reconciliation Thresholds', () => {
    it('allows updating matching tolerance, fee threshold, and solver depth', () => {
      const store = useSettingsStore();

      expect(store.thresholds.exactToleranceVnd).toBe(0);
      expect(store.thresholds.maxFeeVarianceVnd).toBe(50000);
      expect(store.thresholds.splitSolverMaxK).toBe(8);

      store.updateThresholds({
        maxFeeVarianceVnd: 75000,
        splitSolverMaxK: 10,
      });

      expect(store.thresholds.maxFeeVarianceVnd).toBe(75000);
      expect(store.thresholds.splitSolverMaxK).toBe(10);
      expect(store.thresholds.exactToleranceVnd).toBe(0); // Unchanged
    });
  });

  describe('P105 & P106: Security, Backup & Disaster Recovery (DR)', () => {
    it('verifies Zero Cloud Egress lock and HSM credentials', () => {
      const store = useSettingsStore();

      expect(store.security.zeroEgressStrict).toBe(true);
      expect(store.security.hsmProvider).toContain('VNPT-CA');
      expect(store.security.tamperProofChain).toBe(true);
    });

    it('creates instant snapshot backups and verifies DR drill successfully', () => {
      const store = useSettingsStore();

      const snapshotHash = store.triggerBackupNow();
      expect(snapshotHash.startsWith('SNAP-')).toBe(true);
      expect(store.backup.isBackingUp).toBe(false);

      const drReport = store.verifyDisasterRecovery();
      expect(drReport).toContain('RTO:');
      expect(drReport).toContain('RPO:');
      expect(drReport).toContain('Pass 100%');
    });

    it('switches navigation subtabs properly', () => {
      const store = useSettingsStore();

      store.setActiveTab('BACKUP');
      expect(store.activeTab).toBe('BACKUP');

      store.setActiveTab('THRESHOLDS');
      expect(store.activeTab).toBe('THRESHOLDS');
    });
  });
});

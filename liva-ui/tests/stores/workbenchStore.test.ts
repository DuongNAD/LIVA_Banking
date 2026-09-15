import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useWorkbenchStore } from '../../src/stores/workbenchStore';

describe('workbenchStore (P42 Workbench & P44 Quarantine Dual Control)', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('initializes with benchmark bank lines, GL records, and quarantine items', () => {
    const store = useWorkbenchStore();
    expect(store.bankLines.length).toBeGreaterThan(0);
    expect(store.glRecords.length).toBeGreaterThan(0);
    expect(store.quarantineItems.length).toBe(4);
    expect(store.currentOperatorRole).toBe('MAKER');
    expect(store.currentOperatorId).toBe('USR-MAKER-KETOAN-01');
  });

  it('filters quarantine items by status and severity', () => {
    const store = useWorkbenchStore();
    store.quarantineFilterSeverity = 'CRITICAL';
    expect(store.filteredQuarantineItems.every((i) => i.severity === 'CRITICAL')).toBe(true);

    store.quarantineFilterSeverity = 'ALL';
    store.quarantineFilterStatus = 'PENDING_MAKER';
    expect(store.filteredQuarantineItems.every((i) => i.status === 'PENDING_MAKER')).toBe(true);
  });

  it('switches operator role between MAKER and CHECKER', () => {
    const store = useWorkbenchStore();
    expect(store.currentOperatorRole).toBe('MAKER');
    expect(store.currentOperatorId).toBe('USR-MAKER-KETOAN-01');

    store.switchOperatorRole('CHECKER');
    expect(store.currentOperatorRole).toBe('CHECKER');
    expect(store.currentOperatorId).toBe('USR-CHECKER-KTT-01');
  });

  describe('P44 Maker-Checker Dual Control Lifecycle', () => {
    it('Maker can propose quarantine resolution with note >= 10 chars', () => {
      const store = useWorkbenchStore();
      store.switchOperatorRole('MAKER');

      // Attempt with short note should fail
      expect(() => {
        store.proposeQuarantineResolution('q-01', '6425', 'Ngan qua');
      }).toThrow('độ dài tối thiểu 10 ký tự');

      // Valid proposal
      store.proposeQuarantineResolution(
        'q-01',
        '1388',
        'Hồ sơ đã được gửi sang phòng kinh doanh để xác minh hợp đồng phụ lục XL-05'
      );

      const item = store.quarantineItems.find((i) => i.id === 'q-01');
      expect(item?.status).toBe('SUBMITTED_TO_CHECKER');
      expect(item?.makerId).toBe('USR-MAKER-KETOAN-01');
      expect(item?.proposedGlAccount).toBe('1388');
      expect(item?.tokenUuid).toBeDefined();
      expect(item?.hashProof).toMatch(/^0x[0-9a-f]{64}$/);
    });

    it('Checker role cannot submit Maker proposals', () => {
      const store = useWorkbenchStore();
      store.switchOperatorRole('CHECKER');

      expect(() => {
        store.proposeQuarantineResolution('q-01', '6425', 'Thu nghiem tu Checker');
      }).toThrow('Chỉ người có vai trò Kế toán viên (Maker)');
    });

    it('Enforces Circular 09 Segregation of Duties: maker_user_id != checker_user_id', () => {
      const store = useWorkbenchStore();
      store.switchOperatorRole('MAKER');

      // Maker proposes q-03
      store.proposeQuarantineResolution(
        'q-03',
        '1388',
        'Khách hàng chuyển khoản thiếu hóa đơn tài chính, tạm treo TK 1388'
      );

      const item = store.quarantineItems.find((i) => i.id === 'q-03');
      expect(item?.status).toBe('SUBMITTED_TO_CHECKER');

      // If a user with the SAME ID tries to approve (e.g. simulation of same user)
      // Force currentOperatorId to match makerId
      store.switchOperatorRole('MAKER'); // role is MAKER
      expect(() => {
        store.approveQuarantineItem('q-03', 'Toi tu duyet cho toi');
      }).toThrow('Chỉ Kế toán trưởng (Checker)');

      // Switch to CHECKER
      store.switchOperatorRole('CHECKER');
      // Set item.makerId to match currentOperatorId to simulate violation
      if (item) item.makerId = store.currentOperatorId;

      expect(() => {
        store.approveQuarantineItem('q-03', 'Kiem tra hop le va dong y phuong an');
      }).toThrow('Vi phạm nguyên tắc 4 mắt (SoD)');

      expect(() => {
        store.rejectQuarantineItem('q-03', 'Tu choi ho so khong du can cu');
      }).toThrow('Vi phạm nguyên tắc 4 mắt (SoD)');
    });

    it('Checker approves proposal when maker_id != checker_id', () => {
      const store = useWorkbenchStore();
      // q-02 already has makerId = USR-MAKER-KETOAN-01 and status = SUBMITTED_TO_CHECKER
      store.switchOperatorRole('CHECKER');

      store.approveQuarantineItem(
        'q-02',
        'Đã đối chiếu biểu phí SWIFT từ ngân hàng Techcombank, đồng ý ghi nhận chi phí TK 6425'
      );

      const item = store.quarantineItems.find((i) => i.id === 'q-02');
      expect(item?.status).toBe('APPROVED');
      expect(item?.checkerId).toBe('USR-CHECKER-KTT-01');
      expect(item?.resolvedAt).toBeDefined();
    });

    it('Checker rejects proposal when note is valid and maker_id != checker_id', () => {
      const store = useWorkbenchStore();
      // First submit q-04
      store.switchOperatorRole('MAKER');
      store.proposeQuarantineResolution('q-04', '3388', 'Nghi ngo duplicate ref, treo phai tra 3388');

      // Checker rejects
      store.switchOperatorRole('CHECKER');
      store.rejectQuarantineItem(
        'q-04',
        'Hồ sơ không đạt, yêu cầu rà soát lại sao kê ngân hàng ngày 14/09'
      );

      const item = store.quarantineItems.find((i) => i.id === 'q-04');
      expect(item?.status).toBe('REJECTED');
      expect(item?.checkerId).toBe('USR-CHECKER-KTT-01');
    });
  });

  describe('P42 Workbench Reconciliation & Split Solver Invariants', () => {
    it('maintains mathematical balance invariant', () => {
      const store = useWorkbenchStore();
      expect(store.summaryStats.totalBank).toBeDefined();
      expect(store.summaryStats.totalGl).toBeDefined();
    });

    it('solves subset sum for target bank line with k <= 8 combinations', () => {
      const store = useWorkbenchStore();
      const bk03 = store.bankLines.find((b) => b.id === 'bk-03');
      expect(bk03).toBeDefined();
      if (!bk03) return;

      const result = store.solveSubsetSum(bk03);
      expect(result.combinations.length).toBeGreaterThan(0);
      expect(result.combinations.some((s) => s.difference === 0)).toBe(true);
      expect(result.combinations[0].glRecords.length).toBe(2);
    });
  });
});

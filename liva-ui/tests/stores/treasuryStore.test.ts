import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useTreasuryStore } from '../../src/stores/treasuryStore';

describe('treasuryStore (P60–P65 Ngân Quỹ & Kế Hoạch Dòng Tiền)', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  describe('P60 Cash Concentration & Multi-bank Accounts', () => {
    it('initializes multi-bank accounts and computes total cash position', () => {
      const store = useTreasuryStore();
      expect(store.accounts.length).toBe(4);

      const sum = store.accounts.reduce((acc, a) => acc + a.balance, 0);
      expect(store.totalCashPosition).toBe(sum);
      expect(store.totalCashPosition).toBe(2660000000); // 2.66 Tỷ VND
    });

    it('identifies sweeping opportunities when collections accounts exceed thresholds', () => {
      const store = useTreasuryStore();
      const recs = store.sweepingRecommendations;

      // TCB balance is 680M with threshold 500M and target buffer 200M => excess 480M
      expect(recs.length).toBeGreaterThan(0);
      const tcbRec = recs.find((r) => r.source.bankCode === 'TCB');
      expect(tcbRec).toBeDefined();
      expect(tcbRec?.target.bankCode).toBe('VCB');
      expect(tcbRec?.excessAmount).toBe(480000000); // 680M - 200M
    });

    it('executes cash sweeping and updates account balances atomically', () => {
      const store = useTreasuryStore();
      const initialVcb = store.accounts.find((a) => a.id === 'acc-vcb-01')!.balance;
      const initialTcb = store.accounts.find((a) => a.id === 'acc-tcb-01')!.balance;

      const sweepAmount = 200000000;
      store.executeCashSweeping('acc-tcb-01', 'acc-vcb-01', sweepAmount);

      const afterVcb = store.accounts.find((a) => a.id === 'acc-vcb-01')!.balance;
      const afterTcb = store.accounts.find((a) => a.id === 'acc-tcb-01')!.balance;

      expect(afterVcb).toBe(initialVcb + sweepAmount);
      expect(afterTcb).toBe(initialTcb - sweepAmount);
      // Total cash remains invariant
      expect(store.totalCashPosition).toBe(2660000000);
    });

    it('throws error when executing sweep with insufficient source balance', () => {
      const store = useTreasuryStore();
      expect(() => {
        store.executeCashSweeping('acc-mbb-01', 'acc-vcb-01', 99999999999);
      }).toThrow('Số dư nguồn không đủ để quét vốn');
    });
  });

  describe('P61 Rolling 30/60/90-Day Cashflow Sentinel', () => {
    it('generates cashflow projections according to the chosen horizon', () => {
      const store = useTreasuryStore();

      store.projectionHorizonDays = 30;
      expect(store.projectedCashflowData.length).toBe(30);

      store.projectionHorizonDays = 60;
      expect(store.projectedCashflowData.length).toBe(60);

      store.projectionHorizonDays = 90;
      expect(store.projectedCashflowData.length).toBe(90);
    });

    it('computes lowest projected point, runway days, and liquidity health', () => {
      const store = useTreasuryStore();
      expect(store.lowestProjectedPoint).toBeDefined();
      expect(store.lowestProjectedPoint?.projectedBalance).toBeDefined();

      expect(store.runwayDays).toBeGreaterThan(0);
      expect(['SAFE', 'WARNING', 'CRITICAL']).toContain(store.liquidityHealthStatus.level);
    });
  });

  describe('P62 Circular 09 Maker-Checker Dual Control Payment Orders', () => {
    it('allows Maker to create payment orders and blocks Checker from creating', () => {
      const store = useTreasuryStore();

      // Checker cannot create
      store.switchOperatorRole('CHECKER');
      expect(() => {
        store.createPaymentOrder({
          sourceAccountId: 'acc-vcb-01',
          beneficiaryName: 'Nhà cung cấp ABC',
          beneficiaryBank: 'TCB',
          beneficiaryAccount: '9988776655',
          amount: 50000000,
          memo: 'Thanh toán đợt 1',
          priority: 'NORMAL',
        });
      }).toThrow('Chỉ người có vai trò Thủ quỹ / Thanh toán (Maker) mới được lập lệnh chi tiền');

      // Maker can create
      store.switchOperatorRole('MAKER');
      const newOrder = store.createPaymentOrder({
        sourceAccountId: 'acc-vcb-01',
        beneficiaryName: 'Nhà cung cấp ABC',
        beneficiaryBank: 'TCB',
        beneficiaryAccount: '9988776655',
        amount: 50000000,
        memo: 'Thanh toán đợt 1',
        priority: 'NORMAL',
        makerNote: 'Hóa đơn đầy đủ',
      });

      expect(newOrder.status).toBe('SUBMITTED_TO_CHECKER');
      expect(newOrder.makerId).toBe('USR-TREASURY-MAKER-01');
      expect(newOrder.orderNumber).toMatch(/^UNC-/);
      expect(store.paymentOrders[0].id).toBe(newOrder.id);
    });

    it('enforces Segregation of Duties (SoD): Maker cannot approve their own order', () => {
      const store = useTreasuryStore();
      store.switchOperatorRole('MAKER');

      const order = store.createPaymentOrder({
        sourceAccountId: 'acc-vcb-01',
        beneficiaryName: 'Nhà cung cấp XYZ',
        beneficiaryBank: 'BIDV',
        beneficiaryAccount: '11223344',
        amount: 30000000,
        memo: 'Tạm ứng',
        priority: 'NORMAL',
      });

      // Role check: Maker cannot approve
      expect(() => {
        store.approvePaymentOrder(order.id, 'Duyệt luôn đi');
      }).toThrow('Chỉ Kế toán trưởng hoặc Giám đốc tài chính (Checker) mới có quyền duyệt lệnh chi');

      // If Maker attempts to approve under same identity, SoD blocks it
      order.makerId = 'USR-CFO-CHECKER-01'; // Force makerId to match checker
      store.switchOperatorRole('CHECKER');
      expect(() => {
        store.approvePaymentOrder(order.id, 'Duyệt chính lệnh của mình');
      }).toThrow('Vi phạm nguyên tắc 4 mắt (SoD)');
    });

    it('allows Checker to approve valid order with adequate comment', () => {
      const store = useTreasuryStore();
      store.switchOperatorRole('CHECKER');

      // pay-01 was created by USR-TREASURY-MAKER-01, status SUBMITTED_TO_CHECKER
      const target = store.paymentOrders.find((p) => p.id === 'pay-01')!;
      expect(target.status).toBe('SUBMITTED_TO_CHECKER');

      // Reject comment under 5 chars
      expect(() => {
        store.approvePaymentOrder('pay-01', 'ok');
      }).toThrow('Cần nhập ý kiến phê duyệt tối thiểu 5 ký tự');

      store.approvePaymentOrder('pay-01', 'Duyệt thanh toán đúng hợp đồng');
      expect(target.status).toBe('APPROVED');
      expect(target.checkerId).toBe('USR-CFO-CHECKER-01');
      expect(target.checkerNote).toBe('Duyệt thanh toán đúng hợp đồng');
      expect(target.approvedAt).toBeDefined();
    });

    it('allows Checker to reject order with reason', () => {
      const store = useTreasuryStore();
      store.switchOperatorRole('CHECKER');

      const target = store.paymentOrders.find((p) => p.id === 'pay-01')!;
      store.rejectPaymentOrder('pay-01', 'Thiếu biên bản nghiệm thu đợt 2');

      expect(target.status).toBe('REJECTED');
      expect(target.checkerNote).toBe('Thiếu biên bản nghiệm thu đợt 2');
    });

    it('executes disbursement only for APPROVED orders and deducts balance', () => {
      const store = useTreasuryStore();

      // pay-01 is SUBMITTED_TO_CHECKER, cannot execute
      expect(() => {
        store.executePaymentDisbursement('pay-01');
      }).toThrow('Chỉ lệnh chi đã được phê duyệt mới được giải ngân');

      // pay-02 is already APPROVED for amount 185,000,000 from acc-tcb-01
      const tcbAcc = store.accounts.find((a) => a.id === 'acc-tcb-01')!;
      const initialBalance = tcbAcc.balance;

      store.executePaymentDisbursement('pay-02');

      const order2 = store.paymentOrders.find((p) => p.id === 'pay-02')!;
      expect(order2.status).toBe('EXECUTED');
      expect(order2.executedAt).toBeDefined();
      expect(order2.bankRefNumber).toMatch(/^FT\d+/);
      expect(tcbAcc.balance).toBe(initialBalance - 185000000);
    });

    it('blocks disbursement if source balance is insufficient', () => {
      const store = useTreasuryStore();
      const order2 = store.paymentOrders.find((p) => p.id === 'pay-02')!;
      const tcbAcc = store.accounts.find((a) => a.id === 'acc-tcb-01')!;

      tcbAcc.balance = 50000; // Drop balance to 50k VND
      order2.status = 'APPROVED';

      expect(() => {
        store.executePaymentDisbursement('pay-02');
      }).toThrow('không đủ để chi');
    });
  });
});

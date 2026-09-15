import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { logger } from '../utils/logger';

export type BankPartner = 'VCB' | 'TCB' | 'BIDV' | 'MBB';
export type PaymentStatus = 'DRAFT' | 'SUBMITTED_TO_CHECKER' | 'APPROVED' | 'EXECUTED' | 'REJECTED';
export type PaymentPriority = 'NORMAL' | 'URGENT' | 'PAYROLL_TAX';

export interface TreasuryAccount {
  id: string;
  bankCode: BankPartner;
  bankName: string;
  accountNumber: string;
  accountType: 'MASTER_POOL' | 'COLLECTIONS' | 'DISBURSEMENT';
  balance: number; // Integer VND
  targetBuffer: number; // Ngưỡng đệm mục tiêu
  sweepingThreshold: number; // Ngưỡng kích hoạt quét vốn
  themeColor: string;
}

export interface CashflowDataPoint {
  dayIndex: number;
  date: string;
  inflow: number;
  outflow: number;
  netFlow: number;
  projectedBalance: number;
  isBreached: boolean;
}

export interface PaymentOrder {
  id: string;
  paymentGuid: string;
  orderNumber: string;
  sourceAccountId: string;
  beneficiaryName: string;
  beneficiaryBank: string;
  beneficiaryAccount: string;
  amount: number; // Integer VND
  memo: string;
  priority: PaymentPriority;
  status: PaymentStatus;
  makerId: string;
  makerNote?: string;
  submittedAt?: string;
  checkerId?: string;
  checkerNote?: string;
  approvedAt?: string;
  hsmTokenUuid?: string;
  executedAt?: string;
  bankRefNumber?: string;
}

export const useTreasuryStore = defineStore('treasury', () => {
  // 1. Multi-bank Accounts (P60)
  const accounts = ref<TreasuryAccount[]>([
    {
      id: 'acc-vcb-01',
      bankCode: 'VCB',
      bankName: 'Vietcombank — Trụ sở chính',
      accountNumber: '00710009821',
      accountType: 'MASTER_POOL',
      balance: 1450000000, // 1.45 Tỷ
      targetBuffer: 1000000000,
      sweepingThreshold: 2000000000,
      themeColor: '#10b981',
    },
    {
      id: 'acc-tcb-01',
      bankCode: 'TCB',
      bankName: 'Techcombank — Chi nhánh Sài Gòn',
      accountNumber: '1903456789',
      accountType: 'COLLECTIONS',
      balance: 680000000, // 680 Tr
      targetBuffer: 200000000,
      sweepingThreshold: 500000000,
      themeColor: '#ef4444',
    },
    {
      id: 'acc-bidv-01',
      bankCode: 'BIDV',
      bankName: 'BIDV — Chi nhánh Chợ Lớn',
      accountNumber: '1201000456',
      accountType: 'DISBURSEMENT',
      balance: 320000000, // 320 Tr
      targetBuffer: 300000000,
      sweepingThreshold: 300000000,
      themeColor: '#3b82f6',
    },
    {
      id: 'acc-mbb-01',
      bankCode: 'MBB',
      bankName: 'MBBank — Chi nhánh Gia Định',
      accountNumber: '0680100123',
      accountType: 'COLLECTIONS',
      balance: 210000000, // 210 Tr
      targetBuffer: 100000000,
      sweepingThreshold: 300000000,
      themeColor: '#8b5cf6',
    },
  ]);

  // Operator ID & Role state for Treasury
  const currentOperatorRole = ref<'MAKER' | 'CHECKER'>('MAKER');
  const currentOperatorId = computed(() =>
    currentOperatorRole.value === 'MAKER' ? 'USR-TREASURY-MAKER-01' : 'USR-CFO-CHECKER-01'
  );

  // 2. Rolling Cashflow Horizon (P61)
  const projectionHorizonDays = ref<30 | 60 | 90>(30);
  const minOperatingReserve = ref(500000000); // 500M VND

  // 3. Payment Orders (P62)
  const paymentOrders = ref<PaymentOrder[]>([
    {
      id: 'pay-01',
      paymentGuid: 'guid-pay-20260915-01',
      orderNumber: 'UNC-VCB-2026-081',
      sourceAccountId: 'acc-vcb-01',
      beneficiaryName: 'Công ty Cổ phần Xây Lắp Điện Miền Nam',
      beneficiaryBank: 'BIDV',
      beneficiaryAccount: '12010009988',
      amount: 350000000,
      memo: 'Thanh toán đợt 2 gói thầu cơ điện MEP tòa nhà văn phòng',
      priority: 'NORMAL',
      status: 'SUBMITTED_TO_CHECKER',
      makerId: 'USR-TREASURY-MAKER-01',
      makerNote: 'Đã kiểm tra đầy đủ hồ sơ nghiệm thu kỹ thuật và hóa đơn VAT điện tử hợp lệ.',
      submittedAt: new Date(Date.now() - 3600000).toISOString(),
      hsmTokenUuid: 'hsm-tok-78912',
    },
    {
      id: 'pay-02',
      paymentGuid: 'guid-pay-20260915-02',
      orderNumber: 'UNC-TCB-2026-042',
      sourceAccountId: 'acc-tcb-01',
      beneficiaryName: 'Cục Thuế TP. Hồ Chí Minh',
      beneficiaryBank: 'VietinBank',
      beneficiaryAccount: '711A009123',
      amount: 185000000,
      memo: 'Nộp thuế Giá trị gia tăng (VAT) quý 3/2026',
      priority: 'PAYROLL_TAX',
      status: 'APPROVED',
      makerId: 'USR-TREASURY-MAKER-01',
      makerNote: 'Tờ khai thuế đã được ký số nộp qua Thuedientu.gdt.gov.vn',
      submittedAt: new Date(Date.now() - 7200000).toISOString(),
      checkerId: 'USR-CFO-CHECKER-01',
      checkerNote: 'Chấp thuận chi nộp thuế kịp thời hạn quy định ngày 20.',
      approvedAt: new Date(Date.now() - 1800000).toISOString(),
      hsmTokenUuid: 'hsm-tok-65432',
    },
  ]);

  const activePaymentOrderId = ref<string | null>(null);

  // --- Computed Views ---

  const totalCashPosition = computed(() => {
    return accounts.value.reduce((acc, a) => acc + a.balance, 0);
  });

  const activePaymentOrder = computed(() => {
    if (!activePaymentOrderId.value) return null;
    return paymentOrders.value.find((p) => p.id === activePaymentOrderId.value) || null;
  });

  /**
   * P61: Sinh dữ liệu mô phỏng dự báo dòng tiền 30/60/90 ngày
   */
  const projectedCashflowData = computed<CashflowDataPoint[]>(() => {
    const points: CashflowDataPoint[] = [];
    const horizon = projectionHorizonDays.value;
    let runningBalance = totalCashPosition.value;

    const startDate = new Date();

    for (let i = 1; i <= horizon; i++) {
      const d = new Date(startDate);
      d.setDate(d.getDate() + i);
      const dateStr = d.toISOString().split('T')[0];

      // Realistic business cash cycles:
      // Day 15 & 30: Large collections from enterprise B2B customers
      // Day 20: Tax & VAT disbursement
      // Day 25: Payroll disbursement
      let inflow = 15000000 + ((i * 37) % 25) * 1000000;
      let outflow = 18000000 + ((i * 29) % 20) * 1000000;

      if (i % 15 === 0) inflow += 350000000; // Customer invoice receipts
      if (i === 5 || i === 20) outflow += 180000000; // Tax & supplier bills
      if (i === 25) outflow += 420000000; // Monthly payroll

      const netFlow = inflow - outflow;
      runningBalance += netFlow;

      points.push({
        dayIndex: i,
        date: dateStr,
        inflow,
        outflow,
        netFlow,
        projectedBalance: runningBalance,
        isBreached: runningBalance < minOperatingReserve.value,
      });
    }

    return points;
  });

  const lowestProjectedPoint = computed(() => {
    if (projectedCashflowData.value.length === 0) return null;
    return projectedCashflowData.value.reduce((min, p) =>
      p.projectedBalance < min.projectedBalance ? p : min
    );
  });

  const runwayDays = computed(() => {
    const breachIndex = projectedCashflowData.value.findIndex((p) => p.isBreached);
    if (breachIndex === -1) return 999; // Không chạm ngưỡng thủng đệm
    return breachIndex + 1;
  });

  const liquidityHealthStatus = computed(() => {
    const runway = runwayDays.value;
    if (runway <= 14) {
      return {
        level: 'CRITICAL',
        title: 'CẢNH BÁO NGUY CƠ THÂM HỤT THANH KHOẢN CAO',
        desc: `Dự kiến số dư tiền mặt sẽ thủng ngưỡng đệm an toàn 500M VND trong ${runway} ngày tới. Cần huy động vốn hoặc điều chuyển gấp!`,
      };
    } else if (runway <= 30) {
      return {
        level: 'WARNING',
        title: 'CẢNH BÁO ÁP LỰC THANH KHOẢN TRUNG HẠN',
        desc: `Dòng tiền dự kiến tiệm cận ngưỡng đệm thanh khoản sau ${runway} ngày. Cần theo dõi tiến độ thu hồi công nợ.`,
      };
    } else {
      return {
        level: 'SAFE',
        title: 'TRẠNG THÁI THANH KHOẢN TỐI ƯU & AN TOÀN',
        desc: 'Số dư khả dụng duy trì liên tục trên 500M VND trong toàn bộ kỳ dự báo.',
      };
    }
  });

  const sweepingRecommendations = computed(() => {
    const recs: {
      source: TreasuryAccount;
      target: TreasuryAccount;
      excessAmount: number;
    }[] = [];

    const masterAcc = accounts.value.find((a) => a.accountType === 'MASTER_POOL');
    if (!masterAcc) return recs;

    accounts.value.forEach((acc) => {
      if (acc.accountType === 'COLLECTIONS' && acc.balance > acc.sweepingThreshold) {
        const excess = acc.balance - acc.targetBuffer;
        if (excess > 0) {
          recs.push({
            source: acc,
            target: masterAcc,
            excessAmount: excess,
          });
        }
      }
    });

    return recs;
  });

  // --- Actions ---

  function switchOperatorRole(role: 'MAKER' | 'CHECKER') {
    currentOperatorRole.value = role;
    logger.info(`Treasury Operator chuyển sang: ${role} (${currentOperatorId.value})`);
  }

  function selectPaymentOrder(id: string | null) {
    activePaymentOrderId.value = id;
  }

  /**
   * P62: Lập lệnh chi tiền mới (Maker)
   */
  function createPaymentOrder(params: {
    sourceAccountId: string;
    beneficiaryName: string;
    beneficiaryBank: string;
    beneficiaryAccount: string;
    amount: number;
    memo: string;
    priority: PaymentPriority;
    makerNote?: string;
  }): PaymentOrder {
    if (currentOperatorRole.value !== 'MAKER') {
      throw new Error('Chỉ người có vai trò Thủ quỹ / Thanh toán (Maker) mới được lập lệnh chi tiền');
    }
    if (params.amount <= 0) {
      throw new Error('Số tiền thanh toán phải lớn hơn 0 VND');
    }

    const guid = typeof crypto !== 'undefined' && crypto.randomUUID ? crypto.randomUUID() : `guid-pay-${Date.now()}`;
    const orderNo = `UNC-${Date.now().toString().slice(-6)}`;

    const newOrder: PaymentOrder = {
      id: `pay-${Date.now()}`,
      paymentGuid: guid,
      orderNumber: orderNo,
      sourceAccountId: params.sourceAccountId,
      beneficiaryName: params.beneficiaryName,
      beneficiaryBank: params.beneficiaryBank,
      beneficiaryAccount: params.beneficiaryAccount,
      amount: params.amount,
      memo: params.memo,
      priority: params.priority,
      status: 'SUBMITTED_TO_CHECKER',
      makerId: currentOperatorId.value,
      makerNote: params.makerNote,
      submittedAt: new Date().toISOString(),
      hsmTokenUuid: `hsm-${guid.slice(0, 8)}`,
    };

    paymentOrders.value.unshift(newOrder);
    logger.info(`Đã tạo và trình duyệt lệnh chi tiền ${orderNo} số tiền ${params.amount} VND`);
    return newOrder;
  }

  /**
   * P62: Phê duyệt lệnh chi tiền (Checker) với ràng buộc SoD
   */
  function approvePaymentOrder(orderId: string, checkerNote: string) {
    const order = paymentOrders.value.find((p) => p.id === orderId);
    if (!order) throw new Error(`Không tìm thấy lệnh chi ${orderId}`);

    if (currentOperatorRole.value !== 'CHECKER') {
      throw new Error('Chỉ Kế toán trưởng hoặc Giám đốc tài chính (Checker) mới có quyền duyệt lệnh chi');
    }

    // Circular 09 Strict Segregation of Duties Invariant
    if (order.makerId === currentOperatorId.value) {
      throw new Error(`Vi phạm nguyên tắc 4 mắt (SoD): Người duyệt (${currentOperatorId.value}) không được trùng người lập lệnh (${order.makerId})!`);
    }

    if (!checkerNote || checkerNote.trim().length < 5) {
      throw new Error('Cần nhập ý kiến phê duyệt tối thiểu 5 ký tự');
    }

    order.status = 'APPROVED';
    order.checkerId = currentOperatorId.value;
    order.checkerNote = checkerNote.trim();
    order.approvedAt = new Date().toISOString();
    logger.info(`Lệnh chi ${order.orderNumber} đã được phê duyệt thành công`);
  }

  /**
   * P62: Từ chối lệnh chi tiền (Checker)
   */
  function rejectPaymentOrder(orderId: string, reason: string) {
    const order = paymentOrders.value.find((p) => p.id === orderId);
    if (!order) throw new Error(`Không tìm thấy lệnh chi ${orderId}`);

    if (currentOperatorRole.value !== 'CHECKER') {
      throw new Error('Chỉ Kế toán trưởng hoặc Giám đốc tài chính (Checker) mới có quyền từ chối lệnh chi');
    }

    if (order.makerId === currentOperatorId.value) {
      throw new Error(`Vi phạm nguyên tắc 4 mắt (SoD): Người từ chối không được trùng người lập lệnh!`);
    }

    order.status = 'REJECTED';
    order.checkerId = currentOperatorId.value;
    order.checkerNote = reason;
    logger.info(`Lệnh chi ${order.orderNumber} đã bị từ chối với lý do: ${reason}`);
  }

  /**
   * P62: Khởi chạy giải ngân thực tế (Execution)
   */
  function executePaymentDisbursement(orderId: string) {
    const order = paymentOrders.value.find((p) => p.id === orderId);
    if (!order) throw new Error(`Không tìm thấy lệnh chi ${orderId}`);

    if (order.status !== 'APPROVED') {
      throw new Error('Chỉ lệnh chi đã được phê duyệt mới được giải ngân');
    }

    const sourceAcc = accounts.value.find((a) => a.id === order.sourceAccountId);
    if (!sourceAcc) throw new Error('Không tìm thấy tài khoản trích tiền');

    if (sourceAcc.balance < order.amount) {
      throw new Error(`Số dư tài khoản (${sourceAcc.balance} VND) không đủ để chi ${order.amount} VND!`);
    }

    sourceAcc.balance -= order.amount;
    order.status = 'EXECUTED';
    order.executedAt = new Date().toISOString();
    order.bankRefNumber = `FT${Date.now().toString().slice(-8)}`;

    logger.info(`Đã giải ngân thành công lệnh chi ${order.orderNumber}, mã FT: ${order.bankRefNumber}`);
  }

  /**
   * P60: Kích hoạt điều chuyển tập trung vốn (Physical Sweeping)
   */
  function executeCashSweeping(sourceId: string, targetId: string, amount: number) {
    const src = accounts.value.find((a) => a.id === sourceId);
    const tgt = accounts.value.find((a) => a.id === targetId);

    if (!src || !tgt) throw new Error('Tài khoản nguồn hoặc đích không hợp lệ');
    if (src.balance < amount) throw new Error('Số dư nguồn không đủ để quét vốn');

    src.balance -= amount;
    tgt.balance += amount;

    logger.info(`Đã quét vốn ${amount} VND từ ${src.bankCode} sang ${tgt.bankCode} thành công`);
  }

  return {
    accounts,
    currentOperatorRole,
    currentOperatorId,
    projectionHorizonDays,
    minOperatingReserve,
    paymentOrders,
    activePaymentOrderId,
    totalCashPosition,
    activePaymentOrder,
    projectedCashflowData,
    lowestProjectedPoint,
    runwayDays,
    liquidityHealthStatus,
    sweepingRecommendations,
    switchOperatorRole,
    selectPaymentOrder,
    createPaymentOrder,
    approvePaymentOrder,
    rejectPaymentOrder,
    executePaymentDisbursement,
    executeCashSweeping,
  };
});

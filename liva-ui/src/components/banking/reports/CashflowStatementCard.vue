<script setup lang="ts">
/**
 * CashflowStatementCard.vue — P93 Báo Cáo Lưu Chuyển Tiền Tệ (Mẫu B03-DN)
 * =========================================================================
 * Báo cáo lưu chuyển tiền tệ theo phương pháp trực tiếp chuẩn Thông tư 200/2014/TT-BTC.
 * Tách biệt 3 dòng tiền: Hoạt động kinh doanh, Hoạt động đầu tư và Hoạt động tài chính.
 */
import { useReportStore } from '../../../stores/reportStore';

const store = useReportStore();

function formatVnd(val: number): string {
  if (val === 0) return '0 ₫';
  const prefix = val < 0 ? '- ' : '+ ';
  return `${prefix}${Math.abs(val).toLocaleString('vi-VN')} ₫`;
}

function formatAbsolute(val: number): string {
  return `${val.toLocaleString('vi-VN')} ₫`;
}
</script>

<template>
  <div class="cashflow-card">
    <div class="card-header">
      <div>
        <div class="header-tags">
          <span class="vas-badge">MẪU B03-DN (TT 200/2014/TT-BTC)</span>
          <span class="method-pill">Phương pháp Trực tiếp (Direct Method)</span>
        </div>
        <h3 class="card-title">Báo Cáo Lưu Chuyển Tiền Tệ (Statement of Cash Flows)</h3>
        <p class="card-desc">
          Đối chiếu dòng tiền vào/ra thực tế qua các tài khoản ngân hàng trong kỳ {{ store.currentPeriod }}.
        </p>
      </div>

      <div class="summary-box">
        <div class="summary-label">Lưu Chuyển Tiền Thuần Trong Kỳ</div>
        <div class="summary-val font-mono" :class="store.cashFlow.netCashFlow >= 0 ? 'pos' : 'neg'">
          {{ formatVnd(store.cashFlow.netCashFlow) }}
        </div>
      </div>
    </div>

    <!-- 3 Activity Blocks -->
    <div class="blocks-grid">
      <!-- 1. Hoạt động kinh doanh -->
      <div class="flow-block">
        <div class="block-header">
          <div class="block-title">I. Dòng Tiền Từ Hoạt Động Kinh Doanh (Operating)</div>
          <span class="code-badge">Mã 20</span>
        </div>
        <div class="flow-lines">
          <div class="flow-line">
            <span class="line-label">1. Tiền thu từ bán hàng, cung cấp dịch vụ</span>
            <span class="line-val pos font-mono">+ {{ formatAbsolute(store.cashFlow.operatingInflow) }}</span>
          </div>
          <div class="flow-line">
            <span class="line-label">2. Tiền chi trả người cung cấp hàng hóa, dịch vụ & nhân công</span>
            <span class="line-val neg font-mono">- {{ formatAbsolute(store.cashFlow.operatingOutflow) }}</span>
          </div>
        </div>
        <div class="block-footer">
          <span>Lưu chuyển tiền thuần từ HĐKD (Mã 20)</span>
          <strong class="font-mono" :class="store.cashFlow.netOperatingFlow >= 0 ? 'pos' : 'neg'">
            {{ formatVnd(store.cashFlow.netOperatingFlow) }}
          </strong>
        </div>
      </div>

      <!-- 2. Hoạt động đầu tư -->
      <div class="flow-block">
        <div class="block-header">
          <div class="block-title">II. Dòng Tiền Từ Hoạt Động Đầu Tư (Investing)</div>
          <span class="code-badge">Mã 30</span>
        </div>
        <div class="flow-lines">
          <div class="flow-line">
            <span class="line-label">1. Tiền thu hồi đầu tư, thanh lý nhượng bán TSCĐ</span>
            <span class="line-val pos font-mono">+ {{ formatAbsolute(store.cashFlow.investingInflow) }}</span>
          </div>
          <div class="flow-line">
            <span class="line-label">2. Tiền chi mua sắm tài sản cố định & thiết bị công nghệ</span>
            <span class="line-val neg font-mono">- {{ formatAbsolute(store.cashFlow.investingOutflow) }}</span>
          </div>
        </div>
        <div class="block-footer">
          <span>Lưu chuyển tiền thuần từ HĐ đầu tư (Mã 30)</span>
          <strong class="font-mono" :class="store.cashFlow.netInvestingFlow >= 0 ? 'pos' : 'neg'">
            {{ formatVnd(store.cashFlow.netInvestingFlow) }}
          </strong>
        </div>
      </div>

      <!-- 3. Hoạt động tài chính -->
      <div class="flow-block">
        <div class="block-header">
          <div class="block-title">III. Dòng Tiền Từ Hoạt Động Tài Chính (Financing)</div>
          <span class="code-badge">Mã 40</span>
        </div>
        <div class="flow-lines">
          <div class="flow-line">
            <span class="line-label">1. Tiền thu từ phát hành cổ phiếu, nhận vốn góp chủ sở hữu</span>
            <span class="line-val pos font-mono">+ {{ formatAbsolute(store.cashFlow.financingInflow) }}</span>
          </div>
          <div class="flow-line">
            <span class="line-label">2. Tiền chi trả nợ gốc vay ngân hàng (VCB/TCB)</span>
            <span class="line-val neg font-mono">- {{ formatAbsolute(store.cashFlow.financingOutflow) }}</span>
          </div>
        </div>
        <div class="block-footer">
          <span>Lưu chuyển tiền thuần từ HĐ tài chính (Mã 40)</span>
          <strong class="font-mono" :class="store.cashFlow.netFinancingFlow >= 0 ? 'pos' : 'neg'">
            {{ formatVnd(store.cashFlow.netFinancingFlow) }}
          </strong>
        </div>
      </div>
    </div>

    <!-- Reconcile Cash Invariants Footer -->
    <div class="reconcile-footer">
      <div class="reconcile-step">
        <span class="step-num">Mã 60</span>
        <span class="step-label">Tiền & tương đương tiền đầu kỳ:</span>
        <strong class="font-mono">{{ formatAbsolute(store.cashFlow.openingCash) }}</strong>
      </div>
      <div class="operator">+</div>
      <div class="reconcile-step">
        <span class="step-num">Mã 50</span>
        <span class="step-label">Lưu chuyển thuần trong kỳ:</span>
        <strong class="font-mono" :class="store.cashFlow.netCashFlow >= 0 ? 'pos' : 'neg'">
          {{ formatVnd(store.cashFlow.netCashFlow) }}
        </strong>
      </div>
      <div class="operator">=</div>
      <div class="reconcile-step highlight">
        <span class="step-num">Mã 70</span>
        <span class="step-label">Tiền & tương đương tiền cuối kỳ:</span>
        <strong class="font-mono text-gold">{{ formatAbsolute(store.cashFlow.closingCash) }}</strong>
      </div>
    </div>
  </div>
</template>

<style scoped>
.cashflow-card {
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.header-tags {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.vas-badge {
  font-size: 11px;
  font-weight: 700;
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.12);
  border: 1px solid rgba(56, 189, 248, 0.3);
  padding: 2px 8px;
  border-radius: 4px;
}

.method-pill {
  font-size: 11px;
  color: #a78bfa;
  background: rgba(167, 139, 250, 0.1);
  border: 1px solid rgba(167, 139, 250, 0.25);
  padding: 2px 8px;
  border-radius: 4px;
}

.card-title {
  font-size: 16px;
  font-weight: 700;
  color: #f8fafc;
  margin: 0 0 4px 0;
}

.card-desc {
  font-size: 12px;
  color: #94a3b8;
  margin: 0;
}

.summary-box {
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 8px 16px;
  text-align: right;
}

.summary-label {
  font-size: 11px;
  color: #94a3b8;
}

.summary-val {
  font-size: 18px;
  font-weight: 700;
}

.blocks-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 14px;
}

@media (max-width: 1024px) {
  .blocks-grid {
    grid-template-columns: 1fr;
  }
}

.flow-block {
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 12px;
}

.block-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  padding-bottom: 8px;
}

.block-title {
  font-size: 12px;
  font-weight: 700;
  color: #f1f5f9;
}

.code-badge {
  font-size: 10px;
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.1);
  padding: 1px 6px;
  border-radius: 3px;
  font-family: monospace;
}

.flow-lines {
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-size: 11px;
}

.flow-line {
  display: flex;
  justify-content: space-between;
  gap: 8px;
}

.line-label {
  color: #94a3b8;
}

.line-val {
  font-size: 12px;
  white-space: nowrap;
}

.block-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  padding-top: 8px;
  font-size: 12px;
  color: #cbd5e1;
}

.reconcile-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 12px 20px;
  font-size: 13px;
}

.reconcile-step {
  display: flex;
  align-items: center;
  gap: 8px;
}

.reconcile-step.highlight {
  background: rgba(245, 158, 11, 0.1);
  padding: 4px 12px;
  border-radius: 6px;
  border: 1px solid rgba(245, 158, 11, 0.25);
}

.step-num {
  font-size: 10px;
  color: #38bdf8;
  background: rgba(56, 189, 248, 0.1);
  padding: 1px 5px;
  border-radius: 3px;
  font-family: monospace;
}

.step-label {
  color: #94a3b8;
}

.operator {
  font-size: 16px;
  font-weight: 700;
  color: #64748b;
}

.pos {
  color: #10b981;
}

.neg {
  color: #f87171;
}

.text-gold {
  color: #f59e0b;
}
</style>

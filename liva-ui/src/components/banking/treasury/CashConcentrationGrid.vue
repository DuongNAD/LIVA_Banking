<script setup lang="ts">
/**
 * CashConcentrationGrid.vue — P60 Multi-Bank Cash Concentration Pool
 * ====================================================================
 * Visualizes multi-bank liquidity positions, target buffers,
 * and automated cash sweeping recommendations to Master Pool.
 */
import { useTreasuryStore } from '../../../stores/treasuryStore';
import type { TreasuryAccount } from '../../../stores/treasuryStore';

const store = useTreasuryStore();

function formatVnd(val: number): string {
  return `${val.toLocaleString('vi-VN')} ₫`;
}

function handleExecuteSweep(rec: { source: TreasuryAccount; target: TreasuryAccount; excessAmount: number }) {
  store.executeCashSweeping(rec.source.id, rec.target.id, rec.excessAmount);
}
</script>

<template>
  <div class="cash-concentration-module">
    <!-- Top Sweeping Recommendations Alert Banner -->
    <div v-if="store.sweepingRecommendations.length > 0" class="sweeping-alert-box">
      <div class="alert-icon">⚡</div>
      <div class="alert-body">
        <div class="alert-headline">Phát hiện số dư thặng dư vượt ngưỡng — Khuyến nghị quét vốn tự động (Sweeping):</div>
        <div class="recs-list">
          <div
            v-for="(rec, idx) in store.sweepingRecommendations"
            :key="idx"
            class="rec-item"
          >
            <span class="rec-text">
              Điều chuyển <strong>{{ formatVnd(rec.excessAmount) }}</strong> từ <strong>{{ rec.source.bankName }}</strong> về <strong>{{ rec.target.bankName }}</strong>
            </span>
            <button class="btn-sweep-action" @click="handleExecuteSweep(rec)">
              Quét Vốn Ngay
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Accounts Cards Grid -->
    <div class="accounts-grid">
      <div
        v-for="acc in store.accounts"
        :key="acc.id"
        class="treasury-card"
        :class="{ 'is-master': acc.accountType === 'MASTER_POOL' }"
      >
        <!-- Card Header -->
        <div class="card-top">
          <div class="bank-badge-group">
            <span class="bank-pill" :class="acc.bankCode.toLowerCase()">{{ acc.bankCode }}</span>
            <span class="acc-type-pill" :class="acc.accountType.toLowerCase()">
              {{ acc.accountType === 'MASTER_POOL' ? 'MASTER POOL' : acc.accountType === 'COLLECTIONS' ? 'THU TIỀN' : 'CHI TIỀN' }}
            </span>
          </div>
          <span class="acc-no">{{ acc.accountNumber }}</span>
        </div>

        <div class="acc-name">{{ acc.bankName }}</div>

        <!-- Current Balance Display -->
        <div class="balance-metric">
          <span class="metric-lbl">Số dư thanh toán khả dụng:</span>
          <span class="metric-number">{{ formatVnd(acc.balance) }}</span>
        </div>

        <!-- Buffer Progress Track -->
        <div class="buffer-section">
          <div class="buffer-labels">
            <span>Ngưỡng đệm mục tiêu: {{ formatVnd(acc.targetBuffer) }}</span>
            <span v-if="acc.balance >= acc.targetBuffer" class="surplus-tag text-green">Thặng dư</span>
            <span v-else class="surplus-tag text-red">Thiếu hụt đệm</span>
          </div>
          <div class="progress-track">
            <div
              class="progress-bar"
              :style="{
                width: `${Math.min(100, Math.round((acc.balance / acc.targetBuffer) * 100))}%`,
                backgroundColor: acc.balance >= acc.targetBuffer ? '#10b981' : '#f59e0b',
              }"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.cash-concentration-module {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.sweeping-alert-box {
  display: flex;
  gap: 12px;
  background: rgba(245, 158, 11, 0.1);
  border: 1px solid rgba(245, 158, 11, 0.3);
  border-radius: 8px;
  padding: 14px 18px;
  color: #fbbf24;
}

.alert-icon {
  font-size: 20px;
}

.alert-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.alert-headline {
  font-size: 13px;
  font-weight: 700;
}

.recs-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.rec-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #0f172a;
  border-radius: 6px;
  padding: 8px 12px;
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.rec-text {
  font-size: 12px;
  color: #e2e8f0;
}

.btn-sweep-action {
  background: #f59e0b;
  color: #0f172a;
  border: none;
  border-radius: 4px;
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 800;
  cursor: pointer;
  transition: background 0.15s ease;
}

.btn-sweep-action:hover {
  background: #d97706;
}

.accounts-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 16px;
}

.treasury-card {
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  transition: transform 0.15s ease, border-color 0.15s ease;
}

.treasury-card:hover {
  border-color: rgba(255, 255, 255, 0.2);
}

.treasury-card.is-master {
  border-color: rgba(16, 185, 129, 0.4);
  box-shadow: 0 4px 20px rgba(16, 185, 129, 0.08);
}

.card-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.bank-badge-group {
  display: flex;
  align-items: center;
  gap: 6px;
}

.bank-pill {
  font-size: 11px;
  font-weight: 800;
  padding: 2px 8px;
  border-radius: 4px;
}

.bank-pill.vcb {
  background: rgba(16, 185, 129, 0.2);
  color: #34d399;
}

.bank-pill.tcb {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
}

.bank-pill.bidv {
  background: rgba(59, 130, 246, 0.2);
  color: #60a5fa;
}

.bank-pill.mbb {
  background: rgba(168, 85, 247, 0.2);
  color: #c084fc;
}

.acc-type-pill {
  font-size: 9px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.06);
  color: #94a3b8;
}

.acc-type-pill.master_pool {
  background: rgba(16, 185, 129, 0.2);
  color: #10b981;
}

.acc-no {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: #64748b;
}

.acc-name {
  font-size: 13px;
  font-weight: 600;
  color: #f1f5f9;
}

.balance-metric {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.metric-lbl {
  font-size: 11px;
  color: #64748b;
}

.metric-number {
  font-family: 'JetBrains Mono', monospace;
  font-size: 18px;
  font-weight: 800;
  color: #f8fafc;
}

.buffer-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 4px;
}

.buffer-labels {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11px;
  color: #94a3b8;
}

.surplus-tag.text-green {
  color: #34d399;
  font-weight: 700;
}

.surplus-tag.text-red {
  color: #f59e0b;
  font-weight: 700;
}

.progress-track {
  height: 6px;
  background: #0f172a;
  border-radius: 3px;
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  border-radius: 3px;
  transition: width 0.3s ease;
}
</style>

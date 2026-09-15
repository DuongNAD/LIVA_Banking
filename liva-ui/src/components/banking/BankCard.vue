<script setup lang="ts">
/**
 * BankCard.vue
 * Individual Bank Account Summary Card displaying:
 * - Bank Name & Code
 * - Account Number
 * - Opening Balance (Số dư đầu kỳ)
 * - Closing Balance (Số dư cuối kỳ)
 * - Reconciled Amount (Đã đối soát)
 * - Discrepancy Amount (Chênh lệch)
 * - Last Sync Time & Invariant Progress Track
 */
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    bankName: string;
    bankCode: 'VCB' | 'TCB' | 'BIDV';
    accountNumber?: string;
    openingBalance?: number;
    closingBalance?: number;
    balance?: number;
    reconciled?: number;
    unreconciled?: number;
    discrepancy?: number;
    lastSync: string;
  }>(),
  {
    accountNumber: '****8921',
    openingBalance: 0,
    closingBalance: 0,
    balance: 0,
    reconciled: 0,
    unreconciled: 0,
    discrepancy: 0,
  }
);

function formatVnd(val: number): string {
  return new Intl.NumberFormat('en-US').format(val) + ' VND';
}

const displayBalance = computed(() => {
  return props.balance !== undefined && props.balance > 0 ? props.balance : (props.closingBalance || 0);
});

const displayDiscrepancy = computed(() => {
  if (props.unreconciled !== undefined && props.unreconciled > 0) return props.unreconciled;
  if (props.discrepancy !== undefined && props.discrepancy > 0) return props.discrepancy;
  return props.unreconciled ?? props.discrepancy ?? 0;
});

const progressPercentage = computed(() => {
  if (displayBalance.value <= 0) return 0;
  return Math.min(100, Math.max(0, ((props.reconciled || 0) / displayBalance.value) * 100));
});
</script>

<template>
  <div class="bank-card">
    <!-- Header: Bank Icon + Name + Account Number -->
    <div class="bank-card-header">
      <div class="bank-info">
        <!-- VCB Icon (Green shield) -->
        <div v-if="bankCode === 'VCB'" class="bank-icon-vcb">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none">
            <path d="M12 2L4 5V11.5C4 16.5 7.5 21.2 12 22.5C16.5 21.2 20 16.5 20 11.5V5L12 2Z" fill="#10b981" />
            <path d="M12 6L7 11H10V16H14V11H17L12 6Z" fill="#ffffff" />
          </svg>
        </div>

        <!-- TCB Icon (Red dual rhombuses) -->
        <div v-else-if="bankCode === 'TCB'" class="bank-icon-tcb">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none">
            <rect x="7" y="3" width="7" height="7" transform="rotate(45 7 3)" fill="#ef4444" />
            <rect x="14" y="10" width="7" height="7" transform="rotate(45 14 10)" fill="#ef4444" />
          </svg>
        </div>

        <!-- Fallback/BIDV Icon (Blue bank) -->
        <div v-else class="bank-icon-bidv">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="#0284c7">
            <path d="M3 21h18M3 10h18M5 10v11M9 10v11M15 10v11M19 10v11M12 2l10 5H2z" />
          </svg>
        </div>

        <div class="bank-identity">
          <span class="bank-name-text">{{ bankName }}</span>
          <span class="account-badge font-mono">{{ accountNumber }}</span>
        </div>
      </div>

      <button class="more-options-btn" title="Chi tiết tài khoản">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
          <circle cx="5" cy="12" r="2" />
          <circle cx="12" cy="12" r="2" />
          <circle cx="19" cy="12" r="2" />
        </svg>
      </button>
    </div>

    <!-- Balance Overview: Closing vs Opening -->
    <div class="balance-row">
      <span class="balance-label">Số dư cuối kỳ:</span>
      <span class="balance-value">{{ formatVnd(displayBalance) }}</span>
    </div>

    <div v-if="openingBalance > 0" class="opening-balance-row">
      <span class="opening-label">Số dư đầu kỳ:</span>
      <span class="opening-val font-mono">{{ formatVnd(openingBalance) }}</span>
    </div>

    <!-- Reconciliation Progress Bar -->
    <div class="progress-track">
      <div class="progress-fill" :style="{ width: `${progressPercentage}%` }" />
    </div>

    <!-- Reconciled / Discrepancy Metrics -->
    <div class="metrics-grid">
      <div class="metric-row">
        <span class="metric-label">Đã đối soát:</span>
        <span class="metric-val reconciled-val">{{ formatVnd(reconciled || 0) }}</span>
      </div>
      <div class="metric-row">
        <span class="metric-label">Chênh lệch:</span>
        <span
          class="metric-val unreconciled-val"
          :class="{ 'text-red font-bold': displayDiscrepancy > 0, 'text-green': displayDiscrepancy === 0 }"
        >
          {{ formatVnd(displayDiscrepancy) }}
        </span>
      </div>
    </div>

    <!-- Footer: Last sync & status indicator -->
    <div class="bank-card-footer">
      <span class="last-sync-text">Last sync: {{ lastSync }}</span>
      <span v-if="displayDiscrepancy === 0" class="status-pill status-balanced">Cân bằng 100%</span>
      <span v-else class="status-pill status-pending">Chờ xử lý</span>
    </div>
  </div>
</template>

<style scoped>
.bank-card {
  background: #ffffff;
  border-radius: 12px;
  border: 1px solid #e2e8f0;
  padding: 18px 20px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
  display: flex;
  flex-direction: column;
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

.bank-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.06);
}

.bank-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.bank-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.bank-identity {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.bank-name-text {
  font-size: 14px;
  font-weight: 700;
  color: #1e293b;
}

.account-badge {
  font-size: 11px;
  color: #64748b;
  background: #f1f5f9;
  padding: 1px 6px;
  border-radius: 4px;
  width: fit-content;
}

.bank-icon-vcb,
.bank-icon-tcb,
.bank-icon-bidv {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background-color: #f8fafc;
}

.more-options-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
}

.more-options-btn:hover {
  background: #f1f5f9;
  color: #475569;
}

.balance-row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  margin-bottom: 4px;
}

.balance-label {
  font-size: 12px;
  color: #64748b;
  font-weight: 500;
}

.balance-value {
  font-size: 17px;
  font-weight: 800;
  color: #0f172a;
  font-feature-settings: 'tnum';
}

.opening-balance-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11px;
  color: #94a3b8;
  margin-bottom: 10px;
}

.progress-track {
  height: 5px;
  background-color: #f1f5f9;
  border-radius: 3px;
  overflow: hidden;
  margin-bottom: 12px;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #10b981 0%, #059669 100%);
  border-radius: 3px;
  transition: width 0.3s ease;
}

.metrics-grid {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding-top: 4px;
  margin-bottom: 12px;
  border-top: 1px dashed #f1f5f9;
}

.metric-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
}

.metric-label {
  color: #64748b;
}

.metric-val {
  font-weight: 600;
  font-feature-settings: 'tnum';
}

.reconciled-val {
  color: #10b981;
}

.unreconciled-val {
  color: #ef4444;
}

.text-red {
  color: #ef4444 !important;
}

.text-green {
  color: #10b981 !important;
}

.bank-card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-top: 1px solid #f1f5f9;
  padding-top: 8px;
}

.last-sync-text {
  font-size: 11px;
  color: #94a3b8;
}

.status-pill {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 4px;
}

.status-balanced {
  background: #ecfdf5;
  color: #059669;
}

.status-pending {
  background: #fef2f2;
  color: #dc2626;
}
</style>

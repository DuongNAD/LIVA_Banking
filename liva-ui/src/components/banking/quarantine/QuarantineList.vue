<script setup lang="ts">
/**
 * QuarantineList.vue — P44 Quarantine Queue Master List
 * ======================================================
 * Displays list of quarantined transactions with:
 * - Severity filters (CRITICAL, HIGH, MEDIUM)
 * - Status filters (PENDING_MAKER, SUBMITTED_TO_CHECKER, APPROVED, REJECTED)
 * - Search by TxCode and Memo
 * - Circular 09 / Decree 13 badge indicators
 */
import { ref, computed } from 'vue';
import { useWorkbenchStore } from '../../../stores/workbenchStore';
import type { QuarantineItem, QuarantineExceptionType } from '../../../stores/workbenchStore';

const store = useWorkbenchStore();
const searchQuery = ref('');

const exceptionLabels: Record<QuarantineExceptionType, { label: string; icon: string }> = {
  AML_HIGH_VALUE: { label: 'AML Giá trị lớn (≥400Tr)', icon: '🚨' },
  FEE_OVER_LIMIT: { label: 'Lệch phí vượt trần (>22k)', icon: '💸' },
  UNKNOWN_PARTNER: { label: 'Chưa rõ đối tác', icon: '❓' },
  DUPLICATE_REF: { label: 'Trùng mã tham chiếu', icon: '⚠️' },
  REVERSAL_SUSPECT: { label: 'Nghi ngờ giao dịch đảo', icon: '🔄' },
};

const displayItems = computed(() => {
  return store.filteredQuarantineItems.filter((item) => {
    if (!searchQuery.value.trim()) return true;
    const q = searchQuery.value.toLowerCase();
    return (
      item.txCode.toLowerCase().includes(q) ||
      item.memo.toLowerCase().includes(q) ||
      item.accountNumber.toLowerCase().includes(q)
    );
  });
});

function formatVnd(amount: number): string {
  const prefix = amount > 0 ? '+' : '';
  return `${prefix}${amount.toLocaleString('vi-VN')} ₫`;
}

function selectItem(item: QuarantineItem) {
  store.selectQuarantineItem(item.id);
}
</script>

<template>
  <div class="quarantine-list-pane">
    <!-- Header with Counter & Filters -->
    <div class="list-header">
      <div class="header-title-row">
        <div class="title-with-badge">
          <span class="pane-title">Hàng Đợi Cách Ly</span>
          <span class="count-badge">{{ displayItems.length }}</span>
        </div>
        <div class="aml-mandate-badge" title="Áp dụng Quyết định 11/2023/QĐ-TTg & Thông tư 09/2020">
          <span>🛡️ QĐ11 & TT09</span>
        </div>
      </div>

      <!-- Search Input -->
      <div class="search-box">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8" />
          <line x1="21" y1="21" x2="16.65" y2="16.65" />
        </svg>
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Tìm mã GD, nội dung memo..."
          class="search-input"
        />
      </div>

      <!-- Filter Controls -->
      <div class="filter-row">
        <div class="filter-group">
          <label class="filter-label">Trạng thái:</label>
          <select v-model="store.quarantineFilterStatus" class="filter-select">
            <option value="ALL">Tất cả trạng thái</option>
            <option value="PENDING_MAKER">Chờ Maker đề xuất</option>
            <option value="SUBMITTED_TO_CHECKER">Chờ KTT phê duyệt</option>
            <option value="APPROVED">Đã phê duyệt</option>
            <option value="REJECTED">Đã từ chối</option>
          </select>
        </div>

        <div class="filter-group">
          <label class="filter-label">Mức độ:</label>
          <select v-model="store.quarantineFilterSeverity" class="filter-select">
            <option value="ALL">Tất cả mức độ</option>
            <option value="CRITICAL">Nghiêm trọng (CRITICAL)</option>
            <option value="HIGH">Cao (HIGH)</option>
            <option value="MEDIUM">Trung bình (MEDIUM)</option>
          </select>
        </div>
      </div>
    </div>

    <!-- Items Scroll Container -->
    <div class="items-container">
      <div v-if="displayItems.length === 0" class="empty-state">
        <span class="empty-icon">✓</span>
        <p class="empty-title">Không có giao dịch cách ly</p>
        <p class="empty-sub">Tất cả giao dịch đều đạt chuẩn đối soát hoặc bộ lọc không khớp</p>
      </div>

      <div
        v-for="item in displayItems"
        :key="item.id"
        class="quarantine-card"
        :class="{
          selected: store.selectedQuarantineId === item.id,
          'severity-critical': item.severity === 'CRITICAL',
          'severity-high': item.severity === 'HIGH',
          'severity-medium': item.severity === 'MEDIUM',
        }"
        @click="selectItem(item)"
      >
        <!-- Card Top: Bank & TxCode & Severity -->
        <div class="card-top">
          <div class="bank-tx-meta">
            <span class="bank-pill" :class="item.bankCode.toLowerCase()">{{ item.bankCode }}</span>
            <span class="tx-code">{{ item.txCode }}</span>
          </div>
          <span
            class="severity-badge"
            :class="item.severity.toLowerCase()"
          >
            {{ item.severity }}
          </span>
        </div>

        <!-- Exception Type Banner -->
        <div class="exception-banner">
          <span class="exception-icon">{{ exceptionLabels[item.exceptionType]?.icon || '⚠️' }}</span>
          <span class="exception-name">{{ exceptionLabels[item.exceptionType]?.label || item.exceptionType }}</span>
        </div>

        <!-- Memo Snippet -->
        <div class="memo-text" :title="item.memo">
          {{ item.memo }}
        </div>

        <!-- Card Footer: Amount & Status Badge -->
        <div class="card-footer">
          <div
            class="amount-display"
            :class="{ credit: item.amount > 0, debit: item.amount < 0 }"
          >
            {{ formatVnd(item.amount) }}
          </div>

          <div class="status-indicator">
            <span
              v-if="item.status === 'PENDING_MAKER'"
              class="status-pill status-pending-maker"
            >
              Chờ Maker
            </span>
            <span
              v-else-if="item.status === 'SUBMITTED_TO_CHECKER'"
              class="status-pill status-submitted-checker"
            >
              Chờ KTT duyệt
            </span>
            <span
              v-else-if="item.status === 'APPROVED'"
              class="status-pill status-approved"
            >
              Đã duyệt
            </span>
            <span
              v-else-if="item.status === 'REJECTED'"
              class="status-pill status-rejected"
            >
              Từ chối
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.quarantine-list-pane {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #0f172a;
  border-right: 1px solid rgba(255, 255, 255, 0.08);
  overflow: hidden;
}

.list-header {
  padding: 16px;
  background: #1e293b;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.header-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.title-with-badge {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pane-title {
  font-size: 15px;
  font-weight: 700;
  color: #f1f5f9;
  letter-spacing: -0.01em;
}

.count-badge {
  padding: 2px 8px;
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
}

.aml-mandate-badge {
  font-size: 11px;
  color: #94a3b8;
  background: rgba(255, 255, 255, 0.05);
  padding: 3px 8px;
  border-radius: 4px;
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 6px;
  padding: 6px 10px;
  color: #94a3b8;
}

.search-input {
  background: transparent;
  border: none;
  color: #f8fafc;
  font-size: 13px;
  width: 100%;
  outline: none;
}

.search-input::placeholder {
  color: #64748b;
}

.filter-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.filter-label {
  font-size: 11px;
  color: #94a3b8;
}

.filter-select {
  background: #0f172a;
  color: #e2e8f0;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 6px;
  padding: 4px 8px;
  font-size: 12px;
  outline: none;
  cursor: pointer;
}

.items-container {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.quarantine-card {
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 12px;
  cursor: pointer;
  transition: all 0.15s ease-in-out;
  display: flex;
  flex-direction: column;
  gap: 8px;
  position: relative;
}

.quarantine-card:hover {
  background: #273549;
  border-color: rgba(255, 255, 255, 0.15);
}

.quarantine-card.selected {
  border-color: #38bdf8;
  box-shadow: 0 0 0 1px #38bdf8, 0 4px 12px rgba(56, 189, 248, 0.15);
  background: #1e2c44;
}

.card-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.bank-tx-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.bank-pill {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 6px;
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

.tx-code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  font-weight: 600;
  color: #e2e8f0;
}

.severity-badge {
  font-size: 10px;
  font-weight: 800;
  padding: 2px 6px;
  border-radius: 4px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.severity-badge.critical {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.severity-badge.high {
  background: rgba(245, 158, 11, 0.2);
  color: #fbbf24;
  border: 1px solid rgba(245, 158, 11, 0.3);
}

.severity-badge.medium {
  background: rgba(2, 132, 199, 0.2);
  color: #38bdf8;
  border: 1px solid rgba(2, 132, 199, 0.3);
}

.exception-banner {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 600;
  color: #f8fafc;
}

.exception-icon {
  font-size: 13px;
}

.memo-text {
  font-size: 12px;
  color: #94a3b8;
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-break: break-all;
}

.card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  padding-top: 8px;
  margin-top: 4px;
}

.amount-display {
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  font-weight: 700;
}

.amount-display.credit {
  color: #10b981;
}

.amount-display.debit {
  color: #f43f5e;
}

.status-pill {
  font-size: 11px;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 4px;
}

.status-pending-maker {
  background: rgba(245, 158, 11, 0.15);
  color: #fbbf24;
  border: 1px solid rgba(245, 158, 11, 0.25);
}

.status-submitted-checker {
  background: rgba(56, 189, 248, 0.15);
  color: #38bdf8;
  border: 1px solid rgba(56, 189, 248, 0.25);
}

.status-approved {
  background: rgba(16, 185, 129, 0.15);
  color: #34d399;
  border: 1px solid rgba(16, 185, 129, 0.25);
}

.status-rejected {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.25);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 16px;
  text-align: center;
  color: #64748b;
}

.empty-icon {
  font-size: 28px;
  margin-bottom: 8px;
  color: #10b981;
}

.empty-title {
  font-size: 14px;
  font-weight: 600;
  color: #94a3b8;
  margin-bottom: 4px;
}

.empty-sub {
  font-size: 12px;
  color: #64748b;
}
</style>

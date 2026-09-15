<script setup lang="ts">
import { useWorkbenchStore, type BankStatementLine } from '../../../stores/workbenchStore';

const store = useWorkbenchStore();

function formatVnd(val: number): string {
  const formatted = new Intl.NumberFormat('vi-VN').format(Math.abs(val)) + ' ₫';
  return val < 0 ? `-${formatted}` : `+${formatted}`;
}

function handleDragStart(e: DragEvent, item: BankStatementLine) {
  if (e.dataTransfer) {
    e.dataTransfer.setData('application/json', JSON.stringify({ type: 'BANK', id: item.id }));
    e.dataTransfer.effectAllowed = 'copyMove';
  }
}

function openSplitSolver(item: BankStatementLine) {
  store.splitSolverTarget = item;
}
</script>

<template>
  <div class="column-panel bank-column">
    <!-- Header -->
    <div class="column-header">
      <div class="header-title-row">
        <div class="title-with-badge">
          <span class="column-badge bank-badge">CỘT 1</span>
          <h3 class="column-title">Sao Kê Ngân Hàng</h3>
        </div>
        <span class="counter-badge">{{ store.filteredBankLines.length }} dòng</span>
      </div>

      <!-- Controls -->
      <div class="column-toolbar">
        <div class="search-box">
          <svg class="search-icon" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd" />
          </svg>
          <input
            v-model="store.bankSearchQuery"
            type="text"
            placeholder="Tìm mã GD, VietQR, số tiền..."
            class="toolbar-input"
          />
        </div>

        <div class="filter-row">
          <select v-model="store.bankFilterBank" class="toolbar-select">
            <option value="ALL">Tất cả ngân hàng</option>
            <option value="VCB">Vietcombank</option>
            <option value="TCB">Techcombank</option>
            <option value="BIDV">BIDV</option>
          </select>

          <select v-model="store.bankFilterStatus" class="toolbar-select">
            <option value="UNMATCHED">Chưa đối soát</option>
            <option value="MATCHED">Đã khớp</option>
            <option value="ALL">Tất cả</option>
          </select>
        </div>
      </div>
    </div>

    <!-- List of Bank Lines -->
    <div class="column-content">
      <div v-if="store.filteredBankLines.length === 0" class="empty-state">
        <span class="empty-icon">🏦</span>
        <p>Không có giao dịch ngân hàng nào phù hợp bộ lọc.</p>
      </div>

      <div
        v-for="item in store.filteredBankLines"
        :key="item.id"
        class="transaction-card bank-card"
        :class="{
          'is-selected': store.selectedBankLineIds.includes(item.id),
          'is-matched': item.isMatched,
        }"
        :draggable="!item.isMatched"
        @dragstart="handleDragStart($event, item)"
      >
        <div class="card-top-row">
          <div class="left-meta">
            <input
              type="checkbox"
              :checked="store.selectedBankLineIds.includes(item.id)"
              :disabled="item.isMatched"
              class="card-checkbox"
              @change="store.toggleBankSelection(item.id)"
            />
            <span class="bank-pill" :class="item.bankCode.toLowerCase()">{{ item.bankCode }}</span>
            <span class="tx-code">{{ item.txCode }}</span>
          </div>
          <div class="amount-badge" :class="item.amount >= 0 ? 'credit' : 'debit'">
            {{ formatVnd(item.amount) }}
          </div>
        </div>

        <div class="card-memo">
          {{ item.memo }}
        </div>

        <div class="card-bottom-row">
          <div class="timestamp">
            {{ item.date }} · {{ item.time }}
          </div>

          <div class="actions-group">
            <button
              v-if="!item.isMatched && item.amount > 0"
              class="btn-mini btn-split"
              title="Tìm các hóa đơn ERP để khớp 1:N (k <= 8)"
              @click="openSplitSolver(item)"
            >
              Split Solver (1:N)
            </button>
            <span v-if="item.isMatched" class="matched-tag">✓ Đã ghép</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.column-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #ffffff;
  border-radius: 10px;
  border: 1px solid #e2e8f0;
  overflow: hidden;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
}

.column-header {
  padding: 14px 16px;
  background: #f8fafc;
  border-bottom: 1px solid #e2e8f0;
}

.header-title-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.title-with-badge {
  display: flex;
  align-items: center;
  gap: 8px;
}

.column-badge {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 4px;
}

.bank-badge {
  background: #e0f2fe;
  color: #0369a1;
}

.column-title {
  font-size: 15px;
  font-weight: 700;
  color: #0f172a;
  margin: 0;
}

.counter-badge {
  font-size: 12px;
  font-weight: 600;
  background: #f1f5f9;
  color: #475569;
  padding: 2px 8px;
  border-radius: 9999px;
}

.column-toolbar {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.search-box {
  position: relative;
  width: 100%;
}

.search-icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  width: 15px;
  height: 15px;
  color: #94a3b8;
}

.toolbar-input {
  width: 100%;
  padding: 7px 10px 7px 32px;
  font-size: 12px;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  outline: none;
  background: #ffffff;
}

.toolbar-input:focus {
  border-color: #0284c7;
  box-shadow: 0 0 0 2px rgba(2, 132, 199, 0.15);
}

.filter-row {
  display: flex;
  gap: 8px;
}

.toolbar-select {
  flex: 1;
  padding: 6px 8px;
  font-size: 12px;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  background: #ffffff;
  color: #334155;
  outline: none;
}

.column-content {
  flex: 1;
  padding: 12px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: #94a3b8;
  font-size: 13px;
  text-align: center;
}

.empty-icon {
  font-size: 32px;
  margin-bottom: 8px;
}

.transaction-card {
  padding: 10px 12px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  cursor: grab;
  transition: all 0.15s ease;
  user-select: none;
}

.transaction-card:hover {
  border-color: #94a3b8;
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.05);
}

.transaction-card:active {
  cursor: grabbing;
}

.transaction-card.is-selected {
  border-color: #0284c7;
  background: #f0f9ff;
}

.transaction-card.is-matched {
  opacity: 0.6;
  background: #f8fafc;
  cursor: default;
}

.card-top-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.left-meta {
  display: flex;
  align-items: center;
  gap: 6px;
}

.card-checkbox {
  cursor: pointer;
}

.bank-pill {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 4px;
}

.bank-pill.vcb { background: #dcfce7; color: #15803d; }
.bank-pill.tcb { background: #fee2e2; color: #b91c1c; }
.bank-pill.bidv { background: #dbeafe; color: #1d4ed8; }

.tx-code {
  font-size: 12px;
  font-weight: 600;
  color: #475569;
  font-family: monospace;
}

.amount-badge {
  font-size: 13px;
  font-weight: 700;
  font-family: monospace;
}

.amount-badge.credit { color: #16a34a; }
.amount-badge.debit { color: #dc2626; }

.card-memo {
  font-size: 12px;
  color: #1e293b;
  line-height: 1.4;
  margin-bottom: 8px;
  word-break: break-word;
}

.card-bottom-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
  color: #64748b;
}

.actions-group {
  display: flex;
  gap: 6px;
}

.btn-mini {
  font-size: 11px;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 4px;
  border: 1px solid #cbd5e1;
  background: #f8fafc;
  cursor: pointer;
  transition: background 0.15s;
}

.btn-mini:hover {
  background: #e2e8f0;
}

.btn-split {
  background: #f5f3ff;
  border-color: #c4b5fd;
  color: #6d28d9;
}

.btn-split:hover {
  background: #ede9fe;
}

.matched-tag {
  color: #16a34a;
  font-weight: 600;
  font-size: 11px;
}
</style>

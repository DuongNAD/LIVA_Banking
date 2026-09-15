<script setup lang="ts">
import { ref } from 'vue';
import { useWorkbenchStore, type GlRecord } from '../../../stores/workbenchStore';

const store = useWorkbenchStore();
const dragOverGlId = ref<string | null>(null);

function formatVnd(val: number): string {
  const formatted = new Intl.NumberFormat('vi-VN').format(Math.abs(val)) + ' ₫';
  return val < 0 ? `-${formatted}` : `+${formatted}`;
}

function handleDragOver(e: DragEvent, glId: string) {
  e.preventDefault();
  dragOverGlId.value = glId;
  if (e.dataTransfer) {
    e.dataTransfer.dropEffect = 'copy';
  }
}

function handleDragLeave() {
  dragOverGlId.value = null;
}

function handleDrop(e: DragEvent, targetGl: GlRecord) {
  e.preventDefault();
  dragOverGlId.value = null;
  const rawData = e.dataTransfer?.getData('application/json');
  if (!rawData) return;

  try {
    const payload = JSON.parse(rawData);
    if (payload.type === 'BANK' && payload.id) {
      // Ghép ngay lập tức 1 dòng Bank với dòng GL này
      store.createManualMatch([payload.id], [targetGl.id]);
    }
  } catch {
    // Ignore invalid drop payloads
  }
}

function handleDragStart(e: DragEvent, item: GlRecord) {
  if (e.dataTransfer) {
    e.dataTransfer.setData('application/json', JSON.stringify({ type: 'GL', id: item.id }));
    e.dataTransfer.effectAllowed = 'copyMove';
  }
}
</script>

<template>
  <div class="column-panel gl-column">
    <!-- Header -->
    <div class="column-header">
      <div class="header-title-row">
        <div class="title-with-badge">
          <span class="column-badge gl-badge">CỘT 2</span>
          <h3 class="column-title">Sổ Cái ERP (MISA / FAST)</h3>
        </div>
        <span class="counter-badge">{{ store.filteredGlRecords.length }} chứng từ</span>
      </div>

      <!-- Controls -->
      <div class="column-toolbar">
        <div class="search-box">
          <svg class="search-icon" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd" />
          </svg>
          <input
            v-model="store.glSearchQuery"
            type="text"
            placeholder="Tìm số HĐ, đối tác, mã số thuế..."
            class="toolbar-input"
          />
        </div>

        <div class="filter-row">
          <select v-model="store.glFilterSource" class="toolbar-select">
            <option value="ALL">Tất cả ERP</option>
            <option value="MISA">MISA</option>
            <option value="FAST">FAST</option>
            <option value="BRAVO">BRAVO</option>
          </select>

          <select v-model="store.glFilterStatus" class="toolbar-select">
            <option value="UNMATCHED">Chưa đối soát</option>
            <option value="MATCHED">Đã khớp</option>
            <option value="ALL">Tất cả</option>
          </select>
        </div>
      </div>
    </div>

    <!-- List of GL Records -->
    <div class="column-content">
      <div v-if="store.filteredGlRecords.length === 0" class="empty-state">
        <span class="empty-icon">📑</span>
        <p>Không có chứng từ sổ cái nào phù hợp bộ lọc.</p>
      </div>

      <div
        v-for="item in store.filteredGlRecords"
        :key="item.id"
        class="transaction-card gl-card"
        :class="{
          'is-selected': store.selectedGlRecordIds.includes(item.id),
          'is-matched': item.isMatched,
          'is-drag-over': dragOverGlId === item.id,
        }"
        :draggable="!item.isMatched"
        @dragstart="handleDragStart($event, item)"
        @dragover="handleDragOver($event, item.id)"
        @dragleave="handleDragLeave"
        @drop="handleDrop($event, item)"
      >
        <div class="card-top-row">
          <div class="left-meta">
            <input
              type="checkbox"
              :checked="store.selectedGlRecordIds.includes(item.id)"
              :disabled="item.isMatched"
              class="card-checkbox"
              @change="store.toggleGlSelection(item.id)"
            />
            <span class="erp-pill" :class="item.erpSource.toLowerCase()">{{ item.erpSource }}</span>
            <span class="voucher-num">{{ item.voucherNumber }}</span>
            <span class="account-code">TK {{ item.accountCode }}</span>
          </div>
          <div class="amount-badge" :class="item.amount >= 0 ? 'credit' : 'debit'">
            {{ formatVnd(item.amount) }}
          </div>
        </div>

        <div class="partner-title">
          {{ item.partnerName }}
          <span v-if="item.taxId" class="tax-tag">MST: {{ item.taxId }}</span>
        </div>

        <div class="card-desc">
          {{ item.description }}
        </div>

        <div class="card-bottom-row">
          <div class="timestamp">
            Ngày hạch toán: {{ item.date }}
          </div>

          <div class="drop-hint-text">
            <span v-if="dragOverGlId === item.id" class="drop-ready-text">Thả để ghép 1:1!</span>
            <span v-else-if="item.isMatched" class="matched-tag">✓ Đã khớp</span>
            <span v-else class="dropzone-label">Kéo thả sao kê vào đây</span>
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

.gl-badge {
  background: #fef3c7;
  color: #b45309;
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
  border-color: #d97706;
  box-shadow: 0 0 0 2px rgba(217, 119, 6, 0.15);
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
  border-color: #d97706;
  background: #fffbeb;
}

.transaction-card.is-drag-over {
  border-color: #10b981 !important;
  background: #ecfdf5 !important;
  transform: scale(1.01);
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.15);
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

.erp-pill {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 4px;
}

.erp-pill.misa { background: #e0e7ff; color: #3730a3; }
.erp-pill.fast { background: #fae8ff; color: #86198f; }
.erp-pill.bravo { background: #fef2f2; color: #991b1b; }

.voucher-num {
  font-size: 12px;
  font-weight: 700;
  color: #1e293b;
  font-family: monospace;
}

.account-code {
  font-size: 10px;
  font-weight: 600;
  color: #64748b;
  background: #f1f5f9;
  padding: 1px 4px;
  border-radius: 3px;
}

.amount-badge {
  font-size: 13px;
  font-weight: 700;
  font-family: monospace;
}

.amount-badge.credit { color: #16a34a; }
.amount-badge.debit { color: #dc2626; }

.partner-title {
  font-size: 13px;
  font-weight: 600;
  color: #0f172a;
  margin-bottom: 4px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.tax-tag {
  font-size: 10px;
  color: #64748b;
  font-weight: normal;
  font-family: monospace;
}

.card-desc {
  font-size: 12px;
  color: #475569;
  line-height: 1.4;
  margin-bottom: 8px;
}

.card-bottom-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
  color: #64748b;
}

.drop-ready-text {
  color: #15803d;
  font-weight: 700;
}

.dropzone-label {
  font-size: 10px;
  color: #94a3b8;
  font-style: italic;
}

.matched-tag {
  color: #16a34a;
  font-weight: 600;
  font-size: 11px;
}
</style>

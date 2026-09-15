<script setup lang="ts">
import { useWorkbenchStore, type MatchCard } from '../../../stores/workbenchStore';

const store = useWorkbenchStore();

function formatVnd(val: number): string {
  const formatted = new Intl.NumberFormat('vi-VN').format(Math.abs(val)) + ' ₫';
  return val < 0 ? `-${formatted}` : `+${formatted}`;
}

function openUnmatchModal(match: MatchCard) {
  store.unmatchTarget = match;
}

function handleAutoSplitFee(match: MatchCard) {
  store.autoSplitFee(match.id);
}

function handleSubmitBatch() {
  try {
    store.submitBatchToChecker();
    alert(`Batch ${store.activeBatchId} đã được trình Kế toán trưởng duyệt thành công!`);
  } catch (err: unknown) {
    alert(err instanceof Error ? err.message : String(err));
  }
}
</script>

<template>
  <div class="column-panel match-column">
    <!-- Header -->
    <div class="column-header">
      <div class="header-title-row">
        <div class="title-with-badge">
          <span class="column-badge match-badge">CỘT 3</span>
          <h3 class="column-title">Kết Quả Đối Soát & Staging</h3>
        </div>
        <span class="counter-badge">{{ store.filteredMatches.length }} cặp</span>
      </div>

      <!-- Controls & Filters -->
      <div class="column-toolbar">
        <div class="filter-row">
          <select v-model="store.matchFilterTier" class="toolbar-select">
            <option value="ALL">Tất cả phân tầng (Tier 1–3)</option>
            <option value="TIER_1_EXACT">Tier 1: Khớp chính xác 100%</option>
            <option value="TIER_2_FUZZY">Tier 2: Mờ Heuristic</option>
            <option value="TIER_3_SPLIT">Tier 3: Split Solver (P45)</option>
            <option value="MANUAL">Thủ công (Maker)</option>
          </select>

          <label class="checkbox-filter-label">
            <input v-model="store.matchFilterDiffOnly" type="checkbox" />
            <span>Chỉ xem có chênh lệch</span>
          </label>
        </div>
      </div>
    </div>

    <!-- Active Staging Dock (When checkboxes selected in Col 1 & 2) -->
    <div
      v-if="store.selectedBankLineIds.length > 0 || store.selectedGlRecordIds.length > 0"
      class="staging-dock-banner"
    >
      <div class="staging-info">
        <span class="staging-icon">⚡</span>
        <div>
          <div class="staging-title">Đang chọn thủ công</div>
          <div class="staging-counts">
            {{ store.selectedBankLineIds.length }} sao kê · {{ store.selectedGlRecordIds.length }} chứng từ GL
          </div>
        </div>
      </div>

      <div class="staging-actions">
        <button class="btn-clear" @click="store.clearSelections">Hủy chọn</button>
        <button
          class="btn-create-match"
          :disabled="store.selectedBankLineIds.length === 0 || store.selectedGlRecordIds.length === 0"
          @click="() => store.createManualMatch()"
        >
          Ghép Cặp ({{ store.selectedBankLineIds.length }}:{{ store.selectedGlRecordIds.length }})
        </button>
      </div>
    </div>

    <!-- List of Matches -->
    <div class="column-content">
      <div v-if="store.filteredMatches.length === 0" class="empty-state">
        <span class="empty-icon">🤝</span>
        <p>Chưa có cặp đối soát nào phù hợp bộ lọc.</p>
      </div>

      <div
        v-for="item in store.filteredMatches"
        :key="item.id"
        class="match-card"
        :class="{
          'has-diff': item.difference !== 0,
          'has-fee': item.feeAllocated > 0,
        }"
      >
        <!-- Top Tier Pill -->
        <div class="match-top-row">
          <div class="tier-tag" :class="item.tier.toLowerCase()">
            {{ item.tierLabel }}
            <span class="confidence-pill">{{ Math.round(item.confidenceScore * 100) }}%</span>
          </div>

          <div class="status-badge" :class="item.status.toLowerCase()">
            {{ item.status === 'CONFIRMED' ? 'Đã duyệt' : 'Đề xuất' }}
          </div>
        </div>

        <!-- 2-Pane Comparison details -->
        <div class="comparison-grid">
          <!-- Bank side -->
          <div class="grid-side bank-side">
            <div class="side-header">SAO KÊ ({{ item.bankLines.length }})</div>
            <div v-for="b in item.bankLines" :key="b.id" class="sub-item">
              <span class="sub-code">{{ b.txCode }}</span>
              <span class="sub-amount" :class="b.amount >= 0 ? 'credit' : 'debit'">
                {{ formatVnd(b.amount) }}
              </span>
            </div>
            <div class="total-line">
              <span>Tổng:</span>
              <span class="total-val">{{ formatVnd(item.netBankAmount) }}</span>
            </div>
          </div>

          <!-- Divider -->
          <div class="grid-divider">⇄</div>

          <!-- GL side -->
          <div class="grid-side gl-side">
            <div class="side-header">SỔ CÁI ({{ item.glRecords.length }})</div>
            <div v-for="g in item.glRecords" :key="g.id" class="sub-item">
              <span class="sub-code">{{ g.voucherNumber }}</span>
              <span class="sub-amount" :class="g.amount >= 0 ? 'credit' : 'debit'">
                {{ formatVnd(g.amount) }}
              </span>
            </div>
            <div class="total-line">
              <span>Tổng:</span>
              <span class="total-val">{{ formatVnd(item.netGlAmount) }}</span>
            </div>
          </div>
        </div>

        <!-- Difference & Action Row -->
        <div class="diff-summary-row">
          <div class="diff-indicator">
            <span class="diff-label">Chênh lệch:</span>
            <span
              class="diff-value"
              :class="{
                'zero-diff': item.difference === 0,
                'fee-diff': Math.abs(item.difference) >= 1100 && Math.abs(item.difference) <= 22000,
                'err-diff': item.difference !== 0 && (Math.abs(item.difference) < 1100 || Math.abs(item.difference) > 22000),
              }"
            >
              {{ item.difference === 0 ? '0 ₫ (Cân bằng)' : formatVnd(item.difference) }}
            </span>
          </div>

          <div class="card-btn-group">
            <!-- 1-Click Fee Split Button -->
            <button
              v-if="Math.abs(item.difference) >= 1100 && Math.abs(item.difference) <= 22000"
              class="btn-action btn-fee-split"
              title="Tự động trích chênh lệch sang chi phí dịch vụ ngân hàng TK 6425"
              @click="handleAutoSplitFee(item)"
            >
              Tách phí TK 6425
            </button>

            <!-- Unmatch Button -->
            <button
              class="btn-action btn-unmatch"
              title="Hủy đối soát (Bắt buộc nhập lý do vào Hash-chain Audit)"
              @click="openUnmatchModal(item)"
            >
              Hủy đối soát
            </button>
          </div>
        </div>

        <!-- Footnote / Audit note -->
        <div v-if="item.notes" class="match-footnote">
          {{ item.notes }}
        </div>
      </div>
    </div>

    <!-- Batch Bottom Action Bar -->
    <div class="column-footer">
      <div class="footer-stats">
        <div>
          <span class="stat-label">Tỷ lệ tự động:</span>
          <span class="stat-value highlight">{{ store.summaryStats.matchRate }}%</span>
        </div>
        <div>
          <span class="stat-label">Chênh lệch chờ xử lý:</span>
          <span class="stat-value" :class="store.summaryStats.pendingDiscrepancyCount > 0 ? 'text-warn' : 'text-ok'">
            {{ store.summaryStats.pendingDiscrepancyCount }}
          </span>
        </div>
      </div>

      <button
        class="btn-submit-batch"
        :disabled="store.batchStatus !== 'IN_PROGRESS' || store.summaryStats.pendingDiscrepancyCount > 0"
        @click="handleSubmitBatch"
      >
        <span v-if="store.batchStatus === 'IN_PROGRESS'">Trình Kế Toán Trưởng Duyệt</span>
        <span v-else>✓ Đã Trình Duyệt</span>
      </button>
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

.match-badge {
  background: #d1fae5;
  color: #047857;
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

.filter-row {
  display: flex;
  align-items: center;
  gap: 10px;
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

.checkbox-filter-label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: #475569;
  cursor: pointer;
  white-space: nowrap;
}

.staging-dock-banner {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  background: #eff6ff;
  border-bottom: 1px solid #bfdbfe;
}

.staging-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.staging-icon {
  font-size: 18px;
}

.staging-title {
  font-size: 12px;
  font-weight: 700;
  color: #1e40af;
}

.staging-counts {
  font-size: 11px;
  color: #3b82f6;
}

.staging-actions {
  display: flex;
  gap: 6px;
}

.btn-clear {
  padding: 5px 10px;
  font-size: 11px;
  border: 1px solid #cbd5e1;
  background: #ffffff;
  border-radius: 5px;
  cursor: pointer;
}

.btn-create-match {
  padding: 5px 12px;
  font-size: 11px;
  font-weight: 700;
  background: #2563eb;
  color: #ffffff;
  border: none;
  border-radius: 5px;
  cursor: pointer;
}

.btn-create-match:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.column-content {
  flex: 1;
  padding: 12px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
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

.match-card {
  padding: 12px;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.03);
}

.match-card.has-diff {
  border-left: 4px solid #f59e0b;
}

.match-top-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.tier-tag {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.tier-tag.tier_1_exact { background: #dcfce7; color: #15803d; }
.tier-tag.tier_2_fuzzy { background: #dbeafe; color: #1d4ed8; }
.tier-tag.tier_3_split { background: #f3e8ff; color: #7e22ce; }
.tier-tag.manual { background: #fef3c7; color: #b45309; }

.confidence-pill {
  font-size: 9px;
  padding: 1px 4px;
  background: rgba(255, 255, 255, 0.7);
  border-radius: 3px;
}

.status-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 3px;
}

.status-badge.confirmed { background: #e2e8f0; color: #334155; }
.status-badge.proposed { background: #fef08a; color: #854d0e; }

.comparison-grid {
  display: flex;
  align-items: stretch;
  background: #f8fafc;
  border: 1px solid #f1f5f9;
  border-radius: 6px;
  padding: 8px;
  gap: 8px;
}

.grid-side {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.side-header {
  font-size: 10px;
  font-weight: 700;
  color: #64748b;
  margin-bottom: 2px;
}

.sub-item {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
}

.sub-code {
  color: #334155;
  font-family: monospace;
}

.sub-amount {
  font-weight: 600;
  font-family: monospace;
}

.sub-amount.credit { color: #16a34a; }
.sub-amount.debit { color: #dc2626; }

.grid-divider {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #94a3b8;
  font-size: 14px;
}

.total-line {
  margin-top: 4px;
  padding-top: 4px;
  border-top: 1px dashed #cbd5e1;
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  font-weight: 700;
}

.total-val {
  font-family: monospace;
  color: #0f172a;
}

.diff-summary-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 4px;
}

.diff-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
}

.diff-label {
  font-size: 11px;
  color: #64748b;
}

.diff-value {
  font-size: 12px;
  font-weight: 700;
  font-family: monospace;
}

.diff-value.zero-diff { color: #16a34a; }
.diff-value.fee-diff { color: #d97706; }
.diff-value.err-diff { color: #dc2626; }

.card-btn-group {
  display: flex;
  gap: 6px;
}

.btn-action {
  font-size: 11px;
  font-weight: 600;
  padding: 4px 8px;
  border-radius: 4px;
  cursor: pointer;
  border: 1px solid #cbd5e1;
  background: #ffffff;
  transition: all 0.15s;
}

.btn-fee-split {
  background: #fef3c7;
  border-color: #fcd34d;
  color: #b45309;
}

.btn-fee-split:hover {
  background: #fde68a;
}

.btn-unmatch {
  background: #fff1f2;
  border-color: #fecdd3;
  color: #e11d48;
}

.btn-unmatch:hover {
  background: #ffe4e6;
}

.match-footnote {
  font-size: 10px;
  color: #64748b;
  font-style: italic;
  padding: 4px 6px;
  background: #f8fafc;
  border-radius: 4px;
}

.column-footer {
  padding: 12px 16px;
  background: #f8fafc;
  border-top: 1px solid #e2e8f0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.footer-stats {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
}

.stat-label {
  color: #64748b;
  margin-right: 4px;
}

.stat-value {
  font-weight: 700;
}

.stat-value.highlight {
  color: #0284c7;
}

.stat-value.text-warn {
  color: #dc2626;
}

.stat-value.text-ok {
  color: #16a34a;
}

.btn-submit-batch {
  width: 100%;
  padding: 9px;
  font-size: 13px;
  font-weight: 700;
  background: #0f172a;
  color: #ffffff;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
}

.btn-submit-batch:hover:not(:disabled) {
  background: #1e293b;
}

.btn-submit-batch:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>

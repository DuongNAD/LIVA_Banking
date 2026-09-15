<script setup lang="ts">
/**
 * JournalVoucherTable.vue — P50 Journal Entry Proposals & P52 Sync Queue
 * =======================================================================
 * Displays double-entry accounting vouchers synthesized from reconciliation:
 * - Invariant indicator: Delta == 0 VND
 * - Multi-line Debit/Credit breakdown (TK 1121, 131, 331, 6425, 1388)
 * - ERP sync action (Single & Batch) to MISA AMIS / FAST
 * - Payload inspector trigger
 */
import { ref } from 'vue';
import { useLedgerStore } from '../../../stores/ledgerStore';
import type { JournalVoucher } from '../../../stores/ledgerStore';

const store = useLedgerStore();
const isBatchSyncing = ref(false);

function formatVnd(amount: number): string {
  return `${amount.toLocaleString('vi-VN')} ₫`;
}

async function handleBatchPost() {
  isBatchSyncing.value = true;
  try {
    await store.batchPostToErp();
  } finally {
    isBatchSyncing.value = false;
  }
}

async function handleSinglePost(v: JournalVoucher) {
  try {
    await store.postVoucherToErp(v.id);
  } catch (err) {
    // Error recorded in store
  }
}

function openPayload(v: JournalVoucher) {
  store.openPayloadModal(v);
}
</script>

<template>
  <div class="voucher-table-pane">
    <!-- Top Action & Filter Toolbar -->
    <div class="table-toolbar">
      <div class="toolbar-left">
        <!-- Search Input -->
        <div class="search-box">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <input
            v-model="store.searchQuery"
            type="text"
            placeholder="Tìm số chứng từ, đối tác, GUID..."
            class="search-input"
          />
        </div>

        <!-- Status Filter -->
        <div class="filter-group">
          <select v-model="store.filterStatus" class="filter-select">
            <option value="ALL">Tất cả trạng thái</option>
            <option value="DRAFT">Bản nháp (Chờ ghi sổ)</option>
            <option value="POSTED_TO_ERP">Đã đồng bộ ERP</option>
            <option value="SYNC_FAILED">Lỗi đồng bộ</option>
          </select>
        </div>

        <!-- ERP Filter -->
        <div class="filter-group">
          <select v-model="store.filterErp" class="filter-select">
            <option value="ALL">Tất cả ERP</option>
            <option value="MISA">MISA AMIS</option>
            <option value="FAST">FAST Business</option>
            <option value="BRAVO">BRAVO ERP</option>
          </select>
        </div>
      </div>

      <div class="toolbar-right">
        <!-- Default Target ERP Selector -->
        <div class="erp-target-selector">
          <span class="target-lbl">Hệ thống ERP đích:</span>
          <select v-model="store.defaultErp" class="erp-select">
            <option value="MISA">MISA AMIS</option>
            <option value="FAST">FAST Business Online</option>
            <option value="BRAVO">BRAVO ERP</option>
          </select>
        </div>

        <!-- Batch Post Button -->
        <button
          class="btn btn-batch-post"
          :disabled="isBatchSyncing || store.voucherStats.pending === 0"
          @click="handleBatchPost"
        >
          <span v-if="isBatchSyncing" class="spinner">⏳</span>
          <span v-else>⚡</span>
          <span>Đồng Bộ Hàng Loạt ({{ store.voucherStats.pending }})</span>
        </button>
      </div>
    </div>

    <!-- Table Container -->
    <div class="table-container">
      <table class="voucher-table">
        <thead>
          <tr>
            <th class="th-voucher">Số Chứng Từ / Ngày</th>
            <th class="th-partner">Đối Tác & Nội Dung Nghiệp Vụ</th>
            <th class="th-accounts">Định Khoản Đối Ứng (Nợ / Có)</th>
            <th class="th-amount text-right">Tổng Tiền (VND)</th>
            <th class="th-balance text-center">Cân Đối</th>
            <th class="th-status text-center">Trạng Thái ERP</th>
            <th class="th-actions text-right">Thao Tác</th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="store.filteredVouchers.length === 0">
            <td colspan="7" class="empty-cell">
              <div class="empty-wrap">
                <span class="empty-icon">📂</span>
                <p>Chưa có chứng từ bút toán nào phù hợp bộ lọc.</p>
              </div>
            </td>
          </tr>

          <tr
            v-for="v in store.filteredVouchers"
            :key="v.id"
            class="voucher-row"
            :class="{ selected: store.selectedVoucherId === v.id }"
            @click="store.selectVoucher(v.id)"
          >
            <!-- Column 1: Voucher Number & Meta -->
            <td class="td-voucher">
              <div class="voucher-num-box">
                <span class="v-num">{{ v.voucherNumber }}</span>
                <span class="v-date">{{ v.voucherDate }}</span>
              </div>
              <div class="v-guid-tag" :title="`Idempotency GUID: ${v.voucherGuid}`">
                <span>GUID: {{ v.voucherGuid.slice(0, 8) }}...</span>
              </div>
            </td>

            <!-- Column 2: Partner & Description -->
            <td class="td-partner">
              <div class="partner-title">{{ v.partnerName }}</div>
              <div v-if="v.partnerTaxId" class="partner-tax">MST: {{ v.partnerTaxId }}</div>
              <div class="desc-text" :title="v.description">{{ v.description }}</div>
            </td>

            <!-- Column 3: Accounts Posting Details -->
            <td class="td-accounts">
              <div class="accounts-lines">
                <div
                  v-for="l in v.lines"
                  :key="l.id"
                  class="acc-line-item"
                  :class="l.postingType.toLowerCase()"
                >
                  <span class="posting-pill" :class="l.postingType.toLowerCase()">
                    {{ l.postingType === 'DEBIT' ? 'Nợ' : 'Có' }}
                  </span>
                  <span class="acc-code">TK {{ l.accountCode }}</span>
                  <span class="acc-amt">{{ formatVnd(l.amount) }}</span>
                  <span class="acc-name">{{ l.accountName }}</span>
                </div>
              </div>
            </td>

            <!-- Column 4: Total Amount -->
            <td class="td-amount text-right">
              <span class="total-amt-val">{{ formatVnd(v.totalDebit) }}</span>
            </td>

            <!-- Column 5: Balance Invariant Check -->
            <td class="td-balance text-center">
              <span v-if="v.difference === 0" class="balance-badge balanced" title="Cân đối: Tổng Nợ == Tổng Có">
                ✓ Δ=0₫
              </span>
              <span v-else class="balance-badge unbalanced" title="Lệch tiền giữa Nợ và Có">
                ⚠️ Lệch {{ formatVnd(v.difference) }}
              </span>
            </td>

            <!-- Column 6: ERP Status -->
            <td class="td-status text-center">
              <div class="status-wrap">
                <span
                  v-if="v.status === 'POSTED_TO_ERP'"
                  class="status-pill status-posted"
                  :title="`Đã ghi sổ: ${v.erpReference} lúc ${v.postedAt}`"
                >
                  ✓ Đã ghi sổ {{ v.targetErp }}
                </span>
                <span
                  v-else-if="v.status === 'PENDING_POST'"
                  class="status-pill status-pending"
                >
                  ⏳ Đang gửi...
                </span>
                <span
                  v-else-if="v.status === 'SYNC_FAILED'"
                  class="status-pill status-failed"
                  :title="v.errorMessage || 'Lỗi không xác định'"
                >
                  ✕ Thất bại
                </span>
                <span
                  v-else
                  class="status-pill status-draft"
                >
                  Chờ ghi sổ
                </span>

                <span v-if="v.erpReference" class="erp-ref-text">{{ v.erpReference }}</span>
              </div>
            </td>

            <!-- Column 7: Actions -->
            <td class="td-actions text-right">
              <div class="actions-group">
                <!-- Payload Modal Trigger -->
                <button
                  class="btn-icon"
                  title="Xem gói tin MISA JSON / FAST XML"
                  @click.stop="openPayload(v)"
                >
                  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="16 18 22 12 16 6" />
                    <polyline points="8 6 2 12 8 18" />
                  </svg>
                </button>

                <!-- Single Post Button -->
                <button
                  v-if="v.status !== 'POSTED_TO_ERP'"
                  class="btn-mini-post"
                  :disabled="v.status === 'PENDING_POST'"
                  title="Đẩy chứng từ này sang ERP"
                  @click.stop="handleSinglePost(v)"
                >
                  Đẩy ERP
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.voucher-table-pane {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #0f172a;
  overflow: hidden;
}

.table-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 18px;
  background: #1e293b;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  flex-shrink: 0;
  gap: 12px;
}

.toolbar-left, .toolbar-right {
  display: flex;
  align-items: center;
  gap: 10px;
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
  width: 260px;
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

.filter-select, .erp-select {
  background: #0f172a;
  color: #e2e8f0;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 6px;
  padding: 6px 10px;
  font-size: 12px;
  outline: none;
  cursor: pointer;
}

.erp-target-selector {
  display: flex;
  align-items: center;
  gap: 8px;
}

.target-lbl {
  font-size: 12px;
  color: #94a3b8;
}

.btn-batch-post {
  display: flex;
  align-items: center;
  gap: 6px;
  background: #10b981;
  color: #ffffff;
  border: none;
  border-radius: 6px;
  padding: 6px 14px;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.15s ease-in-out;
}

.btn-batch-post:not(:disabled):hover {
  background: #059669;
}

.btn-batch-post:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.table-container {
  flex: 1;
  overflow-y: auto;
}

.voucher-table {
  width: 100%;
  border-collapse: collapse;
  text-align: left;
  font-size: 13px;
}

.voucher-table th {
  background: #182234;
  color: #94a3b8;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  padding: 10px 14px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  position: sticky;
  top: 0;
  z-index: 10;
}

.voucher-row {
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  cursor: pointer;
  transition: background 0.1s ease;
}

.voucher-row:hover {
  background: rgba(255, 255, 255, 0.03);
}

.voucher-row.selected {
  background: rgba(56, 189, 248, 0.08);
}

.voucher-table td {
  padding: 12px 14px;
  vertical-align: top;
}

.td-voucher {
  min-width: 150px;
}

.voucher-num-box {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.v-num {
  font-family: 'JetBrains Mono', monospace;
  font-weight: 700;
  color: #38bdf8;
  font-size: 13px;
}

.v-date {
  font-size: 11px;
  color: #64748b;
}

.v-guid-tag {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: #64748b;
  margin-top: 4px;
}

.td-partner {
  min-width: 220px;
}

.partner-title {
  font-weight: 600;
  color: #f1f5f9;
}

.partner-tax {
  font-size: 11px;
  color: #64748b;
  font-family: 'JetBrains Mono', monospace;
}

.desc-text {
  font-size: 12px;
  color: #94a3b8;
  margin-top: 2px;
  line-height: 1.3;
}

.td-accounts {
  min-width: 280px;
}

.accounts-lines {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.acc-line-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.posting-pill {
  font-size: 10px;
  font-weight: 800;
  padding: 1px 5px;
  border-radius: 3px;
}

.posting-pill.debit {
  background: rgba(16, 185, 129, 0.2);
  color: #34d399;
}

.posting-pill.credit {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
}

.acc-code {
  font-family: 'JetBrains Mono', monospace;
  font-weight: 700;
  color: #f8fafc;
}

.acc-amt {
  font-family: 'JetBrains Mono', monospace;
  font-weight: 600;
  color: #cbd5e1;
}

.acc-name {
  font-size: 11px;
  color: #64748b;
}

.total-amt-val {
  font-family: 'JetBrains Mono', monospace;
  font-weight: 700;
  color: #10b981;
  font-size: 13px;
}

.balance-badge {
  display: inline-block;
  font-size: 11px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 4px;
}

.balance-badge.balanced {
  background: rgba(16, 185, 129, 0.15);
  color: #34d399;
  border: 1px solid rgba(16, 185, 129, 0.3);
}

.balance-badge.unbalanced {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.4);
}

.status-wrap {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.status-pill {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 4px;
}

.status-posted {
  background: rgba(16, 185, 129, 0.2);
  color: #34d399;
  border: 1px solid rgba(16, 185, 129, 0.3);
}

.status-pending {
  background: rgba(245, 158, 11, 0.2);
  color: #fbbf24;
}

.status-failed {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.status-draft {
  background: rgba(255, 255, 255, 0.06);
  color: #94a3b8;
}

.erp-ref-text {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: #64748b;
}

.actions-group {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
}

.btn-icon {
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: #94a3b8;
  border-radius: 4px;
  padding: 5px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.btn-icon:hover {
  background: #334155;
  color: #f8fafc;
}

.btn-mini-post {
  background: #3b82f6;
  color: #ffffff;
  border: none;
  border-radius: 4px;
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-mini-post:hover {
  background: #2563eb;
}

.text-right {
  text-align: right;
}

.text-center {
  text-align: center;
}

.empty-cell {
  padding: 40px !important;
  text-align: center;
  color: #64748b;
}

.empty-wrap {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.empty-icon {
  font-size: 28px;
}
</style>

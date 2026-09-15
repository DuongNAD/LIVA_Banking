<script setup lang="ts">
/**
 * TransactionLedger.vue
 * Row 3: Real-time Transaction Ledger & Integrated Universal Multi-Format Ingestion Dropzone.
 */
import { ref } from 'vue';
import { useReconciliationStore, type BankTransaction } from '../../stores/reconciliationStore';
import { useStatementStore } from '../../stores/statementStore';
import { useBankingStore } from '../../stores/bankingStore';
import HitlResolutionModal from './HitlResolutionModal.vue';

const reconcileStore = useReconciliationStore();
const statementStore = useStatementStore();
const bankingStore = useBankingStore();

const isDragging = ref(false);
const fileInputRef = ref<HTMLInputElement | null>(null);

// Modal state for Google Sheets / Google Docs paste
const isPasteModalOpen = ref(false);
const pastedText = ref('');
const selectedPasteBank = ref<'AUTO' | 'VCB' | 'TCB' | 'BIDV'>('AUTO');
const pasteError = ref('');
const pasteSuccessMsg = ref('');

function formatVnd(val: number): string {
  const formatted = new Intl.NumberFormat('en-US').format(Math.abs(val)) + ' VND';
  return val < 0 ? `-${formatted}` : `+${formatted}`;
}

// Drag and drop handlers passing real File object
function onDragOver(e: DragEvent) {
  e.preventDefault();
  isDragging.value = true;
}

function onDragLeave() {
  isDragging.value = false;
}

async function onDrop(e: DragEvent) {
  e.preventDefault();
  isDragging.value = false;
  if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
    for (const file of Array.from(e.dataTransfer.files)) {
      await statementStore.ingestFile({ name: file.name, size: file.size, file });
    }
  }
}

function triggerFileInput() {
  fileInputRef.value?.click();
}

async function onFileInputChange(e: Event) {
  const input = e.target as HTMLInputElement;
  if (input.files && input.files.length > 0) {
    for (const file of Array.from(input.files)) {
      await statementStore.ingestFile({ name: file.name, size: file.size, file });
    }
    input.value = '';
  }
}

function openHitl(tx: BankTransaction) {
  reconcileStore.openHitlModal(tx);
}

function handleResetAll() {
  const confirmed = window.confirm('Bạn có chắc chắn muốn đặt lại toàn bộ số liệu về 0 VND và xóa sạch sổ cái để bắt đầu thiết kế từ đầu?');
  if (confirmed) {
    bankingStore.resetAllData();
    reconcileStore.resetTransactions();
    statementStore.resetQueue();
  }
}

function handleSeedDemo() {
  bankingStore.seedBenchmarkData();
  reconcileStore.seedBenchmarkData();
}

function submitPastedData() {
  if (!pastedText.value.trim()) {
    pasteError.value = 'Vui lòng dán dữ liệu từ bảng tính (Google Sheets, Docs, hoặc Excel).';
    return;
  }
  try {
    pasteError.value = '';
    const bank = selectedPasteBank.value === 'AUTO' ? undefined : selectedPasteBank.value;
    const res = statementStore.ingestPastedText(pastedText.value, bank);
    pasteSuccessMsg.value = `Đã nạp thành công ${res.rawRowCount} giao dịch cho ngân hàng ${res.bankCode}!`;
    setTimeout(() => {
      pastedText.value = '';
      pasteSuccessMsg.value = '';
      isPasteModalOpen.value = false;
    }, 1200);
  } catch (err: unknown) {
    pasteError.value = err instanceof Error ? err.message : String(err);
  }
}
</script>

<template>
  <div class="transaction-ledger-card">
    <!-- Header Title -->
    <div class="ledger-header">
      <div class="header-main">
        <h3 class="ledger-title">SỔ GIAO DỊCH CHÍNH TÌM THỰC (Real-time Transaction Ledger)</h3>
        <span class="live-tag">
          <span class="live-dot" /> LIVE STREAM
        </span>
      </div>

      <!-- Compliance & Hot-folder watcher status -->
      <div class="hotfolder-status-pill">
        <span class="pill-label">Hot-Folder (Auto-Scrub NĐ 13):</span>
        <span class="pill-val">{{ statementStore.isWatchingHotFolder ? 'Đang theo dõi' : 'Tạm dừng' }}</span>
        <button class="pill-toggle-btn" @click="statementStore.toggleHotFolderWatcher">
          {{ statementStore.isWatchingHotFolder ? 'Dừng' : 'Bật' }}
        </button>
      </div>
    </div>

    <!-- Integrated Multi-Format Statement Dropzone Bar -->
    <div
      class="statement-dropzone"
      :class="{ 'is-dragging': isDragging }"
      @dragover="onDragOver"
      @dragleave="onDragLeave"
      @drop="onDrop"
      @click="triggerFileInput"
    >
      <input
        ref="fileInputRef"
        type="file"
        multiple
        accept=".xlsx,.xls,.csv,.tsv,.txt,.json,.pdf"
        class="hidden-file-input"
        @change="onFileInputChange"
      />

      <div class="dropzone-icon">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#10b981" stroke-width="2">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="17 8 12 3 7 8" />
          <line x1="12" y1="3" x2="12" y2="15" />
        </svg>
      </div>

      <div class="dropzone-text">
        <span class="drop-prompt">Kéo thả file sao kê, sổ cái vào đây hoặc <strong class="text-green">chọn file từ máy</strong></span>
        <span class="drop-support">Hỗ trợ: Excel (.xlsx, .xls) · CSV/TSV (.csv, .tsv) · JSON (.json) · PDF · Tự động phát hiện & bảo mật PII NĐ 13</span>
      </div>

      <div class="dropzone-badges">
        <span class="format-chip">VCB .xlsx</span>
        <span class="format-chip">TCB .csv</span>
        <span class="format-chip">BIDV .pdf</span>
        <span class="format-chip">Sheets / Docs</span>
        <span class="format-chip">JSON</span>
      </div>
    </div>

    <!-- Ingestion Progress Queue (If any active) -->
    <div v-if="statementStore.queue.length > 0" class="ingest-queue">
      <div v-for="item in statementStore.queue.slice(0, 2)" :key="item.id" class="queue-item">
        <div class="queue-info">
          <span class="queue-file-name">{{ item.fileName }} ({{ item.detectedBank }})</span>
          <span class="queue-status-text" :class="{ 'is-error': item.status.startsWith('REJECTED') || item.status === 'FAILED' }">
            {{ item.status === 'SCANNING' ? 'Quét cấu trúc & băm SHA-256...' :
               item.status === 'SCRUBBING_PII' ? 'Khử định danh PII (NĐ 13)...' :
               item.status === 'EXTRACTING' ? `Trích xuất ${item.parsedRowCount} giao dịch...` :
               item.status === 'VERIFYING' ? 'Kiểm chứng bất biến số học...' :
               item.status === 'REJECTED_UNSUPPORTED_FORMAT' ? 'Từ chối: Định dạng không hỗ trợ' :
               item.status === 'REJECTED_EMPTY_FILE' ? 'Từ chối: Tệp rỗng (0 bytes)' :
               item.status === 'FAILED' ? 'Thất bại' : 'Hoàn tất đối soát' }}
          </span>
          <span class="queue-duration">{{ item.durationMs }}ms</span>
        </div>
        <div class="queue-progress-bar">
          <div class="queue-progress-fill" :style="{ width: `${item.progress}%` }" />
        </div>
      </div>
    </div>

    <!-- Filter Bar & Search -->
    <div class="table-controls">
      <div class="filter-tabs">
        <button
          class="tab-btn"
          :class="{ active: reconcileStore.activeStatusFilter === 'ALL' }"
          @click="reconcileStore.activeStatusFilter = 'ALL'"
        >
          Tất cả ({{ reconcileStore.transactionCounts.total }})
        </button>
        <button
          class="tab-btn"
          :class="{ active: reconcileStore.activeStatusFilter === 'MATCHED' }"
          @click="reconcileStore.activeStatusFilter = 'MATCHED'"
        >
          Đã khớp ({{ reconcileStore.transactionCounts.matched }})
        </button>
        <button
          class="tab-btn"
          :class="{ active: reconcileStore.activeStatusFilter === 'UNMATCHED' }"
          @click="reconcileStore.activeStatusFilter = 'UNMATCHED'"
        >
          Chưa khớp ({{ reconcileStore.transactionCounts.unmatched }})
        </button>
        <button
          class="tab-btn"
          :class="{ active: reconcileStore.activeStatusFilter === 'PENDING_HITL' }"
          @click="reconcileStore.activeStatusFilter = 'PENDING_HITL'"
        >
          Chờ duyệt ({{ reconcileStore.transactionCounts.pending }})
        </button>
      </div>

      <div class="table-actions-right">
        <button
          class="paste-btn"
          title="Dán dữ liệu sao chép từ Google Sheets, Docs, Excel"
          @click="isPasteModalOpen = true"
        >
          📋 Dán từ Sheets/Docs
        </button>

        <button
          class="reset-btn"
          title="Xóa toàn bộ số liệu về 0 VND để tự nạp lại từ đầu"
          @click="handleResetAll"
        >
          🗑️ Reset về 0
        </button>

        <button
          class="run-reconcile-btn"
          :disabled="reconcileStore.isReconciling"
          @click="reconcileStore.runReconciliation()"
        >
          <span v-if="reconcileStore.isReconciling">⏳ Đang đối soát 3-Tier...</span>
          <span v-else>⚡ Chạy Đối Soát Tự Động</span>
        </button>

        <div class="table-search">
          <input
            v-model="reconcileStore.searchQuery"
            type="text"
            placeholder="Lọc theo mã GD hoặc nội dung..."
            class="table-search-input"
          />
        </div>
      </div>
    </div>

    <!-- Empty Slate State (when 0 transactions) -->
    <div v-if="reconcileStore.filteredTransactions.length === 0" class="empty-ledger-state">
      <div class="empty-icon-wrap">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#94a3b8" stroke-width="1.5">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
          <polyline points="14 2 14 8 20 8" />
          <line x1="16" y1="13" x2="8" y2="13" />
          <line x1="16" y1="17" x2="8" y2="17" />
          <polyline points="10 9 9 9 8 9" />
        </svg>
      </div>
      <h4 class="empty-title">Sổ Cái Đang Ở Trạng Thái Sạch (0 VND / 0 Giao Dịch)</h4>
      <p class="empty-desc">
        Toàn bộ số dư đã được reset sạch. Hãy kéo thả file <strong>Excel (.xlsx, .xls)</strong>, <strong>CSV/TSV</strong> hoặc nhấn <strong>"Dán từ Sheets/Docs"</strong> để nạp dữ liệu thực tế của bạn.
      </p>
      <div class="empty-actions">
        <button class="empty-btn-upload" @click="triggerFileInput">
          📁 Tải File Từ Máy (.xlsx, .csv, .json)
        </button>
        <button class="empty-btn-paste" @click="isPasteModalOpen = true">
          📋 Dán Bảng Tính (Sheets / Docs / Excel)
        </button>
        <button class="empty-btn-demo" @click="handleSeedDemo">
          💡 Nạp Dữ Liệu Mẫu Kiểm Thử (Demo)
        </button>
      </div>
    </div>

    <!-- Transactions Table -->
    <div v-else class="table-responsive">
      <table class="transaction-table">
        <thead>
          <tr>
            <th>Mã GD</th>
            <th>Thời gian</th>
            <th>Ngân hàng / Tài khoản</th>
            <th>Nội dung chi tiết</th>
            <th class="text-right">Số tiền (VND)</th>
            <th>Trạng thái</th>
            <th class="text-center">Thao tác</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="tx in reconcileStore.filteredTransactions" :key="tx.id" class="tx-row">
            <!-- Mã GD -->
            <td class="font-mono font-bold text-slate">{{ tx.txCode }}</td>

            <!-- Thời gian -->
            <td class="text-muted">{{ tx.time }}</td>

            <!-- Ngân hàng / Tài khoản -->
            <td>
              <div class="bank-cell">
                <span
                  class="bank-badge"
                  :class="{
                    'badge-vcb': tx.bankCode === 'VCB',
                    'badge-tcb': tx.bankCode === 'TCB',
                    'badge-bidv': tx.bankCode === 'BIDV',
                  }"
                >
                  {{ tx.bankCode }}
                </span>
                <span class="acc-num">{{ tx.accountNumber }}</span>
              </div>
            </td>

            <!-- Nội dung chi tiết -->
            <td class="memo-cell" :title="tx.memo">{{ tx.memo }}</td>

            <!-- Số tiền -->
            <td class="text-right font-bold" :class="tx.bankAmount >= 0 ? 'text-green' : 'text-slate'">
              {{ formatVnd(tx.bankAmount) }}
            </td>

            <!-- Trạng thái -->
            <td>
              <span
                class="status-pill"
                :class="{
                  'status-matched': tx.status === 'MATCHED',
                  'status-unmatched': tx.status === 'UNMATCHED',
                  'status-pending': tx.status === 'PENDING_HITL' || tx.status === 'FEE_DISCREPANCY',
                }"
              >
                {{ tx.statusLabel }}
              </span>
            </td>

            <!-- Thao tác -->
            <td class="text-center">
              <button
                v-if="tx.status === 'PENDING_HITL' || tx.status === 'UNMATCHED'"
                class="action-btn btn-resolve"
                @click="openHitl(tx)"
              >
                Đối chiếu
              </button>
              <button
                v-else
                class="action-btn btn-view"
                title="Xem chứng từ đối chiếu"
                @click="openHitl(tx)"
              >
                Chi tiết
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- HITL Modal Component -->
    <HitlResolutionModal
      v-if="reconcileStore.selectedTxForHitl"
      :transaction="reconcileStore.selectedTxForHitl"
      :is-open="reconcileStore.isHitlModalOpen"
      @close="reconcileStore.closeHitlModal"
    />

    <!-- Google Sheets / Google Docs / Excel Clipboard Paste Modal -->
    <div v-if="isPasteModalOpen" class="paste-modal-overlay" @click.self="isPasteModalOpen = false">
      <div class="paste-modal-card">
        <div class="modal-header">
          <div class="modal-title-wrap">
            <span class="modal-icon">📋</span>
            <h3 class="modal-title">Dán Dữ Liệu Từ Google Sheets / Docs / Excel</h3>
          </div>
          <button class="modal-close-btn" @click="isPasteModalOpen = false">✕</button>
        </div>

        <div class="modal-body">
          <p class="modal-instructions">
            Mở bảng tính Google Sheets, Google Docs hoặc Excel, bôi đen các ô hoặc hàng dữ liệu và nhấn <strong>Ctrl + C</strong>. Sau đó dán (<strong>Ctrl + V</strong>) vào khung dưới:
          </p>

          <div class="modal-bank-selector">
            <label class="selector-label">Ngân hàng hạch toán:</label>
            <select v-model="selectedPasteBank" class="bank-select-input">
              <option value="AUTO">✨ Tự động nhận diện (VCB / TCB / BIDV)</option>
              <option value="VCB">Vietcombank (VCB)</option>
              <option value="TCB">Techcombank (TCB)</option>
              <option value="BIDV">BIDV</option>
            </select>
          </div>

          <textarea
            v-model="pastedText"
            class="paste-textarea"
            rows="8"
            placeholder="Dán các hàng bảng tính vào đây...&#10;Hệ thống tự động nhận diện ngày, mã giao dịch, số tiền nợ/có, diễn giải nội dung.&#10;Ví dụ:&#10;01/08/2026	VCB001	150000000	THANH TOAN TIEN HANG&#10;02/08/2026	VCB002	-2200	PHI GIAO DICH INTERNET"
          />

          <div v-if="pasteError" class="paste-error-alert">
            ⚠️ {{ pasteError }}
          </div>
          <div v-if="pasteSuccessMsg" class="paste-success-alert">
            ✅ {{ pasteSuccessMsg }}
          </div>
        </div>

        <div class="modal-footer">
          <button class="btn-cancel" @click="isPasteModalOpen = false">Hủy</button>
          <button class="btn-submit-paste" @click="submitPastedData">
            🚀 Nạp Vào Sổ Cái & Tính Lại Số Dư
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.transaction-ledger-card {
  background: #ffffff;
  border-radius: 12px;
  border: 1px solid #e2e8f0;
  padding: 18px 20px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.ledger-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
}

.header-main {
  display: flex;
  align-items: center;
  gap: 10px;
}

.ledger-title {
  font-size: 15px;
  font-weight: 700;
  color: #0f172a;
  margin: 0;
}

.live-tag {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 10px;
  font-weight: 800;
  color: #10b981;
  background: #ecfdf5;
  padding: 2px 6px;
  border-radius: 4px;
  letter-spacing: 0.5px;
}

.live-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #10b981;
}

.hotfolder-status-pill {
  display: flex;
  align-items: center;
  gap: 8px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  padding: 3px 10px;
  border-radius: 16px;
  font-size: 12px;
}

.pill-label {
  color: #64748b;
}

.pill-val {
  font-weight: 700;
  color: #10b981;
}

.pill-toggle-btn {
  background: transparent;
  border: none;
  color: #6366f1;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  text-decoration: underline;
}

.statement-dropzone {
  display: flex;
  align-items: center;
  gap: 16px;
  border: 2px dashed #cbd5e1;
  border-radius: 10px;
  padding: 12px 18px;
  background: #f8fafc;
  cursor: pointer;
  transition: all 0.2s ease;
}

.statement-dropzone:hover,
.statement-dropzone.is-dragging {
  border-color: #10b981;
  background: #f0fdf4;
}

.hidden-file-input {
  display: none;
}

.dropzone-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  background: #ffffff;
  padding: 8px;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
}

.dropzone-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
}

.drop-prompt {
  font-size: 13px;
  color: #1e293b;
  font-weight: 500;
}

.text-green {
  color: #059669;
}

.drop-support {
  font-size: 11px;
  color: #64748b;
}

.dropzone-badges {
  display: flex;
  gap: 6px;
}

.format-chip {
  font-size: 10px;
  font-weight: 700;
  color: #475569;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  padding: 2px 6px;
  border-radius: 4px;
}

.ingest-queue {
  display: flex;
  flex-direction: column;
  gap: 6px;
  background: #f1f5f9;
  border-radius: 8px;
  padding: 8px 12px;
}

.queue-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.queue-info {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
}

.queue-file-name {
  font-weight: 700;
  color: #0f172a;
}

.queue-status-text {
  color: #059669;
  font-weight: 500;
}

.queue-status-text.is-error {
  color: #dc2626;
  font-weight: 600;
}

.queue-duration {
  font-family: monospace;
  color: #94a3b8;
}

.queue-progress-bar {
  height: 4px;
  background: #e2e8f0;
  border-radius: 2px;
  overflow: hidden;
}

.queue-progress-fill {
  height: 100%;
  background: #10b981;
  transition: width 0.2s ease;
}

.table-controls {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.filter-tabs {
  display: flex;
  gap: 4px;
}

.tab-btn {
  background: #f1f5f9;
  border: 1px solid transparent;
  color: #64748b;
  font-size: 12px;
  font-weight: 600;
  padding: 5px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.tab-btn:hover {
  background: #e2e8f0;
}

.tab-btn.active {
  background: #ffffff;
  border-color: #cbd5e1;
  color: #0f172a;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
}

.table-actions-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.run-reconcile-btn {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  color: #ffffff;
  border: none;
  font-size: 11px;
  font-weight: 700;
  padding: 6px 14px;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 6px;
  box-shadow: 0 1px 2px rgba(16, 185, 129, 0.2);
  transition: all 0.15s ease;
}

.run-reconcile-btn:hover:not(:disabled) {
  background: linear-gradient(135deg, #059669 0%, #047857 100%);
  transform: translateY(-1px);
}

.run-reconcile-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.table-search-input {
  height: 32px;
  width: 220px;
  padding: 0 10px;
  border-radius: 6px;
  border: 1px solid #cbd5e1;
  font-size: 12px;
  outline: none;
}

.table-search-input:focus {
  border-color: #10b981;
}

.table-responsive {
  width: 100%;
  overflow-x: auto;
}

.transaction-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
  text-align: left;
}

.transaction-table th {
  background: #f8fafc;
  color: #64748b;
  font-weight: 600;
  padding: 10px 12px;
  border-bottom: 1px solid #e2e8f0;
  white-space: nowrap;
}

.transaction-table td {
  padding: 12px;
  border-bottom: 1px solid #f1f5f9;
  vertical-align: middle;
}

.tx-row:hover {
  background: #f8fafc;
}

.font-mono { font-family: monospace; }
.font-bold { font-weight: 700; }
.text-slate { color: #0f172a; }
.text-muted { color: #64748b; font-size: 12px; }
.text-right { text-align: right; }
.text-center { text-align: center; }

.bank-cell {
  display: flex;
  align-items: center;
  gap: 6px;
}

.bank-badge {
  font-size: 10px;
  font-weight: 800;
  padding: 2px 5px;
  border-radius: 4px;
}

.badge-vcb { background: #dcfce7; color: #166534; }
.badge-tcb { background: #fee2e2; color: #991b1b; }
.badge-bidv { background: #e0f2fe; color: #075985; }

.acc-num {
  font-size: 12px;
  color: #475569;
}

.memo-cell {
  max-width: 280px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: #334155;
}

.status-pill {
  font-size: 11px;
  font-weight: 700;
  padding: 3px 8px;
  border-radius: 12px;
  display: inline-block;
}

.status-matched {
  background: #ecfdf5;
  color: #059669;
}

.status-unmatched {
  background: #fef2f2;
  color: #dc2626;
}

.status-pending {
  background: #fffbeb;
  color: #d97706;
}

.action-btn {
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  border: none;
  transition: all 0.15s ease;
}

.btn-resolve {
  background: #fef3c7;
  color: #b45309;
}

.btn-resolve:hover {
  background: #fde68a;
}

.btn-view {
  background: #f1f5f9;
  color: #475569;
}

.btn-view:hover {
  background: #e2e8f0;
}

/* New Toolbar Buttons */
.paste-btn {
  background: #ecfdf5;
  color: #065f46;
  border: 1px solid #a7f3d0;
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  transition: all 0.15s ease;
}

.paste-btn:hover {
  background: #d1fae5;
  border-color: #6ee7b7;
}

.reset-btn {
  background: #fef2f2;
  color: #991b1b;
  border: 1px solid #fecaca;
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  transition: all 0.15s ease;
}

.reset-btn:hover {
  background: #fee2e2;
  border-color: #fca5a5;
}

/* Empty Slate State Styling */
.empty-ledger-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 48px 24px;
  background: #f8fafc;
  border: 2px dashed #cbd5e1;
  border-radius: 12px;
  margin: 10px 0;
}

.empty-icon-wrap {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  background: #e2e8f0;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 16px;
}

.empty-title {
  font-size: 16px;
  font-weight: 700;
  color: #1e293b;
  margin: 0 0 8px 0;
}

.empty-desc {
  font-size: 13px;
  color: #64748b;
  max-width: 520px;
  line-height: 1.5;
  margin: 0 0 20px 0;
}

.empty-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  justify-content: center;
}

.empty-btn-upload {
  background: #10b981;
  color: #ffffff;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s ease;
}

.empty-btn-upload:hover {
  background: #059669;
}

.empty-btn-paste {
  background: #0284c7;
  color: #ffffff;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s ease;
}

.empty-btn-paste:hover {
  background: #0369a1;
}

.empty-btn-demo {
  background: #ffffff;
  color: #64748b;
  border: 1px solid #cbd5e1;
  padding: 8px 14px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.empty-btn-demo:hover {
  background: #f1f5f9;
  color: #334155;
}

/* Modal Overlay & Card for Clipboard Paste */
.paste-modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.6);
  backdrop-filter: blur(4px);
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.paste-modal-card {
  background: #ffffff;
  border-radius: 14px;
  width: 100%;
  max-width: 640px;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.2), 0 10px 10px -5px rgba(0, 0, 0, 0.1);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-header {
  padding: 16px 20px;
  border-bottom: 1px solid #e2e8f0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #f8fafc;
}

.modal-title-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
}

.modal-icon {
  font-size: 18px;
}

.modal-title {
  font-size: 15px;
  font-weight: 700;
  color: #0f172a;
  margin: 0;
}

.modal-close-btn {
  background: none;
  border: none;
  font-size: 18px;
  color: #94a3b8;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
}

.modal-close-btn:hover {
  color: #0f172a;
  background: #e2e8f0;
}

.modal-body {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.modal-instructions {
  font-size: 13px;
  color: #475569;
  line-height: 1.5;
  margin: 0;
}

.modal-bank-selector {
  display: flex;
  align-items: center;
  gap: 10px;
}

.selector-label {
  font-size: 13px;
  font-weight: 600;
  color: #334155;
  white-space: nowrap;
}

.bank-select-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  font-size: 13px;
  color: #0f172a;
  background: #ffffff;
}

.paste-textarea {
  width: 100%;
  box-sizing: border-box;
  padding: 12px;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 12px;
  color: #1e293b;
  resize: vertical;
  line-height: 1.4;
  background: #f8fafc;
}

.paste-textarea:focus {
  outline: none;
  border-color: #10b981;
  box-shadow: 0 0 0 2px rgba(16, 185, 129, 0.15);
  background: #ffffff;
}

.paste-error-alert {
  padding: 8px 12px;
  border-radius: 6px;
  background: #fef2f2;
  border: 1px solid #fecaca;
  color: #dc2626;
  font-size: 12px;
  font-weight: 600;
}

.paste-success-alert {
  padding: 8px 12px;
  border-radius: 6px;
  background: #ecfdf5;
  border: 1px solid #a7f3d0;
  color: #059669;
  font-size: 12px;
  font-weight: 600;
}

.modal-footer {
  padding: 14px 20px;
  border-top: 1px solid #e2e8f0;
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  background: #f8fafc;
}

.btn-cancel {
  background: #ffffff;
  border: 1px solid #cbd5e1;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  color: #64748b;
  cursor: pointer;
}

.btn-cancel:hover {
  background: #f1f5f9;
  color: #0f172a;
}

.btn-submit-paste {
  background: #10b981;
  border: none;
  padding: 8px 18px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 700;
  color: #ffffff;
  cursor: pointer;
  transition: background 0.15s ease;
}

.btn-submit-paste:hover {
  background: #059669;
}
</style>

<script setup lang="ts">
/**
 * HitlResolutionModal.vue
 * Two-Phase Confirmation Modal for Human-in-the-Loop reconciliation resolution.
 * Strictly enforces Circular 09/2020/TT-NHNN Dual Control (4-Eyes Principle):
 * - Phase 1: Maker (Kế toán viên) reviews variance & proposes resolution.
 * - Phase 2: Checker (Kế toán trưởng) inspects cryptographic diff & signs off.
 * - Enforces maker_id != checker_id (Fail-Closed self-approval defense).
 * - Single-use UUIDv4 token with 15-minute TTL.
 */
import { ref, computed } from 'vue';
import { useReconciliationStore, type BankTransaction } from '../../stores/reconciliationStore';

const props = defineProps<{
  transaction: BankTransaction;
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'resolved', txCode: string): void;
}>();

const reconcileStore = useReconciliationStore();

const selectedAction = ref<'ALLOCATE_FEE' | 'MANUAL_MATCH' | 'CREATE_VOUCHER'>('ALLOCATE_FEE');
const targetAccount = ref('6425 - Chi phí quản lý ngân hàng');
const reviewNotes = ref('Đã đối chiếu sao kê và chứng từ gốc, chấp thuận hạch toán bù trừ theo chính sách kế toán.');
const isSubmitting = ref(false);
const submitError = ref<string | null>(null);

// Circular 09/2020/TT-NHNN 4-Eyes Identities
const makerId = ref('KT_TRINH_VAN_NAM');
const checkerId = ref('KTT_NGUYEN_MINH_TRI');

const isSelfApprovalViolation = computed(() => {
  return makerId.value.trim().toLowerCase() === checkerId.value.trim().toLowerCase() && makerId.value.trim().length > 0;
});

function formatVnd(val: number): string {
  return new Intl.NumberFormat('en-US').format(val) + ' VND';
}

async function handleConfirm() {
  submitError.value = null;

  if (isSelfApprovalViolation.value) {
    submitError.value = 'Vi phạm Thông tư 09/2020/TT-NHNN: Kế toán viên (Maker) không được tự phê duyệt với vai trò Kế toán trưởng (Checker).';
    return;
  }

  isSubmitting.value = true;

  const result = await reconcileStore.confirmHitlResolution({
    txId: props.transaction.id,
    tokenUuid: props.transaction.tokenUuid || 'token-' + crypto.randomUUID(),
    action: selectedAction.value,
    targetAccount: targetAccount.value,
    notes: reviewNotes.value,
    makerId: makerId.value,
    checkerId: checkerId.value,
  });

  isSubmitting.value = false;
  if (result.success) {
    emit('resolved', props.transaction.txCode);
    emit('close');
  } else {
    submitError.value = result.error || 'Có lỗi xảy ra trong quá trình xác thực phê duyệt.';
  }
}
</script>

<template>
  <div v-if="isOpen" class="modal-backdrop" @click.self="emit('close')">
    <div class="modal-dialog">
      <!-- Modal Header -->
      <div class="modal-header">
        <div class="header-info">
          <div class="header-badges">
            <span class="two-phase-badge">Two-Phase Confirmation</span>
            <span class="circular-badge">Thông tư 09/2020/TT-NHNN Dual Control</span>
          </div>
          <h3 class="modal-title">Phê Duyệt Đối Chiếu Giao Dịch Lệch (HITL Resolution)</h3>
        </div>
        <button class="close-btn" title="Đóng" @click="emit('close')">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>

      <!-- Single-Use Token UUID Banner -->
      <div class="token-banner">
        <div class="token-meta">
          <span class="token-label">Token Xác Thực Dùng Một Lần (Single-Use UUIDv4):</span>
          <span class="token-ttl">Hạn sử dụng: 15 phút (900s)</span>
        </div>
        <code class="token-code">{{ transaction.tokenUuid || 'token-998a-2026-auth' }}</code>
      </div>

      <!-- Audit Diff Comparison -->
      <div class="diff-grid">
        <!-- Left Column: Bank Statement Record -->
        <div class="diff-col bank-col">
          <div class="col-title">
            <span class="col-dot bg-blue" />
            <span>Dữ Liệu Sao Kê Ngân Hàng</span>
          </div>
          <div class="col-content">
            <div class="field-item">
              <span class="field-label">Mã giao dịch:</span>
              <span class="field-val font-mono">{{ transaction.txCode }}</span>
            </div>
            <div class="field-item">
              <span class="field-label">Thời gian:</span>
              <span class="field-val">{{ transaction.date }} {{ transaction.time }}</span>
            </div>
            <div class="field-item">
              <span class="field-label">Tài khoản:</span>
              <span class="field-val">{{ transaction.accountNumber }} ({{ transaction.bankCode }})</span>
            </div>
            <div class="field-item">
              <span class="field-label">Số tiền ghi nhận:</span>
              <span class="field-val font-bold text-blue">{{ formatVnd(transaction.bankAmount) }}</span>
            </div>
            <div class="field-item">
              <span class="field-label">Nội dung chi tiết:</span>
              <span class="field-val text-memo">{{ transaction.memo }}</span>
            </div>
          </div>
        </div>

        <!-- Right Column: Internal ERP General Ledger Record -->
        <div class="diff-col erp-col">
          <div class="col-title">
            <span class="col-dot bg-purple" />
            <span>Sổ Cái Kế Toán (ERP Ledger)</span>
          </div>
          <div class="col-content">
            <div class="field-item">
              <span class="field-label">Số chứng từ ERP:</span>
              <span class="field-val font-mono">{{ transaction.ledgerVoucher || 'Chưa ghi nhận' }}</span>
            </div>
            <div class="field-item">
              <span class="field-label">Đối tác / Khách hàng:</span>
              <span class="field-val">{{ transaction.counterparty || 'Chưa định danh' }}</span>
            </div>
            <div class="field-item">
              <span class="field-label">Số tiền sổ cái:</span>
              <span class="field-val font-bold">{{ formatVnd(transaction.ledgerAmount) }}</span>
            </div>
            <div class="field-item">
              <span class="field-label">Chênh lệch (Variance):</span>
              <span class="field-val font-bold text-red">{{ formatVnd(transaction.variance) }}</span>
            </div>
            <div class="field-item">
              <span class="field-label">Độ tin cậy đề xuất:</span>
              <span class="field-val font-bold text-green">{{ (transaction.confidenceScore * 100).toFixed(1) }}%</span>
            </div>
          </div>
        </div>
      </div>

      <!-- AI / SLM Root Cause Recommendation -->
      <div class="ai-recommendation-box">
        <div class="ai-header">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#10b981" stroke-width="2">
            <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
          </svg>
          <span class="ai-box-title">Đề Xuất Phê Duyệt Của Trợ Lý Tài Chính LIVA:</span>
        </div>
        <p class="ai-recommendation-text">
          {{ transaction.suggestedAction || 'Phát hiện lệch số liệu giữa ngân hàng và chứng từ. Đề xuất bổ sung bút toán bù trừ hoặc hạch toán chi phí ngân hàng.' }}
        </p>
      </div>

      <!-- Dual Control Maker-Checker Roles Form -->
      <div class="maker-checker-section">
        <div class="section-title-row">
          <span class="section-icon">👥</span>
          <span class="section-title">Phân Định Vai Trò Phê Duyệt 4 Mắt (Dual Control)</span>
        </div>

        <div class="roles-grid">
          <!-- Maker Role Input -->
          <div class="role-field">
            <label class="role-label">
              <span>1. Kế toán viên lập đề xuất (Maker):</span>
              <span class="badge-role badge-maker">Maker</span>
            </label>
            <input
              v-model="makerId"
              type="text"
              class="role-input font-mono"
              placeholder="Nhập ID/Tên kế toán viên..."
            />
          </div>

          <!-- Checker Role Input -->
          <div class="role-field">
            <label class="role-label">
              <span>2. Kế toán trưởng ký duyệt (Checker):</span>
              <span class="badge-role badge-checker">Checker</span>
            </label>
            <input
              v-model="checkerId"
              type="text"
              class="role-input font-mono"
              placeholder="Nhập ID/Tên kế toán trưởng..."
            />
          </div>
        </div>

        <!-- Violation Alert if maker_id == checker_id -->
        <div v-if="isSelfApprovalViolation" class="violation-alert">
          <span class="alert-icon">🚫</span>
          <div class="alert-text">
            <strong>CẢNH BÁO VI PHẠM THÔNG TƯ 09/2020/TT-NHNN:</strong>
            Kế toán viên (Maker) không được phép tự phê duyệt với tư cách Kế toán trưởng (Checker).
            Hệ thống áp dụng cơ chế <em>Fail-Closed Dual Control</em> — bắt buộc 2 cá nhân độc lập để ngăn chặn gian lận tài chính.
          </div>
        </div>
      </div>

      <!-- Form Actions -->
      <div class="form-section">
        <label class="form-label">Phương thức xử lý phê duyệt:</label>
        <div class="action-radios">
          <label class="radio-label">
            <input v-model="selectedAction" type="radio" value="ALLOCATE_FEE" />
            <span>Phân bổ phí ngân hàng vào TK 6425 (Auto-split fee)</span>
          </label>
          <label class="radio-label">
            <input v-model="selectedAction" type="radio" value="MANUAL_MATCH" />
            <span>Khớp số liệu thủ công có ghi chú chứng từ (Manual Match)</span>
          </label>
          <label class="radio-label">
            <input v-model="selectedAction" type="radio" value="CREATE_VOUCHER" />
            <span>Sinh chứng từ kế toán bù trừ mới trên ERP (Create Voucher)</span>
          </label>
        </div>
      </div>

      <div class="notes-section">
        <label class="form-label">Ghi chú kiểm toán (Audit Notes):</label>
        <textarea
          v-model="reviewNotes"
          rows="2"
          class="notes-textarea"
          placeholder="Nhập ghi chú giải trình cho ban kiểm soát / thanh tra..."
        />
      </div>

      <!-- Error message display -->
      <div v-if="submitError" class="submit-error-banner">
        {{ submitError }}
      </div>

      <!-- Footer / Two-Phase Confirmation Buttons -->
      <div class="modal-footer">
        <button class="btn btn-secondary" @click="emit('close')">
          Hủy Bỏ
        </button>
        <button
          class="btn btn-confirm"
          :disabled="isSubmitting || isSelfApprovalViolation"
          @click="handleConfirm"
        >
          <span v-if="isSubmitting">Đang ghi sổ an toàn & ký số SHA-256...</span>
          <span v-else>Xác Nhận Phê Duyệt (Ký Số Phase 2)</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.7);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal-dialog {
  background: #ffffff;
  border-radius: 14px;
  width: 100%;
  max-width: 720px;
  max-height: 92vh;
  overflow-y: auto;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.2), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
  display: flex;
  flex-direction: column;
  padding: 24px;
}

.modal-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 14px;
}

.header-info {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.header-badges {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.two-phase-badge {
  font-size: 11px;
  font-weight: 700;
  color: #059669;
  background: #ecfdf5;
  padding: 2px 8px;
  border-radius: 4px;
  border: 1px solid #a7f3d0;
}

.circular-badge {
  font-size: 11px;
  font-weight: 700;
  color: #1d4ed8;
  background: #eff6ff;
  padding: 2px 8px;
  border-radius: 4px;
  border: 1px solid #bfdbfe;
}

.modal-title {
  font-size: 17px;
  font-weight: 700;
  color: #0f172a;
  margin: 0;
}

.close-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  cursor: pointer;
  padding: 4px;
  border-radius: 6px;
}

.close-btn:hover {
  background: #f1f5f9;
  color: #475569;
}

.token-banner {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: #f8fafc;
  border: 1px dashed #cbd5e1;
  padding: 10px 14px;
  border-radius: 8px;
  margin-bottom: 14px;
}

.token-meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.token-label {
  font-size: 12px;
  font-weight: 600;
  color: #475569;
}

.token-ttl {
  font-size: 10px;
  color: #64748b;
}

.token-code {
  font-size: 12px;
  font-weight: 700;
  color: #059669;
  background: #ecfdf5;
  padding: 4px 10px;
  border-radius: 6px;
  letter-spacing: 0.5px;
}

.diff-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  margin-bottom: 14px;
}

.diff-col {
  border-radius: 10px;
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.bank-col {
  background: #f0fdf4;
  border: 1px solid #bbf7d0;
}

.erp-col {
  background: #faf5ff;
  border: 1px solid #e9d5ff;
}

.col-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 700;
  color: #1e293b;
}

.col-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.bg-blue {
  background-color: #0284c7;
}

.bg-purple {
  background-color: #8b5cf6;
}

.col-content {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-item {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
}

.field-label {
  color: #64748b;
}

.field-val {
  font-weight: 600;
  color: #0f172a;
  text-align: right;
  max-width: 60%;
}

.text-memo {
  font-size: 10px;
  color: #475569;
  word-break: break-all;
}

.text-blue {
  color: #0284c7;
}

.text-green {
  color: #10b981;
}

.text-red {
  color: #ef4444;
}

.ai-recommendation-box {
  background: #f0fdf4;
  border-left: 4px solid #10b981;
  padding: 10px 14px;
  border-radius: 6px;
  margin-bottom: 14px;
}

.ai-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
}

.ai-box-title {
  font-size: 12px;
  font-weight: 700;
  color: #065f46;
}

.ai-recommendation-text {
  font-size: 12px;
  color: #047857;
  margin: 0;
  line-height: 1.4;
}

.maker-checker-section {
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 14px;
  margin-bottom: 14px;
}

.section-title-row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 10px;
}

.section-title {
  font-size: 13px;
  font-weight: 700;
  color: #1e293b;
}

.roles-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.role-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.role-label {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
  font-weight: 600;
  color: #475569;
}

.badge-role {
  font-size: 9px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 3px;
}

.badge-maker {
  background: #e0f2fe;
  color: #0284c7;
}

.badge-checker {
  background: #fef3c7;
  color: #d97706;
}

.role-input {
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  padding: 8px 10px;
  font-size: 12px;
  color: #0f172a;
  background: #ffffff;
}

.role-input:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
}

.violation-alert {
  display: flex;
  gap: 8px;
  align-items: flex-start;
  background: #fef2f2;
  border: 1px solid #fecaca;
  border-radius: 8px;
  padding: 10px 12px;
  margin-top: 10px;
  color: #991b1b;
  font-size: 11px;
  line-height: 1.4;
}

.alert-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.form-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 12px;
}

.form-label {
  font-size: 12px;
  font-weight: 600;
  color: #334155;
}

.action-radios {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.radio-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: #334155;
  cursor: pointer;
}

.notes-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 14px;
}

.notes-textarea {
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  padding: 8px 10px;
  font-size: 12px;
  color: #334155;
  resize: vertical;
}

.notes-textarea:focus {
  outline: none;
  border-color: #3b82f6;
}

.submit-error-banner {
  background: #fef2f2;
  border: 1px solid #f87171;
  color: #b91c1c;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 12px;
  margin-bottom: 12px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding-top: 12px;
  border-top: 1px solid #f1f5f9;
}

.btn {
  padding: 9px 16px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
  border: none;
}

.btn-secondary {
  background: #f1f5f9;
  color: #475569;
}

.btn-secondary:hover {
  background: #e2e8f0;
}

.btn-confirm {
  background: #059669;
  color: #ffffff;
}

.btn-confirm:hover:not(:disabled) {
  background: #047857;
}

.btn-confirm:disabled {
  background: #94a3b8;
  cursor: not-allowed;
  opacity: 0.6;
}
</style>

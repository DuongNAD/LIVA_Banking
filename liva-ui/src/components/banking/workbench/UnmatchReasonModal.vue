<script setup lang="ts">
import { ref, computed } from 'vue';
import { useWorkbenchStore, type UnmatchReasonCode } from '../../../stores/workbenchStore';

const store = useWorkbenchStore();

const match = computed(() => store.unmatchTarget);

const selectedReasonCode = ref<UnmatchReasonCode>('SAI_DOI_TAC');
const reasonNote = ref('');
const errorMessage = ref('');

function formatVnd(val: number): string {
  const formatted = new Intl.NumberFormat('vi-VN').format(Math.abs(val)) + ' ₫';
  return val < 0 ? `-${formatted}` : `+${formatted}`;
}

function handleConfirm() {
  if (!match.value) return;

  if (reasonNote.value.trim().length < 10) {
    errorMessage.value = 'Vui lòng nhập giải trình chi tiết tối thiểu 10 ký tự để lưu vết kiểm toán.';
    return;
  }

  store.unmatchWithReason(match.value.id, selectedReasonCode.value, reasonNote.value.trim());
  closeModal();
}

function closeModal() {
  store.unmatchTarget = null;
  selectedReasonCode.value = 'SAI_DOI_TAC';
  reasonNote.value = '';
  errorMessage.value = '';
}
</script>

<template>
  <div v-if="match" class="modal-backdrop" @click.self="closeModal">
    <div class="modal-dialog unmatch-modal">
      <!-- Header -->
      <div class="modal-header">
        <div class="header-left">
          <span class="modal-badge alert-badge">AUDIT PROTOCOL</span>
          <h3 class="modal-title">Xác Nhận Hủy Đối Soát (Unmatch)</h3>
        </div>
        <button class="btn-close" @click="closeModal">✕</button>
      </div>

      <!-- Match Target Snapshot -->
      <div class="target-box">
        <div class="snapshot-row">
          <div>
            <span class="target-title">CẶP ĐỐI SOÁT ĐƯỢC CHỌN:</span>
            <span class="match-id">{{ match.id }}</span>
            <span class="tier-pill">{{ match.tierLabel }}</span>
          </div>
          <div class="match-amounts">
            Bank: {{ formatVnd(match.netBankAmount) }} ⇄ GL: {{ formatVnd(match.netGlAmount) }}
          </div>
        </div>
      </div>

      <!-- Form Body -->
      <div class="modal-body">
        <div class="audit-warning-banner">
          <span class="warn-icon">🛡️</span>
          <p>
            Theo quy định <strong>Thông tư 09/2020/TT-NHNN</strong> và tiêu chuẩn kiểm toán bất biến,
            thao tác hủy đối soát sẽ lập tức tạo một mắt xích <strong>SHA-256 Hash-Chain</strong> gắn với tài khoản của bạn
            (<code>{{ store.makerUserId }}</code>).
          </p>
        </div>

        <div class="form-group">
          <label class="form-label">Mã Lý Do Hủy (Reason Code) <span class="req">*</span></label>
          <select v-model="selectedReasonCode" class="form-select">
            <option value="SAI_DOI_TAC">SAI_DOI_TAC — Nhầm lẫn tên hoặc tài khoản đối tác</option>
            <option value="LECH_THOI_GIAN">LECH_THOI_GIAN — Lệch kỳ kế toán hoặc ngày chứng từ</option>
            <option value="LECH_PHI">LECH_PHI — Chênh lệch phí chuyển khoản chưa tách</option>
            <option value="HOA_DON_HUY">HOA_DON_HUY — Hóa đơn/Lệnh chi đã bị thu hồi hoặc hủy</option>
            <option value="TRUNG_LAP">TRUNG_LAP — Trùng lặp giao dịch phát sinh</option>
            <option value="KHAC">KHAC — Lý do nghiệp vụ khác (bắt buộc ghi rõ bên dưới)</option>
          </select>
        </div>

        <div class="form-group">
          <label class="form-label">
            Giải Trình Chi Tiết (Justification Memo) <span class="req">* (tối thiểu 10 ký tự)</span>
          </label>
          <textarea
            v-model="reasonNote"
            rows="3"
            placeholder="Ví dụ: Đối tác thanh toán nhầm cho hợp đồng số HD-9901 của chi nhánh miền Nam..."
            class="form-textarea"
          />
          <div v-if="errorMessage" class="error-text">
            ⚠️ {{ errorMessage }}
          </div>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="modal-footer">
        <button class="btn-cancel" @click="closeModal">Đóng (Giữ nguyên)</button>
        <button class="btn-confirm-unmatch" @click="handleConfirm">
          Ký Xác Nhận & Hủy Ghép
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.65);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.modal-dialog {
  background: #ffffff;
  border-radius: 12px;
  width: 90%;
  max-width: 580px;
  overflow: hidden;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.2);
}

.modal-header {
  padding: 16px 20px;
  background: #f8fafc;
  border-bottom: 1px solid #e2e8f0;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.alert-badge {
  font-size: 11px;
  font-weight: 700;
  padding: 2px 8px;
  background: #fee2e2;
  color: #b91c1c;
  border-radius: 4px;
}

.modal-title {
  font-size: 15px;
  font-weight: 700;
  color: #0f172a;
  margin: 0;
}

.btn-close {
  background: none;
  border: none;
  font-size: 16px;
  color: #94a3b8;
  cursor: pointer;
}

.target-box {
  padding: 12px 20px;
  background: #fff1f2;
  border-bottom: 1px solid #fecdd3;
}

.snapshot-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
}

.target-title {
  font-weight: 700;
  color: #9f1239;
  margin-right: 6px;
}

.match-id {
  font-family: monospace;
  color: #334155;
  margin-right: 6px;
}

.tier-pill {
  font-size: 10px;
  background: #ffffff;
  padding: 1px 6px;
  border-radius: 3px;
  color: #881337;
  font-weight: 600;
}

.match-amounts {
  font-family: monospace;
  font-weight: 700;
  color: #0f172a;
}

.modal-body {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.audit-warning-banner {
  display: flex;
  gap: 10px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-left: 3px solid #f59e0b;
  padding: 10px 12px;
  border-radius: 6px;
  font-size: 12px;
  color: #475569;
  line-height: 1.4;
}

.audit-warning-banner p {
  margin: 0;
}

.warn-icon {
  font-size: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-label {
  font-size: 12px;
  font-weight: 600;
  color: #334155;
}

.req {
  color: #dc2626;
}

.form-select,
.form-textarea {
  width: 100%;
  padding: 8px 10px;
  font-size: 13px;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  outline: none;
  background: #ffffff;
  color: #0f172a;
}

.form-select:focus,
.form-textarea:focus {
  border-color: #e11d48;
  box-shadow: 0 0 0 2px rgba(225, 29, 72, 0.15);
}

.error-text {
  font-size: 11px;
  color: #dc2626;
  margin-top: 2px;
}

.modal-footer {
  padding: 14px 20px;
  background: #f8fafc;
  border-top: 1px solid #e2e8f0;
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.btn-cancel {
  padding: 7px 14px;
  font-size: 12px;
  font-weight: 600;
  border: 1px solid #cbd5e1;
  background: #ffffff;
  border-radius: 6px;
  cursor: pointer;
}

.btn-confirm-unmatch {
  padding: 7px 16px;
  font-size: 12px;
  font-weight: 700;
  background: #e11d48;
  color: #ffffff;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
}

.btn-confirm-unmatch:hover {
  background: #be123c;
}
</style>

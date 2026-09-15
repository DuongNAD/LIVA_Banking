<script setup lang="ts">
/**
 * StrReportModal.vue — P82 Mẫu Báo Cáo Giao Dịch Đáng Ngờ (Form STR)
 * ==================================================================
 * Phụ lục II Thông tư số 09/2023/TT-NHNN gửi Cục Phòng, chống rửa tiền.
 */
import { ref } from 'vue';
import { useComplianceStore } from '../../../stores/complianceStore';

defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const store = useComplianceStore();
const officerId = ref('CBT-TUANTHU-HO-01');
const officerNotes = ref(
  'Đã đối chiếu dữ liệu sao kê, phát hiện dấu hiệu bất thường nghiêm trọng. Đề xuất phong tỏa tài khoản và chuyển hồ sơ sang Cục PCRT — NHNN.'
);

function formatVnd(val: number): string {
  return `${val.toLocaleString('vi-VN')} ₫`;
}

function handleSignAndSubmit() {
  if (!store.activeStrModal) return;
  store.signAndSubmitStr(store.activeStrModal.strId, officerId.value, officerNotes.value);
}
</script>

<template>
  <div v-if="isOpen && store.activeStrModal" class="modal-overlay" @click.self="emit('close')">
    <div class="modal-container">
      <div class="modal-header">
        <div class="header-branding">
          <span class="str-badge">STR</span>
          <div>
            <h3 class="modal-title">Báo Cáo Giao Dịch Đáng Ngờ (Form STR)</h3>
            <p class="modal-sub">Phụ lục II Thông tư số 09/2023/TT-NHNN — Cục Phòng, chống rửa tiền</p>
          </div>
        </div>
        <button class="btn-close" @click="emit('close')">✕</button>
      </div>

      <div class="modal-body">
        <!-- Form Header Summary -->
        <div class="summary-grid">
          <div class="s-item">
            <span class="s-lbl">Mã Báo Cáo:</span>
            <strong class="font-mono">{{ store.activeStrModal.strId }}</strong>
          </div>
          <div class="s-item">
            <span class="s-lbl">Ngày Lập:</span>
            <strong class="font-mono">{{ store.activeStrModal.reportDate }}</strong>
          </div>
          <div class="s-item">
            <span class="s-lbl">Dấu Hiệu AML:</span>
            <strong class="text-red">{{ store.activeStrModal.alertType }}</strong>
          </div>
          <div class="s-item">
            <span class="s-lbl">Mức Độ Rủi Ro:</span>
            <strong class="text-red">{{ store.activeStrModal.severity }}</strong>
          </div>
        </div>

        <!-- Section I: Reporting Entity -->
        <div class="form-section">
          <h4 class="sec-title">Phần I: Đơn Vị Báo Cáo</h4>
          <div class="sec-content">
            <strong>{{ store.activeStrModal.reportingEntity }}</strong>
          </div>
        </div>

        <!-- Section II: Suspect Info -->
        <div class="form-section">
          <h4 class="sec-title">Phần II: Đối Tượng Bị Báo Cáo</h4>
          <div class="two-col-box">
            <div>
              <span class="lbl">Họ và tên / Đơn vị đối tác:</span>
              <strong class="val">{{ store.activeStrModal.suspectName }}</strong>
            </div>
            <div>
              <span class="lbl">Số tài khoản giao dịch:</span>
              <strong class="val font-mono">{{ store.activeStrModal.suspectAccount }}</strong>
            </div>
          </div>
        </div>

        <!-- Section III: Transaction Details -->
        <div class="form-section">
          <h4 class="sec-title">Phần III: Chi Tiết Giao Dịch Đáng Ngờ</h4>
          <div class="three-col-box">
            <div>
              <span class="lbl">Số lượng giao dịch:</span>
              <strong class="val font-mono">{{ store.activeStrModal.transactionCount }}</strong>
            </div>
            <div>
              <span class="lbl">Tổng số tiền:</span>
              <strong class="val font-mono text-red">{{ formatVnd(store.activeStrModal.totalVndAmount) }}</strong>
            </div>
            <div>
              <span class="lbl">Loại tiền:</span>
              <strong class="val font-mono">VND</strong>
            </div>
          </div>
        </div>

        <!-- Section IV: Suspicion Grounds & AI Narrative -->
        <div class="form-section">
          <h4 class="sec-title">Phần IV: Căn Cứ Pháp Lý & Diễn Giải Nghi Ngờ (Explainable AI)</h4>
          <div class="narrative-box">
            <div class="statutory-rule font-mono">
              Căn cứ: {{ store.activeStrModal.statutoryRuleRef }}
            </div>
            <p class="narrative-text">
              {{ store.activeStrModal.narrativeSummary }}
            </p>
          </div>
        </div>

        <!-- Section V: Compliance Officer Sign-off -->
        <div class="form-section">
          <h4 class="sec-title">Phần V: Ý Kiến & Ký Duyệt Của Cán Bộ Tuân Thủ</h4>
          <div class="officer-box">
            <div class="officer-input-row">
              <label>Mã cán bộ ký duyệt:</label>
              <input v-model="officerId" class="officer-input font-mono" />
            </div>
            <textarea
              v-model="officerNotes"
              rows="3"
              class="notes-textarea"
              placeholder="Nhập ý kiến và đề xuất biện pháp xử lý..."
            />

            <!-- Merkle Proof Chain Footer -->
            <div class="merkle-proof-row font-mono">
              <span class="m-lbl">Mã băm Merkle bảo chứng bất biến:</span>
              <span class="m-val">{{ store.activeStrModal.merkleProofHash }}</span>
            </div>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <div v-if="store.activeStrModal.signedAt" class="signed-banner">
          ✅ Đã được ký số bởi <strong>{{ store.activeStrModal.officerId }}</strong> lúc {{ store.activeStrModal.signedAt }}
        </div>
        <div class="footer-actions">
          <button class="btn btn-secondary" @click="emit('close')">Đóng</button>
          <button
            v-if="!store.activeStrModal.signedAt"
            class="btn btn-primary"
            @click="handleSignAndSubmit"
          >
            ✍️ Ký Duyệt & Phát Hành Báo Cáo STR
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.65);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1200;
  padding: 20px;
}

.modal-container {
  background: #ffffff;
  border-radius: 12px;
  width: 100%;
  max-width: 820px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.2);
  overflow: hidden;
}

.modal-header {
  padding: 16px 20px;
  background: #0f172a;
  color: #ffffff;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-branding {
  display: flex;
  align-items: center;
  gap: 12px;
}

.str-badge {
  background: #ef4444;
  color: #ffffff;
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  font-weight: 800;
  padding: 4px 8px;
  border-radius: 6px;
}

.modal-title {
  font-size: 15px;
  font-weight: 700;
  margin: 0;
}

.modal-sub {
  font-size: 11px;
  color: #94a3b8;
  margin: 2px 0 0 0;
}

.btn-close {
  background: transparent;
  border: none;
  color: #94a3b8;
  font-size: 16px;
  cursor: pointer;
}

.modal-body {
  padding: 20px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.summary-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 12px 14px;
}

.s-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.s-lbl {
  font-size: 10px;
  color: #64748b;
  text-transform: uppercase;
}

.form-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.sec-title {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  color: #475569;
  letter-spacing: 0.5px;
  margin: 0;
}

.two-col-box,
.three-col-box {
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 12px 14px;
  display: grid;
  gap: 12px;
}

.two-col-box {
  grid-template-columns: repeat(2, 1fr);
}

.three-col-box {
  grid-template-columns: repeat(3, 1fr);
}

.lbl {
  font-size: 11px;
  color: #64748b;
  display: block;
}

.val {
  font-size: 13px;
  color: #0f172a;
}

.narrative-box {
  background: #fffafa;
  border: 1px solid #fecaca;
  border-radius: 8px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.statutory-rule {
  font-size: 11px;
  font-weight: 700;
  color: #b91c1c;
}

.narrative-text {
  font-size: 12px;
  color: #7f1d1d;
  line-height: 1.5;
  margin: 0;
}

.officer-box {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.officer-input-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: #334155;
}

.officer-input {
  padding: 4px 8px;
  border: 1px solid #cbd5e1;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 700;
}

.notes-textarea {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  font-size: 12px;
  box-sizing: border-box;
}

.merkle-proof-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 10px;
  background: #0f172a;
  color: #34d399;
  padding: 6px 10px;
  border-radius: 4px;
  overflow: hidden;
}

.m-lbl {
  color: #94a3b8;
  white-space: nowrap;
}

.m-val {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.modal-footer {
  padding: 14px 20px;
  border-top: 1px solid #e2e8f0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: #f8fafc;
}

.signed-banner {
  font-size: 12px;
  color: #15803d;
  font-weight: 600;
}

.footer-actions {
  display: flex;
  gap: 10px;
  margin-left: auto;
}

.btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.btn-secondary {
  background: #ffffff;
  border: 1px solid #cbd5e1;
  color: #475569;
}

.btn-primary {
  background: #0f172a;
  color: #ffffff;
  border: none;
}

.btn-primary:hover {
  background: #334155;
}

.font-mono {
  font-family: 'JetBrains Mono', monospace;
}

.text-red {
  color: #dc2626;
}
</style>

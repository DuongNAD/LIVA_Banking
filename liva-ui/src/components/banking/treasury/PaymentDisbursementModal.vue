<script setup lang="ts">
/**
 * PaymentDisbursementModal.vue — P62 Maker-Checker Dual Control Payment Orders
 * ==============================================================================
 * Enforces Circular 09/2020/TT-NHNN 4-Eyes principle:
 * maker_user_id != checker_user_id on corporate fund disbursement orders.
 */
import { ref, computed } from 'vue';
import { useTreasuryStore } from '../../../stores/treasuryStore';
import type { PaymentOrder, PaymentPriority } from '../../../stores/treasuryStore';

const props = defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const store = useTreasuryStore();

// Form states for new payment order
const isCreatingNew = ref(false);
const formSourceAccount = ref(store.accounts[0]?.id || '');
const formBeneficiaryName = ref('');
const formBeneficiaryBank = ref('VCB');
const formBeneficiaryAccount = ref('');
const formAmount = ref<number>(50000000);
const formMemo = ref('');
const formPriority = ref<PaymentPriority>('NORMAL');
const formNote = ref('');

// Checker decision state
const checkerDecisionNote = ref('');
const actionError = ref('');
const actionSuccess = ref('');

const selectedOrder = computed(() => store.activePaymentOrder);

function formatVnd(val: number): string {
  return `${val.toLocaleString('vi-VN')} ₫`;
}

function handleCreateOrder() {
  actionError.value = '';
  actionSuccess.value = '';

  try {
    const created = store.createPaymentOrder({
      sourceAccountId: formSourceAccount.value,
      beneficiaryName: formBeneficiaryName.value,
      beneficiaryBank: formBeneficiaryBank.value,
      beneficiaryAccount: formBeneficiaryAccount.value,
      amount: formAmount.value,
      memo: formMemo.value,
      priority: formPriority.value,
      makerNote: formNote.value,
    });
    actionSuccess.value = `Đã lập lệnh chi ${created.orderNumber} và chuyển Kế toán trưởng duyệt!`;
    isCreatingNew.value = false;
    store.selectPaymentOrder(created.id);
  } catch (err: any) {
    actionError.value = err.message;
  }
}

function handleApprove(order: PaymentOrder) {
  actionError.value = '';
  actionSuccess.value = '';

  try {
    store.approvePaymentOrder(order.id, checkerDecisionNote.value || 'Phê duyệt chi theo hồ sơ thanh toán hợp lệ.');
    actionSuccess.value = `Lệnh chi ${order.orderNumber} đã được phê duyệt thành công!`;
    checkerDecisionNote.value = '';
  } catch (err: any) {
    actionError.value = err.message;
  }
}

function handleReject(order: PaymentOrder) {
  actionError.value = '';
  actionSuccess.value = '';

  try {
    store.rejectPaymentOrder(order.id, checkerDecisionNote.value || 'Từ chối duyệt lệnh chi.');
    actionSuccess.value = `Đã từ chối lệnh chi ${order.orderNumber}.`;
    checkerDecisionNote.value = '';
  } catch (err: any) {
    actionError.value = err.message;
  }
}

function handleExecute(order: PaymentOrder) {
  actionError.value = '';
  actionSuccess.value = '';

  try {
    store.executePaymentDisbursement(order.id);
    actionSuccess.value = `Đã giải ngân thành công lệnh chi ${order.orderNumber}! Mã FT: ${order.bankRefNumber}`;
  } catch (err: any) {
    actionError.value = err.message;
  }
}
</script>

<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="emit('close')">
    <div class="payment-modal-box">
      <!-- Modal Header -->
      <div class="modal-header">
        <div class="header-left">
          <span class="modal-tag">P62 DUAL CONTROL</span>
          <h2 class="modal-title">Quản Lý Lệnh Chi Tiền & Phê Duyệt 4 Mắt</h2>
        </div>

        <div class="header-right">
          <!-- Role Toggle for Testing -->
          <div class="role-toggle-group">
            <span class="role-lbl">Vai trò:</span>
            <button
              class="role-pill-btn maker"
              :class="{ active: store.currentOperatorRole === 'MAKER' }"
              @click="store.switchOperatorRole('MAKER')"
            >
              R02 Thủ quỹ (Maker)
            </button>
            <button
              class="role-pill-btn checker"
              :class="{ active: store.currentOperatorRole === 'CHECKER' }"
              @click="store.switchOperatorRole('CHECKER')"
            >
              R03/R04 KTT/CFO (Checker)
            </button>
          </div>

          <button class="btn-close" @click="emit('close')">✕</button>
        </div>
      </div>

      <!-- Action Notification -->
      <div v-if="actionError" class="modal-alert error">
        <span>⚠️ {{ actionError }}</span>
      </div>
      <div v-if="actionSuccess" class="modal-alert success">
        <span>✓ {{ actionSuccess }}</span>
      </div>

      <!-- Modal Body (2-column layout) -->
      <div class="modal-body-grid">
        <!-- Left Column: Payment Orders List -->
        <div class="orders-list-col">
          <div class="list-title-row">
            <span class="title-text">Danh Sách Lệnh Chi ({{ store.paymentOrders.length }})</span>
            <button
              v-if="store.currentOperatorRole === 'MAKER'"
              class="btn-new-order"
              @click="isCreatingNew = true; store.selectPaymentOrder(null)"
            >
              + Tạo Lệnh Chi
            </button>
          </div>

          <div class="orders-scroll-pane">
            <div
              v-for="order in store.paymentOrders"
              :key="order.id"
              class="order-card"
              :class="{ selected: store.activePaymentOrderId === order.id }"
              @click="store.selectPaymentOrder(order.id); isCreatingNew = false"
            >
              <div class="order-top">
                <span class="order-num">{{ order.orderNumber }}</span>
                <span class="order-priority" :class="order.priority.toLowerCase()">{{ order.priority }}</span>
              </div>
              <div class="order-beneficiary">{{ order.beneficiaryName }}</div>
              <div class="order-bottom">
                <span class="order-amt">{{ formatVnd(order.amount) }}</span>
                <span class="order-status-pill" :class="order.status.toLowerCase()">
                  {{ order.status }}
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- Right Column: Create Form or Investigation / Approval Details -->
        <div class="order-detail-col">
          <!-- VIEW 1: Form to Create New Order (When Maker clicks + Tạo Lệnh Chi) -->
          <div v-if="isCreatingNew" class="create-form-pane">
            <h3 class="pane-headline">Lập Lệnh Chi Tiền Điện Tử Mới (Maker)</h3>

            <div class="form-grid">
              <div class="form-group">
                <label>Tài khoản trích tiền (Nguồn):</label>
                <select v-model="formSourceAccount" class="form-input">
                  <option v-for="a in store.accounts" :key="a.id" :value="a.id">
                    {{ a.bankName }} ({{ a.accountNumber }}) — Khả dụng: {{ formatVnd(a.balance) }}
                  </option>
                </select>
              </div>

              <div class="form-group">
                <label>Đơn vị thụ hưởng (Tên công ty/Cá nhân):</label>
                <input v-model="formBeneficiaryName" type="text" placeholder="Công ty CP..." class="form-input" />
              </div>

              <div class="form-row-2">
                <div class="form-group">
                  <label>Ngân hàng thụ hưởng:</label>
                  <input v-model="formBeneficiaryBank" type="text" placeholder="VCB / TCB / BIDV" class="form-input" />
                </div>
                <div class="form-group">
                  <label>Số tài khoản thụ hưởng:</label>
                  <input v-model="formBeneficiaryAccount" type="text" placeholder="0123456789" class="form-input" />
                </div>
              </div>

              <div class="form-row-2">
                <div class="form-group">
                  <label>Số tiền thanh toán (VND):</label>
                  <input v-model.number="formAmount" type="number" step="1000000" class="form-input font-mono" />
                </div>
                <div class="form-group">
                  <label>Mức độ ưu tiên:</label>
                  <select v-model="formPriority" class="form-input">
                    <option value="NORMAL">Bình thường (NORMAL)</option>
                    <option value="URGENT">Khẩn cấp (URGENT)</option>
                    <option value="PAYROLL_TAX">Lương & Thuế (PAYROLL_TAX)</option>
                  </select>
                </div>
              </div>

              <div class="form-group">
                <label>Nội dung chuyển tiền (Memo):</label>
                <textarea v-model="formMemo" rows="2" placeholder="Thanh toán tiền hàng theo hợp đồng..." class="form-input" />
              </div>

              <div class="form-group">
                <label>Căn cứ chứng từ đính kèm (Maker Note):</label>
                <input v-model="formNote" type="text" placeholder="Hóa đơn VAT số 00982, biên bản nghiệm thu..." class="form-input" />
              </div>
            </div>

            <div class="form-actions-bar">
              <button class="btn btn-secondary" @click="isCreatingNew = false">Hủy</button>
              <button
                class="btn btn-primary"
                :disabled="!formBeneficiaryName || !formBeneficiaryAccount || formAmount <= 0"
                @click="handleCreateOrder"
              >
                Trình Duyệt KTT (Checker)
              </button>
            </div>
          </div>

          <!-- VIEW 2: Order Details & Approval / Execution Workspace -->
          <div v-else-if="selectedOrder" class="order-review-pane">
            <div class="review-header">
              <div>
                <span class="review-num">{{ selectedOrder.orderNumber }}</span>
                <span class="review-date">Tạo lúc: {{ selectedOrder.submittedAt || 'N/A' }}</span>
              </div>
              <div class="review-amount">{{ formatVnd(selectedOrder.amount) }}</div>
            </div>

            <div class="review-details-grid">
              <div class="detail-item">
                <span class="d-lbl">Đơn vị thụ hưởng:</span>
                <span class="d-val bold">{{ selectedOrder.beneficiaryName }}</span>
              </div>
              <div class="detail-item">
                <span class="d-lbl">Tài khoản & Ngân hàng:</span>
                <span class="d-val font-mono">{{ selectedOrder.beneficiaryAccount }} ({{ selectedOrder.beneficiaryBank }})</span>
              </div>
              <div class="detail-item full-width">
                <span class="d-lbl">Nội dung chuyển tiền:</span>
                <span class="d-val memo-box">{{ selectedOrder.memo }}</span>
              </div>
              <div class="detail-item full-width">
                <span class="d-lbl">Căn cứ Maker đề xuất ({{ selectedOrder.makerId }}):</span>
                <span class="d-val note-box">{{ selectedOrder.makerNote || 'Đã kiểm tra đầy đủ hồ sơ hợp lệ' }}</span>
              </div>
              <div v-if="selectedOrder.checkerId" class="detail-item full-width">
                <span class="d-lbl">Ý kiến Checker phê duyệt ({{ selectedOrder.checkerId }}):</span>
                <span class="d-val note-box text-green">{{ selectedOrder.checkerNote }} (Lúc: {{ selectedOrder.approvedAt }})</span>
              </div>
            </div>

            <!-- Segregation of Duties Check Banner -->
            <div
              v-if="store.currentOperatorRole === 'CHECKER' && selectedOrder.makerId === store.currentOperatorId"
              class="sod-warning-box"
            >
              🚫 Vi phạm nguyên tắc 4 mắt: Bạn là người đã lập lệnh này. Không thể tự phê duyệt!
            </div>

            <!-- Checker Review Box -->
            <div v-if="store.currentOperatorRole === 'CHECKER' && selectedOrder.status === 'SUBMITTED_TO_CHECKER'" class="checker-decision-box">
              <label class="d-lbl">Ý kiến thẩm tra của Kế toán trưởng / CFO:</label>
              <textarea
                v-model="checkerDecisionNote"
                rows="2"
                placeholder="Nhập ý kiến phê duyệt hoặc lý do từ chối..."
                class="form-input"
              />
              <div class="decision-btn-row">
                <button
                  class="btn btn-approve"
                  :disabled="selectedOrder.makerId === store.currentOperatorId"
                  @click="handleApprove(selectedOrder)"
                >
                  ✓ Phê Duyệt Lệnh Chi
                </button>
                <button
                  class="btn btn-reject"
                  :disabled="selectedOrder.makerId === store.currentOperatorId"
                  @click="handleReject(selectedOrder)"
                >
                  ✕ Bác Bỏ
                </button>
              </div>
            </div>

            <!-- Execution Box (When Approved) -->
            <div v-if="selectedOrder.status === 'APPROVED'" class="disbursement-action-box">
              <div class="disburse-info">
                <span>🛡️ Lệnh chi đã được Kế toán trưởng ký duyệt. Sẵn sàng phát lệnh Core Banking.</span>
              </div>
              <button class="btn btn-disburse" @click="handleExecute(selectedOrder)">
                🚀 Thực Hiện Giải Ngân (Disburse)
              </button>
            </div>

            <!-- Executed Stamp -->
            <div v-if="selectedOrder.status === 'EXECUTED'" class="executed-stamp-box">
              <span class="stamp-icon">✓</span>
              <div>
                <strong>ĐÃ GIẢI NGÂN THÀNH CÔNG VÀO CORE BANKING</strong>
                <p>Mã tham chiếu ngân hàng: {{ selectedOrder.bankRefNumber }} lúc {{ selectedOrder.executedAt }}</p>
              </div>
            </div>
          </div>

          <div v-else class="empty-detail-pane">
            <span>👈 Chọn một lệnh chi từ danh sách bên trái hoặc bấm "Tạo Lệnh Chi"</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 999;
  backdrop-filter: blur(4px);
  padding: 20px;
}

.payment-modal-box {
  background: #111827;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 12px;
  width: 100%;
  max-width: 960px;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px;
  background: #1e293b;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.modal-tag {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 800;
  background: rgba(245, 158, 11, 0.2);
  color: #fbbf24;
  padding: 2px 6px;
  border-radius: 4px;
}

.modal-title {
  font-size: 15px;
  font-weight: 700;
  color: #f8fafc;
  margin: 0;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 14px;
}

.role-toggle-group {
  display: flex;
  align-items: center;
  gap: 6px;
  background: #0f172a;
  padding: 2px 6px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.role-lbl {
  font-size: 11px;
  color: #64748b;
}

.role-pill-btn {
  background: transparent;
  border: none;
  font-size: 11px;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 4px;
  cursor: pointer;
  color: #94a3b8;
}

.role-pill-btn.maker.active {
  background: rgba(245, 158, 11, 0.2);
  color: #fbbf24;
}

.role-pill-btn.checker.active {
  background: rgba(16, 185, 129, 0.2);
  color: #34d399;
}

.btn-close {
  background: transparent;
  border: none;
  color: #94a3b8;
  font-size: 16px;
  cursor: pointer;
}

.modal-alert {
  padding: 10px 20px;
  font-size: 12px;
  font-weight: 600;
}

.modal-alert.error {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
}

.modal-alert.success {
  background: rgba(16, 185, 129, 0.2);
  color: #34d399;
}

.modal-body-grid {
  flex: 1;
  display: grid;
  grid-template-columns: 340px 1fr;
  overflow: hidden;
}

.orders-list-col {
  background: #0f172a;
  border-right: 1px solid rgba(255, 255, 255, 0.08);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.list-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.title-text {
  font-size: 13px;
  font-weight: 700;
  color: #cbd5e1;
}

.btn-new-order {
  background: #3b82f6;
  color: #ffffff;
  border: none;
  border-radius: 4px;
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
}

.orders-scroll-pane {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.order-card {
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 12px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 6px;
  transition: all 0.15s ease;
}

.order-card:hover {
  background: #28364b;
}

.order-card.selected {
  border-color: #38bdf8;
  background: #1a2a44;
}

.order-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.order-num {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  font-weight: 700;
  color: #38bdf8;
}

.order-priority {
  font-size: 9px;
  font-weight: 800;
  padding: 1px 6px;
  border-radius: 3px;
}

.order-priority.normal {
  background: rgba(255, 255, 255, 0.06);
  color: #94a3b8;
}

.order-priority.urgent {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
}

.order-priority.payroll_tax {
  background: rgba(168, 85, 247, 0.2);
  color: #c084fc;
}

.order-beneficiary {
  font-size: 12px;
  font-weight: 600;
  color: #f1f5f9;
}

.order-bottom {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 2px;
}

.order-amt {
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  font-weight: 700;
  color: #10b981;
}

.order-status-pill {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 3px;
}

.order-status-pill.submitted_to_checker {
  background: rgba(245, 158, 11, 0.2);
  color: #fbbf24;
}

.order-status-pill.approved {
  background: rgba(16, 185, 129, 0.2);
  color: #34d399;
}

.order-status-pill.executed {
  background: rgba(56, 189, 248, 0.2);
  color: #38bdf8;
}

.order-status-pill.rejected {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
}

.order-detail-col {
  padding: 20px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.create-form-pane, .order-review-pane {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.pane-headline {
  font-size: 14px;
  font-weight: 700;
  color: #f8fafc;
  margin: 0;
}

.form-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.form-group label {
  font-size: 11px;
  color: #94a3b8;
  font-weight: 600;
}

.form-input {
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 6px;
  padding: 8px 12px;
  color: #f8fafc;
  font-size: 13px;
  outline: none;
}

.form-input:focus {
  border-color: #38bdf8;
}

.form-row-2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.form-actions-bar {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 8px;
}

.btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  border: none;
}

.btn-primary {
  background: #3b82f6;
  color: #ffffff;
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.08);
  color: #e2e8f0;
}

.btn-approve {
  background: #10b981;
  color: #ffffff;
}

.btn-reject {
  background: #ef4444;
  color: #ffffff;
}

.btn-disburse {
  background: #0284c7;
  color: #ffffff;
}

.review-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  padding-bottom: 12px;
}

.review-num {
  font-family: 'JetBrains Mono', monospace;
  font-size: 16px;
  font-weight: 800;
  color: #38bdf8;
  display: block;
}

.review-date {
  font-size: 11px;
  color: #64748b;
}

.review-amount {
  font-family: 'JetBrains Mono', monospace;
  font-size: 20px;
  font-weight: 800;
  color: #10b981;
}

.review-details-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.detail-item.full-width {
  grid-column: 1 / -1;
}

.d-lbl {
  font-size: 11px;
  color: #64748b;
}

.d-val {
  font-size: 13px;
  color: #e2e8f0;
}

.d-val.bold {
  font-weight: 700;
}

.memo-box, .note-box {
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  padding: 8px 12px;
}

.sod-warning-box {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid #ef4444;
  border-radius: 6px;
  padding: 10px;
  color: #fca5a5;
  font-size: 12px;
}

.checker-decision-box {
  background: #0f172a;
  border-radius: 8px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.decision-btn-row {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.disbursement-action-box {
  background: rgba(2, 132, 199, 0.1);
  border: 1px solid rgba(2, 132, 199, 0.3);
  border-radius: 8px;
  padding: 14px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.disburse-info {
  font-size: 12px;
  color: #38bdf8;
}

.executed-stamp-box {
  background: rgba(16, 185, 129, 0.1);
  border: 1px solid rgba(16, 185, 129, 0.3);
  border-radius: 8px;
  padding: 14px;
  display: flex;
  align-items: center;
  gap: 12px;
  color: #34d399;
  font-size: 12px;
}

.stamp-icon {
  font-size: 24px;
}

.empty-detail-pane {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #64748b;
  font-size: 13px;
}

.font-mono {
  font-family: 'JetBrains Mono', monospace;
}
</style>

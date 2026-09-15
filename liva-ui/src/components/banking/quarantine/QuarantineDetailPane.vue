<script setup lang="ts">
/**
 * QuarantineDetailPane.vue — P44 Quarantine Investigation & Dual Control
 * =======================================================================
 * Implements:
 * - Circular 09/2020/TT-NHNN 4-Eyes principle (Maker-Checker dual control)
 * - Strict Segregation of Duties (SoD) Invariant: maker_user_id != checker_user_id
 * - 15-minute TTL Countdown Timer ($t \le 900$s)
 * - Merkle Hash-Chain audit block verification
 * - Maker proposal submission & Checker sign-off/rejection
 */
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useWorkbenchStore } from '../../../stores/workbenchStore';

const store = useWorkbenchStore();

// Form states
const proposedAccount = ref('6425');
const makerNote = ref('');
const checkerNote = ref('');
const actionError = ref('');
const actionSuccess = ref('');

// TTL countdown
const remainingSeconds = ref(900);
let timerInterval: ReturnType<typeof setInterval> | null = null;

const activeItem = computed(() => store.activeQuarantineItem);

// Segregation of Duties (SoD) Check
const isSodViolation = computed(() => {
  if (!activeItem.value || !activeItem.value.makerId) return false;
  return (
    store.currentOperatorRole === 'CHECKER' &&
    activeItem.value.makerId === store.currentOperatorId
  );
});

function calculateRemainingTtl() {
  if (!activeItem.value) {
    remainingSeconds.value = 900;
    return;
  }
  const elapsed = Math.floor((Date.now() - activeItem.value.createdAt) / 1000);
  const left = Math.max(0, activeItem.value.ttlSeconds - elapsed);
  remainingSeconds.value = left;
}

watch(
  () => activeItem.value?.id,
  () => {
    actionError.value = '';
    actionSuccess.value = '';
    makerNote.value = '';
    checkerNote.value = '';
    calculateRemainingTtl();
  }
);

onMounted(() => {
  calculateRemainingTtl();
  timerInterval = setInterval(() => {
    calculateRemainingTtl();
  }, 1000);
});

onUnmounted(() => {
  if (timerInterval) clearInterval(timerInterval);
});

const formattedTtl = computed(() => {
  const m = Math.floor(remainingSeconds.value / 60);
  const s = remainingSeconds.value % 60;
  return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
});

function formatVnd(amount: number): string {
  const prefix = amount > 0 ? '+' : '';
  return `${prefix}${amount.toLocaleString('vi-VN')} ₫`;
}

function handleMakerSubmit() {
  actionError.value = '';
  actionSuccess.value = '';
  if (!activeItem.value) return;

  try {
    store.proposeQuarantineResolution(
      activeItem.value.id,
      proposedAccount.value,
      makerNote.value
    );
    actionSuccess.value = 'Đã trình đề xuất xử lý lên Kế toán trưởng duyệt thành công!';
    makerNote.value = '';
  } catch (err: any) {
    actionError.value = err.message || 'Lỗi khi gửi đề xuất';
  }
}

function handleCheckerApprove() {
  actionError.value = '';
  actionSuccess.value = '';
  if (!activeItem.value) return;

  try {
    store.approveQuarantineItem(activeItem.value.id, checkerNote.value);
    actionSuccess.value = 'Đã phê duyệt giải phóng giao dịch thành công vào sổ cái!';
    checkerNote.value = '';
  } catch (err: any) {
    actionError.value = err.message || 'Lỗi khi phê duyệt';
  }
}

function handleCheckerReject() {
  actionError.value = '';
  actionSuccess.value = '';
  if (!activeItem.value) return;

  try {
    store.rejectQuarantineItem(activeItem.value.id, checkerNote.value);
    actionSuccess.value = 'Đã từ chối đề xuất xử lý giao dịch cách ly!';
    checkerNote.value = '';
  } catch (err: any) {
    actionError.value = err.message || 'Lỗi khi từ chối';
  }
}
</script>

<template>
  <div v-if="!activeItem" class="empty-detail">
    <div class="empty-detail-box">
      <span class="shield-icon">🛡️</span>
      <h3>Chưa chọn giao dịch cách ly</h3>
      <p>Chọn một bản ghi ở danh sách bên trái để kiểm tra chi tiết pháp lý và xử lý quy trình 4 mắt.</p>
    </div>
  </div>

  <div v-else class="detail-pane">
    <!-- Top Action / Header Bar -->
    <div class="detail-header">
      <div class="header-left">
        <div class="tx-headline">
          <span class="bank-badge" :class="activeItem.bankCode.toLowerCase()">
            {{ activeItem.bankCode }}
          </span>
          <span class="tx-title">{{ activeItem.txCode }}</span>
          <span class="tx-timestamp">{{ activeItem.date }} {{ activeItem.time }}</span>
        </div>
        <div class="account-line">
          <span class="acc-label">Số tài khoản:</span>
          <span class="acc-val">{{ activeItem.accountNumber }}</span>
        </div>
      </div>

      <div class="header-right">
        <div class="amount-card" :class="{ credit: activeItem.amount > 0, debit: activeItem.amount < 0 }">
          <span class="amount-type">{{ activeItem.amount > 0 ? 'TIỀN VÀO (CR)' : 'TIỀN RA (DR)' }}</span>
          <span class="amount-number">{{ formatVnd(activeItem.amount) }}</span>
        </div>
      </div>
    </div>

    <!-- Alert / Feedback Notification -->
    <div v-if="actionError" class="feedback-banner error-banner">
      <span class="feedback-icon">⚠️</span>
      <span>{{ actionError }}</span>
    </div>

    <div v-if="actionSuccess" class="feedback-banner success-banner">
      <span class="feedback-icon">✓</span>
      <span>{{ actionSuccess }}</span>
    </div>

    <!-- Body Scroll Area -->
    <div class="detail-content">
      <!-- Section 1: Forensic Investigation & Diagnostics -->
      <div class="section-card forensics-card">
        <div class="card-section-title">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <span>Dữ Liệu Điều Tra Pháp Lý & Giao Dịch Gốc</span>
        </div>

        <div class="diagnostic-grid">
          <div class="diag-item">
            <span class="diag-label">Mã Lý Do Cách Ly:</span>
            <span class="diag-val code-val">{{ activeItem.exceptionType }}</span>
          </div>

          <div class="diag-item">
            <span class="diag-label">Mức Độ Rủi Ro:</span>
            <span
              class="severity-tag"
              :class="activeItem.severity.toLowerCase()"
            >
              {{ activeItem.severity }}
            </span>
          </div>

          <div class="diag-item full-width">
            <span class="diag-label">Nguyên Nhân Kích Hoạt Cách Ly:</span>
            <div class="diag-reason-box">
              {{ activeItem.exceptionReason }}
            </div>
          </div>

          <div class="diag-item full-width">
            <span class="diag-label">Nội Dung Thuyết Minh Ngân Hàng (Raw Narration):</span>
            <div class="raw-memo-box">
              {{ activeItem.memo }}
            </div>
          </div>
        </div>

        <!-- Audit Proof Chain -->
        <div class="audit-chain-footer">
          <div class="chain-item">
            <span class="chain-icon">🔗</span>
            <span class="chain-label">Merkle SHA-256 Proof:</span>
            <span class="chain-hash" :title="activeItem.hashProof">{{ activeItem.hashProof || 'Chưa chốt khối' }}</span>
          </div>
          <div class="chain-item">
            <span class="chain-label">Token UUID:</span>
            <span class="token-uuid">{{ activeItem.tokenUuid }}</span>
          </div>
        </div>
      </div>

      <!-- Section 2: Circular 09 Dual Control Hub -->
      <div class="section-card dual-control-card">
        <div class="card-section-title">
          <div class="title-with-timer">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
            </svg>
            <span>Quy Trình Kiểm Soát Kép 4 Mắt (Thông tư 09/2020/TT-NHNN)</span>
          </div>

          <!-- TTL 15-Minute Countdown -->
          <div class="ttl-countdown" :class="{ 'ttl-warning': remainingSeconds < 180 }">
            <span class="ttl-label">Thời hạn Token:</span>
            <span class="ttl-timer">{{ formattedTtl }}</span>
          </div>
        </div>

        <!-- Role Context & SoD Invariant Display -->
        <div class="sod-status-bar">
          <div class="sod-operator-info">
            <span class="op-role-tag" :class="store.currentOperatorRole.toLowerCase()">
              {{ store.currentOperatorRole === 'MAKER' ? 'R01 Kế Toán Viên (Maker)' : 'R04 Kế Toán Trưởng (Checker)' }}
            </span>
            <span class="op-user-id">{{ store.currentOperatorId }}</span>
          </div>

          <div class="sod-rule-badge">
            <span>Maker-Checker Invariant: maker_id ≠ checker_id</span>
          </div>
        </div>

        <!-- SoD Violation Warning Banner -->
        <div v-if="isSodViolation" class="sod-violation-alert">
          <span class="alert-icon">🚫</span>
          <div class="alert-text">
            <strong>CẢNH BÁO VI PHẠM NGUYÊN TẮC 4 MẮT (SoD):</strong>
            <span>Bạn là người đã tạo đề xuất này ({{ activeItem.makerId }}). Theo quy định Thông tư 09, Kế toán trưởng phê duyệt phải là một cá nhân khác để đảm bảo tính độc lập.</span>
          </div>
        </div>

        <!-- Sub-State 1: Review Existing Proposal (If already submitted) -->
        <div v-if="activeItem.makerProposal" class="existing-proposal-box">
          <div class="proposal-header">
            <span class="badge-maker">ĐỀ XUẤT TỪ MAKER ({{ activeItem.makerId }})</span>
            <span class="proposal-time">{{ activeItem.submittedAt || 'Gần đây' }}</span>
          </div>
          <div class="proposal-details">
            <div class="prop-row">
              <span class="prop-lbl">Tài khoản hạch toán đề xuất:</span>
              <span class="prop-acc">TK {{ activeItem.proposedGlAccount }}</span>
            </div>
            <div class="prop-row">
              <span class="prop-lbl">Căn cứ giải trình:</span>
              <p class="prop-note">{{ activeItem.makerProposal }}</p>
            </div>
          </div>
        </div>

        <!-- Sub-State 2: Resolution Decision Summary (If Approved or Rejected) -->
        <div v-if="activeItem.status === 'APPROVED' || activeItem.status === 'REJECTED'" class="decision-summary-box" :class="activeItem.status.toLowerCase()">
          <div class="decision-header">
            <span class="decision-badge">
              {{ activeItem.status === 'APPROVED' ? '✓ ĐÃ PHÊ DUYỆT BỞI CHECKER' : '✕ ĐÃ BÁC BỎ BỞI CHECKER' }}
            </span>
            <span class="decision-time">{{ activeItem.resolvedAt }}</span>
          </div>
          <div class="decision-body">
            <p><strong>Người ký duyệt:</strong> {{ activeItem.checkerId }}</p>
            <p><strong>Ý kiến phê duyệt:</strong> {{ activeItem.checkerDecisionNote }}</p>
          </div>
        </div>

        <!-- Dynamic Action Form based on Role -->

        <!-- CASE A: MAKER WORKSPACE -->
        <div v-if="store.currentOperatorRole === 'MAKER'" class="role-workspace">
          <div v-if="activeItem.status === 'PENDING_MAKER'" class="maker-form">
            <div class="form-title">Lập Đề Xuất Xử Lý Cách Ly</div>

            <div class="form-field">
              <label class="field-label">Tài Khoản Định Khoản Dự Kiến (GL Account):</label>
              <select v-model="proposedAccount" class="field-select">
                <option value="6425">TK 6425 — Chi phí dịch vụ ngân hàng</option>
                <option value="1388">TK 1388 — Phải thu khác (Chờ xác minh khách hàng)</option>
                <option value="3388">TK 3388 — Phải trả khác (Chờ rà soát đối tác nộp nhầm)</option>
                <option value="811">TK 811 — Chi phí khác (Tổn thất/Sai lệch không thể thu hồi)</option>
                <option value="1121">TK 1121 — Tiền gửi ngân hàng (Chuyển tiếp nội bộ)</option>
              </select>
            </div>

            <div class="form-field">
              <label class="field-label">Ghi Chú Giải Trình Chi Tiết (Tối thiểu 10 ký tự):</label>
              <textarea
                v-model="makerNote"
                rows="3"
                placeholder="Nhập căn cứ nghiệp vụ, số hợp đồng bổ sung hoặc chứng từ làm căn cứ đề xuất..."
                class="field-textarea"
              />
            </div>

            <div class="form-actions">
              <button
                class="btn btn-primary"
                :disabled="makerNote.trim().length < 10"
                @click="handleMakerSubmit"
              >
                Trình Duyệt Kế Toán Trưởng (Checker)
              </button>
            </div>
          </div>

          <div v-else class="maker-wait-notice">
            <span class="info-icon">ℹ️</span>
            <span>Hồ sơ này đã ở trạng thái <strong>{{ activeItem.status }}</strong>. Maker không cần thao tác thêm.</span>
          </div>
        </div>

        <!-- CASE B: CHECKER WORKSPACE -->
        <div v-if="store.currentOperatorRole === 'CHECKER'" class="role-workspace">
          <div v-if="activeItem.status === 'SUBMITTED_TO_CHECKER'" class="checker-form">
            <div class="form-title">Thẩm Quyền Ký Duyệt Của Kế Toán Trưởng</div>

            <div class="form-field">
              <label class="field-label">Ý Kiến Phê Duyệt / Lý Do Từ Chối (Tối thiểu 10 ký tự):</label>
              <textarea
                v-model="checkerNote"
                rows="3"
                placeholder="Nhập nhận xét kiểm soát nội bộ, chỉ đạo điều chỉnh hoặc căn cứ phê duyệt..."
                class="field-textarea"
                :disabled="isSodViolation"
              />
            </div>

            <div class="form-actions checker-actions">
              <button
                class="btn btn-approve"
                :disabled="checkerNote.trim().length < 10 || isSodViolation"
                @click="handleCheckerApprove"
              >
                ✓ Phê Duyệt & Giải Phóng (Approve)
              </button>

              <button
                class="btn btn-reject"
                :disabled="checkerNote.trim().length < 10 || isSodViolation"
                @click="handleCheckerReject"
              >
                ✕ Bác Bỏ Đề Xuất (Reject)
              </button>
            </div>
          </div>

          <div v-else class="checker-wait-notice">
            <span class="info-icon">ℹ️</span>
            <span v-if="activeItem.status === 'PENDING_MAKER'">Đang chờ Kế toán viên (Maker) lập phương án giải trình.</span>
            <span v-else>Hồ sơ đã hoàn tất phê duyệt / kết luận (Trạng thái: <strong>{{ activeItem.status }}</strong>).</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.empty-detail {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  background: #0f172a;
  color: #64748b;
  padding: 32px;
}

.empty-detail-box {
  text-align: center;
  max-width: 400px;
}

.shield-icon {
  font-size: 48px;
  display: block;
  margin-bottom: 16px;
}

.empty-detail-box h3 {
  font-size: 18px;
  font-weight: 700;
  color: #e2e8f0;
  margin-bottom: 8px;
}

.empty-detail-box p {
  font-size: 13px;
  line-height: 1.5;
  color: #94a3b8;
}

.detail-pane {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #0f172a;
  overflow: hidden;
}

.detail-header {
  padding: 18px 24px;
  background: #1e293b;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.tx-headline {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 6px;
}

.bank-badge {
  font-size: 11px;
  font-weight: 800;
  padding: 3px 8px;
  border-radius: 4px;
}

.bank-badge.vcb {
  background: rgba(16, 185, 129, 0.2);
  color: #34d399;
}

.bank-badge.tcb {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
}

.bank-badge.bidv {
  background: rgba(59, 130, 246, 0.2);
  color: #60a5fa;
}

.bank-badge.mbb {
  background: rgba(168, 85, 247, 0.2);
  color: #c084fc;
}

.tx-title {
  font-family: 'JetBrains Mono', monospace;
  font-size: 16px;
  font-weight: 700;
  color: #f8fafc;
}

.tx-timestamp {
  font-size: 12px;
  color: #64748b;
}

.account-line {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
}

.acc-label {
  color: #64748b;
}

.acc-val {
  font-family: 'JetBrains Mono', monospace;
  color: #cbd5e1;
  font-weight: 600;
}

.amount-card {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  padding: 8px 16px;
  background: #0f172a;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.amount-type {
  font-size: 10px;
  font-weight: 700;
  color: #64748b;
  letter-spacing: 0.05em;
}

.amount-number {
  font-family: 'JetBrains Mono', monospace;
  font-size: 18px;
  font-weight: 800;
}

.amount-card.credit .amount-number {
  color: #10b981;
}

.amount-card.debit .amount-number {
  color: #f43f5e;
}

.feedback-banner {
  padding: 10px 24px;
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  font-weight: 600;
}

.feedback-banner.error-banner {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
  border-bottom: 1px solid rgba(239, 68, 68, 0.3);
}

.feedback-banner.success-banner {
  background: rgba(16, 185, 129, 0.2);
  color: #34d399;
  border-bottom: 1px solid rgba(16, 185, 129, 0.3);
}

.detail-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.section-card {
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card-section-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 14px;
  font-weight: 700;
  color: #e2e8f0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  padding-bottom: 12px;
}

.title-with-timer {
  display: flex;
  align-items: center;
  gap: 8px;
}

.ttl-countdown {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  background: rgba(56, 189, 248, 0.1);
  border: 1px solid rgba(56, 189, 248, 0.3);
  border-radius: 6px;
  font-size: 12px;
}

.ttl-countdown.ttl-warning {
  background: rgba(239, 68, 68, 0.15);
  border-color: rgba(239, 68, 68, 0.4);
  color: #f87171;
}

.ttl-timer {
  font-family: 'JetBrains Mono', monospace;
  font-weight: 700;
  color: #38bdf8;
}

.ttl-countdown.ttl-warning .ttl-timer {
  color: #f87171;
}

.diagnostic-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}

.diag-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.diag-item.full-width {
  grid-column: 1 / -1;
}

.diag-label {
  font-size: 11px;
  font-weight: 600;
  color: #64748b;
  text-transform: uppercase;
}

.diag-val.code-val {
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  font-weight: 700;
  color: #38bdf8;
}

.severity-tag {
  display: inline-block;
  font-size: 11px;
  font-weight: 800;
  padding: 2px 8px;
  border-radius: 4px;
  width: fit-content;
}

.severity-tag.critical {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.severity-tag.high {
  background: rgba(245, 158, 11, 0.2);
  color: #fbbf24;
  border: 1px solid rgba(245, 158, 11, 0.3);
}

.severity-tag.medium {
  background: rgba(2, 132, 199, 0.2);
  color: #38bdf8;
  border: 1px solid rgba(2, 132, 199, 0.3);
}

.diag-reason-box {
  background: rgba(239, 68, 68, 0.08);
  border-left: 3px solid #ef4444;
  padding: 10px 14px;
  border-radius: 0 6px 6px 0;
  font-size: 13px;
  line-height: 1.5;
  color: #fca5a5;
}

.raw-memo-box {
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  padding: 10px 14px;
  font-size: 13px;
  line-height: 1.5;
  color: #e2e8f0;
  word-break: break-all;
}

.audit-chain-footer {
  display: flex;
  flex-direction: column;
  gap: 8px;
  background: #0f172a;
  border-radius: 6px;
  padding: 10px 14px;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.chain-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
}

.chain-label {
  color: #64748b;
  font-weight: 600;
}

.chain-hash, .token-uuid {
  font-family: 'JetBrains Mono', monospace;
  color: #94a3b8;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sod-status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #0f172a;
  padding: 10px 14px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.06);
}

.sod-operator-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.op-role-tag {
  font-size: 11px;
  font-weight: 700;
  padding: 3px 8px;
  border-radius: 4px;
}

.op-role-tag.maker {
  background: rgba(245, 158, 11, 0.2);
  color: #fbbf24;
}

.op-role-tag.checker {
  background: rgba(16, 185, 129, 0.2);
  color: #34d399;
}

.op-user-id {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  color: #cbd5e1;
}

.sod-rule-badge {
  font-size: 11px;
  color: #64748b;
  font-family: 'JetBrains Mono', monospace;
}

.sod-violation-alert {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid #ef4444;
  border-radius: 6px;
  padding: 12px 14px;
  color: #fca5a5;
  font-size: 12px;
  line-height: 1.5;
}

.existing-proposal-box {
  background: #0f172a;
  border: 1px solid rgba(56, 189, 248, 0.2);
  border-radius: 8px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.proposal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.badge-maker {
  font-size: 11px;
  font-weight: 700;
  color: #38bdf8;
  letter-spacing: 0.05em;
}

.proposal-time {
  font-size: 11px;
  color: #64748b;
}

.proposal-details {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.prop-row {
  display: flex;
  gap: 8px;
  font-size: 12px;
}

.prop-lbl {
  color: #64748b;
}

.prop-acc {
  font-weight: 700;
  color: #34d399;
}

.prop-note {
  color: #e2e8f0;
  line-height: 1.4;
  margin: 0;
}

.decision-summary-box {
  border-radius: 8px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.decision-summary-box.approved {
  background: rgba(16, 185, 129, 0.1);
  border: 1px solid rgba(16, 185, 129, 0.3);
  color: #34d399;
}

.decision-summary-box.rejected {
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #f87171;
}

.decision-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.decision-badge {
  font-size: 12px;
  font-weight: 800;
}

.decision-body {
  font-size: 12px;
  line-height: 1.5;
  color: #e2e8f0;
}

.role-workspace {
  background: #0f172a;
  border-radius: 8px;
  padding: 16px;
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.form-title {
  font-size: 13px;
  font-weight: 700;
  color: #f1f5f9;
  margin-bottom: 12px;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 12px;
}

.field-label {
  font-size: 12px;
  color: #94a3b8;
  font-weight: 600;
}

.field-select, .field-textarea {
  background: #1e293b;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 6px;
  padding: 8px 12px;
  color: #f8fafc;
  font-size: 13px;
  outline: none;
}

.field-select:focus, .field-textarea:focus {
  border-color: #38bdf8;
}

.field-textarea {
  resize: vertical;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
}

.checker-actions {
  gap: 12px;
}

.btn {
  padding: 8px 18px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  border: none;
  transition: all 0.15s ease-in-out;
}

.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-primary {
  background: #3b82f6;
  color: #ffffff;
}

.btn-primary:not(:disabled):hover {
  background: #2563eb;
}

.btn-approve {
  background: #10b981;
  color: #ffffff;
}

.btn-approve:not(:disabled):hover {
  background: #059669;
}

.btn-reject {
  background: #ef4444;
  color: #ffffff;
}

.btn-reject:not(:disabled):hover {
  background: #dc2626;
}

.maker-wait-notice, .checker-wait-notice {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: #94a3b8;
  padding: 12px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 6px;
}
</style>

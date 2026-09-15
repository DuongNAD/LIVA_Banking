<template>
  <div
    v-if="isOpen && voucher"
    class="fixed inset-0 z-50 overflow-y-auto bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4"
  >
    <div
      class="bg-white rounded-2xl shadow-2xl max-w-2xl w-full border border-slate-200 overflow-hidden flex flex-col max-h-[92vh] animate-in fade-in zoom-in-95 duration-200"
    >
      <!-- Modal Header -->
      <div class="px-6 py-4 bg-slate-900 text-white flex items-center justify-between">
        <div class="flex items-center space-x-3">
          <div class="w-9 h-9 rounded-xl bg-blue-600 flex items-center justify-center text-white shadow-md">
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="20" height="12" x="2" y="6" rx="2"/><circle cx="12" cy="12" r="2"/><path d="M6 12h.01M18 12h.01"/></svg>
          </div>
          <div>
            <h3 class="text-base font-bold tracking-tight">Phê Duyệt Lệnh Chi (Maker-Checker Gate)</h3>
            <p class="text-xs text-slate-300">
              Kiểm soát chéo 2 vòng — Điều 16 & 18 Thông tư 09/2020/TT-NHNN
            </p>
          </div>
        </div>
        <button
          type="button"
          class="text-slate-400 hover:text-white transition p-1.5 rounded-lg hover:bg-slate-800"
          @click="onClose"
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18M6 6l12 12"/></svg>
        </button>
      </div>

      <!-- Modal Body -->
      <div class="p-6 overflow-y-auto space-y-5 text-slate-800 text-sm flex-1">
        <!-- Self-Approval Fail-Closed Alert -->
        <div
          v-if="isSelfApproval"
          class="p-3.5 bg-rose-50 border border-rose-200 rounded-xl flex items-start space-x-3 text-rose-800 text-xs"
        >
          <svg class="w-5 h-5 text-rose-600 shrink-0 mt-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="m4.9 4.9 14.2 14.2"/></svg>
          <div>
            <strong class="font-semibold block">Vi Phạm Nguyên Tắc Bốn Mắt (Circular 09/2020)</strong>
            <span>
              Người lập lệnh (Maker: <code class="font-mono bg-rose-100 px-1 rounded">{{ voucher.makerId }}</code>)
              không được phép tự phê duyệt (Checker). Hệ thống Fail-Closed từ chối thực thi.
            </span>
          </div>
        </div>

        <!-- Voucher Summary Cards -->
        <div class="grid grid-cols-2 gap-4 p-4 bg-slate-50 rounded-xl border border-slate-200">
          <div>
            <span class="text-xs text-slate-400 block font-medium">Mã Lệnh Chi (Voucher ID):</span>
            <span class="font-mono font-bold text-slate-900 text-xs">{{ voucher.voucherId }}</span>
          </div>
          <div>
            <span class="text-xs text-slate-400 block font-medium">Trạng Thái Hiện Tại:</span>
            <span
              class="inline-block px-2.5 py-0.5 rounded-full text-xs font-bold"
              :class="getStatusBadgeClass(voucher.status)"
            >
              {{ voucher.status }}
            </span>
          </div>
          <div>
            <span class="text-xs text-slate-400 block font-medium">Người Lập (Maker):</span>
            <span class="font-semibold text-slate-900">{{ voucher.makerId }}</span>
          </div>
          <div>
            <span class="text-xs text-slate-400 block font-medium">Thời Gian Khởi Tạo:</span>
            <span class="text-xs text-slate-600">{{ formatTimestamp(voucher.createdAt) }}</span>
          </div>
        </div>

        <!-- Payment Details -->
        <div class="border border-slate-200 rounded-xl p-4 space-y-3">
          <h4 class="text-xs font-bold uppercase tracking-wider text-slate-500">
            Thông Tin Chuyển Khoản Ngân Hàng
          </h4>
          <div class="grid grid-cols-2 gap-3 text-xs">
            <div>
              <span class="text-slate-400 block">Đơn Vị Thụ Hưởng:</span>
              <span class="font-bold text-slate-900">{{ voucher.beneficiaryName || 'N/A' }}</span>
            </div>
            <div>
              <span class="text-slate-400 block">Tài Khoản Thụ Hưởng:</span>
              <span class="font-mono font-bold text-blue-700">{{ voucher.beneficiaryAccount }}</span>
              <span class="text-slate-500 ml-1">({{ voucher.beneficiaryBank }})</span>
            </div>
            <div class="col-span-2">
              <span class="text-slate-400 block">Nội Dung Thanh Toán (Mục đích):</span>
              <span class="text-slate-800 font-medium">{{ voucher.purpose }}</span>
            </div>
            <div class="col-span-2 pt-2 border-t border-slate-100 flex items-center justify-between">
              <span class="text-xs font-semibold text-slate-600">Số Tiền Thanh Toán:</span>
              <span class="text-lg font-bold text-rose-600">
                {{ formatVnd(voucher.amountVnd) }} VND
              </span>
            </div>
          </div>
        </div>

        <!-- Token Status & Expiry -->
        <div
          v-if="voucher.status === 'PENDING_APPROVAL'"
          class="p-3 bg-blue-50 border border-blue-200 rounded-xl flex items-center justify-between text-xs text-blue-900"
        >
          <div class="flex items-center space-x-2">
            <span class="font-semibold">Token HITL (UUIDv4):</span>
            <span class="font-mono text-blue-700 text-[11px] truncate max-w-[200px]">
              {{ voucher.hitlToken || 'Đang bảo vệ' }}
            </span>
          </div>
          <div class="text-right">
            <span class="text-slate-500">Thời hạn hiệu lực:</span>
            <span class="font-bold ml-1 text-blue-800">15 phút (900s)</span>
          </div>
        </div>

        <!-- Checker Inputs & Step-Up Auth -->
        <div v-if="voucher.status === 'PENDING_APPROVAL'" class="space-y-3 pt-2">
          <label class="block text-xs font-bold text-slate-700 uppercase tracking-wide">
            Định Danh Người Phê Duyệt (Checker Identity)
          </label>
          <input
            v-model="checkerId"
            type="text"
            placeholder="Nhập checker ID (ví dụ: checker_cfo_01)"
            class="w-full px-3.5 py-2 text-xs rounded-xl border border-slate-300 focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono"
          />

          <!-- Authentication Method Radio -->
          <div>
            <label class="block text-xs font-bold text-slate-700 uppercase tracking-wide mb-1.5">
              Phương Thức Xác Thực Bước 2 (Step-Up Dual Factor)
            </label>
            <div class="grid grid-cols-2 gap-3">
              <label
                class="flex items-center space-x-2.5 p-3 rounded-xl border cursor-pointer transition text-xs"
                :class="authMethod === 'BIOMETRIC_SIM' ? 'border-blue-500 bg-blue-50/50' : 'border-slate-200 bg-white'"
              >
                <input
                  v-model="authMethod"
                  type="radio"
                  value="BIOMETRIC_SIM"
                  class="text-blue-600"
                />
                <div>
                  <span class="font-bold block text-slate-900">Sinh Trắc Học FIDO2</span>
                  <span class="text-[11px] text-slate-500">Mô phỏng FaceID / Vân tay</span>
                </div>
              </label>

              <label
                class="flex items-center space-x-2.5 p-3 rounded-xl border cursor-pointer transition text-xs"
                :class="authMethod === 'SMS_OTP_SIM' ? 'border-blue-500 bg-blue-50/50' : 'border-slate-200 bg-white'"
              >
                <input
                  v-model="authMethod"
                  type="radio"
                  value="SMS_OTP_SIM"
                  class="text-blue-600"
                />
                <div>
                  <span class="font-bold block text-slate-900">Mã Xác Thực SMS OTP</span>
                  <span class="text-[11px] text-slate-500">Mã ngẫu nhiên 6 chữ số</span>
                </div>
              </label>
            </div>
          </div>

          <!-- Simulated Step-up Button -->
          <div class="flex items-center justify-between p-3 bg-slate-50 border border-slate-200 rounded-xl text-xs">
            <span class="text-slate-600">
              Trạng thái xác thực 2 bước:
              <strong :class="isStepUpVerified ? 'text-emerald-700 font-bold' : 'text-amber-600'">
                {{ isStepUpVerified ? 'Đã xác thực thành công' : 'Chưa kích hoạt xác thực' }}
              </strong>
            </span>
            <button
              type="button"
              class="px-3 py-1.5 rounded-lg text-xs font-semibold transition"
              :class="isStepUpVerified ? 'bg-slate-200 text-slate-600' : 'bg-blue-600 hover:bg-blue-700 text-white shadow-sm'"
              @click="triggerStepUpSimulation"
            >
              {{ isStepUpVerified ? 'Xác thực lại' : 'Kích hoạt xác thực ngay' }}
            </button>
          </div>

          <!-- Rejection Reason Input (Collapsible) -->
          <div v-if="isRejectMode" class="space-y-1.5 p-3 bg-rose-50 border border-rose-200 rounded-xl text-xs">
            <label class="block font-bold text-rose-800">
              Lý Do Từ Chối Lệnh Chi (Bắt buộc theo TT 09/2020):
            </label>
            <textarea
              v-model="rejectReason"
              rows="2"
              placeholder="Ghi rõ lý do từ chối để Maker bổ sung chứng từ..."
              class="w-full p-2 rounded-lg border border-rose-300 text-slate-800 focus:outline-none focus:ring-1 focus:ring-rose-500"
            ></textarea>
          </div>
        </div>

        <!-- Error Message -->
        <div v-if="actionError" class="p-3 bg-rose-100 border border-rose-300 text-rose-800 rounded-xl text-xs">
          {{ actionError }}
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="px-6 py-4 bg-slate-50 border-t border-slate-200 flex items-center justify-between">
        <button
          type="button"
          class="px-4 py-2 rounded-xl text-xs font-semibold text-slate-600 hover:bg-slate-200 transition"
          @click="onClose"
        >
          Đóng
        </button>

        <div v-if="voucher.status === 'PENDING_APPROVAL'" class="flex items-center space-x-3">
          <button
            v-if="!isRejectMode"
            type="button"
            class="px-4 py-2 rounded-xl text-xs font-semibold text-rose-700 hover:bg-rose-100 transition border border-rose-300"
            @click="isRejectMode = true"
          >
            Từ Chối Lệnh
          </button>
          <button
            v-else
            type="button"
            class="px-4 py-2 rounded-xl text-xs font-semibold bg-rose-600 hover:bg-rose-700 text-white shadow-sm transition"
            @click="handleReject"
          >
            Xác Nhận Từ Chối
          </button>

          <button
            type="button"
            class="px-5 py-2 rounded-xl text-xs font-bold text-white transition shadow-sm"
            :class="canApprove ? 'bg-emerald-600 hover:bg-emerald-700' : 'bg-slate-300 cursor-not-allowed'"
            :disabled="!canApprove"
            @click="handleApprove"
          >
            Phê Duyệt Lệnh Chi (Approve)
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { PaymentVoucher, AuthMethod, VoucherStatus } from '../../types/treasury';

const props = withDefaults(
  defineProps<{
    isOpen: boolean;
    voucher: PaymentVoucher | null;
    defaultCheckerId?: string;
  }>(),
  {
    defaultCheckerId: 'checker_cfo_01',
  }
);

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'approve', payload: { voucherId: string; checkerId: string; authMethod: AuthMethod }): void;
  (e: 'reject', payload: { voucherId: string; checkerId: string; reason: string }): void;
}>();

const checkerId = ref(props.defaultCheckerId);
const authMethod = ref<AuthMethod>('BIOMETRIC_SIM');
const isStepUpVerified = ref(false);
const isRejectMode = ref(false);
const rejectReason = ref('');
const actionError = ref('');

const isSelfApproval = computed(() => {
  if (!props.voucher) return false;
  return props.voucher.makerId.trim() === checkerId.value.trim();
});

const canApprove = computed(() => {
  return (
    Boolean(props.voucher) &&
    props.voucher?.status === 'PENDING_APPROVAL' &&
    checkerId.value.trim() !== '' &&
    !isSelfApproval.value &&
    isStepUpVerified.value
  );
});

function triggerStepUpSimulation() {
  actionError.value = '';
  isStepUpVerified.value = true;
}

function handleApprove() {
  if (!props.voucher) return;
  actionError.value = '';

  if (isSelfApproval.value) {
    actionError.value = 'Vi phạm Thông tư 09/2020: Maker không thể tự phê duyệt chính voucher của mình!';
    return;
  }

  if (!checkerId.value || checkerId.value.trim() === '') {
    actionError.value = 'Vui lòng nhập Checker ID hợp lệ!';
    return;
  }

  emit('approve', {
    voucherId: props.voucher.voucherId,
    checkerId: checkerId.value.trim(),
    authMethod: authMethod.value,
  });
}

function handleReject() {
  if (!props.voucher) return;
  actionError.value = '';

  if (isSelfApproval.value) {
    actionError.value = 'Vi phạm Thông tư 09/2020: Maker không thể từ chối voucher của chính mình với tư cách checker!';
    return;
  }

  if (!checkerId.value || checkerId.value.trim() === '') {
    actionError.value = 'Vui lòng nhập Checker ID hợp lệ!';
    return;
  }

  const reason = rejectReason.value.trim() || 'Từ chối bởi checker: Thiếu hồ sơ thanh toán gốc';
  emit('reject', {
    voucherId: props.voucher.voucherId,
    checkerId: checkerId.value.trim(),
    reason,
  });
}

function onClose() {
  actionError.value = '';
  isRejectMode.value = false;
  isStepUpVerified.value = false;
  emit('close');
}

function formatVnd(val: number): string {
  return Number(val || 0).toLocaleString('vi-VN');
}

function formatTimestamp(isoStr?: string): string {
  if (!isoStr) return '';
  try {
    const d = new Date(isoStr);
    return d.toLocaleString('vi-VN');
  } catch {
    return isoStr;
  }
}

function getStatusBadgeClass(status: VoucherStatus): string {
  switch (status) {
    case 'APPROVED':
      return 'bg-emerald-100 text-emerald-800';
    case 'PENDING_APPROVAL':
      return 'bg-amber-100 text-amber-800';
    case 'REJECTED':
      return 'bg-rose-100 text-rose-800';
    case 'SETTLED':
      return 'bg-blue-100 text-blue-800';
    case 'EXPIRED':
      return 'bg-slate-200 text-slate-700';
    default:
      return 'bg-slate-100 text-slate-600';
  }
}
</script>

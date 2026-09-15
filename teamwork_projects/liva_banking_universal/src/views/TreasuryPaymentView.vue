<template>
  <div class="treasury-payment-view space-y-6 animate-in fade-in duration-200">
    <!-- View Header & Role Switcher -->
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm flex flex-col md:flex-row md:items-center md:justify-between gap-4">
      <div>
        <h1 class="text-xl font-bold text-slate-900 tracking-tight flex items-center space-x-2.5">
          <div class="w-8 h-8 rounded-xl bg-slate-900 text-white flex items-center justify-center">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="20" height="12" x="2" y="6" rx="2"/><circle cx="12" cy="12" r="2"/><path d="M6 12h.01M18 12h.01"/></svg>
          </div>
          <span>Bàn Làm Việc Kiểm Soát Viên & Quản Trị Ngân Quỹ (Maker-Checker Desk)</span>
        </h1>
        <p class="text-xs text-slate-500 mt-0.5">
          Tuân thủ Điều 16 & 18 Thông tư 09/2020/TT-NHNN — Kiểm soát kép độc lập, thẩm định chứng từ, xác thực OTP/Chữ ký số và đóng dấu Sổ cái Merkle Tree
        </p>
      </div>

      <!-- Identity & Role Switcher -->
      <div class="flex items-center space-x-2 bg-slate-100 p-1 rounded-xl border border-slate-200 self-start md:self-auto text-xs">
        <span class="text-slate-500 font-medium px-2">Phân quyền tác nghiệp:</span>
        <button
          type="button"
          class="px-3 py-1.5 rounded-lg font-bold transition-all flex items-center space-x-1.5 cursor-pointer"
          :class="treasuryStore.currentUserRole === 'MAKER' ? 'bg-white text-slate-900 shadow-xs' : 'text-slate-600 hover:text-slate-900'"
          @click="setRole('maker_accountant_01', 'MAKER', 'Lê Hoàng Phúc (Cán bộ Vận hành / Maker)')"
        >
          <svg class="w-3.5 h-3.5 text-blue-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 20h9"/><path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"/></svg>
          <span>Maker (Lập Lệnh)</span>
        </button>
        <button
          type="button"
          class="px-3 py-1.5 rounded-lg font-bold transition-all flex items-center space-x-1.5 cursor-pointer"
          :class="treasuryStore.currentUserRole === 'CHECKER' ? 'bg-blue-600 text-white shadow-xs' : 'text-slate-600 hover:text-slate-900'"
          @click="setRole('checker_cfo_01', 'CHECKER', 'Nguyễn Văn Minh (Kiểm soát viên / Checker)')"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
          <span>Checker (Duyệt Lệnh)</span>
        </button>
      </div>
    </div>

    <!-- KPI Summary Grid -->
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      <div class="p-4 bg-white rounded-2xl border border-slate-200 shadow-sm space-y-1">
        <span class="text-xs text-slate-500 block font-medium">Chờ Phê Duyệt (Pending):</span>
        <div class="text-2xl font-bold font-mono text-amber-600">
          {{ treasuryStore.pendingVouchers.length }} <span class="text-xs font-normal text-slate-500">Lệnh</span>
        </div>
        <span class="text-[10px] text-slate-400 block">Tổng: {{ formatVnd(treasuryStore.totalPendingVnd) }} VND</span>
      </div>

      <div class="p-4 bg-white rounded-2xl border border-slate-200 shadow-sm space-y-1">
        <span class="text-xs text-slate-500 block font-medium">Đã Phê Duyệt (Approved):</span>
        <div class="text-2xl font-bold font-mono text-emerald-600">
          {{ treasuryStore.approvedVouchers.length }} <span class="text-xs font-normal text-slate-500">Lệnh</span>
        </div>
        <span class="text-[10px] text-slate-400 block">Kèm chữ ký số HMAC-SHA256</span>
      </div>

      <div class="p-4 bg-white rounded-2xl border border-slate-200 shadow-sm space-y-1">
        <span class="text-xs text-slate-500 block font-medium">Đã Quyết Toán (Settled):</span>
        <div class="text-2xl font-bold font-mono text-blue-600">
          {{ treasuryStore.settledVouchers.length }} <span class="text-xs font-normal text-slate-500">Lệnh</span>
        </div>
        <span class="text-[10px] text-slate-400 block">Tổng chi: {{ formatVnd(treasuryStore.totalDisbursedVnd) }} VND</span>
      </div>

      <div class="p-4 bg-white rounded-2xl border border-slate-200 shadow-sm space-y-1">
        <span class="text-xs text-slate-500 block font-medium">Gốc Cây Merkle (Root):</span>
        <div class="text-xs font-bold font-mono text-indigo-700 truncate" :title="treasuryStore.merkleRoot">
          {{ treasuryStore.merkleRoot.slice(0, 16) }}...
        </div>
        <span class="text-[10px] text-emerald-600 font-semibold block">● Tamper-Evident 100%</span>
      </div>
    </div>

    <!-- Active Workstation Tab Navigation (Clean Dedicated Functional Sub-Pages) -->
    <div class="flex items-center space-x-2 border-b border-slate-200 pb-2 text-xs font-bold overflow-x-auto">
      <button
        type="button"
        class="px-4 py-2 rounded-xl transition flex items-center space-x-2 cursor-pointer whitespace-nowrap"
        :class="activeDeskTab === 'APPRAISAL' ? 'bg-slate-900 text-white shadow-sm' : 'bg-white text-slate-600 hover:bg-slate-100 border border-slate-200'"
        @click="activeDeskTab = 'APPRAISAL'"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/><rect x="8" y="2" width="8" height="4" rx="1" ry="1"/></svg>
        <span>Thẩm định & Ký duyệt</span>
        <span
          v-if="treasuryStore.pendingVouchers.length > 0"
          class="px-1.5 py-0.5 rounded-full text-[10px] font-mono"
          :class="activeDeskTab === 'APPRAISAL' ? 'bg-amber-500 text-slate-950 font-bold' : 'bg-amber-100 text-amber-800'"
        >
          {{ treasuryStore.pendingVouchers.length }}
        </span>
      </button>

      <button
        type="button"
        class="px-4 py-2 rounded-xl transition flex items-center space-x-2 cursor-pointer whitespace-nowrap"
        :class="activeDeskTab === 'CREATE' ? 'bg-slate-900 text-white shadow-sm' : 'bg-white text-slate-600 hover:bg-slate-100 border border-slate-200'"
        @click="activeDeskTab = 'CREATE'"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        <span>Lập lệnh chi</span>
      </button>

      <button
        type="button"
        class="px-4 py-2 rounded-xl transition flex items-center space-x-2 cursor-pointer whitespace-nowrap"
        :class="activeDeskTab === 'ALL_VOUCHERS' ? 'bg-slate-900 text-white shadow-sm' : 'bg-white text-slate-600 hover:bg-slate-100 border border-slate-200'"
        @click="activeDeskTab = 'ALL_VOUCHERS'"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>
        <span>Sổ lệnh chi</span>
      </button>

      <button
        type="button"
        class="px-4 py-2 rounded-xl transition flex items-center space-x-2 cursor-pointer whitespace-nowrap"
        :class="activeDeskTab === 'MERKLE_AUDIT' ? 'bg-slate-900 text-white shadow-sm' : 'bg-white text-slate-600 hover:bg-slate-100 border border-slate-200'"
        @click="activeDeskTab = 'MERKLE_AUDIT'"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 3v18"/><path d="m8 8 4-5 4 5"/><path d="M3 14h18"/><path d="m8 19 4 2 4-2"/></svg>
        <span>Bằng chứng Merkle</span>
      </button>
    </div>

    <!-- TAB 1: Dedicated Checker Appraisal Desk & Document Drawer -->
    <div v-if="activeDeskTab === 'APPRAISAL'" class="space-y-6">
      <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">
        <!-- Left: Pending Vouchers List (5 cols) -->
        <div class="lg:col-span-5 bg-white rounded-2xl border border-slate-200 shadow-sm p-5 space-y-3 flex flex-col">
          <div class="flex items-center justify-between pb-2 border-b border-slate-100">
            <h3 class="text-xs font-bold uppercase tracking-wider text-slate-700 flex items-center space-x-2">
              <svg class="w-3.5 h-3.5 text-amber-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
              <span>Danh Sách Lệnh Chờ Thẩm Định ({{ treasuryStore.pendingVouchers.length }})</span>
            </h3>
            <span class="text-[11px] text-slate-400">Ưu tiên theo thời hạn TTL</span>
          </div>

          <div v-if="treasuryStore.pendingVouchers.length === 0" class="p-8 text-center bg-slate-50 rounded-xl border border-slate-200 text-slate-500 text-xs">
            <div class="w-10 h-10 rounded-full bg-emerald-100 text-emerald-700 flex items-center justify-center mx-auto mb-2">
              <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 6 9 17l-5-5"/></svg>
            </div>
            <p class="font-bold text-slate-800">Không có lệnh chi nào chờ thẩm định!</p>
            <p class="text-[11px] mt-0.5">Tất cả lệnh chi đã được phê duyệt hoặc xử lý khép lại.</p>
          </div>

          <div v-else class="space-y-2.5 max-h-[500px] overflow-y-auto pr-1">
            <div
              v-for="v in treasuryStore.pendingVouchers"
              :key="v.voucherId"
              class="p-3.5 rounded-xl border transition cursor-pointer text-xs space-y-2"
              :class="selectedVoucher?.voucherId === v.voucherId ? 'border-blue-600 bg-blue-50/50 shadow-xs ring-1 ring-blue-500' : 'border-slate-200 bg-slate-50/60 hover:bg-white'"
              @click="selectedVoucher = v"
            >
              <div class="flex items-center justify-between">
                <span class="font-mono font-bold text-slate-900">{{ v.voucherId }}</span>
                <span v-if="v.amountVnd >= 400000000" class="px-1.5 py-0.5 rounded font-bold text-[10px] bg-rose-100 text-rose-800 border border-rose-200">
                  ≥ 400M (QĐ 11/2023)
                </span>
              </div>

              <div class="flex items-baseline justify-between">
                <div class="font-bold text-sm text-slate-900 font-mono">
                  {{ formatVnd(v.amountVnd) }} VND
                </div>
                <span class="text-[10px] text-slate-500 font-mono">{{ v.beneficiaryBank }}</span>
              </div>

              <p class="text-slate-600 line-clamp-1 text-[11px]" :title="v.purpose">{{ v.purpose }}</p>

              <div class="flex items-center justify-between pt-1 border-t border-slate-200/60 text-[10px] text-slate-400">
                <span>Maker: <strong class="text-slate-700">{{ v.makerId }}</strong></span>
                <span class="text-amber-600 font-semibold font-mono">15m TTL</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Right: Document Appraisal Drawer / Panel (7 cols) -->
        <div class="lg:col-span-7 bg-white rounded-2xl border border-slate-200 shadow-sm p-6 space-y-5 flex flex-col justify-between">
          <div v-if="!selectedVoucher" class="p-12 text-center text-slate-400 text-xs space-y-2 my-auto">
            <div class="w-12 h-12 rounded-2xl bg-slate-100 text-slate-500 flex items-center justify-center mx-auto mb-2">
              <svg class="w-6 h-6" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
            </div>
            <p class="font-bold text-slate-700">Chọn một lệnh chi từ danh sách bên trái để mở hồ sơ thẩm định chứng từ</p>
            <p class="text-[11px]">Kiểm soát viên cần rà soát chứng từ gốc, tài khoản thụ hưởng và nội dung trước khi ký số phê duyệt</p>
          </div>

          <div v-else class="space-y-5">
            <!-- Dossier Header -->
            <div class="flex items-center justify-between pb-3 border-b border-slate-100">
              <div>
                <span class="text-[10px] font-bold uppercase tracking-wider text-slate-400 block">Hồ Sơ Thẩm Định Chứng Từ Lệnh Chi</span>
                <h2 class="text-base font-bold text-slate-900 font-mono">{{ selectedVoucher.voucherId }}</h2>
              </div>
              <span class="px-2.5 py-1 rounded-full text-xs font-bold bg-amber-100 text-amber-800">
                {{ selectedVoucher.status }}
              </span>
            </div>

            <!-- Anti-Self-Approval Fail-Closed Banner -->
            <div
              v-if="isSelfApproval"
              class="p-4 bg-rose-50 border-2 border-rose-300 rounded-xl flex items-start space-x-3 text-rose-900 text-xs animate-in shake duration-200"
            >
              <svg class="w-5 h-5 text-rose-600 shrink-0 mt-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="m4.9 4.9 14.2 14.2"/></svg>
              <div>
                <strong class="font-bold text-rose-900 block text-xs">VI PHẠM ĐIỀU 16 & 18 THÔNG TƯ 09/2020/TT-NHNN</strong>
                <p class="text-[11px] text-rose-800 mt-0.5 leading-relaxed">
                  Cán bộ lập lệnh (Maker: <code class="bg-rose-200 px-1 py-0.5 rounded font-mono">{{ selectedVoucher.makerId }}</code>) không được phép tự phê duyệt lệnh chi của chính mình.
                  Hệ thống <strong>Fail-Closed</strong> khóa nút phê duyệt để bảo đảm nguyên tắc bốn mắt bắt buộc.
                </p>
              </div>
            </div>

            <!-- Dossier Details Grid -->
            <div class="grid grid-cols-2 sm:grid-cols-3 gap-3 text-xs">
              <div class="p-3 rounded-xl bg-slate-50 border border-slate-200 space-y-0.5">
                <span class="text-slate-400 block text-[10px]">Đơn vị thụ hưởng:</span>
                <span class="font-bold text-slate-900">{{ selectedVoucher.beneficiaryName || 'Đối tác thụ hưởng' }}</span>
              </div>

              <div class="p-3 rounded-xl bg-slate-50 border border-slate-200 space-y-0.5">
                <span class="text-slate-400 block text-[10px]">Tài khoản thụ hưởng:</span>
                <span class="font-mono font-bold text-blue-700 select-all">{{ selectedVoucher.beneficiaryAccount }}</span>
              </div>

              <div class="p-3 rounded-xl bg-slate-50 border border-slate-200 space-y-0.5">
                <span class="text-slate-400 block text-[10px]">Kênh thanh toán:</span>
                <span class="font-bold text-indigo-800 font-mono">{{ selectedVoucher.beneficiaryBank }}</span>
              </div>

              <div class="p-3 rounded-xl bg-slate-50 border border-slate-200 space-y-0.5">
                <span class="text-slate-400 block text-[10px]">Người lập lệnh (Maker):</span>
                <span class="font-semibold text-slate-800">{{ selectedVoucher.makerId }}</span>
              </div>

              <div class="p-3 rounded-xl bg-slate-50 border border-slate-200 space-y-0.5">
                <span class="text-slate-400 block text-[10px]">Thời gian khởi tạo:</span>
                <span class="font-mono text-slate-700 text-[11px]">{{ formatTimestamp(selectedVoucher.createdAt) }}</span>
              </div>

              <div class="p-3 rounded-xl bg-slate-50 border border-slate-200 space-y-0.5">
                <span class="text-slate-400 block text-[10px]">Mã Token HITL (UUIDv4):</span>
                <span class="font-mono text-slate-700 text-[10px] truncate block" :title="selectedVoucher.hitlToken || ''">
                  {{ selectedVoucher.hitlToken || 'N/A' }}
                </span>
              </div>
            </div>

            <!-- Amount & Purpose Card -->
            <div class="p-4 rounded-xl bg-slate-900 text-white space-y-2">
              <div class="flex items-center justify-between text-xs text-slate-400">
                <span>Số Tiền Lệnh Chi Cần Phê Duyệt:</span>
                <span class="font-mono text-emerald-400">64-bit Integer VND (0 Float Drift)</span>
              </div>
              <div class="text-2xl font-bold font-mono text-emerald-400">
                {{ formatVnd(selectedVoucher.amountVnd) }} VND
              </div>
              <div class="text-xs text-slate-300 pt-1 border-t border-slate-800">
                <span class="text-slate-400">Nội dung chi:</span> {{ selectedVoucher.purpose }}
              </div>
            </div>

            <!-- Mandatory Explanatory Remarks Field (Circular 09/2020) -->
            <div class="space-y-1.5">
              <div class="flex items-center justify-between">
                <label class="block text-xs font-bold text-slate-700">
                  Biên Bản Giải Trình & Ghi Chú Thẩm Định Của Kiểm Soát Viên (<span class="text-rose-600">Bắt buộc theo TT 09/2020</span>):
                </label>
                <span class="text-[10px] font-semibold" :class="checkerRemarks.trim() ? 'text-emerald-600' : 'text-rose-500'">
                  {{ checkerRemarks.trim() ? 'Đã nhập giải trình' : 'Chưa nhập biên bản' }}
                </span>
              </div>
              <textarea
                v-model="checkerRemarks"
                rows="2"
                class="w-full p-2.5 text-xs rounded-xl border border-slate-300 focus:ring-2 focus:ring-blue-500 focus:outline-none"
                placeholder="Nhập biên bản thẩm định đối chiếu hồ sơ hóa đơn, chứng từ hợp lệ trước khi ký lệnh chi..."
              ></textarea>
            </div>

            <!-- Step-Up Simulated OTP / Digital Signature -->
            <div class="p-3.5 bg-slate-50 border border-slate-200 rounded-xl space-y-2 text-xs">
              <div class="flex items-center justify-between">
                <span class="font-bold text-slate-700">Xác Thực 2 Bước (Step-Up Dual-Control):</span>
                <span class="font-semibold text-xs" :class="isOtpVerified ? 'text-emerald-700' : 'text-amber-600'">
                  {{ isOtpVerified ? 'OTP / Chữ ký số hợp lệ' : 'Chưa kích hoạt xác thực' }}
                </span>
              </div>

              <div class="grid grid-cols-3 gap-2">
                <label
                  class="flex items-center space-x-2 p-2 rounded-lg border cursor-pointer text-[11px]"
                  :class="activeAuthMethod === 'BIOMETRIC_SIM' ? 'bg-white border-blue-500 shadow-xs' : 'bg-slate-100 border-slate-200'"
                >
                  <input v-model="activeAuthMethod" type="radio" value="BIOMETRIC_SIM" />
                  <span class="font-medium">Sinh Trắc Học FIDO2</span>
                </label>

                <label
                  class="flex items-center space-x-2 p-2 rounded-lg border cursor-pointer text-[11px]"
                  :class="activeAuthMethod === 'SMS_OTP_SIM' ? 'bg-white border-blue-500 shadow-xs' : 'bg-slate-100 border-slate-200'"
                >
                  <input v-model="activeAuthMethod" type="radio" value="SMS_OTP_SIM" />
                  <span class="font-medium">SMS OTP 6 Số</span>
                </label>

                <label
                  class="flex items-center space-x-2 p-2 rounded-lg border cursor-pointer text-[11px]"
                  :class="activeAuthMethod === 'HARDWARE_TOKEN' ? 'bg-white border-blue-500 shadow-xs' : 'bg-slate-100 border-slate-200'"
                >
                  <input v-model="activeAuthMethod" type="radio" value="HARDWARE_TOKEN" />
                  <span class="font-medium">Token Chữ Ký Số</span>
                </label>
              </div>

              <div class="flex items-center justify-between pt-1">
                <span class="text-[11px] text-slate-500 font-mono">
                  Mã xác thực: <strong>{{ simulatedOtpCode || '••••••' }}</strong>
                </span>
                <button
                  type="button"
                  class="px-3 py-1 text-xs font-semibold rounded bg-blue-600 hover:bg-blue-700 text-white transition cursor-pointer"
                  @click="generateAndVerifyOtp"
                >
                  {{ isOtpVerified ? 'Xác thực lại' : 'Tạo & Xác Thực OTP' }}
                </button>
              </div>
            </div>

            <!-- Action Buttons -->
            <div class="flex items-center justify-end space-x-3 pt-2 border-t border-slate-100">
              <button
                type="button"
                class="px-4 py-2 text-xs font-bold rounded-xl border border-rose-300 text-rose-700 hover:bg-rose-50 transition cursor-pointer"
                :disabled="!checkerRemarks.trim()"
                @click="handleAppraisalReject"
              >
                Từ Chối Lệnh Chi
              </button>

              <button
                type="button"
                class="px-5 py-2 text-xs font-bold rounded-xl text-white transition shadow-sm flex items-center space-x-1.5 cursor-pointer"
                :class="canApproveCurrent ? 'bg-emerald-600 hover:bg-emerald-700' : 'bg-slate-300 cursor-not-allowed'"
                :disabled="!canApproveCurrent"
                @click="handleAppraisalApprove"
              >
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 6 9 17l-5-5"/></svg>
                <span>Ký Số & Phê Duyệt Lệnh Chi (Approve)</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- TAB 2: Dedicated Maker Voucher Creator Form -->
    <div v-else-if="activeDeskTab === 'CREATE'" class="bg-white rounded-2xl border border-slate-200 shadow-sm p-6 space-y-5">
      <div class="flex items-center justify-between pb-3 border-b border-slate-100">
        <div>
          <h2 class="text-base font-bold text-slate-900">Khởi Tạo Lệnh Điều Chuyển Vốn / Lệnh Chi (Maker Desk)</h2>
          <p class="text-xs text-slate-500">Lập lệnh chi với tài khoản thụ hưởng, kiểm soát rủi ro hạn mức tự động</p>
        </div>
      </div>

      <div class="p-4 bg-slate-50 rounded-xl border border-slate-200 space-y-4">
        <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3 text-xs">
          <div>
            <label class="block font-medium text-slate-700 mb-1">Đơn Vị Thụ Hưởng / Kênh Nhận:</label>
            <input
              v-model="newVoucher.beneficiaryName"
              type="text"
              class="w-full p-2 rounded-lg border border-slate-300 bg-white"
              placeholder="VD: SỞ GIAO DỊCH NHNN / NAPAS"
            />
          </div>

          <div>
            <label class="block font-medium text-slate-700 mb-1">Số Tài Khoản / Mã Hạn Mức:</label>
            <input
              v-model="newVoucher.beneficiaryAccount"
              type="text"
              class="w-full p-2 rounded-lg border border-slate-300 bg-white font-mono"
              placeholder="VD: 01-CITAD-SBV-VND"
            />
          </div>

          <div>
            <label class="block font-medium text-slate-700 mb-1">Kênh Thanh Toán / Quyết Toán Thụ Hưởng:</label>
            <select v-model="newVoucher.beneficiaryBank" class="w-full p-2 rounded-lg border border-slate-300 bg-white">
              <option value="CITAD_SBV">Sở Giao Dịch NHNN (Kênh CITAD / IBPS Giá Trị Cao)</option>
              <option value="NAPAS">Công ty CP Thanh toán Quốc gia VN (Kênh NAPAS 24/7)</option>
              <option value="NOSTRO_BILATERAL">Kênh Thanh toán Song phương & Tài khoản Nostro/Vostro</option>
              <option value="SWIFT_INTL">Cổng Chuyển tiền & Điện báo Quốc tế SWIFT</option>
            </select>
          </div>

          <div>
            <label class="block font-medium text-slate-700 mb-1">Số Tiền Điều Chuyển / Chi (VND):</label>
            <input
              v-model.number="newVoucher.amountVnd"
              type="number"
              step="10000000"
              class="w-full p-2 rounded-lg border border-slate-300 bg-white font-mono font-bold"
              placeholder="VD: 50000000"
            />
          </div>

          <div class="sm:col-span-2">
            <label class="block font-medium text-slate-700 mb-1">Nội Dung Chi Tiết / Hóa Đơn:</label>
            <input
              v-model="newVoucher.purpose"
              type="text"
              class="w-full p-2 rounded-lg border border-slate-300 bg-white"
              placeholder="VD: Thanh toán bù trừ đợt 1 hợp đồng HD-2026-99"
            />
          </div>
        </div>

        <div class="flex justify-end space-x-2 pt-2 border-t border-slate-200">
          <button
            type="button"
            class="px-4 py-2 text-xs font-semibold rounded-lg bg-slate-200 text-slate-700 hover:bg-slate-300 cursor-pointer"
            @click="activeDeskTab = 'ALL_VOUCHERS'"
          >
            Xem Danh Sách Sổ Lệnh
          </button>
          <button
            type="button"
            class="px-5 py-2 text-xs font-bold rounded-lg bg-emerald-600 hover:bg-emerald-700 text-white shadow-xs cursor-pointer"
            :disabled="!newVoucher.amountVnd || !newVoucher.beneficiaryAccount"
            @click="handleCreateVoucher"
          >
            Lưu & Trình Duyệt Ngay
          </button>
        </div>
      </div>
    </div>

    <!-- TAB 3: Full Vouchers Ledger -->
    <div v-else-if="activeDeskTab === 'ALL_VOUCHERS'" class="bg-white rounded-2xl border border-slate-200 shadow-sm p-5 space-y-4">
      <div class="flex items-center justify-between pb-3 border-b border-slate-100">
        <div>
          <h2 class="text-base font-bold text-slate-900">Danh Sách Lệnh Điều Chuyển Vốn & Lệnh Chi Liên Ngân Hàng</h2>
          <p class="text-xs text-slate-500">Cán bộ lập lệnh (Maker) và Kiểm soát viên phê duyệt (Checker) độc lập tuyệt đối</p>
        </div>

        <button
          type="button"
          class="px-3.5 py-2 text-xs font-bold rounded-xl bg-slate-900 hover:bg-slate-800 text-white transition shadow-sm flex items-center space-x-1.5 cursor-pointer"
          @click="activeDeskTab = 'CREATE'"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          <span>+ Lập Lệnh Điều Chuyển / Chi</span>
        </button>
      </div>

      <!-- Vouchers Table -->
      <div class="overflow-x-auto">
        <table class="w-full text-left text-xs">
          <thead class="bg-slate-50 text-slate-600 font-semibold border-b border-slate-200">
            <tr>
              <th class="p-3">Mã Lệnh</th>
              <th class="p-3">Người Lập (Maker)</th>
              <th class="p-3">Đơn Vị Thụ Hưởng</th>
              <th class="p-3 text-right">Số Tiền (VND)</th>
              <th class="p-3">Nội Dung Thanh Toán</th>
              <th class="p-3 text-center">Trạng Thái</th>
              <th class="p-3 text-right">Hành Động</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-100">
            <tr
              v-for="v in treasuryStore.vouchers"
              :key="v.voucherId"
              class="hover:bg-slate-50/80 transition"
            >
              <td class="p-3 align-middle font-mono font-bold text-slate-900 whitespace-nowrap">
                {{ v.voucherId }}
              </td>

              <td class="p-3 align-middle whitespace-nowrap">
                <span class="text-slate-800 font-medium">{{ v.makerId }}</span>
              </td>

              <td class="p-3 align-middle">
                <div class="font-bold text-slate-900">{{ v.beneficiaryName || 'Đối tác thụ hưởng' }}</div>
                <div class="text-[10px] text-slate-400 font-mono">
                  {{ v.beneficiaryBank }} - {{ v.beneficiaryAccount }}
                </div>
              </td>

              <td class="p-3 align-middle text-right font-mono font-bold text-slate-900 whitespace-nowrap">
                {{ formatVnd(v.amountVnd) }}
              </td>

              <td class="p-3 align-middle max-w-[220px]">
                <p class="truncate text-slate-600" :title="v.purpose">{{ v.purpose }}</p>
                <span v-if="v.signatureHmac" class="text-[10px] font-mono text-emerald-600 truncate block" :title="v.signatureHmac">
                  HMAC: {{ v.signatureHmac.slice(0, 16) }}...
                </span>
              </td>

              <td class="p-3 align-middle text-center whitespace-nowrap">
                <span
                  class="inline-block px-2.5 py-0.5 rounded-full text-[10px] font-bold"
                  :class="getStatusBadgeClass(v.status)"
                >
                  {{ v.status }}
                </span>
              </td>

              <td class="p-3 align-middle text-right whitespace-nowrap">
                <button
                  v-if="v.status === 'PENDING_APPROVAL'"
                  type="button"
                  class="px-3 py-1.5 text-xs font-bold rounded-lg bg-blue-600 hover:bg-blue-700 text-white shadow-xs transition cursor-pointer"
                  @click="openAppraisalDrawer(v)"
                >
                  Thẩm Định & Duyệt
                </button>

                <button
                  v-else-if="v.status === 'DRAFT'"
                  type="button"
                  class="px-3 py-1.5 text-xs font-bold rounded-lg bg-emerald-600 hover:bg-emerald-700 text-white shadow-xs transition cursor-pointer"
                  @click="treasuryStore.submitVoucher(v.voucherId)"
                >
                  Trình Duyệt
                </button>

                <button
                  v-else-if="v.status === 'APPROVED'"
                  type="button"
                  class="px-3 py-1.5 text-xs font-bold rounded-lg bg-slate-900 hover:bg-slate-800 text-white shadow-xs transition cursor-pointer"
                  @click="treasuryStore.settleVoucher(v.voucherId)"
                >
                  Quyết Toán
                </button>

                <span v-else class="text-xs text-slate-400">
                  Đã khép lại
                </span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- TAB 4: Merkle Tree Cryptographic Forward Audit Ledger & Proof Steps Inspector -->
    <div v-else-if="activeDeskTab === 'MERKLE_AUDIT'" class="space-y-6">
      <!-- Merkle Proof Card -->
      <MerkleProofCard
        :merkle-root="treasuryStore.merkleRoot"
        :audit-entries="treasuryStore.auditEntries"
        :leaf-count="treasuryStore.approvedVouchers.length + treasuryStore.settledVouchers.length"
      />

      <!-- Interactive O(log N) Proof Step Inspector & Ledger Verifier -->
      <div class="bg-white rounded-2xl border border-slate-200 shadow-sm p-5 space-y-4">
        <div class="flex items-center justify-between pb-3 border-b border-slate-100">
          <div>
            <h3 class="text-sm font-bold text-slate-900">Công Cụ Kiểm Định Bằng Chứng Merkle O(log N) & Toàn Vẹn Sổ Cái</h3>
            <p class="text-xs text-slate-500">Tái tính toán băm mật mã FIPS 180-4 độc lập, bảo đảm không có khối nào bị giả mạo</p>
          </div>

          <button
            type="button"
            class="px-4 py-2 text-xs font-bold rounded-xl bg-indigo-600 hover:bg-indigo-700 text-white shadow-xs transition flex items-center space-x-1.5 cursor-pointer"
            @click="runFullLedgerIntegrityCheck"
          >
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><path d="m9 12 2 2 4-4"/></svg>
            <span>Kiểm Tra Toàn Vẹn Chuỗi Khối (Full Chain Check)</span>
          </button>
        </div>

        <div v-if="ledgerIntegrityResult" class="p-3 rounded-xl border text-xs font-semibold" :class="ledgerIntegrityResult.isValid ? 'bg-emerald-50 text-emerald-800 border-emerald-200' : 'bg-rose-50 text-rose-800 border-rose-200'">
          {{ ledgerIntegrityResult.isValid ? 'Toàn bộ chuỗi khối (Forward Hash Chain) đạt tính toàn vẹn 100%. Không phát hiện khối nào bị can thiệp dữ liệu.' : ledgerIntegrityResult.details }}
        </div>

        <!-- Inclusion Proof Tool -->
        <div class="p-4 bg-slate-50 rounded-xl border border-slate-200 space-y-3 text-xs">
          <div class="flex flex-wrap items-center justify-between gap-2">
            <label class="font-bold text-slate-700">Chọn Lệnh Chi Đã Duyệt Để Kiểm Tra Bằng Chứng Bao Hàm (Inclusion Proof):</label>
            <select
              v-model="selectedProofVoucherId"
              class="p-2 rounded-lg border border-slate-300 bg-white font-mono text-xs font-semibold"
              @change="computeProofForSelected"
            >
              <option value="">-- Chọn lệnh chi đã phê duyệt --</option>
              <option
                v-for="v in eligibleProofVouchers"
                :key="v.voucherId"
                :value="v.voucherId"
              >
                {{ v.voucherId }} ({{ formatVnd(v.amountVnd) }} VND - {{ v.status }})
              </option>
            </select>
          </div>

          <!-- Proof Steps Display -->
          <div v-if="generatedProofSteps.length > 0" class="space-y-2 pt-2 border-t border-slate-200">
            <div class="flex items-center justify-between text-[11px] font-mono">
              <span class="text-slate-500">Mã Băm Lá (Leaf Hash): <strong class="text-slate-900">{{ currentLeafHash.slice(0, 24) }}...</strong></span>
              <span class="text-indigo-700 font-bold">Số bước O(log N): {{ generatedProofSteps.length }} bước</span>
            </div>

            <div class="space-y-1.5 font-mono text-[10px]">
              <div
                v-for="(step, sIdx) in generatedProofSteps"
                :key="sIdx"
                class="p-2.5 rounded-lg bg-white border border-slate-200 flex items-center justify-between"
              >
                <div class="flex items-center space-x-2">
                  <span class="px-1.5 py-0.5 rounded font-bold uppercase" :class="step.position === 'left' ? 'bg-blue-100 text-blue-800' : 'bg-purple-100 text-purple-800'">
                    {{ step.position }}
                  </span>
                  <span class="text-slate-600">Bước #{{ sIdx + 1 }}: {{ step.hash }}</span>
                </div>
              </div>
            </div>

            <div class="flex items-center justify-between pt-2">
              <span v-if="proofVerificationStatus" class="font-bold text-xs" :class="proofVerificationStatus.includes('hợp lệ') ? 'text-emerald-600' : 'text-rose-600'">
                {{ proofVerificationStatus }}
              </span>
              <span v-else></span>

              <button
                type="button"
                class="px-4 py-2 text-xs font-bold rounded-lg bg-slate-900 hover:bg-slate-800 text-white transition shadow-xs cursor-pointer"
                @click="verifySelectedProof"
              >
                Xác Thực Bằng Chứng Mật Mã (Verify Proof)
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Toast Notification -->
    <div
      v-if="toastMessage"
      class="fixed bottom-6 right-6 z-50 p-4 bg-slate-900 text-white text-xs font-semibold rounded-2xl shadow-xl border border-slate-700 flex items-center space-x-2 animate-in slide-in-from-bottom-3"
    >
      <div class="w-4 h-4 rounded-full bg-emerald-500 text-slate-950 flex items-center justify-center font-bold">
        <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"/></svg>
      </div>
      <span>{{ toastMessage }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useTreasuryStore } from '../stores/treasuryStore';
import MerkleProofCard from '../components/treasury/MerkleProofCard.vue';
import type { PaymentVoucher, VoucherStatus, AuthMethod, MerkleProofStep } from '../types/treasury';
import {
  generateMerkleProof,
  verifyMerkleProof,
  computeMerkleLeaf,
} from '../engine/treasury/merkleAudit';

const treasuryStore = useTreasuryStore();

const activeDeskTab = ref<'APPRAISAL' | 'CREATE' | 'ALL_VOUCHERS' | 'MERKLE_AUDIT'>('APPRAISAL');
const selectedVoucher = ref<PaymentVoucher | null>(null);
const checkerRemarks = ref('');
const activeAuthMethod = ref<AuthMethod>('BIOMETRIC_SIM');
const isOtpVerified = ref(false);
const simulatedOtpCode = ref('');
const toastMessage = ref('');

// Merkle verification state
const selectedProofVoucherId = ref('');
const currentLeafHash = ref('');
const generatedProofSteps = ref<MerkleProofStep[]>([]);
const proofVerificationStatus = ref('');
const ledgerIntegrityResult = ref<{ isValid: boolean; brokenAt?: number; details?: string } | null>(null);

const newVoucher = ref({
  beneficiaryName: '',
  beneficiaryAccount: '',
  beneficiaryBank: 'CITAD_SBV',
  amountVnd: 50_000_000,
  purpose: '',
});

onMounted(() => {
  // If there are pending vouchers, auto-select the first one for appraisal
  if (treasuryStore.pendingVouchers.length > 0) {
    selectedVoucher.value = treasuryStore.pendingVouchers[0];
  }
});

function setRole(userId: string, role: 'MAKER' | 'CHECKER', name: string) {
  treasuryStore.setCurrentUser(userId, role, name);
  if (role === 'CHECKER') {
    activeDeskTab.value = 'APPRAISAL';
  }
}

const isSelfApproval = computed(() => {
  if (!selectedVoucher.value) return false;
  return selectedVoucher.value.makerId.trim() === treasuryStore.currentUserId.trim();
});

const canApproveCurrent = computed(() => {
  return (
    Boolean(selectedVoucher.value) &&
    selectedVoucher.value?.status === 'PENDING_APPROVAL' &&
    !isSelfApproval.value &&
    isOtpVerified.value &&
    checkerRemarks.value.trim().length > 0
  );
});

function openAppraisalDrawer(v: PaymentVoucher) {
  selectedVoucher.value = v;
  activeDeskTab.value = 'APPRAISAL';
  isOtpVerified.value = false;
  simulatedOtpCode.value = '';
  checkerRemarks.value = '';
}

function generateAndVerifyOtp() {
  const code = Math.floor(100000 + Math.random() * 900000).toString();
  simulatedOtpCode.value = code;
  isOtpVerified.value = true;
  showToast(`Mã xác thực OTP ${code} đã được áp dụng thành công.`);
}

function handleAppraisalApprove() {
  if (!selectedVoucher.value || !canApproveCurrent.value) return;

  const vId = selectedVoucher.value.voucherId;
  const token = selectedVoucher.value.hitlToken || undefined;

  treasuryStore.approveVoucher(
    vId,
    treasuryStore.currentUserId,
    activeAuthMethod.value,
    token
  );

  showToast(`Đã ký số và phê duyệt lệnh chi ${vId} thành công!`);

  // Reset form and select next pending voucher if any
  isOtpVerified.value = false;
  simulatedOtpCode.value = '';
  checkerRemarks.value = '';
  selectedVoucher.value = treasuryStore.pendingVouchers[0] || null;
}

function handleAppraisalReject() {
  if (!selectedVoucher.value || !checkerRemarks.value.trim()) return;

  const vId = selectedVoucher.value.voucherId;
  const token = selectedVoucher.value.hitlToken || undefined;

  treasuryStore.rejectVoucher(
    vId,
    treasuryStore.currentUserId,
    checkerRemarks.value.trim(),
    token
  );

  showToast(`Đã từ chối lệnh chi ${vId}. Biên bản giải trình đã được lưu trữ.`);

  isOtpVerified.value = false;
  simulatedOtpCode.value = '';
  checkerRemarks.value = '';
  selectedVoucher.value = treasuryStore.pendingVouchers[0] || null;
}

function handleCreateVoucher() {
  if (!newVoucher.value.amountVnd || !newVoucher.value.beneficiaryAccount) return;

  const created = treasuryStore.createVoucher(
    treasuryStore.currentUserId,
    newVoucher.value.beneficiaryAccount,
    newVoucher.value.beneficiaryBank,
    newVoucher.value.amountVnd,
    newVoucher.value.purpose || 'Chi thanh toán đối tác liên ngân hàng',
    newVoucher.value.beneficiaryName || 'Đơn vị thụ hưởng'
  );

  treasuryStore.submitVoucher(created.voucherId);

  newVoucher.value = {
    beneficiaryName: '',
    beneficiaryAccount: '',
    beneficiaryBank: 'CITAD_SBV',
    amountVnd: 50_000_000,
    purpose: '',
  };
  activeDeskTab.value = 'ALL_VOUCHERS';
  showToast(`Đã tạo và trình duyệt lệnh chi ${created.voucherId}!`);
}

// Merkle Proof Inspection Logic
const eligibleProofVouchers = computed(() => {
  return treasuryStore.vouchers.filter(
    (v) => v.status === 'APPROVED' || v.status === 'SETTLED'
  );
});

function computeProofForSelected() {
  proofVerificationStatus.value = '';
  if (!selectedProofVoucherId.value) {
    generatedProofSteps.value = [];
    currentLeafHash.value = '';
    return;
  }

  const leaves = eligibleProofVouchers.value.map((v) =>
    v.merkleLeafHash || computeMerkleLeaf(v)
  );

  const idx = eligibleProofVouchers.value.findIndex(
    (v) => v.voucherId === selectedProofVoucherId.value
  );

  if (idx >= 0 && leaves.length > 0) {
    const proof = generateMerkleProof(leaves, idx);
    currentLeafHash.value = proof.leaf;
    generatedProofSteps.value = proof.proof;
  }
}

function verifySelectedProof() {
  if (!currentLeafHash.value) return;
  const isValid = verifyMerkleProof(
    currentLeafHash.value,
    generatedProofSteps.value,
    treasuryStore.merkleRoot
  );

  if (isValid) {
    proofVerificationStatus.value =
      'Bằng chứng hợp lệ 100%: Khối nằm trong cây Merkle mà không cần tiết lộ giao dịch khác.';
  } else {
    proofVerificationStatus.value = 'Bằng chứng không khớp hoặc dữ liệu bị thay đổi.';
  }
}

function runFullLedgerIntegrityCheck() {
  ledgerIntegrityResult.value = treasuryStore.verifyAuditIntegrity();
}

function showToast(msg: string) {
  toastMessage.value = msg;
  setTimeout(() => {
    toastMessage.value = '';
  }, 3500);
}

function formatVnd(val?: number): string {
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
      return 'bg-amber-100 text-amber-800 animate-pulse';
    case 'SETTLED':
      return 'bg-blue-100 text-blue-800';
    case 'REJECTED':
      return 'bg-rose-100 text-rose-800';
    case 'DRAFT':
    default:
      return 'bg-slate-100 text-slate-700';
  }
}
</script>

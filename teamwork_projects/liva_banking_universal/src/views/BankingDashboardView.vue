<template>
  <div class="banking-dashboard space-y-6 animate-in fade-in duration-200">
    <!-- Top Action Bar & Channel Overview -->
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm flex flex-col md:flex-row md:items-center md:justify-between gap-4">
      <div>
        <h1 class="text-xl font-bold text-slate-900 tracking-tight flex items-center space-x-2.5">
          <div class="w-8 h-8 rounded-xl bg-slate-900 text-white flex items-center justify-center">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 21h18"/><path d="M3 10h18"/><path d="M5 6l7-3 7 3"/><path d="M4 10v11"/><path d="M20 10v11"/><path d="M8 14v4"/><path d="M12 14v4"/><path d="M16 14v4"/></svg>
          </div>
          <span>Bàn Giao Dịch Thanh Khoản & Quyết Toán Liên Ngân Hàng (Treasury Desk)</span>
        </h1>
        <p class="text-xs text-slate-500 mt-0.5">
          Hệ thống tác nghiệp nội bộ Ngân hàng Thương mại — Giám sát vị thế thanh khoản các kênh CITAD (NHNN), NAPAS 24/7, Song phương & SWIFT
        </p>
      </div>

      <!-- Quick Action Buttons -->
      <div class="flex items-center space-x-2 self-start md:self-auto text-xs">
        <button
          type="button"
          class="px-3.5 py-2 text-xs font-bold rounded-xl bg-amber-500 hover:bg-amber-400 text-slate-950 transition shadow-xs flex items-center space-x-1.5 cursor-pointer"
          @click="openRebalanceModal()"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 3 4 7l4 4"/><path d="M4 7h16"/><path d="m16 21 4-4-4-4"/><path d="M20 17H4"/></svg>
          <span>Điều Chuyển Vốn</span>
        </button>

        <button
          type="button"
          class="px-3.5 py-2 text-xs font-bold rounded-xl bg-slate-900 hover:bg-slate-800 text-white transition shadow-xs flex items-center space-x-1.5 cursor-pointer"
          @click="$emit('navigate', 'reconciliation')"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m16 16 3-8 3 8c-.87.65-1.92 1-3 1s-2.13-.35-3-1Z"/><path d="m2 16 3-8 3 8c-.87.65-1.92 1-3 1s-2.13-.35-3-1Z"/><path d="M7 21h10"/><path d="M12 3v18"/><path d="M3 7h2c2 0 5-1 7-2 2 1 5 2 7 2h2"/></svg>
          <span>Đối Soát Quyết Toán</span>
        </button>
      </div>
    </div>

    <!-- Reserve Breach Warning Banner (if any channel breaches limit) -->
    <div
      v-if="bankingStore.hasReserveBreach"
      class="p-4 rounded-2xl bg-rose-50 border border-rose-300 text-rose-900 flex items-center justify-between shadow-xs animate-bounce-short"
    >
      <div class="flex items-center space-x-3">
        <div class="w-9 h-9 rounded-xl bg-rose-200 text-rose-800 flex items-center justify-center shrink-0">
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
        </div>
        <div>
          <h4 class="text-sm font-bold">CẢNH BÁO VI PHẠM HẠN MỨC DỰ TRỮ THANH KHOẢN (LIQUIDITY LIMIT BREACH)</h4>
          <p class="text-xs text-rose-700">
            Có kênh thanh toán có số dư khả dụng thấp hơn mức dự trữ an toàn tối thiểu quy định. Cần điều chuyển vốn ngay lập tức!
          </p>
        </div>
      </div>
      <button
        type="button"
        class="px-4 py-2 text-xs font-bold rounded-xl bg-rose-600 hover:bg-rose-700 text-white transition shadow-sm whitespace-nowrap cursor-pointer flex items-center space-x-1.5"
        @click="openRebalanceModal()"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 3 4 7l4 4"/><path d="M4 7h16"/><path d="m16 21 4-4-4-4"/><path d="M20 17H4"/></svg>
        <span>Điều Chuyển Bù Đắp Ngay</span>
      </button>
    </div>

    <!-- Clean Functional Subpage Navigation Tabs -->
    <div class="flex items-center space-x-2 border-b border-slate-200 pb-2 text-xs font-bold overflow-x-auto">
      <button
        type="button"
        class="px-4 py-2 rounded-xl transition flex items-center space-x-2 cursor-pointer whitespace-nowrap"
        :class="activeTab === 'CHANNELS' ? 'bg-slate-900 text-white shadow-sm' : 'bg-white text-slate-600 hover:bg-slate-100 border border-slate-200'"
        @click="activeTab = 'CHANNELS'"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="20" height="12" x="2" y="6" rx="2"/><circle cx="12" cy="12" r="2"/><path d="M6 12h.01M18 12h.01"/></svg>
        <span>Kênh thanh toán & Hạn mức</span>
        <span
          v-if="bankingStore.breachedChannels.length > 0"
          class="px-1.5 py-0.5 rounded-full text-[10px] font-mono bg-rose-500 text-white font-bold"
        >
          {{ bankingStore.breachedChannels.length }}
        </span>
      </button>

      <button
        type="button"
        class="px-4 py-2 rounded-xl transition flex items-center space-x-2 cursor-pointer whitespace-nowrap"
        :class="activeTab === 'REBALANCE_LOGS' ? 'bg-slate-900 text-white shadow-sm' : 'bg-white text-slate-600 hover:bg-slate-100 border border-slate-200'"
        @click="activeTab = 'REBALANCE_LOGS'"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 3 4 7l4 4"/><path d="M4 7h16"/><path d="m16 21 4-4-4-4"/><path d="M20 17H4"/></svg>
        <span>Nhật ký điều chuyển vốn</span>
        <span class="px-1.5 py-0.5 rounded-full text-[10px] font-mono bg-slate-100 text-slate-700">
          {{ bankingStore.rebalanceLogs.length }}
        </span>
      </button>

      <button
        type="button"
        class="px-4 py-2 rounded-xl transition flex items-center space-x-2 cursor-pointer whitespace-nowrap"
        :class="activeTab === 'MERKLE_AUDIT' ? 'bg-slate-900 text-white shadow-sm' : 'bg-white text-slate-600 hover:bg-slate-100 border border-slate-200'"
        @click="activeTab = 'MERKLE_AUDIT'"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 3v18"/><path d="m8 8 4-5 4 5"/><path d="M3 14h18"/><path d="m8 19 4 2 4-2"/></svg>
        <span>Sổ cái kiểm toán Merkle</span>
      </button>

      <button
        type="button"
        class="px-4 py-2 rounded-xl transition flex items-center space-x-2 cursor-pointer whitespace-nowrap"
        :class="activeTab === 'KPIS' ? 'bg-slate-900 text-white shadow-sm' : 'bg-white text-slate-600 hover:bg-slate-100 border border-slate-200'"
        @click="activeTab = 'KPIS'"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 3v18h18"/><path d="m19 9-5 5-4-4-3 3"/></svg>
        <span>Báo cáo thanh khoản</span>
      </button>
    </div>

    <!-- TAB 1: Interbank Settlement Channels & Liquidity Risk Limits -->
    <div v-if="activeTab === 'CHANNELS'" class="space-y-4">
      <!-- Channel Switcher Filter Pills -->
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-1.5 p-1 bg-white rounded-xl border border-slate-200 text-xs font-semibold overflow-x-auto">
          <button
            type="button"
            class="px-3 py-1.5 rounded-lg transition-all whitespace-nowrap cursor-pointer"
            :class="bankingStore.selectedBankPersona === 'ALL' ? 'bg-slate-900 text-white shadow-xs font-bold' : 'text-slate-600 hover:text-slate-900'"
            @click="bankingStore.setBankPersona('ALL')"
          >
            Tất Cả Kênh ({{ bankingStore.accounts.length }})
          </button>
          <button
            type="button"
            class="px-3 py-1.5 rounded-lg transition-all flex items-center space-x-1 whitespace-nowrap cursor-pointer"
            :class="bankingStore.selectedBankPersona === 'CITAD' ? 'bg-emerald-700 text-white shadow-xs font-bold' : 'text-slate-600 hover:text-slate-900'"
            @click="bankingStore.setBankPersona('CITAD')"
          >
            <span class="w-2 h-2 rounded-full bg-emerald-400"></span>
            <span>CITAD (NHNN)</span>
          </button>
          <button
            type="button"
            class="px-3 py-1.5 rounded-lg transition-all flex items-center space-x-1 whitespace-nowrap cursor-pointer"
            :class="bankingStore.selectedBankPersona === 'NAPAS' ? 'bg-sky-700 text-white shadow-xs font-bold' : 'text-slate-600 hover:text-slate-900'"
            @click="bankingStore.setBankPersona('NAPAS')"
          >
            <span class="w-2 h-2 rounded-full bg-sky-400"></span>
            <span>NAPAS 24/7</span>
          </button>
          <button
            type="button"
            class="px-3 py-1.5 rounded-lg transition-all flex items-center space-x-1 whitespace-nowrap cursor-pointer"
            :class="bankingStore.selectedBankPersona === 'BILATERAL' ? 'bg-amber-700 text-white shadow-xs font-bold' : 'text-slate-600 hover:text-slate-900'"
            @click="bankingStore.setBankPersona('BILATERAL')"
          >
            <span class="w-2 h-2 rounded-full bg-amber-400"></span>
            <span>Song Phương & Nostro</span>
          </button>
          <button
            type="button"
            class="px-3 py-1.5 rounded-lg transition-all flex items-center space-x-1 whitespace-nowrap cursor-pointer"
            :class="bankingStore.selectedBankPersona === 'SWIFT' ? 'bg-purple-700 text-white shadow-xs font-bold' : 'text-slate-600 hover:text-slate-900'"
            @click="bankingStore.setBankPersona('SWIFT')"
          >
            <span class="w-2 h-2 rounded-full bg-purple-400"></span>
            <span>SWIFT</span>
          </button>
        </div>

        <span class="text-xs text-slate-400">
          Hiển thị: {{ bankingStore.displayedAccounts.length }} / {{ bankingStore.accounts.length }} kênh
        </span>
      </div>

      <!-- Channel Cards Grid -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <div
          v-for="acc in bankingStore.displayedAccounts"
          :key="acc.accountNumber"
          class="bg-white rounded-2xl border p-5 space-y-4 shadow-sm hover:shadow-md transition duration-200 flex flex-col justify-between"
          :class="[getAccountBorderClass(acc.bankCode), acc.balanceVnd < acc.minReserveVnd ? 'ring-2 ring-rose-400' : '']"
        >
          <div class="space-y-3">
            <!-- Channel Header Badge -->
            <div class="flex items-start justify-between">
              <div class="flex items-center space-x-2.5">
                <div
                  class="w-9 h-9 rounded-xl flex items-center justify-center font-bold text-white text-xs shadow-xs"
                  :style="{ backgroundColor: acc.brandColor }"
                >
                  {{ acc.channelCode || acc.bankCode }}
                </div>
                <div>
                  <h3 class="text-sm font-bold text-slate-900">{{ acc.channelCode || acc.bankCode }}</h3>
                  <p class="text-[11px] text-slate-500 truncate max-w-[150px]" :title="acc.clearingAuthority || acc.channelName">
                    {{ acc.clearingAuthority || acc.channelName }}
                  </p>
                </div>
              </div>

              <!-- Status Badge -->
              <span
                v-if="acc.balanceVnd < acc.minReserveVnd"
                class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-rose-100 text-rose-800"
              >
                Dưới Hạn Mức
              </span>
              <span
                v-else
                class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-emerald-100 text-emerald-800"
              >
                An Toàn
              </span>
            </div>

            <!-- Account / Channel Details -->
            <div class="p-3 bg-slate-50 rounded-xl space-y-1 text-xs">
              <div class="flex justify-between text-slate-500">
                <span>Mã TK / Hạn mức:</span>
                <span class="font-mono font-bold text-slate-900 select-all truncate max-w-[120px]">{{ acc.accountNumber }}</span>
              </div>
              <div class="flex justify-between text-slate-500">
                <span>Giờ Cut-off:</span>
                <span class="font-semibold text-slate-800">{{ acc.cutoffTime || '24/7' }}</span>
              </div>
              <div class="text-[10px] text-slate-400 truncate pt-0.5" :title="acc.accountName">
                {{ acc.accountName }}
              </div>
            </div>

            <!-- Balance Value -->
            <div class="pt-1">
              <div class="flex items-center justify-between text-[11px] text-slate-400 font-medium">
                <span>Số Dư Khả Dụng:</span>
                <span>Dự trữ tối thiểu: {{ formatVnd(acc.minReserveVnd) }} VND</span>
              </div>
              <div class="text-xl font-bold font-mono text-slate-900 mt-0.5">
                {{ formatVnd(acc.balanceVnd) }} <span class="text-xs font-normal text-slate-500">VND</span>
              </div>

              <!-- Progress bar of Reserve Coverage -->
              <div class="w-full bg-slate-100 rounded-full h-1.5 mt-1.5 overflow-hidden">
                <div
                  class="h-1.5 rounded-full transition-all duration-300"
                  :class="acc.balanceVnd >= acc.minReserveVnd ? 'bg-emerald-500' : 'bg-rose-500'"
                  :style="{ width: Math.min(100, Math.round((acc.balanceVnd / (acc.minReserveVnd || 1)) * 100)) + '%' }"
                ></div>
              </div>
              <div class="flex justify-between text-[10px] text-slate-400 mt-1">
                <span>Tỷ lệ bao phủ: {{ ((acc.balanceVnd / (acc.minReserveVnd || 1)) * 100).toFixed(0) }}%</span>
                <span :class="acc.balanceVnd >= acc.minReserveVnd ? 'text-emerald-600 font-semibold' : 'text-rose-600 font-bold'">
                  {{ acc.balanceVnd >= acc.minReserveVnd ? 'Đạt chuẩn' : 'Vi phạm' }}
                </span>
              </div>
            </div>

            <!-- Monthly Inflow / Outflow & Channel Net Flow -->
            <div class="space-y-1.5 pt-1">
              <div class="grid grid-cols-2 gap-2 text-[11px]">
                <div class="p-2 rounded-lg bg-emerald-50/70 border border-emerald-100">
                  <span class="text-emerald-700 block text-[10px]">Vào (Inflow):</span>
                  <span class="font-bold text-emerald-900 font-mono text-xs">+{{ formatVnd(acc.inflowMonthVnd) }}</span>
                </div>
                <div class="p-2 rounded-lg bg-rose-50/70 border border-rose-100">
                  <span class="text-rose-700 block text-[10px]">Ra (Outflow):</span>
                  <span class="font-bold text-rose-900 font-mono text-xs">-{{ formatVnd(acc.outflowMonthVnd) }}</span>
                </div>
              </div>
              <div
                class="px-2 py-1 rounded-md text-[11px] font-mono flex items-center justify-between"
                :class="(acc.inflowMonthVnd - acc.outflowMonthVnd) >= 0 ? 'bg-emerald-50 text-emerald-800' : 'bg-rose-50 text-rose-800'"
              >
                <span class="text-[10px] uppercase font-sans font-semibold">Dòng tiền ròng kênh:</span>
                <span class="font-bold">
                  {{ (acc.inflowMonthVnd - acc.outflowMonthVnd) >= 0 ? '+' : '' }}{{ formatVnd(acc.inflowMonthVnd - acc.outflowMonthVnd) }}
                </span>
              </div>
            </div>
          </div>

          <!-- Bottom Card Action -->
          <div class="pt-3 border-t border-slate-100 flex items-center justify-between text-xs">
            <button
              type="button"
              class="text-xs font-semibold text-amber-700 hover:text-amber-900 flex items-center space-x-1 cursor-pointer"
              @click="openRebalanceModal(acc.bankCode)"
            >
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 3 4 7l4 4"/><path d="M4 7h16"/><path d="m16 21 4-4-4-4"/><path d="M20 17H4"/></svg>
              <span>Điều vốn</span>
            </button>
            <button
              type="button"
              class="font-semibold text-blue-600 hover:text-blue-800 flex items-center space-x-1 cursor-pointer"
              @click="onSelectAccount(acc.bankCode)"
            >
              <span>Đối Soát Kênh</span>
              <span>→</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- TAB 2: Intraday Cash Rebalancing Log -->
    <div v-else-if="activeTab === 'REBALANCE_LOGS'" class="bg-white rounded-2xl border border-slate-200 shadow-sm p-5 space-y-4">
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 pb-3 border-b border-slate-100">
        <div>
          <h2 class="text-base font-bold text-slate-900">Nhật Ký Điều Chuyển Vốn Thanh Khoản (Intraday & Overnight)</h2>
          <p class="text-xs text-slate-500">Lịch sử các lệnh điều phối vốn bù trừ giữa các kênh CITAD, NAPAS, Nostro</p>
        </div>
        <button
          type="button"
          class="px-3.5 py-2 text-xs font-bold rounded-xl bg-amber-500 hover:bg-amber-400 text-slate-950 transition shadow-xs flex items-center space-x-1.5 cursor-pointer self-start sm:self-auto"
          @click="openRebalanceModal()"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
          <span>+ Lập Lệnh Điều Vốn</span>
        </button>
      </div>

      <div v-if="bankingStore.rebalanceLogs.length === 0" class="text-center py-8 text-slate-400 text-xs">
        Chưa có lệnh điều chuyển vốn nào trong phiên.
      </div>

      <div v-else class="overflow-x-auto">
        <table class="w-full text-left text-xs">
          <thead class="bg-slate-50 text-slate-600 font-semibold border-b border-slate-200">
            <tr>
              <th class="p-3">Mã Lệnh</th>
              <th class="p-3">Kênh Chuyển → Nhận</th>
              <th class="p-3 text-right">Số Tiền (VND)</th>
              <th class="p-3">Mục Đích Nghiệp Vụ</th>
              <th class="p-3">Người Thực Hiện</th>
              <th class="p-3">Thời Gian</th>
              <th class="p-3 text-center">Trạng Thái</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-100">
            <tr
              v-for="log in bankingStore.rebalanceLogs"
              :key="log.id"
              class="hover:bg-slate-50/80 transition"
            >
              <td class="p-3 font-mono font-bold text-slate-900 whitespace-nowrap">{{ log.id }}</td>
              <td class="p-3 whitespace-nowrap font-mono font-bold text-slate-800">
                {{ log.fromChannel }} <span class="text-slate-400 mx-1">→</span> {{ log.toChannel }}
              </td>
              <td class="p-3 text-right font-mono font-bold text-emerald-700 whitespace-nowrap">
                {{ formatVnd(log.amountVnd) }}
              </td>
              <td class="p-3 text-slate-600 max-w-xs truncate">{{ log.purpose }}</td>
              <td class="p-3 text-slate-700 whitespace-nowrap">{{ log.executedBy }}</td>
              <td class="p-3 text-slate-400 whitespace-nowrap">{{ formatTimestamp(log.timestamp) }}</td>
              <td class="p-3 text-center whitespace-nowrap">
                <span
                  class="px-2 py-0.5 rounded-full text-[10px] font-bold uppercase"
                  :class="log.status === 'COMPLETED' ? 'bg-emerald-100 text-emerald-800' : 'bg-rose-100 text-rose-800'"
                >
                  {{ log.status }}
                </span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- TAB 3: Merkle Audit Ledger Stream & Compliance -->
    <div v-else-if="activeTab === 'MERKLE_AUDIT'" class="bg-white rounded-2xl border border-slate-200 shadow-sm p-6 space-y-4">
      <div class="flex items-center justify-between pb-3 border-b border-slate-100">
        <div>
          <h2 class="text-base font-bold text-slate-900">Sổ Bất Biến Forward Merkle & Chuỗi Khối Kiểm Toán</h2>
          <p class="text-xs text-slate-500">Mỗi sự kiện điều vốn và duyệt chi được băm mật mã SHA-256 chuỗi tiếp nối</p>
        </div>
        <span class="px-2.5 py-1 rounded-lg text-xs font-mono bg-slate-100 text-slate-700 font-semibold">
          {{ treasuryStore.auditEntries.length }} Khối ghi nhận
        </span>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <!-- Events List -->
        <div class="space-y-2 max-h-[400px] overflow-y-auto pr-1 text-xs">
          <div
            v-for="(entry, idx) in treasuryStore.auditEntries.slice(-8).reverse()"
            :key="idx"
            class="p-3 bg-slate-50 rounded-xl border border-slate-200 space-y-1"
          >
            <div class="flex items-center justify-between text-[11px]">
              <span class="font-bold uppercase font-mono text-slate-800">{{ entry.event }}</span>
              <span class="text-slate-400">{{ formatTimestamp(entry.timestamp) }}</span>
            </div>
            <div class="text-[10px] text-slate-500 font-mono truncate">
              Hash: {{ (entry.currentHash || '').slice(0, 36) }}...
            </div>
            <div class="text-[10px] text-slate-400">
              Tác nhân: <strong class="text-slate-700">{{ entry.actor }}</strong>
            </div>
          </div>
        </div>

        <!-- Compliance & Security Status -->
        <div class="p-5 bg-slate-50 rounded-xl border border-slate-200 space-y-4 text-xs flex flex-col justify-between">
          <div class="space-y-3">
            <h3 class="font-bold text-slate-900 uppercase tracking-wider text-[11px]">Tiêu Chuẩn Toàn Vẹn Hệ Thống</h3>
            <div class="space-y-2">
              <div class="flex items-center justify-between p-2.5 bg-white rounded-lg border border-slate-200">
                <span class="text-slate-600">Độc lập môi trường:</span>
                <span class="font-semibold text-emerald-700 font-mono">100% Client-Side</span>
              </div>
              <div class="flex items-center justify-between p-2.5 bg-white rounded-lg border border-slate-200">
                <span class="text-slate-600">Thông tư TT 09/2020:</span>
                <span class="font-semibold text-blue-700">Maker-Checker Bốn Mắt</span>
              </div>
              <div class="flex items-center justify-between p-2.5 bg-white rounded-lg border border-slate-200">
                <span class="text-slate-600">Hạn mức rủi ro thanh khoản:</span>
                <span class="font-semibold" :class="bankingStore.hasReserveBreach ? 'text-rose-600 font-bold' : 'text-emerald-700'">
                  {{ bankingStore.hasReserveBreach ? 'Có Kênh Vi Phạm Hạn Mức' : 'Trong Ngưỡng Cho Phép' }}
                </span>
              </div>
            </div>
          </div>

          <div class="p-3 bg-slate-900 text-white rounded-xl text-center text-[11px] font-mono">
            LIVA Universal Banking • Production Ready
          </div>
        </div>
      </div>
    </div>

    <!-- TAB 4: Financial KPIs & Overview Grid -->
    <div v-else-if="activeTab === 'KPIS'" class="space-y-4">
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4">
        <!-- 1. Total Liquid Capital -->
        <div class="p-5 bg-gradient-to-br from-slate-900 to-slate-800 text-white rounded-2xl shadow-sm space-y-2 border border-slate-700">
          <div class="flex items-center justify-between text-xs text-slate-300">
            <span class="font-medium">Tổng Thanh Khoản</span>
            <span class="px-2 py-0.5 rounded-md bg-emerald-500/20 text-emerald-300 font-mono text-[11px]">VND</span>
          </div>
          <div class="text-2xl font-bold font-mono text-emerald-400 tracking-tight">
            {{ formatVnd(displayedTotalBalance) }}
          </div>
          <div class="flex items-center space-x-2 text-[11px] text-slate-400 pt-1 border-t border-slate-700/60">
            <span class="text-emerald-400 font-semibold">● Nội bộ 100%</span>
            <span class="truncate">{{ bankingStore.selectedBankPersona === 'ALL' ? '4 Kênh liên ngân hàng' : 'Kênh chuyên biệt' }}</span>
          </div>
        </div>

        <!-- 2. Net Cash Flow Month -->
        <div class="p-5 bg-white rounded-2xl shadow-sm border border-slate-200 space-y-2">
          <div class="flex items-center justify-between text-xs text-slate-500">
            <span class="font-medium">Dòng Tiền Ròng Tháng</span>
            <span
              class="px-2 py-0.5 rounded-md text-[10px] font-bold"
              :class="bankingStore.netFlowMonthVnd >= 0 ? 'bg-emerald-100 text-emerald-800' : 'bg-rose-100 text-rose-800'"
            >
              {{ bankingStore.netFlowMonthVnd >= 0 ? 'DƯ DƯƠNG' : 'THÂM HỤT' }}
            </span>
          </div>
          <div
            class="text-2xl font-bold font-mono tracking-tight"
            :class="bankingStore.netFlowMonthVnd >= 0 ? 'text-emerald-600' : 'text-rose-600'"
          >
            {{ bankingStore.netFlowMonthVnd >= 0 ? '+' : '' }}{{ formatVnd(bankingStore.netFlowMonthVnd) }}
          </div>
          <div class="text-[11px] text-slate-400 pt-1 border-t border-slate-100 flex items-center justify-between">
            <span class="text-emerald-600 font-mono">+{{ formatVnd(bankingStore.totalInflowMonthVnd) }}</span>
            <span class="text-rose-600 font-mono">-{{ formatVnd(bankingStore.totalOutflowMonthVnd) }}</span>
          </div>
        </div>

        <!-- 3. Liquidity Runway -->
        <div class="p-5 bg-white rounded-2xl shadow-sm border border-slate-200 space-y-2">
          <div class="flex items-center justify-between text-xs text-slate-500">
            <span class="font-medium">Độ Dài Thanh Khoản</span>
            <svg class="w-4 h-4 text-slate-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
          </div>
          <div class="text-2xl font-bold text-slate-900 font-mono">
            {{ bankingStore.liquidityRunwayDays }} <span class="text-sm font-normal text-slate-500">Ngày</span>
          </div>
          <div class="text-[11px] text-slate-400 pt-1 border-t border-slate-100 flex items-center space-x-1">
            <span class="text-emerald-600 font-semibold">An toàn</span>
            <span class="truncate">Mức chi {{ formatVnd(bankingStore.dailyAverageBurnVnd) }} VND/ngày</span>
          </div>
        </div>

        <!-- 4. Reconciliation Match Rate -->
        <div class="p-5 bg-white rounded-2xl shadow-sm border border-slate-200 space-y-2">
          <div class="flex items-center justify-between text-xs text-slate-500">
            <span class="font-medium">Tỷ Lệ Đối Soát</span>
            <svg class="w-4 h-4 text-emerald-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="m9 12 2 2 4-4"/></svg>
          </div>
          <div class="text-2xl font-bold text-emerald-600 font-mono">
            {{ reconStore.matchRate.toFixed(1) }}%
          </div>
          <div class="text-[11px] text-slate-400 pt-1 border-t border-slate-100 flex items-center justify-between">
            <span>Chuẩn: ≥ 99.8%</span>
            <span class="text-emerald-600 font-semibold font-mono">{{ reconStore.matchedCount }} Khớp</span>
          </div>
        </div>

        <!-- 5. Pending Dual-Control & Limit Health -->
        <div class="p-5 bg-white rounded-2xl shadow-sm border border-slate-200 space-y-2">
          <div class="flex items-center justify-between text-xs text-slate-500">
            <span class="font-medium">Kiểm Soát & Hạn Mức</span>
            <svg class="w-4 h-4 text-amber-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="m9 12 2 2 4-4"/></svg>
          </div>
          <div class="flex items-baseline space-x-2">
            <div class="text-2xl font-bold text-amber-600 font-mono">
              {{ treasuryStore.pendingVouchers.length }}
              <span class="text-xs font-normal text-slate-500">Lệnh</span>
            </div>
            <div
              class="text-2xl font-bold font-mono"
              :class="bankingStore.hasReserveBreach ? 'text-rose-600' : 'text-emerald-600'"
            >
              / {{ bankingStore.breachedChannels.length }}
              <span class="text-xs font-normal text-slate-500">Vi phạm</span>
            </div>
          </div>
          <div class="text-[11px] text-slate-400 pt-1 border-t border-slate-100 flex items-center justify-between">
            <span class="text-amber-600 font-medium">Bốn mắt TT 09</span>
            <span :class="bankingStore.hasReserveBreach ? 'text-rose-600 font-bold' : 'text-emerald-600 font-medium'">
              {{ bankingStore.hasReserveBreach ? 'Nguy cơ cạn vốn' : 'Hạn mức chuẩn' }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Modal: Intraday Cash Rebalancing Tool -->
    <div
      v-if="isRebalanceModalOpen"
      class="fixed inset-0 z-50 overflow-y-auto bg-slate-900/60 backdrop-blur-xs flex items-center justify-center p-4 animate-in fade-in"
    >
      <div class="bg-white rounded-2xl shadow-2xl max-w-lg w-full border border-slate-200 overflow-hidden flex flex-col">
        <!-- Header -->
        <div class="px-6 py-4 bg-slate-900 text-white flex items-center justify-between">
          <div class="flex items-center space-x-3">
            <div class="w-8 h-8 rounded-lg bg-amber-500 flex items-center justify-center text-slate-950 font-bold">
              <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 3 4 7l4 4"/><path d="M4 7h16"/><path d="m16 21 4-4-4-4"/><path d="M20 17H4"/></svg>
            </div>
            <div>
              <h3 class="text-base font-bold">Lệnh Điều Chuyển Vốn Thanh Khoản Nội Bộ</h3>
              <p class="text-xs text-slate-400">Tác nghiệp điều chuyển giữa các tài khoản bù trừ thanh toán</p>
            </div>
          </div>
          <button
            type="button"
            class="text-slate-400 hover:text-white transition p-1 rounded-lg cursor-pointer"
            @click="closeRebalanceModal()"
          >
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18M6 6l12 12"/></svg>
          </button>
        </div>

        <!-- Form Body -->
        <div class="p-6 space-y-4 text-xs">
          <!-- Alert feedback if any -->
          <div
            v-if="rebalanceFeedback"
            class="p-3 rounded-xl text-xs font-semibold"
            :class="rebalanceFeedback.success ? 'bg-emerald-50 text-emerald-800 border border-emerald-200' : 'bg-rose-50 text-rose-800 border border-rose-200'"
          >
            {{ rebalanceFeedback.message || rebalanceFeedback.error }}
          </div>

          <!-- Channel Pickers -->
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="block font-semibold text-slate-700 mb-1">Kênh Nguồn (Trích Nợ):</label>
              <select
                v-model="rebalanceForm.fromChannel"
                class="w-full px-3 py-2 bg-slate-50 border border-slate-200 rounded-xl font-medium focus:ring-2 focus:ring-amber-500 outline-hidden"
              >
                <option v-for="a in bankingStore.accounts" :key="a.bankCode" :value="a.bankCode">
                  {{ a.bankCode }} - {{ formatVnd(a.balanceVnd) }} VND
                </option>
              </select>
            </div>
            <div>
              <label class="block font-semibold text-slate-700 mb-1">Kênh Đích (Ghi Có):</label>
              <select
                v-model="rebalanceForm.toChannel"
                class="w-full px-3 py-2 bg-slate-50 border border-slate-200 rounded-xl font-medium focus:ring-2 focus:ring-amber-500 outline-hidden"
              >
                <option v-for="a in bankingStore.accounts" :key="a.bankCode" :value="a.bankCode">
                  {{ a.bankCode }} - {{ formatVnd(a.balanceVnd) }} VND
                </option>
              </select>
            </div>
          </div>

          <!-- Amount Input & Quick Presets -->
          <div>
            <div class="flex items-center justify-between mb-1">
              <label class="font-semibold text-slate-700">Số Tiền Điều Chuyển (VND):</label>
              <span class="text-slate-400 font-mono font-bold">{{ formatVnd(rebalanceForm.amountVnd) }} VND</span>
            </div>
            <input
              v-model.number="rebalanceForm.amountVnd"
              type="number"
              step="10000000"
              min="1000000"
              class="w-full px-3 py-2 bg-slate-50 border border-slate-200 rounded-xl font-mono text-base font-bold text-slate-900 focus:ring-2 focus:ring-amber-500 outline-hidden"
            />
            <div class="flex items-center space-x-1.5 mt-2">
              <button
                type="button"
                class="px-2.5 py-1 rounded-lg bg-slate-100 hover:bg-slate-200 text-slate-700 font-mono text-[11px] cursor-pointer"
                @click="rebalanceForm.amountVnd = 50_000_000"
              >
                50M
              </button>
              <button
                type="button"
                class="px-2.5 py-1 rounded-lg bg-slate-100 hover:bg-slate-200 text-slate-700 font-mono text-[11px] cursor-pointer"
                @click="rebalanceForm.amountVnd = 100_000_000"
              >
                100M
              </button>
              <button
                type="button"
                class="px-2.5 py-1 rounded-lg bg-slate-100 hover:bg-slate-200 text-slate-700 font-mono text-[11px] cursor-pointer"
                @click="rebalanceForm.amountVnd = 200_000_000"
              >
                200M
              </button>
              <button
                type="button"
                class="px-2.5 py-1 rounded-lg bg-slate-100 hover:bg-slate-200 text-slate-700 font-mono text-[11px] cursor-pointer"
                @click="rebalanceForm.amountVnd = 500_000_000"
              >
                500M
              </button>
            </div>
          </div>

          <!-- Purpose -->
          <div>
            <label class="block font-semibold text-slate-700 mb-1">Mục Đích Nghiệp Vụ:</label>
            <input
              v-model="rebalanceForm.purpose"
              type="text"
              class="w-full px-3 py-2 bg-slate-50 border border-slate-200 rounded-xl text-xs focus:ring-2 focus:ring-amber-500 outline-hidden"
              placeholder="VD: Cân đối thanh khoản bù trừ cuối ngày NAPAS"
            />
          </div>

          <!-- Live Balance Preview Box -->
          <div class="p-3.5 bg-slate-50 rounded-xl border border-slate-200 space-y-2">
            <h4 class="font-bold text-slate-700 uppercase text-[10px] tracking-wider">Xem Trước Vị Thế Sau Điều Chuyển</h4>
            <div class="grid grid-cols-2 gap-3">
              <!-- Source Preview -->
              <div class="p-2.5 bg-white rounded-lg border border-slate-200">
                <span class="text-[10px] text-slate-400 block font-semibold">Kênh nguồn: {{ rebalanceForm.fromChannel }}</span>
                <div class="flex justify-between items-baseline mt-1">
                  <span class="text-slate-500 text-[11px]">Hiện tại:</span>
                  <span class="font-mono font-medium">{{ formatVnd(sourceAcc?.balanceVnd) }}</span>
                </div>
                <div class="flex justify-between items-baseline">
                  <span class="text-slate-700 font-semibold text-[11px]">Sau chuyển:</span>
                  <span
                    class="font-mono font-bold"
                    :class="sourcePostBalance < 0 ? 'text-rose-600' : 'text-slate-900'"
                  >
                    {{ formatVnd(sourcePostBalance) }}
                  </span>
                </div>
                <div
                  v-if="isSourceBreaching"
                  class="text-[10px] font-bold text-rose-600 mt-1"
                >
                  Số dư sau chuyển sẽ dưới ngưỡng dự trữ ({{ formatVnd(sourceAcc?.minReserveVnd) }})!
                </div>
              </div>

              <!-- Target Preview -->
              <div class="p-2.5 bg-white rounded-lg border border-slate-200">
                <span class="text-[10px] text-slate-400 block font-semibold">Kênh đích: {{ rebalanceForm.toChannel }}</span>
                <div class="flex justify-between items-baseline mt-1">
                  <span class="text-slate-500 text-[11px]">Hiện tại:</span>
                  <span class="font-mono font-medium">{{ formatVnd(targetAcc?.balanceVnd) }}</span>
                </div>
                <div class="flex justify-between items-baseline">
                  <span class="text-slate-700 font-semibold text-[11px]">Sau chuyển:</span>
                  <span class="font-mono font-bold text-emerald-600">
                    {{ formatVnd(targetPostBalance) }}
                  </span>
                </div>
                <div class="text-[10px] text-emerald-600 mt-1 font-semibold">
                  Dự trữ an toàn
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Footer -->
        <div class="px-6 py-4 bg-slate-50 border-t border-slate-200 flex items-center justify-between">
          <button
            type="button"
            class="px-4 py-2 text-xs font-semibold rounded-xl text-slate-600 hover:text-slate-900 hover:bg-slate-200 transition cursor-pointer"
            @click="closeRebalanceModal()"
          >
            Đóng
          </button>
          <button
            type="button"
            class="px-5 py-2 text-xs font-bold rounded-xl text-slate-950 transition shadow-sm"
            :class="canExecuteRebalance ? 'bg-amber-500 hover:bg-amber-400 cursor-pointer' : 'bg-slate-300 text-slate-500 cursor-not-allowed'"
            :disabled="!canExecuteRebalance"
            @click="executeRebalance()"
          >
            Xác Nhận Điều Chuyển Vốn
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, reactive } from 'vue';
import { useBankingStore } from '../stores/bankingStore';
import { useReconciliationStore } from '../stores/reconciliationStore';
import { useAmlStore } from '../stores/amlStore';
import { useTreasuryStore } from '../stores/treasuryStore';
import type { BankCode } from '../types/banking';

const emit = defineEmits<{
  (e: 'navigate', tab: 'dashboard' | 'reconciliation' | 'aml' | 'treasury'): void;
}>();

const bankingStore = useBankingStore();
const reconStore = useReconciliationStore();
const amlStore = useAmlStore();
const treasuryStore = useTreasuryStore();

const activeTab = ref<'CHANNELS' | 'REBALANCE_LOGS' | 'MERKLE_AUDIT' | 'KPIS'>('CHANNELS');

const displayedTotalBalance = computed(() => {
  if (bankingStore.selectedBankPersona === 'ALL') {
    return bankingStore.totalLiquidVnd;
  }
  return bankingStore.currentPersonaAccount?.balanceVnd ?? 0;
});

// Rebalancing State
const isRebalanceModalOpen = ref(false);
const rebalanceFeedback = ref<{ success?: boolean; message?: string; error?: string } | null>(null);

const rebalanceForm = reactive({
  fromChannel: 'CITAD' as BankCode,
  toChannel: 'NAPAS' as BankCode,
  amountVnd: 100_000_000,
  purpose: 'Cân đối thanh khoản bù trừ cuối ngày',
});

const sourceAcc = computed(() =>
  bankingStore.accounts.find(
    (a) => a.bankCode === rebalanceForm.fromChannel || a.channelCode === rebalanceForm.fromChannel
  )
);

const targetAcc = computed(() =>
  bankingStore.accounts.find(
    (a) => a.bankCode === rebalanceForm.toChannel || a.channelCode === rebalanceForm.toChannel
  )
);

const sourcePostBalance = computed(() => (sourceAcc.value?.balanceVnd ?? 0) - rebalanceForm.amountVnd);
const targetPostBalance = computed(() => (targetAcc.value?.balanceVnd ?? 0) + rebalanceForm.amountVnd);

const isSourceBreaching = computed(() => {
  if (!sourceAcc.value?.minReserveVnd) return false;
  return sourcePostBalance.value < sourceAcc.value.minReserveVnd;
});

const canExecuteRebalance = computed(() => {
  if (rebalanceForm.fromChannel === rebalanceForm.toChannel) return false;
  if (rebalanceForm.amountVnd <= 0) return false;
  if (!sourceAcc.value || sourceAcc.value.balanceVnd < rebalanceForm.amountVnd) return false;
  return true;
});

function openRebalanceModal(defaultSource?: BankCode) {
  if (defaultSource) {
    rebalanceForm.fromChannel = defaultSource;
    // pick a different target
    const alt = bankingStore.accounts.find((a) => a.bankCode !== defaultSource);
    if (alt) rebalanceForm.toChannel = alt.bankCode;
  }
  rebalanceFeedback.value = null;
  isRebalanceModalOpen.value = true;
}

function closeRebalanceModal() {
  isRebalanceModalOpen.value = false;
  rebalanceFeedback.value = null;
}

function executeRebalance() {
  const res = bankingStore.rebalanceLiquidity(
    rebalanceForm.fromChannel,
    rebalanceForm.toChannel,
    rebalanceForm.amountVnd,
    rebalanceForm.purpose,
    treasuryStore.currentUserId || 'CB-OPERATOR-01'
  );

  if (res.success) {
    treasuryStore.auditLedger.append(
      treasuryStore.currentUserId || 'CB-OPERATOR-01',
      'FUNDS_REBALANCED',
      {
        from: rebalanceForm.fromChannel,
        to: rebalanceForm.toChannel,
        amountVnd: rebalanceForm.amountVnd,
        purpose: rebalanceForm.purpose,
        logId: res.logId,
      }
    );
    rebalanceFeedback.value = { success: true, message: res.message };
    setTimeout(() => {
      closeRebalanceModal();
    }, 1500);
  } else {
    rebalanceFeedback.value = { success: false, error: res.error };
  }
}

function onSelectAccount(bankCode: BankCode) {
  bankingStore.setBankPersona(bankCode);
  if (bankCode === 'CITAD' || bankCode === 'VCB') {
    reconStore.loadSampleDataset('CITAD');
  } else if (bankCode === 'NAPAS' || bankCode === 'TCB') {
    reconStore.loadSampleDataset('NAPAS');
  } else if (bankCode === 'BILATERAL' || bankCode === 'BIDV') {
    reconStore.loadSampleDataset('BILATERAL');
  } else {
    reconStore.loadSampleDataset('ALL');
  }
  emit('navigate', 'reconciliation');
}

function formatVnd(val?: number): string {
  return Number(val || 0).toLocaleString('vi-VN');
}

function formatTimestamp(isoStr?: string): string {
  if (!isoStr) return '';
  try {
    const d = new Date(isoStr);
    return d.toLocaleTimeString('vi-VN', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  } catch {
    return isoStr;
  }
}

function getAccountBorderClass(bankCode: BankCode): string {
  switch (bankCode) {
    case 'CITAD':
    case 'VCB':
      return 'border-emerald-200 hover:border-emerald-400';
    case 'NAPAS':
    case 'TCB':
      return 'border-sky-200 hover:border-sky-400';
    case 'BILATERAL':
    case 'BIDV':
      return 'border-amber-200 hover:border-amber-400';
    case 'SWIFT':
      return 'border-purple-200 hover:border-purple-400';
    default:
      return 'border-slate-200';
  }
}
</script>

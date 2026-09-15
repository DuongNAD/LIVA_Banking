<template>
  <div class="reconciliation-workbench space-y-4 animate-in fade-in duration-200">
    <!-- 1. Enterprise Workbench Header -->
    <div class="bg-white p-4 rounded-xl border border-slate-200 shadow-xs flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
      <div>
        <div class="flex items-center space-x-2.5">
          <div class="w-8 h-8 rounded-lg bg-slate-900 text-white flex items-center justify-center shadow-xs">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m16 16 3-8 3 8c-.87.65-1.92 1-3 1s-2.13-.35-3-1Z"/><path d="m2 16 3-8 3 8c-.87.65-1.92 1-3 1s-2.13-.35-3-1Z"/><path d="M7 21h10"/><path d="M12 3v18"/><path d="M3 7h2c2 0 5-1 7-2 2 1 5 2 7 2h2"/></svg>
          </div>
          <div>
            <h1 class="text-base sm:text-lg font-bold text-slate-900 tracking-tight flex items-center space-x-2">
              <span>Đối Soát Quyết Toán Liên Ngân Hàng</span>
              <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-slate-100 text-slate-700 border border-slate-200">3-Tier Engine</span>
            </h1>
            <p class="text-[11px] text-slate-500 hidden sm:block">
              Đối chiếu tự động giữa Sổ Nhật ký Core Banking nội bộ với Bảng kê quyết toán bù trừ NAPAS & CITAD
            </p>
          </div>
        </div>
      </div>

      <div class="flex items-center space-x-2 self-end sm:self-auto">
        <button
          type="button"
          class="px-3.5 py-2 text-xs font-bold rounded-lg bg-emerald-700 hover:bg-emerald-800 text-white transition shadow-xs flex items-center space-x-1.5 cursor-pointer whitespace-nowrap"
          :disabled="reconStore.isReconciling"
          @click="reconStore.executeReconciliation"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"/></svg>
          <span>{{ reconStore.isReconciling ? 'Đang đối soát...' : 'Chạy đối soát' }}</span>
        </button>
      </div>
    </div>

    <!-- 2. Functional Sub-Pages Navigation Bar -->
    <nav class="bg-white px-2.5 py-1.5 rounded-xl border border-slate-200 shadow-xs flex items-center justify-between gap-2 overflow-x-auto">
      <div class="flex items-center space-x-1 text-xs font-semibold">
        <!-- Sub-page 1: Sổ Đối Chiếu -->
        <button
          type="button"
          class="px-3 py-1.5 rounded-lg transition flex items-center space-x-1.5 cursor-pointer whitespace-nowrap"
          :class="activeSubTab === 'tables' ? 'bg-slate-900 text-white font-bold' : 'text-slate-600 hover:text-slate-900 hover:bg-slate-100'"
          @click="activeSubTab = 'tables'"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 3h18v18H3z"/><path d="M3 9h18"/><path d="M12 9v12"/></svg>
          <span>Sổ đối chiếu</span>
          <span class="ml-1 px-1.5 py-0.2 rounded-full text-[10px] font-mono opacity-80" :class="activeSubTab === 'tables' ? 'bg-slate-800 text-slate-200' : 'bg-slate-100 text-slate-600'">
            {{ reconStore.bankTransactions.length }}
          </span>
        </button>

        <!-- Sub-page 2: Xử Lý Tra Soát -->
        <button
          type="button"
          class="px-3 py-1.5 rounded-lg transition flex items-center space-x-1.5 cursor-pointer whitespace-nowrap"
          :class="activeSubTab === 'quarantine' ? 'bg-slate-900 text-white font-bold' : 'text-slate-600 hover:text-slate-900 hover:bg-slate-100'"
          @click="activeSubTab = 'quarantine'"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><line x1="12" x2="12" y1="9" y2="13"/><line x1="12" x2="12.01" y1="17" y2="17"/></svg>
          <span>Xử lý tra soát</span>
          <span
            v-if="reconStore.quarantinedItems.length > 0"
            class="ml-1 px-1.5 py-0.2 rounded-full text-[10px] font-bold"
            :class="activeSubTab === 'quarantine' ? 'bg-amber-400 text-slate-950' : 'bg-amber-500 text-white'"
          >
            {{ reconStore.quarantinedItems.length }}
          </span>
        </button>

        <!-- Sub-page 3: Báo Cáo Cân Đối -->
        <button
          type="button"
          class="px-3 py-1.5 rounded-lg transition flex items-center space-x-1.5 cursor-pointer whitespace-nowrap"
          :class="activeSubTab === 'overview' ? 'bg-slate-900 text-white font-bold' : 'text-slate-600 hover:text-slate-900 hover:bg-slate-100'"
          @click="activeSubTab = 'overview'"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="m9 12 2 2 4-4"/></svg>
          <span>Báo cáo cân đối</span>
        </button>

        <!-- Sub-page 4: Nạp Dữ Liệu -->
        <button
          type="button"
          class="px-3 py-1.5 rounded-lg transition flex items-center space-x-1.5 cursor-pointer whitespace-nowrap"
          :class="activeSubTab === 'import' ? 'bg-slate-900 text-white font-bold' : 'text-slate-600 hover:text-slate-900 hover:bg-slate-100'"
          @click="activeSubTab = 'import'"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" x2="12" y1="3" y2="15"/></svg>
          <span>Nạp dữ liệu</span>
        </button>

        <!-- Sub-page 5: Nhật Ký Kiểm Toán -->
        <button
          type="button"
          class="px-3 py-1.5 rounded-lg transition flex items-center space-x-1.5 cursor-pointer whitespace-nowrap"
          :class="activeSubTab === 'audit' ? 'bg-slate-900 text-white font-bold' : 'text-slate-600 hover:text-slate-900 hover:bg-slate-100'"
          @click="activeSubTab = 'audit'"
        >
          <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" x2="8" y1="13" y2="13"/><line x1="16" x2="8" y1="17" y2="17"/></svg>
          <span>Nhật ký kiểm toán</span>
          <span v-if="reconStore.reconciliationSummary" class="ml-1 px-1.5 py-0.2 rounded-full text-[10px] font-mono opacity-80" :class="activeSubTab === 'audit' ? 'bg-slate-800 text-slate-200' : 'bg-slate-100 text-slate-600'">
            {{ reconStore.reconciliationSummary.matches.length }}
          </span>
        </button>
      </div>

      <div class="hidden lg:flex items-center space-x-1.5 text-[11px] text-slate-500 font-medium">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
        <span>Kênh: <strong class="text-slate-700">Toàn hệ thống</strong></span>
      </div>
    </nav>

    <!-- ================================================================= -->
    <!-- SUB-PAGE 1: SỔ ĐỐI CHIẾU SONG SONG (Tables View) -->
    <!-- ================================================================= -->
    <div v-show="activeSubTab === 'tables'" class="space-y-4">
      <!-- Quick Filter & Summary Bar -->
      <div class="bg-white p-3 rounded-xl border border-slate-200 shadow-xs flex flex-wrap items-center justify-between gap-3 text-xs">
        <div class="flex items-center space-x-1.5">
          <span class="text-slate-500 font-medium">Lọc trạng thái:</span>
          <button
            type="button"
            class="px-2.5 py-1 rounded-md text-[11px] font-semibold transition cursor-pointer"
            :class="tableFilter === 'ALL' ? 'bg-slate-900 text-white' : 'bg-slate-100 text-slate-700 hover:bg-slate-200'"
            @click="tableFilter = 'ALL'"
          >
            Tất cả ({{ reconStore.bankTransactions.length }})
          </button>
          <button
            type="button"
            class="px-2.5 py-1 rounded-md text-[11px] font-semibold transition cursor-pointer"
            :class="tableFilter === 'MATCHED' ? 'bg-emerald-700 text-white' : 'bg-emerald-50 text-emerald-800 hover:bg-emerald-100'"
            @click="tableFilter = 'MATCHED'"
          >
            Đã khớp
          </button>
          <button
            type="button"
            class="px-2.5 py-1 rounded-md text-[11px] font-semibold transition cursor-pointer"
            :class="tableFilter === 'UNMATCHED' ? 'bg-amber-700 text-white' : 'bg-amber-50 text-amber-800 hover:bg-amber-100'"
            @click="tableFilter = 'UNMATCHED'"
          >
            Chưa khớp / Treo ({{ reconStore.quarantinedItems.length }})
          </button>
        </div>

        <div class="flex items-center space-x-2">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Tìm theo mã GD, đối tác, số tiền..."
            class="px-3 py-1 rounded-lg border border-slate-300 text-xs w-56 focus:outline-none focus:ring-1 focus:ring-slate-900"
          />
        </div>
      </div>

      <!-- Side-by-Side Comparative Grid -->
      <div class="grid grid-cols-1 lg:grid-cols-12 gap-4">
        <!-- LEFT PANE: Bank Statement Transactions (6 cols) -->
        <div class="lg:col-span-6 bg-white rounded-xl border border-slate-200 shadow-xs overflow-hidden flex flex-col">
          <div class="p-3.5 border-b border-slate-200 bg-slate-50 flex items-center justify-between">
            <div class="flex items-center space-x-2">
              <svg class="w-4 h-4 text-slate-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 21h18"/><path d="M3 10h18"/><path d="M5 6l7-3 7 3"/><path d="M4 10v11"/><path d="M20 10v11"/><path d="M8 14v3"/><path d="M12 14v3"/><path d="M16 14v3"/></svg>
              <h3 class="text-xs font-bold uppercase tracking-wider text-slate-800">
                Bảng Kê Quyết Toán Bù Trừ Liên Ngân Hàng
              </h3>
            </div>
            <span class="text-xs text-slate-500 font-mono">{{ filteredBankTransactions.length }} Giao dịch</span>
          </div>

          <!-- Table Container -->
          <div class="overflow-x-auto max-h-[560px] overflow-y-auto">
            <table class="w-full text-left text-xs">
              <thead class="bg-slate-100/80 text-slate-600 font-semibold border-b border-slate-200 sticky top-0">
                <tr>
                  <th class="p-2.5">Ngày / Mã GD</th>
                  <th class="p-2.5">Nội Dung / Đối Tác</th>
                  <th class="p-2.5 text-right">Phát Sinh (VND)</th>
                  <th class="p-2.5 text-right">Số Dư</th>
                  <th class="p-2.5 text-center">Trạng Thái</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-slate-100">
                <tr
                  v-for="tx in filteredBankTransactions"
                  :key="tx.id"
                  class="hover:bg-slate-50/80 transition"
                >
                  <td class="p-2.5 align-top">
                    <div class="font-medium text-slate-800 whitespace-nowrap">{{ tx.date }}</div>
                    <div class="text-[10px] font-mono text-slate-400">{{ tx.txCode }}</div>
                  </td>
                  <td class="p-2.5 align-top max-w-[200px]">
                    <div class="text-slate-800 font-medium line-clamp-2" :title="tx.narration">
                      {{ tx.narration }}
                    </div>
                    <div v-if="tx.counterparty" class="text-[10px] text-slate-500 font-semibold">
                      {{ tx.counterparty }}
                    </div>
                  </td>
                  <td class="p-2.5 align-top text-right whitespace-nowrap font-mono">
                    <span v-if="tx.credit > 0" class="text-emerald-700 font-bold">
                      +{{ formatVnd(tx.credit) }}
                    </span>
                    <span v-else-if="tx.debit > 0" class="text-rose-700 font-bold">
                      -{{ formatVnd(tx.debit) }}
                    </span>
                  </td>
                  <td class="p-2.5 align-top text-right whitespace-nowrap font-mono text-slate-700">
                    {{ formatVnd(tx.balance) }}
                  </td>
                  <td class="p-2.5 align-top text-center">
                    <span
                      class="inline-block px-2 py-0.5 rounded text-[10px] font-bold"
                      :class="getMatchStatusBadge(tx.id)"
                    >
                      {{ getMatchStatusText(tx.id) }}
                    </span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <!-- RIGHT PANE: Accounting Internal Ledger (6 cols) -->
        <div class="lg:col-span-6 bg-white rounded-xl border border-slate-200 shadow-xs overflow-hidden flex flex-col">
          <div class="p-3.5 border-b border-slate-200 bg-slate-50 flex items-center justify-between">
            <div class="flex items-center space-x-2">
              <svg class="w-4 h-4 text-slate-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1-2.5-2.5Z"/><path d="M6 6h10"/><path d="M6 10h10"/></svg>
              <h3 class="text-xs font-bold uppercase tracking-wider text-slate-800">
                Sổ Nhật Ký Giao Dịch Core Banking Nội Bộ
              </h3>
            </div>
            <span class="text-xs text-slate-500 font-mono">{{ filteredLedgerEntries.length }} Bản ghi</span>
          </div>

          <!-- Table Container -->
          <div class="overflow-x-auto max-h-[560px] overflow-y-auto">
            <table class="w-full text-left text-xs">
              <thead class="bg-slate-100/80 text-slate-600 font-semibold border-b border-slate-200 sticky top-0">
                <tr>
                  <th class="p-2.5">Số HĐ / Ngày</th>
                  <th class="p-2.5">Khách Hàng / Đối Tác</th>
                  <th class="p-2.5">Diễn Giải Hạch Toán</th>
                  <th class="p-2.5 text-right">Số Tiền (VND)</th>
                  <th class="p-2.5 text-center">Đối Chiếu</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-slate-100">
                <tr
                  v-for="entry in filteredLedgerEntries"
                  :key="entry.id"
                  class="hover:bg-slate-50/80 transition"
                >
                  <td class="p-2.5 align-top whitespace-nowrap">
                    <div class="font-bold text-slate-900 font-mono">{{ entry.docNo }}</div>
                    <div class="text-[10px] text-slate-400">{{ entry.entryDate }}</div>
                  </td>
                  <td class="p-2.5 align-top max-w-[160px]">
                    <span class="font-medium text-slate-800 line-clamp-1" :title="entry.partnerName">
                      {{ entry.partnerName }}
                    </span>
                  </td>
                  <td class="p-2.5 align-top max-w-[160px]">
                    <span class="text-slate-600 line-clamp-2" :title="entry.description">
                      {{ entry.description }}
                    </span>
                  </td>
                  <td class="p-2.5 align-top text-right whitespace-nowrap font-mono font-bold text-slate-900">
                    {{ formatVnd(entry.amount) }}
                  </td>
                  <td class="p-2.5 align-top text-center">
                    <span
                      class="inline-block px-2 py-0.5 rounded text-[10px] font-bold"
                      :class="getLedgerMatchStatusBadge(entry.id)"
                    >
                      {{ getLedgerMatchStatusText(entry.id) }}
                    </span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </div>

    <!-- ================================================================= -->
    <!-- SUB-PAGE 2: HÀNG ĐỢI XỬ LÝ NGOẠI LỆ HITL (Quarantine View) -->
    <!-- ================================================================= -->
    <div v-show="activeSubTab === 'quarantine'" class="bg-white rounded-xl border border-slate-200 shadow-xs p-5 space-y-4">
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 pb-3 border-b border-slate-100">
        <div class="flex items-center space-x-2.5">
          <div class="w-8 h-8 rounded-lg bg-amber-500/10 text-amber-700 flex items-center justify-center">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><line x1="12" x2="12" y1="9" y2="13"/><line x1="12" x2="12.01" y1="17" y2="17"/></svg>
          </div>
          <div>
            <h2 class="text-sm font-bold text-slate-900">
              Hàng Đợi Xử Lý Ngoại Lệ & Giao Dịch Treo (Exception Queue - HITL)
            </h2>
            <p class="text-xs text-slate-500">
              Xử lý các giao dịch cách ly chưa khớp tự động: Lệch phí chuyển tiền, trễ phiên bù trừ hoặc thiếu hóa đơn đối ứng
            </p>
          </div>
        </div>

        <div class="flex items-center space-x-2">
          <span
            class="px-3 py-1 rounded-full text-xs font-bold"
            :class="reconStore.quarantinedItems.length > 0 ? 'bg-amber-100 text-amber-900' : 'bg-emerald-100 text-emerald-800'"
          >
            {{ reconStore.quarantinedItems.length }} Giao Dịch Cần Xử Lý
          </span>
        </div>
      </div>

      <!-- Empty state when no quarantined items -->
      <div
        v-if="reconStore.quarantinedItems.length === 0"
        class="p-8 bg-slate-50 rounded-xl border border-slate-200 text-center space-y-2"
      >
        <div class="w-10 h-10 rounded-full bg-emerald-100 text-emerald-700 flex items-center justify-center mx-auto">
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
        </div>
        <p class="text-xs font-bold text-slate-800">Không có giao dịch nào bị treo trong hàng đợi cách ly!</p>
        <p class="text-[11px] text-slate-500">Tất cả giao dịch đã được đối khớp 3 tầng tự động hoặc đã được giải tỏa hoàn tất.</p>
      </div>

      <!-- Quarantined items list -->
      <div v-else class="space-y-3">
        <div
          v-for="item in reconStore.quarantinedItems"
          :key="item.txId"
          class="p-4 rounded-xl border border-amber-200 bg-amber-50/20 hover:bg-amber-50/40 transition space-y-3"
        >
          <!-- Top metadata row -->
          <div class="flex flex-wrap items-center justify-between gap-2 text-xs">
            <div class="flex items-center space-x-2">
              <span class="px-2 py-0.5 rounded font-mono font-bold bg-amber-100 text-amber-900 border border-amber-300 text-[10px]">
                CHỜ XỬ LÝ NGOẠI LỆ
              </span>
              <span class="text-[11px] text-slate-500">Mã GD: <strong class="text-slate-800 font-mono">{{ item.rawTransaction.txCode }}</strong></span>
            </div>

            <div class="text-right text-[11px] text-slate-500 font-mono flex items-center space-x-2">
              <span>Ngày: {{ item.rawTransaction.date }}</span>
              <span class="px-1.5 py-0.2 rounded bg-blue-50 text-blue-700 text-[10px]" :title="'Token: ' + item.hitlToken">Hạn: 15p</span>
            </div>
          </div>

          <!-- Transaction details & suspected cause grid -->
          <div class="grid grid-cols-1 md:grid-cols-12 gap-4 text-xs">
            <!-- Col 1: Transaction details (5 cols) -->
            <div class="md:col-span-5 space-y-1">
              <span class="text-slate-400 text-[10px] block font-medium">Chi Tiết Giao Dịch Ngân Hàng:</span>
              <div class="font-bold text-sm text-slate-900 font-mono">
                {{ formatVnd(item.amount) }} VND
              </div>
              <p class="text-slate-700 font-medium line-clamp-2" :title="item.rawTransaction.narration">
                {{ item.rawTransaction.narration }}
              </p>
              <div v-if="item.rawTransaction.counterparty" class="text-[11px] text-slate-500">
                Đối tác: <strong class="text-slate-700">{{ item.rawTransaction.counterparty }}</strong>
              </div>
            </div>

            <!-- Col 2: Suspected cause & variance (4 cols) -->
            <div class="md:col-span-4 space-y-1.5 p-3 rounded-lg bg-white border border-slate-200">
              <span class="text-slate-400 text-[10px] block font-medium">Phân Tích Nguyên Nhân Nghi Vấn:</span>
              <div>
                <span
                  class="inline-block px-2 py-0.5 rounded text-[10px] font-bold border"
                  :class="reconStore.getSuspectedCause(item).badgeClass"
                >
                  {{ reconStore.getSuspectedCause(item).description }}
                </span>
              </div>
              <div class="text-[11px] text-slate-600 font-mono">
                Số tiền chênh lệch: <strong class="text-rose-600">{{ formatVnd(item.amount) }} VND</strong>
              </div>
              <div v-if="reconStore.escalatedTxMap[item.txId]" class="text-[10px] font-semibold text-indigo-700">
                Đã chuyển KSV: "{{ reconStore.escalatedTxMap[item.txId].remarks }}"
              </div>
            </div>

            <!-- Col 3: Interactive Resolution Actions (3 cols) -->
            <div class="md:col-span-3 flex flex-col justify-center space-y-1.5">
              <button
                type="button"
                class="w-full px-3 py-1.5 text-xs font-bold rounded-lg bg-emerald-700 hover:bg-emerald-800 text-white transition shadow-xs flex items-center justify-center space-x-1.5 cursor-pointer"
                @click="openOverrideModal(item)"
              >
                <span>Khớp thủ công (Manual Match)</span>
              </button>

              <button
                type="button"
                class="w-full px-3 py-1.5 text-xs font-bold rounded-lg bg-blue-700 hover:bg-blue-800 text-white transition shadow-xs flex items-center justify-center space-x-1.5 cursor-pointer"
                @click="openAllocateFeeModal(item)"
              >
                <span>Hạch toán phí dịch vụ (TK 6425)</span>
              </button>

              <button
                type="button"
                class="w-full px-3 py-1.5 text-xs font-bold rounded-lg bg-slate-900 hover:bg-slate-800 text-white transition shadow-xs flex items-center justify-center space-x-1.5 cursor-pointer"
                @click="openEscalateModal(item)"
              >
                <span>Trình duyệt tra soát (Escalate)</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- ================================================================= -->
    <!-- SUB-PAGE 3: TỔNG QUAN CÂN BẰNG KÉP & PHÂN TẦNG (Overview View) -->
    <!-- ================================================================= -->
    <div v-show="activeSubTab === 'overview'" class="space-y-4">
      <div class="grid grid-cols-1 lg:grid-cols-12 gap-4">
        <!-- Balance Invariant Card (7 cols) -->
        <div class="lg:col-span-7 bg-white p-5 rounded-xl border border-slate-200 shadow-xs space-y-3">
          <div class="flex items-center justify-between pb-2.5 border-b border-slate-100">
            <div class="flex items-center space-x-2">
              <div class="w-7 h-7 rounded-lg bg-slate-100 text-slate-700 flex items-center justify-center">
                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="m9 12 2 2 4-4"/></svg>
              </div>
              <h3 class="text-xs font-bold uppercase tracking-wider text-slate-800">
                Bảo Chứng Cân Bằng Kế Toán Kép (Double-Entry Balance Invariant)
              </h3>
            </div>
            <span
              class="px-2.5 py-0.5 rounded-full text-xs font-bold flex items-center space-x-1"
              :class="reconStore.isBalanced ? 'bg-emerald-100 text-emerald-800' : 'bg-rose-100 text-rose-800'"
            >
              <span>{{ reconStore.isBalanced ? 'Cân Bằng Chuẩn Tuyệt Đối' : 'Phát Hiện Lệch Số Dư' }}</span>
            </span>
          </div>

          <!-- Formula Numbers Grid -->
          <div class="grid grid-cols-2 sm:grid-cols-4 gap-2.5 text-center text-xs">
            <div class="p-3 rounded-lg bg-slate-50 border border-slate-200">
              <span class="text-slate-500 block text-[10px] uppercase font-semibold">Số dư đầu kỳ:</span>
              <span class="font-bold font-mono text-slate-900 text-xs">
                {{ formatVnd(reconStore.balanceInvariantReport?.openingBalance) }}
              </span>
            </div>

            <div class="p-3 rounded-lg bg-emerald-50/70 border border-emerald-200">
              <span class="text-emerald-800 block text-[10px] uppercase font-semibold">+ Phát sinh Có:</span>
              <span class="font-bold font-mono text-emerald-900 text-xs">
                {{ formatVnd(reconStore.balanceInvariantReport?.totalCredit) }}
              </span>
            </div>

            <div class="p-3 rounded-lg bg-rose-50/70 border border-rose-200">
              <span class="text-rose-800 block text-[10px] uppercase font-semibold">- Phát sinh Nợ:</span>
              <span class="font-bold font-mono text-rose-900 text-xs">
                {{ formatVnd(reconStore.balanceInvariantReport?.totalDebit) }}
              </span>
            </div>

            <div class="p-3 rounded-lg bg-blue-50/70 border border-blue-200">
              <span class="text-blue-800 block text-[10px] uppercase font-semibold">= Số dư cuối kỳ:</span>
              <span class="font-bold font-mono text-blue-900 text-xs">
                {{ formatVnd(reconStore.balanceInvariantReport?.closingBalance) }}
              </span>
            </div>
          </div>

          <div class="flex items-center justify-between text-xs text-slate-500 pt-1">
            <span class="text-slate-500 text-[11px]">Định dạng tiền tệ VND chuẩn số học nguyên 64-bit</span>
            <span class="font-mono text-xs" :class="reconStore.discrepancyAmount === 0 ? 'text-emerald-700 font-bold' : 'text-rose-700 font-bold'">
              Chênh lệch: {{ formatVnd(reconStore.discrepancyAmount) }} VND
            </span>
          </div>
        </div>

        <!-- 3-Tier Match Breakdown Badges (5 cols) -->
        <div class="lg:col-span-5 bg-white p-5 rounded-xl border border-slate-200 shadow-xs space-y-3 flex flex-col justify-between">
          <div class="flex items-center justify-between pb-2.5 border-b border-slate-100">
            <h3 class="text-xs font-bold uppercase tracking-wider text-slate-800">
              Phân Tầng Đối Soát (3-Tier Resolution)
            </h3>
            <span class="text-xs font-bold text-emerald-700 font-mono">
              Tỷ lệ khớp: {{ reconStore.matchRate.toFixed(1) }}%
            </span>
          </div>

          <div class="grid grid-cols-2 gap-2 text-xs">
            <!-- Tier 1 -->
            <div class="p-2.5 rounded-lg bg-emerald-50 border border-emerald-200 flex items-center justify-between">
              <div>
                <span class="font-bold text-emerald-900 block text-xs">Tier 1: Tuyệt Đối</span>
                <span class="text-[10px] text-emerald-700">1:1 Khớp số tiền & mã</span>
              </div>
              <span class="text-base font-bold font-mono text-emerald-800">{{ reconStore.tier1Count }}</span>
            </div>

            <!-- Tier 2 -->
            <div class="p-2.5 rounded-lg bg-blue-50 border border-blue-200 flex items-center justify-between">
              <div>
                <span class="font-bold text-blue-900 block text-xs">Tier 2: Mờ Heuristic</span>
                <span class="text-[10px] text-blue-700">Bóc tách phí & ±72h</span>
              </div>
              <span class="text-base font-bold font-mono text-blue-800">{{ reconStore.tier2Count }}</span>
            </div>

            <!-- Tier 3 -->
            <div class="p-2.5 rounded-lg bg-purple-50 border border-purple-200 flex items-center justify-between">
              <div>
                <span class="font-bold text-purple-900 block text-xs">Tier 3: Phân Tách</span>
                <span class="text-[10px] text-purple-700">Ghép 1:N / N:1 solver</span>
              </div>
              <span class="text-base font-bold font-mono text-purple-800">{{ reconStore.tier3Count }}</span>
            </div>

            <!-- HITL Quarantine -->
            <div class="p-2.5 rounded-lg bg-amber-50 border border-amber-200 flex items-center justify-between">
              <div>
                <span class="font-bold text-amber-900 block text-xs">Quarantine HITL</span>
                <span class="text-[10px] text-amber-700">Hàng đợi xử lý tay</span>
              </div>
              <span class="text-base font-bold font-mono text-amber-800">{{ reconStore.hitlQuarantineCount }}</span>
            </div>
          </div>

          <div class="text-[11px] text-slate-500 flex items-center justify-between pt-1">
            <span>Tổng sao kê: <strong class="text-slate-800">{{ reconStore.totalBankTxCount }}</strong></span>
            <span>Bản ghi sổ cái: <strong class="text-slate-800">{{ reconStore.totalLedgerCount }}</strong></span>
          </div>
        </div>
      </div>
    </div>

    <!-- ================================================================= -->
    <!-- SUB-PAGE 4: NẠP DỮ LIỆU & BENCHMARK (Import View) -->
    <!-- ================================================================= -->
    <div v-show="activeSubTab === 'import'" class="space-y-4">
      <!-- Standard Benchmark Bar (keeps required string for test contract) -->
      <div class="bg-white p-5 rounded-xl border border-slate-200 shadow-xs space-y-3">
        <div class="flex items-center justify-between border-b border-slate-100 pb-2.5">
          <div class="flex items-center space-x-2">
            <svg class="w-4 h-4 text-slate-700" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1-2.5-2.5Z"/></svg>
            <h3 class="text-xs font-bold uppercase tracking-wider text-slate-800">
              Bộ Dữ Liệu (Standard Benchmark):
            </h3>
            <span class="text-xs text-slate-500">Tải nhanh bảng kê quyết toán thực tế từ kho chuẩn</span>
          </div>
          <span class="text-xs font-mono text-emerald-700 font-semibold">
            Đang nạp: {{ reconStore.activeDatasetName }}
          </span>
        </div>

        <div class="flex flex-wrap items-center gap-2 text-xs">
          <button
            type="button"
            class="px-3 py-1.5 rounded-lg bg-emerald-800 hover:bg-emerald-700 text-white font-semibold transition border border-emerald-700 flex items-center space-x-1.5 cursor-pointer"
            @click="handleSelectSample('CITAD')"
          >
            <span>1. Kênh CITAD (NHNN) Excel (.xlsx)</span>
          </button>

          <button
            type="button"
            class="px-3 py-1.5 rounded-lg bg-sky-800 hover:bg-sky-700 text-white font-semibold transition border border-sky-700 flex items-center space-x-1.5 cursor-pointer"
            @click="handleSelectSample('NAPAS')"
          >
            <span>2. Kênh NAPAS 24/7 CSV (BOM)</span>
          </button>

          <button
            type="button"
            class="px-3 py-1.5 rounded-lg bg-amber-800 hover:bg-amber-700 text-white font-semibold transition border border-amber-700 flex items-center space-x-1.5 cursor-pointer"
            @click="handleSelectSample('BILATERAL')"
          >
            <span>3. Kênh Song Phương & Tra Soát (.csv)</span>
          </button>

          <button
            type="button"
            class="px-3 py-1.5 rounded-lg bg-purple-800 hover:bg-purple-700 text-white font-semibold transition border border-purple-700 flex items-center space-x-1.5 cursor-pointer"
            @click="handleSelectSample('ALL')"
          >
            <span>4. Quyết Toán Toàn Kênh (CITAD + NAPAS + Song Phương)</span>
          </button>

          <button
            type="button"
            class="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-white font-semibold transition border border-slate-700 flex items-center space-x-1.5 cursor-pointer"
            @click="handleSelectSample('SHEETS')"
          >
            <span>5. Dán Bảng Tính Tra Soát</span>
          </button>
        </div>
      </div>

      <!-- File Dropzone & Paste Tabs Container -->
      <div class="bg-white p-5 rounded-xl border border-slate-200 shadow-xs space-y-3">
        <div class="flex items-center space-x-2 border-b border-slate-100 pb-2.5">
          <button
            type="button"
            class="px-3 py-1.5 text-xs font-bold rounded-lg transition cursor-pointer"
            :class="inputTab === 'FILE' ? 'bg-slate-900 text-white' : 'bg-slate-100 text-slate-700 hover:bg-slate-200'"
            @click="inputTab = 'FILE'"
          >
            Kéo Thả Tệp (.xlsx, .csv)
          </button>
          <button
            type="button"
            class="px-3 py-1.5 text-xs font-bold rounded-lg transition cursor-pointer"
            :class="inputTab === 'PASTE' ? 'bg-slate-900 text-white' : 'bg-slate-100 text-slate-700 hover:bg-slate-200'"
            @click="inputTab = 'PASTE'"
          >
            Dán Bảng Tính (Clipboard)
          </button>
        </div>

        <!-- Tab 1: File Dropzone -->
        <div
          v-if="inputTab === 'FILE'"
          class="p-8 rounded-xl border-2 border-dashed transition-all text-center flex flex-col justify-center items-center cursor-pointer min-h-[140px]"
          :class="isDragging ? 'border-emerald-500 bg-emerald-50/30' : 'border-slate-300 hover:border-slate-400 bg-slate-50/50'"
          @dragover.prevent="isDragging = true"
          @dragleave.prevent="isDragging = false"
          @drop.prevent="handleFileDrop"
          @click="triggerFileInput"
        >
          <input
            ref="fileInputRef"
            type="file"
            class="hidden"
            accept=".xlsx,.xls,.csv,.tsv,.txt"
            @change="handleFileSelect"
          />
          <div class="w-10 h-10 rounded-full bg-slate-200/70 flex items-center justify-center text-slate-700 mb-2">
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" x2="12" y1="3" y2="15"/></svg>
          </div>
          <p class="text-xs font-bold text-slate-800">
            Kéo thả tệp sao kê quyết toán bù trừ vào đây hoặc <span class="text-blue-600 underline">chọn tệp từ máy</span>
          </p>
          <p class="text-[11px] text-slate-500 mt-1">
            Định dạng hỗ trợ: CITAD Excel (.xlsx), NAPAS 24/7 (CSV BOM), Sổ phụ Nostro/Vostro (.csv, .tsv, .txt)
          </p>
        </div>

        <!-- Tab 2: Paste Area -->
        <div v-else-if="inputTab === 'PASTE'" class="space-y-2">
          <textarea
            v-model="pastedText"
            rows="5"
            class="w-full text-xs font-mono p-3 rounded-xl border border-slate-300 bg-slate-50 focus:bg-white focus:ring-1 focus:ring-slate-900 focus:outline-none"
            placeholder="Dán dữ liệu quyết toán từ hệ thống bù trừ, bảng tính tra soát liên ngân hàng tại đây..."
          ></textarea>
          <div class="flex items-center justify-between pt-1">
            <span class="text-[11px] text-slate-500">Hệ thống tự động nhận diện phân cách (\t, ;, ,, |) và chuẩn hóa số học VND</span>
            <button
              type="button"
              class="px-4 py-2 text-xs font-bold rounded-lg bg-slate-900 hover:bg-slate-800 text-white transition cursor-pointer"
              :disabled="!pastedText.trim()"
              @click="applyPastedText"
            >
              Bóc Tách & Nhập Dữ Liệu
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- ================================================================= -->
    <!-- SUB-PAGE 5: NHẬT KÝ KHỚP LỆNH & KIỂM TOÁN (Audit View) -->
    <!-- ================================================================= -->
    <div v-show="activeSubTab === 'audit'" class="bg-white rounded-xl border border-slate-200 shadow-xs p-5 space-y-4">
      <div class="flex items-center justify-between pb-2.5 border-b border-slate-100">
        <div class="flex items-center space-x-2.5">
          <div class="w-8 h-8 rounded-lg bg-slate-100 text-slate-700 flex items-center justify-center">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" x2="8" y1="13" y2="13"/><line x1="16" x2="8" y1="17" y2="17"/></svg>
          </div>
          <div>
            <h3 class="text-sm font-bold text-slate-900">Chi Tiết Bằng Chứng Khớp Quyết Toán & Bóc Tách Phí Chuyển Mạch</h3>
            <p class="text-xs text-slate-500">Biên bản kiểm toán khớp lệnh kèm độ tin cậy toán học và giải trình chi tiết</p>
          </div>
        </div>
        <span v-if="reconStore.reconciliationSummary" class="text-xs text-slate-500 font-mono">
          Tổng cộng: {{ reconStore.reconciliationSummary.matches.length }} kết quả
        </span>
      </div>

      <div v-if="reconStore.reconciliationSummary" class="space-y-2.5 max-h-[560px] overflow-y-auto">
        <div
          v-for="match in reconStore.reconciliationSummary.matches"
          :key="match.matchId"
          class="p-3.5 rounded-xl border text-xs flex flex-col md:flex-row md:items-center md:justify-between gap-3"
          :class="getMatchRowClass(match.tier)"
        >
          <div class="space-y-1">
            <div class="flex items-center space-x-2">
              <span class="px-2 py-0.5 rounded text-[10px] font-bold" :class="getTierPillClass(match.tier)">
                {{ match.tier }}
              </span>
              <span class="font-semibold text-slate-800">
                Độ tin cậy: {{ (match.confidence * 100).toFixed(0) }}%
              </span>
              <span v-if="match.feeAmount > 0" class="px-2 py-0.5 rounded text-[10px] font-bold bg-amber-100 text-amber-900">
                Phí TK 6425: {{ formatVnd(match.feeAmount) }} VND
              </span>
            </div>
            <p class="text-slate-700 font-medium">
              {{ match.explanation }}
            </p>
          </div>

          <div class="text-right whitespace-nowrap self-end md:self-auto font-mono">
            <span class="text-slate-400 text-[10px] block">Số tiền khớp:</span>
            <span class="font-bold text-sm text-slate-900">{{ formatVnd(match.matchedAmount) }} VND</span>
          </div>
        </div>
      </div>
      <div v-else class="p-8 text-center text-slate-500 text-xs">
        Chưa có dữ liệu đối khớp. Vui lòng bấm "Chạy Đối Soát Tự Động" để xem nhật ký.
      </div>
    </div>

    <!-- ================================================================= -->
    <!-- MODALS (Override, Fee Allocation, Escalate to Checker) -->
    <!-- ================================================================= -->

    <!-- MODAL 1: Manual Match Override Modal -->
    <div
      v-if="activeModal === 'OVERRIDE' && selectedQuarantineItem"
      class="fixed inset-0 z-50 overflow-y-auto bg-slate-900/60 backdrop-blur-xs flex items-center justify-center p-4"
    >
      <div class="bg-white rounded-2xl shadow-2xl max-w-lg w-full border border-slate-200 overflow-hidden flex flex-col animate-in fade-in zoom-in-95 duration-150">
        <div class="px-5 py-4 bg-emerald-800 text-white flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <h3 class="text-sm font-bold">Khớp Thủ Công Ngoại Lệ (Manual Match)</h3>
          </div>
          <button type="button" class="text-emerald-200 hover:text-white cursor-pointer p-1 rounded-lg" @click="closeModal">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18M6 6l12 12"/></svg>
          </button>
        </div>

        <div class="p-5 space-y-4 text-xs text-slate-800">
          <div class="p-3 bg-slate-50 rounded-xl border border-slate-200 space-y-1">
            <span class="text-slate-500 font-medium">Giao dịch ngân hàng cần đối ứng:</span>
            <div class="font-mono font-bold text-slate-900 text-sm">
              {{ formatVnd(selectedQuarantineItem.amount) }} VND ({{ selectedQuarantineItem.rawTransaction.txCode }})
            </div>
            <p class="text-slate-600">{{ selectedQuarantineItem.rawTransaction.narration }}</p>
          </div>

          <div class="space-y-1.5">
            <label class="block font-bold text-slate-700">Chọn Hóa Đơn Sổ Cái Đối Ứng:</label>
            <select
              v-model="selectedLedgerId"
              class="w-full p-2.5 rounded-lg border border-slate-300 bg-white font-mono text-xs"
            >
              <option value="">-- Không liên kết hóa đơn (Khớp trực tiếp) --</option>
              <option
                v-for="ledger in reconStore.unallocatedLedgers"
                :key="ledger.id"
                :value="ledger.id"
              >
                {{ ledger.docNo }} - {{ ledger.partnerName }} ({{ formatVnd(ledger.amount) }} VND)
              </option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="block font-bold text-slate-700">Biên Bản Giải Trình Đối Chiếu (Audit Trail):</label>
            <textarea
              v-model="overrideRemarks"
              rows="2"
              class="w-full p-2.5 rounded-lg border border-slate-300 text-xs focus:ring-1 focus:ring-slate-900 focus:outline-none"
              placeholder="Ghi rõ căn cứ chứng từ và lý do đối chiếu thủ công..."
            ></textarea>
          </div>
        </div>

        <div class="px-5 py-3 bg-slate-50 border-t border-slate-200 flex justify-end space-x-2">
          <button type="button" class="px-4 py-2 rounded-lg bg-slate-200 text-slate-700 hover:bg-slate-300 font-semibold text-xs cursor-pointer" @click="closeModal">Hủy</button>
          <button type="button" class="px-4 py-2 rounded-lg bg-emerald-700 hover:bg-emerald-800 text-white font-bold text-xs shadow-xs cursor-pointer" @click="confirmManualMatch">Xác Nhận Khớp Thủ Công</button>
        </div>
      </div>
    </div>

    <!-- MODAL 2: Allocate Bank Fee Modal -->
    <div
      v-if="activeModal === 'ALLOCATE_FEE' && selectedQuarantineItem"
      class="fixed inset-0 z-50 overflow-y-auto bg-slate-900/60 backdrop-blur-xs flex items-center justify-center p-4"
    >
      <div class="bg-white rounded-2xl shadow-2xl max-w-lg w-full border border-slate-200 overflow-hidden flex flex-col animate-in fade-in zoom-in-95 duration-150">
        <div class="px-5 py-4 bg-blue-800 text-white flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <h3 class="text-sm font-bold">Hạch Toán Chi Phí Ngân Hàng (TK 6425)</h3>
          </div>
          <button type="button" class="text-blue-200 hover:text-white cursor-pointer p-1 rounded-lg" @click="closeModal">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18M6 6l12 12"/></svg>
          </button>
        </div>

        <div class="p-5 space-y-4 text-xs text-slate-800">
          <div class="p-3 bg-slate-50 rounded-xl border border-slate-200 space-y-1">
            <span class="text-slate-500 font-medium">Giao dịch phát sinh:</span>
            <div class="font-mono font-bold text-slate-900 text-sm">
              {{ formatVnd(selectedQuarantineItem.amount) }} VND
            </div>
            <p class="text-slate-600">{{ selectedQuarantineItem.rawTransaction.narration }}</p>
          </div>

          <div class="space-y-1.5">
            <label class="block font-bold text-slate-700">Số Tiền Phí Phân Bổ (VND):</label>
            <input
              v-model.number="feeAmountInput"
              type="number"
              class="w-full p-2.5 rounded-lg border border-slate-300 font-mono font-bold text-xs"
            />
            <div class="flex items-center space-x-2 pt-1 text-[10px] text-slate-500">
              <span>Gợi ý mức phí chuẩn:</span>
              <button type="button" class="px-2 py-0.5 rounded bg-slate-100 hover:bg-slate-200 cursor-pointer" @click="feeAmountInput = 1100">1.100đ</button>
              <button type="button" class="px-2 py-0.5 rounded bg-slate-100 hover:bg-slate-200 cursor-pointer" @click="feeAmountInput = 2200">2.200đ</button>
              <button type="button" class="px-2 py-0.5 rounded bg-slate-100 hover:bg-slate-200 cursor-pointer" @click="feeAmountInput = 5500">5.500đ</button>
              <button type="button" class="px-2 py-0.5 rounded bg-slate-100 hover:bg-slate-200 cursor-pointer" @click="feeAmountInput = 11000">11.000đ</button>
            </div>
          </div>

          <div class="space-y-1.5">
            <label class="block font-bold text-slate-700">Ghi Chú Hạch Toán:</label>
            <input
              v-model="feeRemarks"
              type="text"
              class="w-full p-2.5 rounded-lg border border-slate-300 text-xs"
            />
          </div>
        </div>

        <div class="px-5 py-3 bg-slate-50 border-t border-slate-200 flex justify-end space-x-2">
          <button type="button" class="px-4 py-2 rounded-lg bg-slate-200 text-slate-700 hover:bg-slate-300 font-semibold text-xs cursor-pointer" @click="closeModal">Hủy</button>
          <button type="button" class="px-4 py-2 rounded-lg bg-blue-700 hover:bg-blue-800 text-white font-bold text-xs shadow-xs cursor-pointer" @click="confirmAllocateFee">Xác Nhận Hạch Toán TK 6425</button>
        </div>
      </div>
    </div>

    <!-- MODAL 3: Escalate to Checker Modal -->
    <div
      v-if="activeModal === 'ESCALATE' && selectedQuarantineItem"
      class="fixed inset-0 z-50 overflow-y-auto bg-slate-900/60 backdrop-blur-xs flex items-center justify-center p-4"
    >
      <div class="bg-white rounded-2xl shadow-2xl max-w-lg w-full border border-slate-200 overflow-hidden flex flex-col animate-in fade-in zoom-in-95 duration-150">
        <div class="px-5 py-4 bg-slate-900 text-white flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <h3 class="text-sm font-bold">Chuyển Kiểm Soát Viên (Escalate to Checker)</h3>
          </div>
          <button type="button" class="text-slate-400 hover:text-white cursor-pointer p-1 rounded-lg" @click="closeModal">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18M6 6l12 12"/></svg>
          </button>
        </div>

        <div class="p-5 space-y-4 text-xs text-slate-800">
          <div class="p-3 bg-slate-50 rounded-xl border border-slate-200 space-y-1">
            <span class="text-slate-500 font-medium">Giao dịch ngoại lệ chuyển thẩm định:</span>
            <div class="font-mono font-bold text-slate-900 text-sm">
              {{ formatVnd(selectedQuarantineItem.amount) }} VND ({{ selectedQuarantineItem.rawTransaction.txCode }})
            </div>
            <p class="text-slate-600">{{ selectedQuarantineItem.rawTransaction.narration }}</p>
          </div>

          <div class="space-y-1.5">
            <label class="block font-bold text-slate-700">Lý Do Chuyển Thẩm Định (Bắt buộc):</label>
            <textarea
              v-model="escalateRemarks"
              rows="3"
              class="w-full p-2.5 rounded-lg border border-slate-300 text-xs focus:ring-1 focus:ring-slate-900 focus:outline-none"
              placeholder="Nhập lý do tra soát hoặc nghi vấn sai lệch để Kiểm Soát Viên xử lý..."
            ></textarea>
          </div>
        </div>

        <div class="px-5 py-3 bg-slate-50 border-t border-slate-200 flex justify-end space-x-2">
          <button type="button" class="px-4 py-2 rounded-lg bg-slate-200 text-slate-700 hover:bg-slate-300 font-semibold text-xs cursor-pointer" @click="closeModal">Hủy</button>
          <button
            type="button"
            class="px-4 py-2 rounded-lg bg-slate-900 hover:bg-slate-800 text-white font-bold text-xs shadow-xs cursor-pointer"
            :disabled="!escalateRemarks.trim()"
            @click="confirmEscalate"
          >
            Chuyển Kiểm Soát Viên Ngay
          </button>
        </div>
      </div>
    </div>

    <!-- Toast Notification -->
    <div
      v-if="resolutionToast"
      class="fixed bottom-6 right-6 z-50 p-4 bg-slate-900 text-white text-xs font-semibold rounded-2xl shadow-xl border border-slate-700 flex items-center space-x-2 animate-in slide-in-from-bottom-3"
    >
      <div class="w-4 h-4 rounded-full bg-emerald-500 text-slate-950 flex items-center justify-center font-bold">
        <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20 6 9 17 4 12"/></svg>
      </div>
      <span>{{ resolutionToast }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useReconciliationStore } from '../stores/reconciliationStore';
import type { MatchTier, HitlQuarantineItem } from '../types/reconciliation';

const reconStore = useReconciliationStore();

// Sub-pages navigation state
type WorkbenchSubTab = 'tables' | 'quarantine' | 'overview' | 'import' | 'audit';
const activeSubTab = ref<WorkbenchSubTab>('tables');

// Table filters & search
const tableFilter = ref<'ALL' | 'MATCHED' | 'UNMATCHED'>('ALL');
const searchQuery = ref('');

const isDragging = ref(false);
const fileInputRef = ref<HTMLInputElement | null>(null);
const pastedText = ref('');
const inputTab = ref<'FILE' | 'PASTE'>('FILE');

// Filtered lists for table search
const filteredBankTransactions = computed(() => {
  let list = reconStore.bankTransactions;
  if (tableFilter.value === 'MATCHED') {
    list = list.filter((tx) => getMatchStatusText(tx.id) !== 'Chưa khớp');
  } else if (tableFilter.value === 'UNMATCHED') {
    list = list.filter((tx) => getMatchStatusText(tx.id) === 'Chưa khớp');
  }
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase().trim();
    list = list.filter((tx) =>
      tx.txCode.toLowerCase().includes(q) ||
      tx.narration.toLowerCase().includes(q) ||
      (tx.counterparty && tx.counterparty.toLowerCase().includes(q)) ||
      String(tx.amount).includes(q)
    );
  }
  return list;
});

const filteredLedgerEntries = computed(() => {
  let list = reconStore.ledgerEntries;
  if (tableFilter.value === 'MATCHED') {
    list = list.filter((entry) => getLedgerMatchStatusText(entry.id) === 'Đã khớp');
  } else if (tableFilter.value === 'UNMATCHED') {
    list = list.filter((entry) => getLedgerMatchStatusText(entry.id) !== 'Đã khớp');
  }
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase().trim();
    list = list.filter((entry) =>
      entry.docNo.toLowerCase().includes(q) ||
      entry.partnerName.toLowerCase().includes(q) ||
      entry.description.toLowerCase().includes(q) ||
      String(entry.amount).includes(q)
    );
  }
  return list;
});

// Exception Queue Interactive State
const selectedQuarantineItem = ref<HitlQuarantineItem | null>(null);
const activeModal = ref<'NONE' | 'OVERRIDE' | 'ALLOCATE_FEE' | 'ESCALATE'>('NONE');
const selectedLedgerId = ref('');
const overrideRemarks = ref('');
const feeAmountInput = ref(0);
const feeRemarks = ref('');
const escalateRemarks = ref('');
const resolutionToast = ref('');

function handleSelectSample(channel: 'CITAD' | 'NAPAS' | 'BILATERAL' | 'ALL' | 'SHEETS') {
  reconStore.loadSampleDataset(channel);
  activeSubTab.value = 'tables';
}

function openOverrideModal(item: HitlQuarantineItem) {
  selectedQuarantineItem.value = item;
  selectedLedgerId.value = (item.candidateLedgerIds && item.candidateLedgerIds[0]) || '';
  overrideRemarks.value = 'Khớp thủ công bởi Cán bộ Vận hành đối chiếu chứng từ gốc';
  activeModal.value = 'OVERRIDE';
}

function openAllocateFeeModal(item: HitlQuarantineItem) {
  selectedQuarantineItem.value = item;
  feeAmountInput.value = item.amount;
  feeRemarks.value = 'Hạch toán chi phí chuyển tiền liên ngân hàng vào TK 6425';
  activeModal.value = 'ALLOCATE_FEE';
}

function openEscalateModal(item: HitlQuarantineItem) {
  selectedQuarantineItem.value = item;
  escalateRemarks.value = '';
  activeModal.value = 'ESCALATE';
}

function closeModal() {
  activeModal.value = 'NONE';
  selectedQuarantineItem.value = null;
}

function confirmManualMatch() {
  if (!selectedQuarantineItem.value) return;
  const targetTxId = selectedQuarantineItem.value.txId;
  const ok = reconStore.resolveManualMatch(
    targetTxId,
    selectedLedgerId.value || undefined,
    overrideRemarks.value
  );
  if (ok) {
    resolutionToast.value = `Đã khớp thủ công thành công giao dịch ${targetTxId}`;
    setTimeout(() => {
      resolutionToast.value = '';
    }, 3500);
  }
  closeModal();
}

function confirmAllocateFee() {
  if (!selectedQuarantineItem.value) return;
  const targetTxId = selectedQuarantineItem.value.txId;
  const amt = feeAmountInput.value;
  const ok = reconStore.resolveAllocateFee(
    targetTxId,
    amt,
    feeRemarks.value
  );
  if (ok) {
    resolutionToast.value = `Đã hạch toán ${amt.toLocaleString('vi-VN')} VND vào TK 6425`;
    setTimeout(() => {
      resolutionToast.value = '';
    }, 3500);
  }
  closeModal();
}

function confirmEscalate() {
  if (!selectedQuarantineItem.value || !escalateRemarks.value.trim()) return;
  const targetTxId = selectedQuarantineItem.value.txId;
  const ok = reconStore.resolveEscalateToChecker(
    targetTxId,
    escalateRemarks.value.trim()
  );
  if (ok) {
    resolutionToast.value = `Đã chuyển giao dịch ${targetTxId} tới Kiểm Soát Viên`;
    setTimeout(() => {
      resolutionToast.value = '';
    }, 3500);
  }
  closeModal();
}

onMounted(() => {
  reconStore.initDefaultDemoData();
});

function triggerFileInput() {
  fileInputRef.value?.click();
}

function handleFileSelect(e: Event) {
  const target = e.target as HTMLInputElement;
  if (target.files && target.files[0]) {
    reconStore.parseUploadedFile(target.files[0]);
    activeSubTab.value = 'tables';
  }
}

function handleFileDrop(e: DragEvent) {
  isDragging.value = false;
  if (e.dataTransfer?.files && e.dataTransfer.files[0]) {
    reconStore.parseUploadedFile(e.dataTransfer.files[0]);
    activeSubTab.value = 'tables';
  }
}

function applyPastedText() {
  if (pastedText.value.trim()) {
    reconStore.parsePastedInput(pastedText.value);
    pastedText.value = '';
    activeSubTab.value = 'tables';
  }
}

function formatVnd(val?: number): string {
  return Number(val || 0).toLocaleString('vi-VN');
}

function getMatchStatusText(txId: string): string {
  if (!reconStore.reconciliationSummary) return 'Chưa khớp';
  const match = reconStore.reconciliationSummary.matches.find((m) =>
    m.bankTransactionIds.includes(txId)
  );
  if (match) {
    switch (match.tier) {
      case 'TIER1_EXACT':
        return 'T1 Tuyệt đối';
      case 'TIER2_FUZZY':
        return 'T2 Mờ / Phí';
      case 'TIER3_SPLIT':
        return 'T3 Phân tách';
      default:
        return 'Đã khớp';
    }
  }
  return 'Chưa khớp';
}

function getMatchStatusBadge(txId: string): string {
  const text = getMatchStatusText(txId);
  if (text.includes('T1')) return 'bg-emerald-100 text-emerald-800';
  if (text.includes('T2')) return 'bg-blue-100 text-blue-800';
  if (text.includes('T3')) return 'bg-purple-100 text-purple-800';
  return 'bg-slate-100 text-slate-500';
}

function getLedgerMatchStatusText(ledgerId: string): string {
  if (!reconStore.reconciliationSummary) return 'Chờ khớp';
  const match = reconStore.reconciliationSummary.matches.find((m) =>
    m.ledgerEntryIds.includes(ledgerId)
  );
  return match ? 'Đã khớp' : 'Chờ khớp';
}

function getLedgerMatchStatusBadge(ledgerId: string): string {
  const text = getLedgerMatchStatusText(ledgerId);
  return text === 'Đã khớp' ? 'bg-emerald-100 text-emerald-800' : 'bg-amber-100 text-amber-800';
}

function getMatchRowClass(tier: MatchTier): string {
  switch (tier) {
    case 'TIER1_EXACT':
      return 'bg-emerald-50/40 border-emerald-200';
    case 'TIER2_FUZZY':
      return 'bg-blue-50/40 border-blue-200';
    case 'TIER3_SPLIT':
      return 'bg-purple-50/40 border-purple-200';
    default:
      return 'bg-slate-50 border-slate-200';
  }
}

function getTierPillClass(tier: MatchTier): string {
  switch (tier) {
    case 'TIER1_EXACT':
      return 'bg-emerald-700 text-white';
    case 'TIER2_FUZZY':
      return 'bg-blue-700 text-white';
    case 'TIER3_SPLIT':
      return 'bg-purple-700 text-white';
    default:
      return 'bg-slate-700 text-white';
  }
}
</script>

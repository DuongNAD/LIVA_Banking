<template>
  <div class="reconciliation-workbench space-y-6 animate-in fade-in duration-200">
    <!-- Workbench Header -->
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm flex flex-col md:flex-row md:items-center md:justify-between gap-4">
      <div>
        <h1 class="text-xl font-bold text-slate-900 tracking-tight flex items-center space-x-2.5">
          <span>⚖️</span>
          <span>Bàn Làm Việc Đối Soát Quyết Toán Liên Ngân Hàng (3-Tier Engine)</span>
        </h1>
        <p class="text-xs text-slate-500 mt-0.5">
          Đối chiếu giữa Sổ Nhật ký Core Banking nội bộ với Bảng kê quyết toán bù trừ từ NAPAS và CITAD (NHNN), bóc tách chênh lệch phí chuyển mạch & xử lý tra soát treo
        </p>
      </div>

      <div class="flex items-center space-x-2">
        <button
          type="button"
          class="px-4 py-2 text-xs font-bold rounded-xl bg-emerald-600 hover:bg-emerald-700 text-white transition shadow-sm flex items-center space-x-2"
          :disabled="reconStore.isReconciling"
          @click="reconStore.executeReconciliation"
        >
          <span>⚡</span>
          <span>{{ reconStore.isReconciling ? 'Đang chạy đối soát...' : 'Chạy Đối Soát Tự Động' }}</span>
        </button>
      </div>
    </div>

    <!-- Quick Load Sample Datasets Bar -->
    <div class="bg-slate-900 text-white p-4 rounded-2xl shadow-sm space-y-3">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-2">
          <span class="text-amber-400 font-bold text-xs">BỘ DỮ LIỆU ĐỐI SOÁT LIÊN NGÂN HÀNG (Standard Benchmark):</span>
          <span class="text-xs text-slate-300">Nhấn 1-click để tải bảng kê quyết toán thực tế từ data/demo_ready/</span>
        </div>
        <span class="text-xs font-mono text-emerald-400 font-semibold truncate max-w-[260px]">
          Hiện hành: {{ reconStore.activeDatasetName }}
        </span>
      </div>

      <div class="flex flex-wrap items-center gap-2 text-xs">
        <button
          type="button"
          class="px-3 py-1.5 rounded-lg bg-emerald-800 hover:bg-emerald-700 text-white font-semibold transition border border-emerald-600 flex items-center space-x-1.5"
          @click="reconStore.loadSampleDataset('CITAD')"
        >
          <span>📊</span>
          <span>1. Kênh CITAD (NHNN) Excel (.xlsx)</span>
        </button>

        <button
          type="button"
          class="px-3 py-1.5 rounded-lg bg-sky-800 hover:bg-sky-700 text-white font-semibold transition border border-sky-600 flex items-center space-x-1.5"
          @click="reconStore.loadSampleDataset('NAPAS')"
        >
          <span>📄</span>
          <span>2. Kênh NAPAS 24/7 CSV (BOM)</span>
        </button>

        <button
          type="button"
          class="px-3 py-1.5 rounded-lg bg-amber-800 hover:bg-amber-700 text-white font-semibold transition border border-amber-600 flex items-center space-x-1.5"
          @click="reconStore.loadSampleDataset('BILATERAL')"
        >
          <span>📋</span>
          <span>3. Kênh Song Phương & Tra Soát (.csv)</span>
        </button>

        <button
          type="button"
          class="px-3 py-1.5 rounded-lg bg-purple-800 hover:bg-purple-700 text-white font-semibold transition border border-purple-600 flex items-center space-x-1.5"
          @click="reconStore.loadSampleDataset('ALL')"
        >
          <span>🌐</span>
          <span>4. Quyết Toán Toàn Kênh (CITAD + NAPAS + Song Phương)</span>
        </button>

        <button
          type="button"
          class="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-white font-semibold transition border border-slate-600 flex items-center space-x-1.5"
          @click="reconStore.loadSampleDataset('SHEETS')"
        >
          <span>📋</span>
          <span>5. Dán Bảng Tính Tra Soát</span>
        </button>
      </div>
    </div>

    <!-- Statement Dropzone & Paste Input Area -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <!-- File Dropzone -->
      <div
        class="bg-white p-5 rounded-2xl border-2 border-dashed transition-all text-center flex flex-col justify-center items-center cursor-pointer min-h-[160px]"
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
        <div class="w-10 h-10 rounded-full bg-slate-100 flex items-center justify-center text-slate-600 text-xl mb-2">
          📂
        </div>
        <p class="text-xs font-bold text-slate-800">
          Kéo thả tệp bảng kê quyết toán bù trừ liên ngân hàng vào đây hoặc <span class="text-blue-600 underline">chọn tệp từ máy</span>
        </p>
        <p class="text-[11px] text-slate-500 mt-1">
          Hỗ trợ: Bảng kê quyết toán CITAD (NHNN), Dữ liệu quyết toán NAPAS 24/7 (BOM CSV), Sổ phụ Nostro/Vostro
        </p>
      </div>

      <!-- Raw Paste Area -->
      <div class="bg-white p-4 rounded-2xl border border-slate-200 shadow-sm flex flex-col justify-between space-y-2">
        <div class="flex items-center justify-between text-xs">
          <span class="font-bold text-slate-700">Dán bảng kê quyết toán / tra soát thô (Clipboard):</span>
          <span class="text-[11px] text-slate-400">Tự động nhận diện phân cách (\t, ;, ,, |)</span>
        </div>
        <textarea
          v-model="pastedText"
          rows="3"
          class="w-full text-xs font-mono p-2.5 rounded-xl border border-slate-300 bg-slate-50 focus:bg-white focus:ring-2 focus:ring-slate-900 focus:outline-none"
          placeholder="Dán dữ liệu quyết toán từ hệ thống bù trừ, bảng tính tra soát liên ngân hàng tại đây..."
        ></textarea>
        <div class="flex items-center justify-between pt-1">
          <span class="text-[10px] text-slate-400">Tự động chuẩn hóa cột & số học VND</span>
          <button
            type="button"
            class="px-3 py-1 text-xs font-semibold rounded-lg bg-slate-900 hover:bg-slate-800 text-white transition"
            :disabled="!pastedText.trim()"
            @click="applyPastedText"
          >
            Bóc Tách & Nhập Dữ Liệu
          </button>
        </div>
      </div>
    </div>

    <!-- Double-Entry Balance Invariant Card & Reconciliation Summary Gauge -->
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-4">
      <!-- Balance Invariant Card (7 cols) -->
      <div class="lg:col-span-7 bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-3">
        <div class="flex items-center justify-between pb-2 border-b border-slate-100">
          <div class="flex items-center space-x-2">
            <span class="text-base">📐</span>
            <div>
              <h3 class="text-xs font-bold uppercase tracking-wider text-slate-700">
                Bảo Chứng Cân Bằng Kế Toán Kép (Double-Entry Balance Invariant)
              </h3>
              <p class="text-[11px] text-slate-400">Công thức: Số dư cuối = Số dư đầu + Tổng Có (Inflow) - Tổng Nợ (Outflow)</p>
            </div>
          </div>
          <span
            class="px-2.5 py-1 rounded-full text-xs font-bold flex items-center space-x-1"
            :class="reconStore.isBalanced ? 'bg-emerald-100 text-emerald-800' : 'bg-rose-100 text-rose-800'"
          >
            <span>{{ reconStore.isBalanced ? '✓ Chuẩn Tuyệt Đối' : '⚠ Lệch Số Dư' }}</span>
          </span>
        </div>

        <!-- Formula Numbers Grid -->
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-2.5 text-center text-xs">
          <div class="p-2.5 rounded-xl bg-slate-50 border border-slate-200">
            <span class="text-slate-400 block text-[10px]">Số dư đầu kỳ:</span>
            <span class="font-bold font-mono text-slate-900 text-xs">
              {{ formatVnd(reconStore.balanceInvariantReport?.openingBalance) }}
            </span>
          </div>

          <div class="p-2.5 rounded-xl bg-emerald-50 border border-emerald-200">
            <span class="text-emerald-700 block text-[10px]">+ Tổng phát sinh Có:</span>
            <span class="font-bold font-mono text-emerald-900 text-xs">
              {{ formatVnd(reconStore.balanceInvariantReport?.totalCredit) }}
            </span>
          </div>

          <div class="p-2.5 rounded-xl bg-rose-50 border border-rose-200">
            <span class="text-rose-700 block text-[10px]">- Tổng phát sinh Nợ:</span>
            <span class="font-bold font-mono text-rose-900 text-xs">
              {{ formatVnd(reconStore.balanceInvariantReport?.totalDebit) }}
            </span>
          </div>

          <div class="p-2.5 rounded-xl bg-blue-50 border border-blue-200">
            <span class="text-blue-700 block text-[10px]">= Số dư cuối kỳ:</span>
            <span class="font-bold font-mono text-blue-900 text-xs">
              {{ formatVnd(reconStore.balanceInvariantReport?.closingBalance) }}
            </span>
          </div>
        </div>

        <div class="flex items-center justify-between text-xs text-slate-500 pt-1">
          <div class="flex items-center space-x-2">
            <span class="text-emerald-600 font-semibold">● Số học nguyên 64-bit (VND):</span>
            <span class="font-mono">Độ lệch sai số IEEE-754: <strong>0 VND</strong></span>
          </div>
          <span class="font-mono text-xs" :class="reconStore.discrepancyAmount === 0 ? 'text-emerald-600 font-bold' : 'text-rose-600 font-bold'">
            Chênh lệch: {{ formatVnd(reconStore.discrepancyAmount) }} VND
          </span>
        </div>
      </div>

      <!-- 3-Tier Match Breakdown Badges (5 cols) -->
      <div class="lg:col-span-5 bg-white p-5 rounded-2xl border border-slate-200 shadow-sm space-y-3 flex flex-col justify-between">
        <div class="flex items-center justify-between pb-2 border-b border-slate-100">
          <h3 class="text-xs font-bold uppercase tracking-wider text-slate-700">
            Phân Tầng Đối Soát (3-Tier Resolution)
          </h3>
          <span class="text-xs font-bold text-emerald-600 font-mono">
            Tỷ lệ: {{ reconStore.matchRate.toFixed(1) }}%
          </span>
        </div>

        <div class="grid grid-cols-2 gap-2 text-xs">
          <!-- Tier 1 -->
          <div class="p-2.5 rounded-xl bg-emerald-50 border border-emerald-200 flex items-center justify-between">
            <div>
              <span class="font-bold text-emerald-900 block text-xs">Tier 1: Tuyệt Đối 1:1</span>
              <span class="text-[10px] text-emerald-700">Khớp số tiền & mã HĐ</span>
            </div>
            <span class="text-base font-bold font-mono text-emerald-800">{{ reconStore.tier1Count }}</span>
          </div>

          <!-- Tier 2 -->
          <div class="p-2.5 rounded-xl bg-blue-50 border border-blue-200 flex items-center justify-between">
            <div>
              <span class="font-bold text-blue-900 block text-xs">Tier 2: Heuristic mờ</span>
              <span class="text-[10px] text-blue-700">Bóc tách phí & ±72h</span>
            </div>
            <span class="text-base font-bold font-mono text-blue-800">{{ reconStore.tier2Count }}</span>
          </div>

          <!-- Tier 3 -->
          <div class="p-2.5 rounded-xl bg-purple-50 border border-purple-200 flex items-center justify-between">
            <div>
              <span class="font-bold text-purple-900 block text-xs">Tier 3: Ghép 1:N / N:1</span>
              <span class="text-[10px] text-purple-700">Subset-Sum solver</span>
            </div>
            <span class="text-base font-bold font-mono text-purple-800">{{ reconStore.tier3Count }}</span>
          </div>

          <!-- HITL Quarantine -->
          <div class="p-2.5 rounded-xl bg-amber-50 border border-amber-200 flex items-center justify-between">
            <div>
              <span class="font-bold text-amber-900 block text-xs">Quarantine HITL</span>
              <span class="text-[10px] text-amber-700">Hàng đợi xử lý tay</span>
            </div>
            <span class="text-base font-bold font-mono text-amber-800">{{ reconStore.hitlQuarantineCount }}</span>
          </div>
        </div>

        <div class="text-[11px] text-slate-400 flex items-center justify-between pt-1">
          <span>Tổng giao dịch sao kê: <strong class="text-slate-800">{{ reconStore.totalBankTxCount }}</strong></span>
          <span>Bản ghi sổ cái: <strong class="text-slate-800">{{ reconStore.totalLedgerCount }}</strong></span>
        </div>
      </div>
    </div>

    <!-- Side-by-Side Comparative Grid -->
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">
      <!-- LEFT PANE: Bank Statement Transactions (6 cols) -->
      <div class="lg:col-span-6 bg-white rounded-2xl border border-slate-200 shadow-sm overflow-hidden flex flex-col">
        <div class="p-4 border-b border-slate-200 bg-slate-50 flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <span class="text-base">🏦</span>
            <h3 class="text-xs font-bold uppercase tracking-wider text-slate-800">
              Bảng Kê Quyết Toán Bù Trừ Liên Ngân Hàng (Interbank Clearing)
            </h3>
          </div>
          <span class="text-xs text-slate-500 font-mono">{{ reconStore.bankTransactions.length }} Giao dịch</span>
        </div>

        <!-- Table Container -->
        <div class="overflow-x-auto max-h-[420px] overflow-y-auto">
          <table class="w-full text-left text-xs">
            <thead class="bg-slate-100/70 text-slate-600 font-semibold border-b border-slate-200 sticky top-0">
              <tr>
                <th class="p-2.5">Ngày / Mã GD</th>
                <th class="p-2.5">Nội Dung Chi Tiết / Đối Tác</th>
                <th class="p-2.5 text-right">Phát Sinh (VND)</th>
                <th class="p-2.5 text-right">Số Dư</th>
                <th class="p-2.5 text-center">Trạng Thái</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-100">
              <tr
                v-for="tx in reconStore.bankTransactions"
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
                    Đối tác: {{ tx.counterparty }}
                  </div>
                </td>
                <td class="p-2.5 align-top text-right whitespace-nowrap font-mono">
                  <span v-if="tx.credit > 0" class="text-emerald-600 font-bold">
                    +{{ formatVnd(tx.credit) }}
                  </span>
                  <span v-else-if="tx.debit > 0" class="text-rose-600 font-bold">
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
      <div class="lg:col-span-6 bg-white rounded-2xl border border-slate-200 shadow-sm overflow-hidden flex flex-col">
        <div class="p-4 border-b border-slate-200 bg-slate-50 flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <span class="text-base">📒</span>
            <h3 class="text-xs font-bold uppercase tracking-wider text-slate-800">
              Sổ Nhật Ký Giao Dịch Core Banking Nội Bộ (Core Banking Ledger)
            </h3>
          </div>
          <span class="text-xs text-slate-500 font-mono">{{ reconStore.ledgerEntries.length }} Bản ghi</span>
        </div>

        <!-- Table Container -->
        <div class="overflow-x-auto max-h-[420px] overflow-y-auto">
          <table class="w-full text-left text-xs">
            <thead class="bg-slate-100/70 text-slate-600 font-semibold border-b border-slate-200 sticky top-0">
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
                v-for="entry in reconStore.ledgerEntries"
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

    <!-- Exception & Unmatched Resolution Queue UI (Feature F06) -->
    <div class="bg-white rounded-2xl border border-slate-200 shadow-sm p-5 space-y-4">
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 pb-3 border-b border-slate-100">
        <div class="flex items-center space-x-3">
          <div class="w-9 h-9 rounded-xl bg-amber-500/20 text-amber-600 flex items-center justify-center font-bold text-base">
            ⚠️
          </div>
          <div>
            <h2 class="text-sm font-bold text-slate-900">
              Hàng Đợi Xử Lý Ngoại Lệ & Giao Dịch Treo (Exception & Unmatched Resolution Queue - HITL)
            </h2>
            <p class="text-xs text-slate-500">
              Quản lý các giao dịch cách ly chưa khớp tự động: Lệch phí chuyển tiền, trễ phiên bù trừ T+2 hoặc thiếu hóa đơn đối ứng (Fail-Closed)
            </p>
          </div>
        </div>

        <div class="flex items-center space-x-2">
          <span
            class="px-3 py-1 rounded-full text-xs font-bold"
            :class="reconStore.quarantinedItems.length > 0 ? 'bg-amber-100 text-amber-800' : 'bg-emerald-100 text-emerald-800'"
          >
            {{ reconStore.quarantinedItems.length }} Giao Dịch Cách Ly
          </span>
        </div>
      </div>

      <!-- Empty state when no quarantined items -->
      <div
        v-if="reconStore.quarantinedItems.length === 0"
        class="p-6 bg-slate-50 rounded-xl border border-slate-200 text-center space-y-1.5"
      >
        <div class="text-2xl">🎉</div>
        <p class="text-xs font-bold text-slate-800">Không có giao dịch nào bị treo trong hàng đợi cách ly!</p>
        <p class="text-[11px] text-slate-500">Tất cả giao dịch đã được đối khớp 3 tầng tự động hoặc đã được giải tỏa hoàn tất.</p>
      </div>

      <!-- Quarantined items list -->
      <div v-else class="space-y-3">
        <div
          v-for="item in reconStore.quarantinedItems"
          :key="item.txId"
          class="p-4 rounded-xl border border-amber-200 bg-amber-50/30 hover:bg-amber-50/60 transition space-y-3"
        >
          <!-- Top metadata row -->
          <div class="flex flex-wrap items-center justify-between gap-2 text-xs">
            <div class="flex items-center space-x-2 flex-wrap">
              <span class="px-2 py-0.5 rounded font-mono font-bold bg-amber-200 text-amber-900 text-[10px]">
                HITL QUARANTINE
              </span>
              <span class="font-mono text-[11px] text-slate-600 truncate max-w-[200px]" :title="item.hitlToken">
                🔑 Token: {{ item.hitlToken }}
              </span>
              <span class="px-2 py-0.5 rounded bg-blue-100 text-blue-800 font-semibold text-[10px]">
                ⏳ TTL: 15 Phút (900s)
              </span>
            </div>

            <div class="text-right text-[11px] text-slate-500 font-mono">
              <span>Mã GD: <strong class="text-slate-800">{{ item.rawTransaction.txCode }}</strong> | Ngày: {{ item.rawTransaction.date }}</span>
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
                👮 Đã chuyển KSV: "{{ reconStore.escalatedTxMap[item.txId].remarks }}"
              </div>
            </div>

            <!-- Col 3: Interactive Resolution Actions (3 cols) -->
            <div class="md:col-span-3 flex flex-col justify-center space-y-1.5">
              <button
                type="button"
                class="w-full px-3 py-1.5 text-xs font-bold rounded-lg bg-emerald-600 hover:bg-emerald-700 text-white transition shadow-xs flex items-center justify-center space-x-1.5"
                @click="openOverrideModal(item)"
              >
                <span>⚡</span>
                <span>Ép Khớp (Manual Override)</span>
              </button>

              <button
                type="button"
                class="w-full px-3 py-1.5 text-xs font-bold rounded-lg bg-blue-600 hover:bg-blue-700 text-white transition shadow-xs flex items-center justify-center space-x-1.5"
                @click="openAllocateFeeModal(item)"
              >
                <span>🏦</span>
                <span>Hạch Toán Phí 6425</span>
              </button>

              <button
                type="button"
                class="w-full px-3 py-1.5 text-xs font-bold rounded-lg bg-slate-900 hover:bg-slate-800 text-white transition shadow-xs flex items-center justify-center space-x-1.5"
                @click="openEscalateModal(item)"
              >
                <span>👮</span>
                <span>Chuyển Kiểm Soát Viên</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Match Results & Audit Breakdown Drawer/Table -->
    <div v-if="reconStore.reconciliationSummary" class="bg-white rounded-2xl border border-slate-200 shadow-sm p-5 space-y-4">
      <div class="flex items-center justify-between pb-2 border-b border-slate-100">
        <div class="flex items-center space-x-2.5">
          <span class="text-base">📋</span>
          <div>
            <h3 class="text-sm font-bold text-slate-900">Chi Tiết Bằng Chứng Khớp Quyết Toán & Bóc Tách Phí Chuyển Mạch (Clearing Match Log)</h3>
            <p class="text-xs text-slate-500">Mỗi khớp lệnh đi kèm độ tin cậy toán học, bóc tách phí NAPAS/CITAD và giải trình kiểm toán</p>
          </div>
        </div>
        <span class="text-xs text-slate-400">
          Tổng cộng: {{ reconStore.reconciliationSummary.matches.length }} kết quả
        </span>
      </div>

      <div class="space-y-2.5">
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
              <span v-if="match.feeAmount > 0" class="px-2 py-0.5 rounded text-[10px] font-bold bg-amber-100 text-amber-800">
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
    </div>

    <!-- MODAL 1: Manual Match Override Modal -->
    <div
      v-if="activeModal === 'OVERRIDE' && selectedQuarantineItem"
      class="fixed inset-0 z-50 overflow-y-auto bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4"
    >
      <div class="bg-white rounded-2xl shadow-2xl max-w-lg w-full border border-slate-200 overflow-hidden flex flex-col animate-in fade-in zoom-in-95 duration-150">
        <div class="px-5 py-4 bg-emerald-700 text-white flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <span class="text-lg">⚡</span>
            <h3 class="text-sm font-bold">Ép Khớp Thủ Công (Manual Match Override)</h3>
          </div>
          <button type="button" class="text-emerald-200 hover:text-white" @click="closeModal">✕</button>
        </div>

        <div class="p-5 space-y-4 text-xs text-slate-800">
          <div class="p-3 bg-slate-50 rounded-xl border border-slate-200 space-y-1">
            <span class="text-slate-500 font-medium">Giao dịch ngân hàng cần ép khớp:</span>
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
            <label class="block font-bold text-slate-700">Giải Trình Ép Khớp (Biên bản kiểm toán):</label>
            <textarea
              v-model="overrideRemarks"
              rows="2"
              class="w-full p-2.5 rounded-lg border border-slate-300 text-xs focus:ring-2 focus:ring-emerald-500 focus:outline-none"
              placeholder="Ghi rõ lý do ép khớp đối soát..."
            ></textarea>
          </div>
        </div>

        <div class="px-5 py-3 bg-slate-50 border-t border-slate-200 flex justify-end space-x-2">
          <button type="button" class="px-4 py-2 rounded-lg bg-slate-200 text-slate-700 hover:bg-slate-300 font-semibold text-xs" @click="closeModal">Hủy</button>
          <button type="button" class="px-4 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-700 text-white font-bold text-xs shadow-xs" @click="confirmManualMatch">Xác Nhận Ép Khớp</button>
        </div>
      </div>
    </div>

    <!-- MODAL 2: Allocate Bank Fee Modal -->
    <div
      v-if="activeModal === 'ALLOCATE_FEE' && selectedQuarantineItem"
      class="fixed inset-0 z-50 overflow-y-auto bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4"
    >
      <div class="bg-white rounded-2xl shadow-2xl max-w-lg w-full border border-slate-200 overflow-hidden flex flex-col animate-in fade-in zoom-in-95 duration-150">
        <div class="px-5 py-4 bg-blue-700 text-white flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <span class="text-lg">🏦</span>
            <h3 class="text-sm font-bold">Hạch Toán Chi Phí Ngân Hàng (TK 6425)</h3>
          </div>
          <button type="button" class="text-blue-200 hover:text-white" @click="closeModal">✕</button>
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
              <button type="button" class="px-2 py-0.5 rounded bg-slate-100 hover:bg-slate-200" @click="feeAmountInput = 1100">1.100đ</button>
              <button type="button" class="px-2 py-0.5 rounded bg-slate-100 hover:bg-slate-200" @click="feeAmountInput = 2200">2.200đ</button>
              <button type="button" class="px-2 py-0.5 rounded bg-slate-100 hover:bg-slate-200" @click="feeAmountInput = 5500">5.500đ</button>
              <button type="button" class="px-2 py-0.5 rounded bg-slate-100 hover:bg-slate-200" @click="feeAmountInput = 11000">11.000đ</button>
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
          <button type="button" class="px-4 py-2 rounded-lg bg-slate-200 text-slate-700 hover:bg-slate-300 font-semibold text-xs" @click="closeModal">Hủy</button>
          <button type="button" class="px-4 py-2 rounded-lg bg-blue-600 hover:bg-blue-700 text-white font-bold text-xs shadow-xs" @click="confirmAllocateFee">Xác Nhận Hạch Toán TK 6425</button>
        </div>
      </div>
    </div>

    <!-- MODAL 3: Escalate to Checker Modal -->
    <div
      v-if="activeModal === 'ESCALATE' && selectedQuarantineItem"
      class="fixed inset-0 z-50 overflow-y-auto bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4"
    >
      <div class="bg-white rounded-2xl shadow-2xl max-w-lg w-full border border-slate-200 overflow-hidden flex flex-col animate-in fade-in zoom-in-95 duration-150">
        <div class="px-5 py-4 bg-slate-900 text-white flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <span class="text-lg">👮</span>
            <h3 class="text-sm font-bold">Chuyển Kiểm Soát Viên (Escalate to Checker)</h3>
          </div>
          <button type="button" class="text-slate-400 hover:text-white" @click="closeModal">✕</button>
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
              class="w-full p-2.5 rounded-lg border border-slate-300 text-xs focus:ring-2 focus:ring-slate-900 focus:outline-none"
              placeholder="Nhập lý do tra soát hoặc nghi vấn sai lệch để Kiểm Soát Viên xử lý..."
            ></textarea>
          </div>
        </div>

        <div class="px-5 py-3 bg-slate-50 border-t border-slate-200 flex justify-end space-x-2">
          <button type="button" class="px-4 py-2 rounded-lg bg-slate-200 text-slate-700 hover:bg-slate-300 font-semibold text-xs" @click="closeModal">Hủy</button>
          <button
            type="button"
            class="px-4 py-2 rounded-lg bg-slate-900 hover:bg-slate-800 text-white font-bold text-xs shadow-xs"
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
      <span class="text-emerald-400 text-sm">✓</span>
      <span>{{ resolutionToast }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useReconciliationStore } from '../stores/reconciliationStore';
import type { MatchTier, HitlQuarantineItem } from '../types/reconciliation';

const reconStore = useReconciliationStore();

const isDragging = ref(false);
const fileInputRef = ref<HTMLInputElement | null>(null);
const pastedText = ref('');

// Exception Queue Interactive State
const selectedQuarantineItem = ref<HitlQuarantineItem | null>(null);
const activeModal = ref<'NONE' | 'OVERRIDE' | 'ALLOCATE_FEE' | 'ESCALATE'>('NONE');
const selectedLedgerId = ref('');
const overrideRemarks = ref('');
const feeAmountInput = ref(0);
const feeRemarks = ref('');
const escalateRemarks = ref('');
const resolutionToast = ref('');

function openOverrideModal(item: HitlQuarantineItem) {
  selectedQuarantineItem.value = item;
  selectedLedgerId.value = (item.candidateLedgerIds && item.candidateLedgerIds[0]) || '';
  overrideRemarks.value = 'Ép khớp thủ công bởi Cán bộ Vận hành đối chiếu chứng từ gốc';
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
    resolutionToast.value = `✓ Đã ép khớp thành công giao dịch ${targetTxId}`;
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
    resolutionToast.value = `✓ Đã hạch toán ${amt.toLocaleString('vi-VN')} VND vào TK 6425`;
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
    resolutionToast.value = `✓ Đã chuyển giao dịch ${targetTxId} tới Kiểm Soát Viên`;
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
  }
}

function handleFileDrop(e: DragEvent) {
  isDragging.value = false;
  if (e.dataTransfer?.files && e.dataTransfer.files[0]) {
    reconStore.parseUploadedFile(e.dataTransfer.files[0]);
  }
}

function applyPastedText() {
  if (pastedText.value.trim()) {
    reconStore.parsePastedInput(pastedText.value);
    pastedText.value = '';
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
      return 'bg-emerald-600 text-white';
    case 'TIER2_FUZZY':
      return 'bg-blue-600 text-white';
    case 'TIER3_SPLIT':
      return 'bg-purple-600 text-white';
    default:
      return 'bg-slate-600 text-white';
  }
}
</script>

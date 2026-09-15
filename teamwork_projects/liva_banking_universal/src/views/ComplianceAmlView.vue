<template>
  <div class="compliance-aml-view space-y-6 animate-in fade-in duration-200">
    <!-- View Header -->
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm flex flex-col md:flex-row md:items-center md:justify-between gap-4">
      <div>
        <h1 class="text-xl font-bold text-slate-900 tracking-tight flex items-center space-x-2.5">
          <div class="w-8 h-8 rounded-xl bg-slate-900 text-white flex items-center justify-center">
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z"/></svg>
          </div>
          <span>Giám Sát Tuân Thủ AML / CTF & Thanh Tra Câu Lệnh AI</span>
        </h1>
        <p class="text-xs text-slate-500 mt-0.5">
          Thông tư 09/2023/TT-NHNN, Quyết định 11/2023/QĐ-TTg & Nghị định 13/2023/NĐ-CP (Zero Cloud Egress)
        </p>
      </div>

      <!-- Live Regulatory Status Indicators -->
      <div class="flex items-center space-x-2 text-xs">
        <span class="inline-flex items-center px-3 py-1.5 rounded-xl font-bold bg-emerald-50 text-emerald-800 border border-emerald-200">
          <span class="w-2 h-2 rounded-full bg-emerald-500 mr-1.5 animate-pulse"></span>
          Cục Bộ 100% (Zero Cloud Leakage)
        </span>
        <span class="inline-flex items-center px-3 py-1.5 rounded-xl font-bold bg-blue-50 text-blue-800 border border-blue-200">
          Cục PCRT — NHNN
        </span>
      </div>
    </div>

    <!-- Subpage Navigation Tabs -->
    <div class="flex items-center space-x-2 border-b border-slate-200 pb-2 text-xs font-bold overflow-x-auto">
      <button
        type="button"
        class="px-4 py-2 rounded-xl transition flex items-center space-x-2 cursor-pointer whitespace-nowrap"
        :class="activeTab === 'ALERTS' ? 'bg-slate-900 text-white shadow-sm' : 'bg-white text-slate-600 hover:bg-slate-100 border border-slate-200'"
        @click="activeTab = 'ALERTS'"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
        <span>Cảnh báo rủi ro & STR</span>
        <span
          v-if="activeAlerts.length > 0"
          class="px-1.5 py-0.5 rounded-full text-[10px] font-mono"
          :class="activeTab === 'ALERTS' ? 'bg-rose-500 text-white font-bold' : 'bg-rose-100 text-rose-800'"
        >
          {{ activeAlerts.length }}
        </span>
      </button>

      <button
        type="button"
        class="px-4 py-2 rounded-xl transition flex items-center space-x-2 cursor-pointer whitespace-nowrap"
        :class="activeTab === 'INSPECTOR' ? 'bg-slate-900 text-white shadow-sm' : 'bg-white text-slate-600 hover:bg-slate-100 border border-slate-200'"
        @click="activeTab = 'INSPECTOR'"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="m9 8 6 4-6 4Z"/></svg>
        <span>Giám sát mô hình AI</span>
      </button>

      <button
        type="button"
        class="px-4 py-2 rounded-xl transition flex items-center space-x-2 cursor-pointer whitespace-nowrap"
        :class="activeTab === 'REGULATORY' ? 'bg-slate-900 text-white shadow-sm' : 'bg-white text-slate-600 hover:bg-slate-100 border border-slate-200'"
        @click="activeTab = 'REGULATORY'"
      >
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
        <span>Khung pháp lý tuân thủ</span>
      </button>
    </div>

    <!-- TAB 1: Circular 09/2023 AML Alert Grid Section -->
    <div v-if="activeTab === 'ALERTS'" class="space-y-4">
      <div class="bg-white rounded-2xl border border-slate-200 shadow-sm p-6 space-y-4">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 pb-3 border-b border-slate-100">
          <div>
            <h2 class="text-base font-bold text-slate-900">
              Nhật Ký Cảnh Báo Gian Lận & Rửa Tiền Đang Hoạt Động (Active Alerts)
            </h2>
            <p class="text-xs text-slate-500">
              Phát hiện từ dữ liệu giao dịch sao kê và bộ kịch bản kiểm thử sandbox chuẩn NHNN
            </p>
          </div>

          <div class="flex items-center space-x-2">
            <button
              type="button"
              class="px-3.5 py-1.5 text-xs font-semibold rounded-lg bg-slate-100 hover:bg-slate-200 text-slate-700 transition flex items-center space-x-1.5 cursor-pointer"
              @click="scanAllCurrentTransactions"
            >
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M8 16H3v5"/></svg>
              <span>Quét Lại Dữ Liệu Sao Kê</span>
            </button>
          </div>
        </div>

        <AmlAlertList
          :alerts="activeAlerts"
          @open-str="onOpenStrModal"
        />
      </div>
    </div>

    <!-- TAB 2: Live Explainable System Prompt Inspector -->
    <div v-else-if="activeTab === 'INSPECTOR'" class="space-y-4">
      <SystemPromptInspector />
    </div>

    <!-- TAB 3: Regulatory Standard Overview -->
    <div v-else-if="activeTab === 'REGULATORY'" class="bg-white rounded-2xl border border-slate-200 shadow-sm p-6 space-y-5">
      <div class="pb-3 border-b border-slate-100">
        <h2 class="text-base font-bold text-slate-900">Khung Pháp Lý Phòng, Chống Rửa Tiền & An Toàn Dữ Liệu</h2>
        <p class="text-xs text-slate-500">Quy chuẩn tuân thủ bắt buộc đối với Ngân hàng Thương mại Việt Nam</p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-4 text-xs">
        <div class="p-4 rounded-xl bg-slate-50 border border-slate-200 space-y-2">
          <div class="flex items-center space-x-2">
            <span class="px-2 py-0.5 rounded font-bold bg-blue-100 text-blue-800 text-[10px]">THÔNG TƯ 09/2023</span>
          </div>
          <h3 class="font-bold text-slate-900 text-sm">Quy Định Hướng Dẫn Luật PCRT</h3>
          <p class="text-slate-600 leading-relaxed">
            Quy định các tiêu chí nhận biết giao dịch đáng ngờ, lập Báo cáo giao dịch đáng ngờ (STR theo Phụ lục II) và thời hạn nộp báo cáo cho Cục PCRT - NHNN.
          </p>
        </div>

        <div class="p-4 rounded-xl bg-slate-50 border border-slate-200 space-y-2">
          <div class="flex items-center space-x-2">
            <span class="px-2 py-0.5 rounded font-bold bg-amber-100 text-amber-800 text-[10px]">QUYẾT ĐỊNH 11/2023</span>
          </div>
          <h3 class="font-bold text-slate-900 text-sm">Ngưỡng Giao Dịch Giá Trị Lớn</h3>
          <p class="text-slate-600 leading-relaxed">
            Ngưỡng bắt buộc báo cáo giao dịch giá trị lớn từ <strong>400,000,000 VND</strong> trở lên trong ngày đối với cả giao dịch tiền mặt và chuyển khoản.
          </p>
        </div>

        <div class="p-4 rounded-xl bg-slate-50 border border-slate-200 space-y-2">
          <div class="flex items-center space-x-2">
            <span class="px-2 py-0.5 rounded font-bold bg-emerald-100 text-emerald-800 text-[10px]">NGHỊ ĐỊNH 13/2023</span>
          </div>
          <h3 class="font-bold text-slate-900 text-sm">Bảo Vệ Dữ Liệu Cá Nhân & Zero Egress</h3>
          <p class="text-slate-600 leading-relaxed">
            100% dữ liệu danh tính khách hàng (PII) được xử lý cục bộ trên máy trạm ngân hàng, không gửi dữ liệu ra máy chủ bên ngoài (Zero Cloud Egress).
          </p>
        </div>
      </div>
    </div>

    <!-- Statutory Form STR Modal (Phụ lục II TT 09/2023/TT-NHNN) -->
    <StrReportModal
      :is-open="amlStore.isStrModalOpen"
      :form="amlStore.generatedStr"
      :official-document-text="amlStore.officialStrDocumentText"
      @close="amlStore.closeStrModal"
      @update-notes="amlStore.updateComplianceNotes"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useAmlStore } from '../stores/amlStore';
import { useReconciliationStore } from '../stores/reconciliationStore';
import SystemPromptInspector from '../components/aml/SystemPromptInspector.vue';
import AmlAlertList from '../components/aml/AmlAlertList.vue';
import StrReportModal from '../components/aml/StrReportModal.vue';
import type { AmlAlert } from '../types/aml';

const amlStore = useAmlStore();
const reconStore = useReconciliationStore();

const activeTab = ref<'ALERTS' | 'INSPECTOR' | 'REGULATORY'>('ALERTS');

onMounted(() => {
  // If sandbox has no alerts evaluated yet, run evaluation
  if (amlStore.sandboxAlerts.length === 0) {
    amlStore.runSandboxEvaluation();
  }
  // Scan any current bank transactions
  scanAllCurrentTransactions();
});

function scanAllCurrentTransactions() {
  if (reconStore.bankTransactions.length > 0) {
    amlStore.scanTransactions(reconStore.bankTransactions);
  }
}

// Merge active scanned alerts with current sandbox alerts if present
const activeAlerts = computed<AmlAlert[]>(() => {
  const scanned = amlStore.alerts;
  const sandbox = amlStore.sandboxAlerts;

  const map = new Map<string, AmlAlert>();
  for (const a of scanned) {
    map.set(a.alertId, a);
  }
  for (const a of sandbox) {
    if (!map.has(a.alertId)) {
      map.set(a.alertId, a);
    }
  }

  return Array.from(map.values());
});

function onOpenStrModal(alert: AmlAlert) {
  amlStore.openStrModal(alert, reconStore.bankTransactions);
}
</script>

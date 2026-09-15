<template>
  <div class="compliance-aml-view space-y-6 animate-in fade-in duration-200">
    <!-- View Header -->
    <div class="bg-white p-5 rounded-2xl border border-slate-200 shadow-sm flex flex-col md:flex-row md:items-center md:justify-between gap-4">
      <div>
        <h1 class="text-xl font-bold text-slate-900 tracking-tight flex items-center space-x-2.5">
          <span>🛡️</span>
          <span>Giám Sát Tuân Thủ AML / CTF & Thanh Tra Câu Lệnh AI</span>
        </h1>
        <p class="text-xs text-slate-500 mt-0.5">
          Thông tư 09/2023/TT-NHNN, Quyết định 11/2023/QĐ-TTg & Nghị định 13/2023/NĐ-CP (Zero Cloud Egress)
        </p>
      </div>

      <!-- Live Regulatory Status Indicators -->
      <div class="flex items-center space-x-2 text-xs">
        <span class="inline-flex items-center px-3 py-1.5 rounded-xl font-bold bg-emerald-100 text-emerald-800 border border-emerald-200">
          <span class="w-2 h-2 rounded-full bg-emerald-500 mr-1.5 animate-pulse"></span>
          Cục Bộ 100% (Zero Cloud Leakage)
        </span>
        <span class="inline-flex items-center px-3 py-1.5 rounded-xl font-bold bg-blue-100 text-blue-800 border border-blue-200">
          ⚖️ Cục PCRT — NHNN
        </span>
      </div>
    </div>

    <!-- Dual Layout: Left Alert Grid + Right System Prompt Inspector -->
    <div class="space-y-6">
      <!-- Live Explainable System Prompt Inspector -->
      <SystemPromptInspector />

      <!-- Circular 09/2023 AML Alert Grid Section -->
      <div class="bg-white rounded-2xl border border-slate-200 shadow-sm p-6 space-y-4">
        <div class="flex items-center justify-between pb-3 border-b border-slate-100">
          <div>
            <h2 class="text-base font-bold text-slate-900">
              Nhật Ký Cảnh Báo Gian Lận & Rửa Tiền Đang Hoạt Động (Active Alerts)
            </h2>
            <p class="text-xs text-slate-500">
              Kết hợp phát hiện từ dữ liệu sao kê nạp vào và bộ sandbox kiểm thử chuẩn
            </p>
          </div>

          <div class="flex items-center space-x-2">
            <button
              type="button"
              class="px-3.5 py-1.5 text-xs font-semibold rounded-lg bg-slate-100 hover:bg-slate-200 text-slate-700 transition"
              @click="scanAllCurrentTransactions"
            >
              🔄 Quét Lại Dữ Liệu Sao Kê
            </button>
          </div>
        </div>

        <AmlAlertList
          :alerts="activeAlerts"
          @open-str="onOpenStrModal"
        />
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
import { computed, onMounted } from 'vue';
import { useAmlStore } from '../stores/amlStore';
import { useReconciliationStore } from '../stores/reconciliationStore';
import SystemPromptInspector from '../components/aml/SystemPromptInspector.vue';
import AmlAlertList from '../components/aml/AmlAlertList.vue';
import StrReportModal from '../components/aml/StrReportModal.vue';
import type { AmlAlert } from '../types/aml';

const amlStore = useAmlStore();
const reconStore = useReconciliationStore();

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

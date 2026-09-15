<template>
  <div class="aml-alert-list space-y-4">
    <div class="flex items-center justify-between pb-2 border-b border-slate-200">
      <div class="flex items-center space-x-3">
        <div class="w-8 h-8 rounded-lg bg-rose-100 text-rose-700 flex items-center justify-center font-bold">
          !
        </div>
        <div>
          <h3 class="text-base font-bold text-slate-900">Danh Sách Cảnh Báo AML / CTF</h3>
          <p class="text-xs text-slate-500">Giám sát theo Thông tư 09/2023/TT-NHNN & Quyết định 11/2023/QĐ-TTg</p>
        </div>
      </div>
      <div class="flex items-center space-x-2">
        <span v-if="criticalCount > 0" class="px-2.5 py-0.5 rounded-full text-xs font-semibold bg-rose-100 text-rose-800">
          {{ criticalCount }} Nghiêm Trọng
        </span>
        <span v-if="highCount > 0" class="px-2.5 py-0.5 rounded-full text-xs font-semibold bg-amber-100 text-amber-800">
          {{ highCount }} Mức Cao
        </span>
        <span class="text-xs text-slate-400">Tổng: {{ alerts.length }}</span>
      </div>
    </div>

    <!-- Empty State -->
    <div v-if="alerts.length === 0" class="text-center py-8 bg-slate-50 rounded-xl border border-dashed border-slate-300">
      <p class="text-sm text-slate-500">Chưa phát hiện giao dịch bất thường hoặc vi phạm AML.</p>
      <p class="text-xs text-slate-400 mt-1">Hệ thống giám sát tự động kích hoạt khi có dữ liệu sao kê.</p>
    </div>

    <!-- Alert Cards -->
    <div v-else class="space-y-3">
      <div
        v-for="alert in alerts"
        :key="alert.alertId"
        class="p-4 rounded-xl border transition-all duration-200"
        :class="getCardBorderClass(alert.severity)"
      >
        <div class="flex items-start justify-between">
          <div class="space-y-1.5 flex-1 pr-4">
            <div class="flex items-center space-x-2">
              <span
                class="px-2 py-0.5 rounded text-xs font-bold"
                :class="getSeverityBadgeClass(alert.severity)"
              >
                {{ alert.severity }}
              </span>
              <span class="px-2 py-0.5 rounded text-xs font-semibold bg-slate-100 text-slate-700">
                {{ formatAnomalyType(alert.anomalyType) }}
              </span>
              <span class="text-xs text-slate-400">
                {{ formatTimestamp(alert.detectedAt) }}
              </span>
            </div>

            <p class="text-sm font-medium text-slate-800 leading-snug">
              {{ alert.reasoning }}
            </p>

            <div class="flex items-center space-x-4 text-xs text-slate-500 pt-1">
              <span>
                Số tiền: <strong class="text-slate-900 font-semibold">{{ formatVnd(alert.totalAmount) }} VND</strong>
              </span>
              <span>
                Giao dịch: <code class="text-slate-700 font-mono">{{ alert.involvedTransactionIds.join(', ') }}</code>
              </span>
              <span class="text-indigo-600 bg-indigo-50 px-2 py-0.5 rounded">
                {{ alert.statutoryRuleRef }}
              </span>
            </div>
          </div>

          <div class="flex items-center space-x-2">
            <button
              v-if="alert.suggestedStrReport"
              type="button"
              class="px-3 py-1.5 text-xs font-semibold rounded-lg bg-rose-600 hover:bg-rose-700 text-white shadow-sm transition"
              @click="$emit('open-str', alert)"
            >
              Lập Form STR
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { AmlAlert, AmlAnomalyType, AmlSeverity } from '../../types/aml';

const props = defineProps<{
  alerts: AmlAlert[];
}>();

defineEmits<{
  (e: 'open-str', alert: AmlAlert): void;
}>();

const criticalCount = computed(() => props.alerts.filter((a) => a.severity === 'CRITICAL').length);
const highCount = computed(() => props.alerts.filter((a) => a.severity === 'HIGH').length);

function formatVnd(val: number): string {
  return Number(val || 0).toLocaleString('vi-VN');
}

function formatTimestamp(isoStr: string): string {
  if (!isoStr) return '';
  try {
    const d = new Date(isoStr);
    return d.toLocaleTimeString('vi-VN', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  } catch {
    return isoStr;
  }
}

function formatAnomalyType(type: AmlAnomalyType): string {
  switch (type) {
    case 'STRUCTURING_SMURFING':
      return 'Gom/Chia Nhỏ (Smurfing)';
    case 'HIGH_VALUE':
      return 'Giá Trị Lớn (>= 400M)';
    case 'RAPID_PASS_THROUGH':
      return 'Trung Chuyển Nhanh';
    case 'NIGHT_VELOCITY':
      return 'Bất Thường Ngoại Giờ';
    case 'WATCHLIST_HIT':
      return 'Từ Khóa Danh Sách Đen';
    default:
      return type;
  }
}

function getSeverityBadgeClass(sev: AmlSeverity): string {
  switch (sev) {
    case 'CRITICAL':
      return 'bg-rose-600 text-white';
    case 'HIGH':
      return 'bg-amber-500 text-white';
    case 'MEDIUM':
      return 'bg-blue-500 text-white';
    default:
      return 'bg-slate-500 text-white';
  }
}

function getCardBorderClass(sev: AmlSeverity): string {
  switch (sev) {
    case 'CRITICAL':
      return 'border-rose-300 bg-rose-50/40 hover:bg-rose-50/70';
    case 'HIGH':
      return 'border-amber-300 bg-amber-50/40 hover:bg-amber-50/70';
    default:
      return 'border-slate-200 bg-white hover:bg-slate-50';
  }
}
</script>

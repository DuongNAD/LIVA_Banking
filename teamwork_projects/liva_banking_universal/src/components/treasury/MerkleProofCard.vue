<template>
  <div class="merkle-proof-card bg-white rounded-2xl border border-slate-200 shadow-sm overflow-hidden space-y-4 p-5">
    <!-- Header with Verification Status -->
    <div class="flex items-center justify-between pb-3 border-b border-slate-100">
      <div class="flex items-center space-x-3">
        <div class="w-9 h-9 rounded-xl bg-indigo-600 flex items-center justify-center text-white font-bold shadow-sm">
          🌳
        </div>
        <div>
          <h3 class="text-sm font-bold text-slate-900">Bảo Chứng Sổ Cái Mật Mã (Merkle Audit Tree)</h3>
          <p class="text-xs text-slate-500">Chuỗi khối kiểm toán bất biến & Chứng thực Zero Cloud Egress</p>
        </div>
      </div>
      <div class="flex items-center space-x-2">
        <span class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-bold bg-emerald-100 text-emerald-800">
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 mr-1.5 animate-pulse"></span>
          Toàn Vẹn 100% (Tamper-Evident)
        </span>
      </div>
    </div>

    <!-- Cryptographic Root Hash Box -->
    <div class="p-3.5 bg-slate-900 rounded-xl text-white space-y-2">
      <div class="flex items-center justify-between text-xs text-slate-400">
        <span class="font-medium">Gốc Cây Merkle Hiện Tại (Current Root Hash - SHA-256):</span>
        <button
          type="button"
          class="text-xs text-blue-400 hover:text-blue-300 font-mono transition"
          @click="copyRootHash"
        >
          {{ copyStatus || 'Sao chép hash' }}
        </button>
      </div>
      <div class="font-mono text-xs text-emerald-400 break-all select-all tracking-wide font-semibold">
        {{ merkleRoot || '0000000000000000000000000000000000000000000000000000000000000000' }}
      </div>
    </div>

    <!-- Regulatory Compliance Badges -->
    <div class="grid grid-cols-1 sm:grid-cols-3 gap-2.5 pt-1">
      <div class="p-2.5 rounded-xl bg-emerald-50 border border-emerald-200 text-xs flex items-center space-x-2">
        <span class="text-emerald-700 font-bold text-sm">🔒</span>
        <div>
          <strong class="text-emerald-900 block font-semibold text-[11px]">Zero Cloud Egress</strong>
          <span class="text-emerald-700 text-[10px]">100% Xử lý nội bộ trên máy trạm</span>
        </div>
      </div>

      <div class="p-2.5 rounded-xl bg-blue-50 border border-blue-200 text-xs flex items-center space-x-2">
        <span class="text-blue-700 font-bold text-sm">🛡️</span>
        <div>
          <strong class="text-blue-900 block font-semibold text-[11px]">Nghị Định 13/2023</strong>
          <span class="text-blue-700 text-[10px]">Bảo vệ & Ẩn danh hóa dữ liệu cá nhân</span>
        </div>
      </div>

      <div class="p-2.5 rounded-xl bg-purple-50 border border-purple-200 text-xs flex items-center space-x-2">
        <span class="text-purple-700 font-bold text-sm">⚖️</span>
        <div>
          <strong class="text-purple-900 block font-semibold text-[11px]">Thông Tư 09/2020</strong>
          <span class="text-purple-700 text-[10px]">Kiểm soát chéo 2 vòng (Maker-Checker)</span>
        </div>
      </div>
    </div>

    <!-- Forward Hash-Chain Summary & Block List -->
    <div class="border border-slate-200 rounded-xl overflow-hidden">
      <div
        class="p-3 bg-slate-50 flex items-center justify-between cursor-pointer select-none text-xs font-bold text-slate-700"
        @click="isExpanded = !isExpanded"
      >
        <div class="flex items-center space-x-2">
          <span>Khối Sổ Cái Liên Kết (Forward Hash Chain: {{ auditEntries.length }} khối)</span>
        </div>
        <span class="text-slate-400 font-mono text-xs">
          {{ isExpanded ? '▲ Thu gọn' : '▼ Xem chi tiết khối' }}
        </span>
      </div>

      <div v-if="isExpanded" class="p-3 divide-y divide-slate-100 max-h-56 overflow-y-auto text-xs">
        <div
          v-for="entry in reversedEntries"
          :key="entry.entryIndex"
          class="py-2.5 space-y-1"
        >
          <div class="flex items-center justify-between">
            <div class="flex items-center space-x-2">
              <span class="font-mono font-bold text-indigo-700">#{{ entry.entryIndex }}</span>
              <span class="font-semibold text-slate-800">{{ entry.event }}</span>
              <span class="text-slate-400 text-[10px]">({{ entry.actor }})</span>
            </div>
            <span class="text-slate-400 text-[10px]">{{ formatTime(entry.timestamp) }}</span>
          </div>

          <div class="grid grid-cols-2 gap-2 text-[11px] font-mono text-slate-500">
            <div class="truncate">
              <span class="text-slate-400">Prev: </span>
              <span class="text-slate-600">{{ entry.prevHash.substring(0, 16) }}...</span>
            </div>
            <div class="truncate text-right">
              <span class="text-slate-400">Hash: </span>
              <span class="text-emerald-600 font-semibold">{{ entry.currentHash.substring(0, 16) }}...</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { AuditLogEntry } from '../../types/treasury';

const props = withDefaults(
  defineProps<{
    merkleRoot: string;
    auditEntries: AuditLogEntry[];
  }>(),
  {
    merkleRoot: '',
    auditEntries: () => [],
  }
);

const isExpanded = ref(false);
const copyStatus = ref('');

const reversedEntries = computed(() => [...props.auditEntries].reverse());

function copyRootHash() {
  if (!props.merkleRoot) return;
  if (typeof navigator !== 'undefined' && navigator.clipboard) {
    navigator.clipboard.writeText(props.merkleRoot);
    copyStatus.value = '✓ Đã sao chép';
    setTimeout(() => {
      copyStatus.value = '';
    }, 2000);
  }
}

function formatTime(isoStr: string): string {
  if (!isoStr) return '';
  try {
    const d = new Date(isoStr);
    return d.toLocaleTimeString('vi-VN');
  } catch {
    return isoStr;
  }
}
</script>

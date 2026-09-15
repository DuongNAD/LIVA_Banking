<template>
  <div>
    <!-- Trigger Button (Floating or Embedded) -->
    <button
      v-if="!isOpen"
      type="button"
      class="fixed bottom-6 right-6 z-40 px-4 py-3 bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-bold rounded-2xl shadow-xl hover:shadow-2xl hover:scale-105 transition-all duration-200 flex items-center space-x-2.5 copilot-drawer-trigger"
      @click="$emit('update:isOpen', true)"
    >
      <span class="text-xl">🤖</span>
      <span class="text-xs">Trợ Lý Copilot 2D</span>
      <span
        v-if="pendingAlertsCount > 0"
        class="w-2.5 h-2.5 rounded-full bg-rose-400 animate-ping ml-1"
      ></span>
    </button>

    <!-- Drawer Overlay Backing -->
    <div
      v-if="isOpen"
      class="fixed inset-0 z-50 bg-slate-900/40 backdrop-blur-xs transition-opacity duration-300"
      @click="$emit('update:isOpen', false)"
    ></div>

    <!-- Sliding 2D Drawer -->
    <aside
      v-if="isOpen"
      class="fixed top-0 right-0 z-50 h-full w-full max-w-md bg-white shadow-2xl border-l border-slate-200 flex flex-col transform transition-transform duration-300 ease-out copilot-drawer"
    >
      <!-- Drawer Header -->
      <div class="px-5 py-4 bg-slate-900 text-white flex items-center justify-between shadow-md">
        <div class="flex items-center space-x-3">
          <div class="w-9 h-9 rounded-xl bg-blue-600 flex items-center justify-center font-bold text-white shadow-sm">
            🤖
          </div>
          <div>
            <h3 class="text-sm font-bold tracking-tight">LIVA Banking Copilot</h3>
            <p class="text-[11px] text-slate-300">Trợ Lý Quản Trị Ngân Quỹ & Đối Soát Liên Ngân Hàng 2D</p>
          </div>
        </div>
        <button
          type="button"
          class="text-slate-400 hover:text-white p-1 rounded-lg transition"
          @click="$emit('update:isOpen', false)"
        >
          ✕
        </button>
      </div>

      <!-- Live Context Pills -->
      <div class="px-4 py-2 bg-slate-50 border-b border-slate-200 grid grid-cols-2 gap-2 text-[11px]">
        <div class="p-2 rounded-lg bg-white border border-slate-200">
          <span class="text-slate-400 block text-[10px]">Thanh Khoản:</span>
          <span class="font-bold text-slate-900">{{ formatVnd(liveBalance) }} VND</span>
        </div>
        <div class="p-2 rounded-lg bg-white border border-slate-200">
          <span class="text-slate-400 block text-[10px]">Tỷ Lệ Khớp:</span>
          <span class="font-bold text-emerald-600">{{ matchRate }}% (Đạt chuẩn)</span>
        </div>
      </div>

      <!-- Chat History (Scrollable) -->
      <div ref="chatContainer" class="flex-1 p-4 overflow-y-auto space-y-3.5 text-xs">
        <div
          v-for="msg in messages"
          :key="msg.id"
          class="flex flex-col"
          :class="msg.sender === 'user' ? 'items-end' : 'items-start'"
        >
          <div
            class="max-w-[85%] rounded-2xl px-4 py-2.5 leading-relaxed shadow-xs"
            :class="
              msg.sender === 'user'
                ? 'bg-blue-600 text-white rounded-tr-xs'
                : 'bg-slate-100 text-slate-800 rounded-tl-xs border border-slate-200'
            "
          >
            <p class="whitespace-pre-wrap">{{ msg.text }}</p>

            <!-- Metrics Card if provided -->
            <div
              v-if="msg.metrics && Object.keys(msg.metrics).length > 0"
              class="mt-2 pt-2 border-t text-[11px] font-mono"
              :class="msg.sender === 'user' ? 'border-blue-400 text-blue-100' : 'border-slate-200 text-slate-600'"
            >
              <div v-if="msg.metrics.runwayDays !== undefined">
                <span>Dự báo an toàn: </span>
                <strong>{{ msg.metrics.runwayDays }} ngày</strong>
              </div>
              <div v-if="msg.metrics.matchRate !== undefined">
                <span>Tỷ lệ đối soát: </span>
                <strong>{{ msg.metrics.matchRate }}%</strong>
              </div>
              <div v-if="msg.metrics.topExpenseAmount !== undefined">
                <span>Chi phí cao nhất: </span>
                <strong>{{ formatVnd(msg.metrics.topExpenseAmount) }} VND</strong>
              </div>
              <div v-if="msg.metrics.alertCount !== undefined">
                <span>Cảnh báo AML: </span>
                <strong>{{ msg.metrics.alertCount }} phát hiện</strong>
              </div>
            </div>
          </div>
          <span class="text-[10px] text-slate-400 mt-1 px-1">
            {{ formatTime(msg.timestamp) }}
          </span>
        </div>
      </div>

      <!-- Quick Prompt Chips -->
      <div class="px-4 py-2 border-t border-slate-100 bg-slate-50 space-y-1.5">
        <span class="text-[10px] font-bold text-slate-400 uppercase tracking-wider block">Gợi ý câu hỏi:</span>
        <div class="flex flex-wrap gap-1.5">
          <button
            v-for="chip in quickChips"
            :key="chip"
            type="button"
            class="px-2.5 py-1 rounded-full text-[11px] bg-white border border-slate-300 text-slate-700 hover:border-blue-500 hover:text-blue-600 transition"
            @click="submitQuickQuery(chip)"
          >
            {{ chip }}
          </button>
        </div>
      </div>

      <!-- Input Bar -->
      <div class="p-3 bg-white border-t border-slate-200 flex items-center space-x-2">
        <input
          v-model="inputQuery"
          type="text"
          placeholder="Hỏi về thanh khoản, tỷ lệ đối soát, AML..."
          class="flex-1 px-3.5 py-2 text-xs rounded-xl border border-slate-300 focus:outline-none focus:ring-2 focus:ring-blue-500"
          @keydown.enter.prevent="handleSend"
        />
        <button
          type="button"
          class="px-4 py-2 rounded-xl bg-blue-600 hover:bg-blue-700 text-white font-bold text-xs transition shadow-sm"
          :disabled="!inputQuery.trim()"
          @click="handleSend"
        >
          Gửi
        </button>
      </div>
    </aside>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue';
import type { CopilotMessage } from '../../stores/treasuryStore';

const props = withDefaults(
  defineProps<{
    isOpen: boolean;
    messages: CopilotMessage[];
    liveBalance?: number;
    matchRate?: number;
    pendingAlertsCount?: number;
  }>(),
  {
    liveBalance: 6_200_000_000,
    matchRate: 99.8,
    pendingAlertsCount: 3,
  }
);

const emit = defineEmits<{
  (e: 'update:isOpen', val: boolean): void;
  (e: 'send', query: string): void;
}>();

const inputQuery = ref('');
const chatContainer = ref<HTMLElement | null>(null);

const quickChips = [
  'Tình hình thanh khoản hiện tại?',
  'Tỷ lệ đối soát tháng 8 đạt bao nhiêu?',
  'Khoản chi phí lớn nhất trong kỳ?',
  'Có giao dịch nào đáng ngờ không?',
  'Lệnh chi đang chờ phê duyệt?',
];

function handleSend() {
  if (!inputQuery.value.trim()) return;
  const q = inputQuery.value.trim();
  inputQuery.value = '';
  emit('send', q);
  scrollToBottom();
}

function submitQuickQuery(chipText: string) {
  emit('send', chipText);
  scrollToBottom();
}

function scrollToBottom() {
  nextTick(() => {
    if (chatContainer.value) {
      chatContainer.value.scrollTop = chatContainer.value.scrollHeight;
    }
  });
}

function formatVnd(val: number): string {
  return Number(val || 0).toLocaleString('vi-VN');
}

function formatTime(isoStr: string): string {
  if (!isoStr) return '';
  try {
    const d = new Date(isoStr);
    return d.toLocaleTimeString('vi-VN', { hour: '2-digit', minute: '2-digit' });
  } catch {
    return isoStr;
  }
}
</script>

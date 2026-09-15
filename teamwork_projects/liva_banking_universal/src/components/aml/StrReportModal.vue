<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 overflow-y-auto bg-slate-900/60 backdrop-blur-sm flex items-center justify-center p-4">
    <div class="bg-white rounded-2xl shadow-2xl max-w-3xl w-full border border-slate-200 overflow-hidden flex flex-col max-h-[90vh]">
      <!-- Modal Header -->
      <div class="px-6 py-4 bg-slate-900 text-white flex items-center justify-between">
        <div class="flex items-center space-x-3">
          <div class="w-8 h-8 rounded-lg bg-rose-600 flex items-center justify-center font-bold text-white text-sm">
            STR
          </div>
          <div>
            <h3 class="text-base font-bold">Báo Cáo Giao Dịch Đáng Ngờ (Form STR)</h3>
            <p class="text-xs text-slate-300">Phụ lục II Thông tư số 09/2023/TT-NHNN — Cục Phòng, chống rửa tiền</p>
          </div>
        </div>
        <button
          type="button"
          class="text-slate-400 hover:text-white transition p-1 rounded-lg cursor-pointer"
          @click="$emit('close')"
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6 6 18M6 6l12 12"/></svg>
        </button>
      </div>

      <!-- Modal Body (Scrollable) -->
      <div class="p-6 overflow-y-auto space-y-6 flex-1 text-slate-800 text-sm">
        <!-- Form Header Summary -->
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-4 p-4 bg-slate-50 rounded-xl border border-slate-200 text-xs">
          <div>
            <span class="text-slate-500 block">Đơn Vị Báo Cáo:</span>
            <span class="font-bold text-slate-900">{{ form?.reportingEntity }}</span>
          </div>
          <div>
            <span class="text-slate-500 block">Ngày Lập:</span>
            <span class="font-bold text-slate-900">{{ form?.reportDate }}</span>
          </div>
          <div>
            <span class="text-slate-500 block">Dấu Hiệu:</span>
            <span class="font-bold text-rose-600">{{ form?.alertType }}</span>
          </div>
          <div>
            <span class="text-slate-500 block">Mức Độ Rủi Ro:</span>
            <span class="font-bold text-slate-900">{{ form?.severity || 'CRITICAL' }}</span>
          </div>
        </div>

        <!-- Section II: Suspect Info -->
        <div class="border-t border-slate-200 pt-4 space-y-2">
          <h4 class="text-xs font-bold uppercase tracking-wider text-slate-500">Phần II: Đối Tượng Bị Báo Cáo</h4>
          <div class="grid grid-cols-2 gap-4 bg-white p-3 rounded-lg border border-slate-200">
            <div>
              <span class="text-xs text-slate-400 block">Họ và tên / Đơn vị:</span>
              <span class="font-semibold text-slate-900">{{ form?.suspectName }}</span>
            </div>
            <div>
              <span class="text-xs text-slate-400 block">Số tài khoản giao dịch:</span>
              <span class="font-mono font-semibold text-slate-900">{{ form?.suspectAccount }}</span>
            </div>
          </div>
        </div>

        <!-- Section III: Transaction Details -->
        <div class="border-t border-slate-200 pt-4 space-y-2">
          <h4 class="text-xs font-bold uppercase tracking-wider text-slate-500">Phần III: Chi Tiết Giao Dịch Đáng Ngờ</h4>
          <div class="grid grid-cols-3 gap-4 bg-white p-3 rounded-lg border border-slate-200">
            <div>
              <span class="text-xs text-slate-400 block">Số lượng giao dịch:</span>
              <span class="font-semibold text-slate-900">{{ form?.transactionCount }}</span>
            </div>
            <div>
              <span class="text-xs text-slate-400 block">Tổng số tiền:</span>
              <span class="font-bold text-rose-600">{{ formatVnd(form?.totalVndAmount) }} VND</span>
            </div>
            <div>
              <span class="text-xs text-slate-400 block">Đồng tiền:</span>
              <span class="font-semibold text-slate-900">VND</span>
            </div>
          </div>
        </div>

        <!-- Section IV: Suspicion Grounds & Narrative -->
        <div class="border-t border-slate-200 pt-4 space-y-2">
          <h4 class="text-xs font-bold uppercase tracking-wider text-slate-500">Phần IV: Căn Cứ Pháp Lý & Lý Do Nghi Ngờ (Explainable AI)</h4>
          <div class="p-4 bg-slate-50 rounded-xl border border-slate-200 space-y-2">
            <div class="text-xs text-indigo-700 font-semibold">
              Căn cứ: {{ form?.statutoryRuleRef }}
            </div>
            <p class="text-xs leading-relaxed text-slate-700 whitespace-pre-line font-mono bg-white p-3 rounded border border-slate-200">
              {{ form?.narrativeSummary }}
            </p>
          </div>
        </div>

        <!-- Section V: Officer Notes -->
        <div class="border-t border-slate-200 pt-4 space-y-2">
          <h4 class="text-xs font-bold uppercase tracking-wider text-slate-500">Phần V: Ý Kiến & Biện Pháp Xử Lý Của Cán Bộ Tuân Thủ</h4>
          <textarea
            :value="form?.complianceOfficerNotes"
            rows="3"
            class="w-full text-xs p-3 rounded-xl border border-slate-300 focus:ring-2 focus:ring-slate-900 focus:outline-none"
            placeholder="Nhập ghi chú hoặc đề xuất biện pháp xử lý..."
            @input="onNotesChange(($event.target as HTMLTextAreaElement).value)"
          ></textarea>
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="px-6 py-4 bg-slate-50 border-t border-slate-200 flex items-center justify-between">
        <div class="flex items-center space-x-2">
          <button
            type="button"
            class="px-3 py-2 text-xs font-semibold rounded-lg bg-white border border-slate-300 hover:bg-slate-100 text-slate-700 transition"
            @click="copyJson"
          >
            {{ copyJsonStatus ? 'Đã Sao Chép JSON' : 'Sao Chép JSON' }}
          </button>
          <button
            type="button"
            class="px-3 py-2 text-xs font-semibold rounded-lg bg-white border border-slate-300 hover:bg-slate-100 text-slate-700 transition"
            @click="copyDocument"
          >
            {{ copyDocStatus ? 'Đã Sao Chép Văn Bản' : 'Sao Chép Báo Cáo' }}
          </button>
        </div>
        <div class="flex items-center space-x-2">
          <button
            type="button"
            class="px-4 py-2 text-xs font-semibold rounded-lg bg-slate-900 hover:bg-slate-800 text-white shadow-sm transition"
            @click="downloadReport"
          >
            Tải File Báo Cáo (.txt)
          </button>
          <button
            type="button"
            class="px-4 py-2 text-xs font-semibold rounded-lg bg-slate-200 hover:bg-slate-300 text-slate-700 transition"
            @click="$emit('close')"
          >
            Đóng
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import type { FormStrData } from '../../types/aml';

const props = defineProps<{
  isOpen: boolean;
  form: FormStrData | null;
  officialDocumentText?: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'update-notes', notes: string): void;
}>();

const copyJsonStatus = ref(false);
const copyDocStatus = ref(false);

function formatVnd(val?: number): string {
  return Number(val || 0).toLocaleString('vi-VN');
}

function onNotesChange(val: string) {
  emit('update-notes', val);
}

function copyJson() {
  if (!props.form) return;
  navigator.clipboard.writeText(JSON.stringify(props.form, null, 2));
  copyJsonStatus.value = true;
  setTimeout(() => (copyJsonStatus.value = false), 2000);
}

function copyDocument() {
  if (!props.officialDocumentText) return;
  navigator.clipboard.writeText(props.officialDocumentText);
  copyDocStatus.value = true;
  setTimeout(() => (copyDocStatus.value = false), 2000);
}

function downloadReport() {
  if (!props.officialDocumentText || !props.form) return;
  const blob = new Blob([props.officialDocumentText], { type: 'text/plain;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `STR_${props.form.reportDate}_${props.form.alertType}.txt`;
  a.click();
  URL.revokeObjectURL(url);
}
</script>

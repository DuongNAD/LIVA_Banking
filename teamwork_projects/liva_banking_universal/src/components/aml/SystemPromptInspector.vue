<template>
  <div class="system-prompt-inspector bg-white rounded-2xl border border-slate-200 shadow-sm overflow-hidden">
    <!-- Header -->
    <div class="p-6 border-b border-slate-200 flex flex-wrap items-center justify-between gap-4 bg-slate-50/50">
      <div class="flex items-center space-x-3">
        <div class="w-10 h-10 rounded-xl bg-slate-900 text-white flex items-center justify-center font-mono font-bold text-lg">
          AI
        </div>
        <div>
          <h2 class="text-lg font-bold text-slate-900">Explainable System Prompt Inspector</h2>
          <p class="text-xs text-slate-500">
            Thanh tra câu lệnh hệ thống & Tinh chỉnh siêu tham số giám sát AML (Thông tư 09/2023/TT-NHNN)
          </p>
        </div>
      </div>

      <div class="flex items-center space-x-3">
        <button
          type="button"
          class="px-3.5 py-2 text-xs font-semibold rounded-xl bg-white border border-slate-300 text-slate-700 hover:bg-slate-100 transition shadow-sm"
          @click="store.resetToSbvDefaults"
        >
          Khôi Phục Chuẩn NHNN
        </button>
        <button
          type="button"
          class="px-4 py-2 text-xs font-semibold rounded-xl bg-slate-900 text-white hover:bg-slate-800 transition shadow-sm flex items-center space-x-1.5"
          @click="saveConfig"
        >
          <span>Lưu Tham Số & Prompt</span>
        </button>
      </div>
    </div>

    <!-- Dual-Pane Layout -->
    <div class="grid grid-cols-1 lg:grid-cols-12 divide-y lg:divide-y-0 lg:divide-x divide-slate-200">
      <!-- LEFT PANE: Active Prompt & Hyperparameters (7 cols) -->
      <div class="lg:col-span-7 p-6 space-y-6">
        <div>
          <div class="flex items-center justify-between mb-2">
            <label class="text-xs font-bold uppercase tracking-wider text-slate-500">
              Active AML System Prompt (Chỉ Thị Giám Sát Cục Bộ)
            </label>
            <span class="text-xs text-emerald-600 font-medium">● Không Rò Rỉ Dữ Liệu (Zero Egress)</span>
          </div>
          <textarea
            v-model="promptInput"
            rows="9"
            class="w-full font-mono text-xs p-4 rounded-xl border border-slate-300 bg-slate-900 text-slate-100 focus:ring-2 focus:ring-slate-900 focus:outline-none leading-relaxed"
            placeholder="System prompt text..."
            @input="onPromptChange"
          ></textarea>
        </div>

        <!-- Hyperparameter Tuner -->
        <div class="space-y-4">
          <h3 class="text-xs font-bold uppercase tracking-wider text-slate-500">
            Siêu Tham Số Rà Soát Định Lượng (Deterministic Thresholds)
          </h3>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <!-- 1. High-Value Threshold -->
            <div class="p-3.5 bg-slate-50 rounded-xl border border-slate-200 space-y-1">
              <div class="flex justify-between text-xs">
                <span class="font-medium text-slate-700">Ngưỡng Giao Dịch Lớn (VND):</span>
                <span class="font-bold text-slate-900 font-mono">{{ formatVnd(localConfig.highValueThreshold) }}</span>
              </div>
              <input
                v-model.number="localConfig.highValueThreshold"
                type="range"
                min="100000000"
                max="1000000000"
                step="50000000"
                class="w-full accent-slate-900"
                @change="applyConfigChanges"
              />
              <span class="text-[10px] text-slate-400 block">Chuẩn QĐ 11/2023: 400.000.000 VND</span>
            </div>

            <!-- 2. Structuring Window Hours -->
            <div class="p-3.5 bg-slate-50 rounded-xl border border-slate-200 space-y-1">
              <div class="flex justify-between text-xs">
                <span class="font-medium text-slate-700">Cửa Sổ Gom Tiền / Smurfing:</span>
                <span class="font-bold text-slate-900 font-mono">{{ localConfig.structuringWindowHours }} Giờ</span>
              </div>
              <input
                v-model.number="localConfig.structuringWindowHours"
                type="range"
                min="12"
                max="72"
                step="12"
                class="w-full accent-slate-900"
                @change="applyConfigChanges"
              />
              <span class="text-[10px] text-slate-400 block">≥ 3 giao dịch dưới 400M trong thời gian này</span>
            </div>

            <!-- 3. Pass-Through Drain Rate -->
            <div class="p-3.5 bg-slate-50 rounded-xl border border-slate-200 space-y-1">
              <div class="flex justify-between text-xs">
                <span class="font-medium text-slate-700">Tỷ Lệ Rút Tiền Trung Chuyển:</span>
                <span class="font-bold text-slate-900 font-mono">{{ (localConfig.passThroughMinDrainRate * 100).toFixed(0) }}%</span>
              </div>
              <input
                v-model.number="localConfig.passThroughMinDrainRate"
                type="range"
                min="0.80"
                max="0.99"
                step="0.05"
                class="w-full accent-slate-900"
                @change="applyConfigChanges"
              />
              <span class="text-[10px] text-slate-400 block">Vào ≥ 100M, rút ra trong ≤ 30 phút</span>
            </div>

            <!-- 4. Night Window -->
            <div class="p-3.5 bg-slate-50 rounded-xl border border-slate-200 space-y-1">
              <div class="flex justify-between text-xs">
                <span class="font-medium text-slate-700">Khung Giờ Ngoại Giờ & Hạn Mức:</span>
                <span class="font-bold text-slate-900 font-mono">{{ localConfig.nightStartHour }}:00 - 0{{ localConfig.nightEndHour }}:00</span>
              </div>
              <div class="flex items-center space-x-2 pt-1">
                <input
                  v-model.number="localConfig.nightMinAmount"
                  type="number"
                  step="10000000"
                  class="w-full text-xs p-1.5 rounded border border-slate-300 font-mono"
                  @change="applyConfigChanges"
                />
                <span class="text-xs text-slate-500 whitespace-nowrap">VND</span>
              </div>
              <span class="text-[10px] text-slate-400 block">Mặc định: 50.000.000 VND</span>
            </div>
          </div>
        </div>
      </div>

      <!-- RIGHT PANE: Real-Time Evaluation Sandbox (5 cols) -->
      <div class="lg:col-span-5 p-6 bg-slate-50/30 flex flex-col justify-between space-y-6">
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <h3 class="text-xs font-bold uppercase tracking-wider text-slate-500">
              Phòng Thử Nghiệm Đối Kháng (Evaluation Sandbox)
            </h3>
            <span class="text-xs font-semibold px-2 py-0.5 rounded bg-indigo-50 text-indigo-700">
              4 Kịch Bản Sẵn
            </span>
          </div>

          <!-- Preset Selector -->
          <div class="space-y-2">
            <div
              v-for="preset in AML_SANDBOX_PRESETS"
              :key="preset.id"
              class="p-3 rounded-xl border transition cursor-pointer"
              :class="
                store.sandboxPresetId === preset.id
                  ? 'border-slate-900 bg-white shadow-sm ring-1 ring-slate-900'
                  : 'border-slate-200 bg-white/70 hover:bg-white'
              "
              @click="store.selectPreset(preset.id)"
            >
              <div class="flex items-center justify-between">
                <span class="text-xs font-bold text-slate-900">{{ preset.name }}</span>
                <span class="text-[10px] font-mono text-slate-400">{{ preset.transactions.length }} GD</span>
              </div>
              <p class="text-[11px] text-slate-500 mt-1 leading-snug">{{ preset.description }}</p>
            </div>
          </div>

          <!-- Action Button -->
          <button
            type="button"
            class="w-full py-2.5 px-4 rounded-xl bg-slate-900 hover:bg-slate-800 text-white font-semibold text-xs transition shadow-sm flex items-center justify-center space-x-2"
            :disabled="store.isEvaluating"
            @click="store.runSandboxEvaluation"
          >
            <span v-if="store.isEvaluating">Đang Đánh Giá...</span>
            <span v-else>Kiểm Thử Prompt Với Kịch Bản Này</span>
          </button>

          <!-- Sandbox Results -->
          <div class="space-y-2 pt-2">
            <div class="flex items-center justify-between text-xs">
              <span class="font-bold text-slate-700">Kết Quả Phân Tích Của AI Engine:</span>
              <span
                class="px-2 py-0.5 rounded-full text-[10px] font-bold"
                :class="store.sandboxAlerts.length > 0 ? 'bg-rose-100 text-rose-800' : 'bg-emerald-100 text-emerald-800'"
              >
                {{ store.sandboxAlerts.length }} Cảnh Báo
              </span>
            </div>

            <div v-if="store.sandboxAlerts.length === 0" class="p-4 bg-white rounded-xl border border-slate-200 text-center text-xs text-slate-500">
              Kịch bản an toàn. Không phát hiện vi phạm theo cấu hình hiện tại.
            </div>

            <div v-else class="space-y-2 max-h-60 overflow-y-auto">
              <div
                v-for="alert in store.sandboxAlerts"
                :key="alert.alertId"
                class="p-3 bg-white rounded-xl border border-rose-200 space-y-1.5 text-xs"
              >
                <div class="flex items-center justify-between">
                  <span class="font-bold text-rose-700">{{ alert.anomalyType }}</span>
                  <span class="text-[10px] font-bold px-1.5 py-0.5 rounded bg-rose-600 text-white">
                    {{ alert.severity }}
                  </span>
                </div>
                <p class="text-slate-700 text-[11px] leading-snug">{{ alert.reasoning }}</p>
                <div class="flex items-center justify-between pt-1 text-[10px] text-slate-400">
                  <span>{{ alert.statutoryRuleRef }}</span>
                  <button
                    type="button"
                    class="text-rose-600 font-bold hover:underline"
                    @click="store.openStrModal(alert, store.sandboxTransactions)"
                  >
                    Xem Form STR →
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- STR Modal Container -->
    <StrReportModal
      :is-open="store.isStrModalOpen"
      :form="store.generatedStr"
      :official-document-text="store.officialStrDocumentText"
      @close="store.closeStrModal"
      @update-notes="store.updateComplianceNotes"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue';
import { useAmlStore, AML_SANDBOX_PRESETS } from '../../stores/amlStore';
import StrReportModal from './StrReportModal.vue';

const store = useAmlStore();

const promptInput = ref(store.activePromptText);
const localConfig = reactive({ ...store.config });

watch(
  () => store.activePromptText,
  (val) => {
    promptInput.value = val;
  }
);

watch(
  () => store.config,
  (cfg) => {
    Object.assign(localConfig, cfg);
  },
  { deep: true }
);

function onPromptChange() {
  store.updatePromptText(promptInput.value);
}

function applyConfigChanges() {
  store.updateConfig(localConfig);
  promptInput.value = store.activePromptText;
  store.runSandboxEvaluation();
}

function saveConfig() {
  applyConfigChanges();
}

function formatVnd(val?: number): string {
  return Number(val || 0).toLocaleString('vi-VN');
}
</script>

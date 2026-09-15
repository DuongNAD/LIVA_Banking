<script setup lang="ts">
/**
 * FinancialAssistantDrawer.vue
 * Pure 2D Conversational Assistant Drawer with 12-Bar Reactive SVG Soundwave.
 * Zero WebGL / Zero 3D Canvas.
 */
import { ref, onMounted, onUnmounted } from 'vue';

defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

export interface ChatMessage {
  id: string;
  sender: 'user' | 'assistant';
  time: string;
  text: string;
  isWarning?: boolean;
  actionButtons?: Array<{ label: string; action: string }>;
}

const messages = ref<ChatMessage[]>([
  {
    id: 'msg-1',
    sender: 'user',
    time: '10:29 AM',
    text: 'Tổng số dư khả dụng hôm nay là bao nhiêu và tuần tới có rủi ro thiếu hụt không?',
  },
  {
    id: 'msg-2',
    sender: 'assistant',
    time: '10:29 AM',
    text: `Dạ, tổng vị thế tiền mặt khả dụng hiện tại là 2,235,830,000 VND trên 2 tài khoản chính:
• Vietcombank: 1,450,230,000 VND (Đã đối soát 99.97%)
• Techcombank: 785,600,000 VND (Đã đối soát 99.93%)

⚠️ CẢNH BÁO DÒNG TIỀN TUẦN TỚI:
Vào ngày Thứ Năm (18/10), doanh nghiệp có lịch thanh toán thuế VAT 800,000,000 VND và nhà cung cấp 1,200,000,000 VND. Số dư tài khoản thanh toán VCB có nguy cơ chạm ngưỡng an toàn tối thiểu nếu các khoản thu nợ khách hàng chưa kịp về.

💡 KHUYẾN NGHỊ ĐIỀU PHỐI VỐN:
Điều chuyển 500,000,000 VND từ Techcombank sang Vietcombank trước 14:00 ngày 17/10 để đảm bảo thanh khoản.`,
    isWarning: true,
    actionButtons: [
      { label: 'Phê duyệt lệnh điều phối vốn', action: 'APPROVE_TRANSFER' },
      { label: 'Xem chi tiết dòng tiền 30 ngày', action: 'VIEW_CASHFLOW' },
    ],
  },
]);

const inputText = ref('');
const voiceState = ref<'PASSIVE' | 'LISTENING' | 'PROCESSING' | 'SPEAKING'>('PASSIVE');
const isListening = ref(false);

// 12-bar reactive soundwave energy values
const soundwaveBars = ref<number[]>([15, 25, 45, 70, 90, 100, 85, 60, 40, 25, 18, 10]);

let animationTimer: ReturnType<typeof setInterval> | null = null;

function updateSoundwave() {
  if (voiceState.value === 'LISTENING' || voiceState.value === 'SPEAKING') {
    soundwaveBars.value = soundwaveBars.value.map(() => Math.floor(Math.random() * 85) + 15);
  } else if (voiceState.value === 'PROCESSING') {
    soundwaveBars.value = soundwaveBars.value.map((_, i) => Math.floor(Math.sin((Date.now() / 200) + i) * 30 + 40));
  } else {
    soundwaveBars.value = [10, 15, 20, 25, 30, 25, 20, 18, 15, 12, 10, 8];
  }
}

onMounted(() => {
  animationTimer = setInterval(updateSoundwave, 120);
});

onUnmounted(() => {
  if (animationTimer) clearInterval(animationTimer);
});

function toggleVoice() {
  if (isListening.value) {
    // Stop listening / Barge-in trigger
    isListening.value = false;
    voiceState.value = 'PROCESSING';
    setTimeout(() => {
      voiceState.value = 'PASSIVE';
    }, 600);
  } else {
    isListening.value = true;
    voiceState.value = 'LISTENING';
  }
}

import { useBankingStore } from '../../stores/bankingStore';

const bankingStore = useBankingStore();

function formatVnd(val: number): string {
  return new Intl.NumberFormat('en-US').format(val) + ' VND';
}

function sendQuery(textToSend?: string) {
  const query = textToSend || inputText.value.trim();
  if (!query) return;

  const now = new Date().toLocaleTimeString('vi-VN', { hour: '2-digit', minute: '2-digit' });
  messages.value.push({
    id: `msg-${Date.now()}`,
    sender: 'user',
    time: now,
    text: query,
  });

  inputText.value = '';
  voiceState.value = 'PROCESSING';

  setTimeout(() => {
    voiceState.value = 'SPEAKING';
    const qLower = query.toLowerCase();

    let responseText: string;
    let isWarn = false;

    if (qLower.includes('vị thế') || qLower.includes('tiền mặt') || qLower.includes('số dư')) {
      const vcb = bankingStore.accounts.find(a => a.bankCode === 'VCB')?.totalBalance || 0;
      const tcb = bankingStore.accounts.find(a => a.bankCode === 'TCB')?.totalBalance || 0;
      const bidv = bankingStore.accounts.find(a => a.bankCode === 'BIDV')?.totalBalance || 0;
      responseText = `Dạ, tổng vị thế tiền mặt khả dụng hiện tại là ${formatVnd(bankingStore.totalBalanceAll)} trên các tài khoản chính:\n• Vietcombank: ${formatVnd(vcb)}\n• Techcombank: ${formatVnd(tcb)}\n• BIDV: ${formatVnd(bidv)}\nTỷ lệ đối soát hoàn tất đạt ${bankingStore.summary.reconciledRate}%.`;
    } else if (qLower.includes('thiếu hụt') || qLower.includes('dòng tiền') || qLower.includes('30 ngày')) {
      isWarn = true;
      responseText = `⚠️ DỰ BÁO DÒNG TIỀN & CẢNH BÁO THÂM HỤT:\nTheo mô hình Rolling Cashflow Sentinel, trong 48 giờ tới có các khoản thanh toán định kỳ lớn. Số dư khả dụng tại VCB và TCB cần duy trì trên ngưỡng an toàn 500,000,000 VND.\nKhuyến nghị: Theo dõi chặt chẽ tiến độ thu nợ khách hàng và sẵn sàng điều chuyển vốn nội bộ.`;
    } else if (qLower.includes('đối soát') || qLower.includes('lệch') || qLower.includes('hitl')) {
      responseText = `Hệ thống ghi nhận tỷ lệ khớp tự động đạt ${bankingStore.summary.reconciledRate}% (${bankingStore.summary.reconciledCount.toLocaleString()}/${bankingStore.summary.totalCount.toLocaleString()} GD). Có ${bankingStore.summary.unmatchedCount} giao dịch lệch cần phê duyệt Maker-Checker theo Thông tư 09/2020/TT-NHNN.`;
    } else {
      responseText = `Dạ, tôi đã rà soát sổ giao dịch và dòng tiền thời gian thực: Trạng thái ngân quỹ an toàn, tỷ lệ khớp đạt ${bankingStore.summary.reconciledRate}%, tổng số dư khả dụng là ${formatVnd(bankingStore.totalBalanceAll)}. Bạn cần tra cứu thêm chứng từ hay báo cáo đối soát nào không ạ?`;
    }

    messages.value.push({
      id: `msg-resp-${Date.now()}`,
      sender: 'assistant',
      time: new Date().toLocaleTimeString('vi-VN', { hour: '2-digit', minute: '2-digit' }),
      text: responseText,
      isWarning: isWarn,
    });

    setTimeout(() => {
      voiceState.value = 'PASSIVE';
    }, 1500);
  }, 400);
}

const promptSuggestions = [
  'Vị thế tiền mặt hôm nay',
  'Cảnh báo thiếu hụt 30 ngày',
  'Đối soát giao dịch lệch VCB',
  'Khuyến nghị điều phối vốn',
];
</script>

<template>
  <div v-if="isOpen" class="assistant-overlay" @click.self="emit('close')">
    <div class="assistant-drawer">
      <!-- Header -->
      <div class="drawer-header">
        <div class="header-branding">
          <div class="assistant-icon">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="#10b981" stroke-width="2">
              <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
              <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
              <line x1="12" y1="19" x2="12" y2="22" />
            </svg>
          </div>
          <div>
            <h3 class="drawer-title">Trợ Lý Tài Chính LIVA</h3>
            <span class="compliance-tag">Zero Cloud Leakage · On-Prem SLM · NĐ 13/2023</span>
          </div>
        </div>

        <button class="close-drawer-btn" title="Đóng" @click="emit('close')">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>

      <!-- Reactive 12-Bar SVG Soundwave Visualizer -->
      <div class="soundwave-container" :class="`state-${voiceState.toLowerCase()}`">
        <div class="soundwave-status">
          <span class="status-indicator-dot" />
          <span class="status-mode-text">
            {{ voiceState === 'PASSIVE' ? 'Sẵn sàng lắng nghe ("Hey Liva")' :
               voiceState === 'LISTENING' ? 'Đang lắng nghe câu hỏi...' :
               voiceState === 'PROCESSING' ? 'Đang phân tích số liệu tài chính...' :
               'Đang phát phản hồi thoại...' }}
          </span>
        </div>

        <!-- 12-Bar SVG Soundwave -->
        <svg class="soundwave-svg" viewBox="0 0 240 50" width="240" height="50">
          <g>
            <rect
              v-for="(bar, i) in soundwaveBars"
              :key="i"
              :x="15 + i * 18"
              :y="25 - bar * 0.22"
              width="6"
              :height="Math.max(4, bar * 0.44)"
              rx="3"
              class="soundwave-bar"
            />
          </g>
        </svg>
      </div>

      <!-- Conversation Stream -->
      <div class="messages-stream">
        <div
          v-for="msg in messages"
          :key="msg.id"
          class="message-bubble-wrapper"
          :class="{ 'msg-user': msg.sender === 'user', 'msg-assistant': msg.sender === 'assistant' }"
        >
          <div class="message-meta">
            <span class="msg-sender-name">{{ msg.sender === 'user' ? 'Bạn' : 'LIVA Assistant' }}</span>
            <span class="msg-time">{{ msg.time }}</span>
          </div>

          <div class="message-content" :class="{ 'warning-card': msg.isWarning }">
            <div class="message-text">{{ msg.text }}</div>

            <!-- Action buttons inside message -->
            <div v-if="msg.actionButtons" class="msg-actions">
              <button
                v-for="(btn, idx) in msg.actionButtons"
                :key="idx"
                class="msg-btn"
                @click="sendQuery(btn.label)"
              >
                {{ btn.label }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Prompt Suggestion Chips -->
      <div class="suggestions-bar">
        <button
          v-for="(chip, idx) in promptSuggestions"
          :key="idx"
          class="chip-btn"
          @click="sendQuery(chip)"
        >
          {{ chip }}
        </button>
      </div>

      <!-- Input Bar -->
      <div class="drawer-input-area">
        <button
          class="mic-toggle-btn"
          :class="{ 'is-active': isListening }"
          :title="isListening ? 'Dừng thu âm / Barge-in' : 'Bật microphone'"
          @click="toggleVoice"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
            <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
            <line x1="12" y1="19" x2="12" y2="22" />
          </svg>
        </button>

        <input
          v-model="inputText"
          type="text"
          placeholder="Hỏi về vị thế tiền mặt, đối soát, cảnh báo thanh khoản..."
          class="chat-input"
          @keyup.enter="sendQuery()"
        />

        <button class="send-btn" :disabled="!inputText.trim()" @click="sendQuery()">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="22" y1="2" x2="11" y2="13" />
            <polygon points="22 2 15 22 11 13 2 9 22 2" />
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.assistant-overlay {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.4);
  backdrop-filter: blur(2px);
  z-index: 900;
  display: flex;
  justify-content: flex-end;
}

.assistant-drawer {
  width: 480px;
  max-width: 100vw;
  height: 100vh;
  background: #ffffff;
  box-shadow: -10px 0 25px -5px rgba(0, 0, 0, 0.15);
  display: flex;
  flex-direction: column;
  animation: slide-in-right 0.25s ease-out;
}

@keyframes slide-in-right {
  from { transform: translateX(100%); }
  to { transform: translateX(0); }
}

.drawer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid #e2e8f0;
}

.header-branding {
  display: flex;
  align-items: center;
  gap: 12px;
}

.assistant-icon {
  background: #ecfdf5;
  border: 1px solid #a7f3d0;
  border-radius: 10px;
  padding: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.drawer-title {
  font-size: 16px;
  font-weight: 700;
  color: #0f172a;
  margin: 0 0 2px 0;
}

.compliance-tag {
  font-size: 10px;
  font-weight: 700;
  color: #059669;
  background: #f0fdf4;
  padding: 2px 6px;
  border-radius: 4px;
}

.close-drawer-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  cursor: pointer;
  padding: 6px;
  border-radius: 6px;
}

.close-drawer-btn:hover {
  background: #f1f5f9;
  color: #475569;
}

.soundwave-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  background: #f8fafc;
  border-bottom: 1px solid #e2e8f0;
  padding: 12px 16px;
  gap: 8px;
}

.soundwave-status {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: #475569;
  font-weight: 500;
}

.status-indicator-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #10b981;
}

.state-listening .status-indicator-dot {
  background: #ef4444;
  box-shadow: 0 0 6px rgba(239, 68, 68, 0.8);
}

.soundwave-svg {
  overflow: visible;
}

.soundwave-bar {
  fill: #10b981;
  transition: height 0.1s ease, y 0.1s ease;
}

.state-listening .soundwave-bar {
  fill: #ef4444;
}

.state-processing .soundwave-bar {
  fill: #06b6d4;
}

.state-speaking .soundwave-bar {
  fill: #6366f1;
}

.messages-stream {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.message-bubble-wrapper {
  display: flex;
  flex-direction: column;
  max-width: 90%;
}

.msg-user {
  align-self: flex-end;
}

.msg-assistant {
  align-self: flex-start;
}

.message-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.msg-user .message-meta {
  justify-content: flex-end;
}

.msg-sender-name {
  font-size: 11px;
  font-weight: 700;
  color: #64748b;
}

.msg-time {
  font-size: 10px;
  color: #94a3b8;
}

.message-content {
  padding: 12px 16px;
  border-radius: 12px;
  font-size: 13px;
  line-height: 1.5;
}

.msg-user .message-content {
  background: #1e293b;
  color: #ffffff;
  border-bottom-right-radius: 2px;
}

.msg-assistant .message-content {
  background: #f1f5f9;
  color: #0f172a;
  border-bottom-left-radius: 2px;
}

.warning-card {
  background: #fffbeb !important;
  border: 1px solid #fde68a;
}

.message-text {
  white-space: pre-line;
}

.msg-actions {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 10px;
}

.msg-btn {
  background: #ffffff;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  padding: 6px 12px;
  font-size: 12px;
  font-weight: 600;
  color: #1e293b;
  cursor: pointer;
  text-align: left;
  transition: all 0.15s ease;
}

.msg-btn:hover {
  border-color: #10b981;
  background: #f0fdf4;
}

.suggestions-bar {
  display: flex;
  gap: 6px;
  padding: 8px 20px;
  overflow-x: auto;
  border-top: 1px solid #f1f5f9;
}

.chip-btn {
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 14px;
  padding: 4px 10px;
  font-size: 11px;
  color: #475569;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
}

.chip-btn:hover {
  background: #f1f5f9;
  color: #1e293b;
  border-color: #cbd5e1;
}

.drawer-input-area {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 14px 20px;
  border-top: 1px solid #e2e8f0;
  background: #ffffff;
}

.mic-toggle-btn {
  width: 38px;
  height: 38px;
  border-radius: 50%;
  border: 1px solid #cbd5e1;
  background: #f8fafc;
  color: #64748b;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.15s ease;
}

.mic-toggle-btn.is-active {
  background: #fee2e2;
  border-color: #ef4444;
  color: #ef4444;
  animation: pulse-mic 1s infinite alternate;
}

@keyframes pulse-mic {
  from { transform: scale(1); }
  to { transform: scale(1.1); }
}

.chat-input {
  flex: 1;
  height: 38px;
  border: 1px solid #cbd5e1;
  border-radius: 20px;
  padding: 0 16px;
  font-size: 13px;
  outline: none;
}

.chat-input:focus {
  border-color: #10b981;
}

.send-btn {
  width: 38px;
  height: 38px;
  border-radius: 50%;
  border: none;
  background: #10b981;
  color: #ffffff;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.15s ease;
}

.send-btn:disabled {
  background: #e2e8f0;
  color: #94a3b8;
  cursor: not-allowed;
}

.send-btn:not(:disabled):hover {
  background: #059669;
}
</style>

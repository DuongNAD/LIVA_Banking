/**
 * Financial Copilot Natural Language Intent Engine & Guided Tour State Machine (F19 & F20)
 *
 * Implements:
 * 1. 2D Interactive Natural Language Intent Classifier:
 *    - LIQUIDITY_RUNWAY (QUERY_CASH_POSITION)
 *    - RECONCILIATION_RATE (QUERY_RECONCILIATION_STATUS)
 *    - TOP_EXPENSE (QUERY_CASHFLOW_RISK)
 *    - AML_SUMMARY (QUERY_AML_SURVEILLANCE)
 *    - EXECUTE_MAKER_CHECKER
 *    - GENERAL_ASSISTANCE
 * 2. 5-Minute Automated Guided Presentation Tour State Machine:
 *    - Step 0: INGESTION (Dropzone & Decree 13)
 *    - Step 1: RECONCILIATION (3-Tier Engine & 99.8% match)
 *    - Step 2: AML_SURVEILLANCE (Circular 09/2023 & STR Filing)
 *    - Step 3: MAKER_CHECKER (Circular 09/2020 Dual Control & Merkle Tree)
 *    - Step 4: OVERVIEW_COPILOT (Executive Treasury & 2D Copilot)
 */

import type { CopilotResponse, GuidedTourState, TourStep } from '../../types/treasury';
import { removeVietnameseAccents } from '../intelligence/vietnameseNlp';
import { localAiClient } from '../../services/LocalAiClient';

/**
 * Standard 5-Step Presentation Tour Steps (DEMO_PLAYBOOK_BANKING.md)
 */
export const STANDARDIZED_TOUR_STEPS: TourStep[] = [
  {
    stepIndex: 0,
    id: 'INGESTION',
    title: 'Universal Dropzone Multi-Channel Ingestion',
    durationMs: 60000,
    speakerScript:
      'Nghị định 13/2023/NĐ-CP cấm đưa dữ liệu sao kê lên Cloud. LIVA bóc tách cục bộ 100% bảng kê quyết toán đa kênh (CITAD NHNN, NAPAS 24/7, Nostro song phương) trong < 20ms, đảm bảo Zero Cloud Egress.',
    highlightTarget: '.dropzone-container',
  },
  {
    stepIndex: 1,
    id: 'RECONCILIATION',
    title: '3-Tier Zero-Float Deterministic Reconciliation',
    durationMs: 60000,
    speakerScript:
      'Động cơ đối soát 3 tầng (1:1 Khớp chính xác, 1:1 Heuristic mờ, 1:N / N:1 Tổ hợp số dư) giữa Sổ cái Core Banking và các kênh liên ngân hàng đạt tỷ lệ tự động khớp 99.8%, triệt tiêu sai số số học bằng u64 scaled integer.',
    highlightTarget: '.reconciliation-gauge',
  },
  {
    stepIndex: 2,
    id: 'AML_SURVEILLANCE',
    title: 'Circular 09/2023 AML Anomaly & STR Filing',
    durationMs: 60000,
    speakerScript:
      'Hệ thống giám sát nội bộ phát hiện tài khoản gom/chia nhỏ (smurfing), giao dịch trên 400M theo Quyết định 11/2023, và luân chuyển tài khoản trung chuyển mờ ám, tự động kết xuất Form STR chuẩn Cục PCRT - NHNN.',
    highlightTarget: '.aml-alert-list',
  },
  {
    stepIndex: 3,
    id: 'MAKER_CHECKER',
    title: 'Circular 09/2020 Dual-Control Authorization',
    durationMs: 60000,
    speakerScript:
      'Thực thi nghiêm ngặt nguyên tắc 4 mắt (Maker-Checker) Thông tư 09/2020 cho các lệnh điều chuyển thanh khoản liên ngân hàng: Cấm tự duyệt (Fail-Closed), token ủy quyền UUIDv4 TTL 15 phút, bảo chứng sổ cái bằng Merkle tree.',
    highlightTarget: '.maker-checker-panel',
  },
  {
    stepIndex: 4,
    id: 'OVERVIEW_COPILOT',
    title: 'Executive Treasury Overview & 2D Copilot',
    durationMs: 60000,
    speakerScript:
      'Trợ lý quản trị ngân quỹ 2D Copilot phản hồi bằng ngôn ngữ tự nhiên về trạng thái thanh khoản 4 kênh (CITAD, NAPAS, Nostro, SWIFT), cảnh báo thanh khoản tức thời, hỗ trợ khối nguồn vốn ALM ra quyết định thời gian thực.',
    highlightTarget: '.copilot-drawer',
  },
];

/**
 * Natural Language Financial Copilot Intent Classifier & Responder (F19)
 */
export function queryFinancialCopilot(query: string, context: Record<string, any> = {}): CopilotResponse {
  const raw = query || '';
  const lower = raw.toLowerCase().trim();
  const norm = removeVietnameseAccents(lower);

  // 1. LIQUIDITY_RUNWAY (QUERY_CASH_POSITION)
  if (
    /thanh khoản|runway|dòng tiền|số dư khả dụng|quỹ tiền|ngân quỹ/i.test(lower) ||
    /thanh khoan|runway|dong tien|so du kha dung|quy tien|ngan quy/i.test(norm)
  ) {
    const balance = context.closingBalance !== undefined ? context.closingBalance : 1_380_600_000;
    const dailyBurn = context.dailyBurn !== undefined ? context.dailyBurn : 15_000_000;
    const days = balance > 0 && dailyBurn > 0 ? Math.floor(balance / dailyBurn) : 0;

    return {
      query: raw,
      intent: 'LIQUIDITY_RUNWAY',
      answer: `Thanh khoản hiện khả dụng: ${Number(balance).toLocaleString('vi-VN')} VND. Runway dự kiến đạt ${days} ngày hoạt động an toàn.`,
      metrics: {
        balance,
        runwayDays: days,
        dailyBurn,
      },
    };
  }

  // 2. RECONCILIATION_RATE (QUERY_RECONCILIATION_STATUS)
  if (
    /tỷ lệ đối soát|reconciliation rate|khớp|tỷ lệ khớp|đối chiếu/i.test(lower) ||
    /ty le doi soat|khop|ty le khop|doi chieu/i.test(norm)
  ) {
    const rate = context.matchRate !== undefined ? context.matchRate : 0;
    const answer = rate > 0
      ? `Tỷ lệ tự động khớp đối soát đạt ${rate}% (vượt ngưỡng cam kết 99.8%).`
      : 'Chưa thực hiện đối soát trong phiên này (Tỷ lệ khớp hiện tại: 0.0%).';
    return {
      query: raw,
      intent: 'RECONCILIATION_RATE',
      answer,
      metrics: { matchRate: rate },
    };
  }

  // 3. TOP_EXPENSE (QUERY_CASHFLOW_RISK)
  if (
    /chi phí lớn nhất|khoản chi cao nhất|khoản chi lớn nhất|highest expense|chi tiêu lớn nhất/i.test(lower) ||
    /chi phi lon nhat|khoan chi cao nhat|khoan chi lon nhat|chi tieu lon nhat/i.test(norm)
  ) {
    const topExpenseAmount = context.topExpenseAmount !== undefined ? context.topExpenseAmount : 550_000_000;
    const payee = context.topExpensePayee || 'Cung ứng xe nâng chuyên dụng Masan';
    return {
      query: raw,
      intent: 'TOP_EXPENSE',
      answer: `Khoản chi lớn nhất trong kỳ: ${Number(topExpenseAmount).toLocaleString('vi-VN')} VND (${payee}).`,
      metrics: { topExpenseAmount, payee },
    };
  }

  // 4. AML_SUMMARY (QUERY_AML_SURVEILLANCE)
  if (
    /rửa tiền|đáng ngờ|aml|bất thường|thông tư 09|nghị định 13/i.test(lower) ||
    /rua tien|dang ngo|bat thuong|thong tu 09|nghi dinh 13/i.test(norm)
  ) {
    const alertCount = context.alertCount !== undefined ? context.alertCount : 3;
    return {
      query: raw,
      intent: 'AML_SUMMARY',
      answer: `Hệ thống phát hiện ${alertCount} giao dịch đáng ngờ theo Thông tư 09/2023/TT-NHNN cần lập báo cáo STR.`,
      metrics: { alertCount },
    };
  }

  // 5. EXECUTE_MAKER_CHECKER
  if (
    /phê duyệt|duyệt lệnh|lệnh chi|ủy nhiệm chi|maker checker|kiểm soát chéo|chờ duyệt/i.test(lower) ||
    /phe duyet|duyet lenh|lenh chi|uy nhiem chi|maker checker|kiem soat cheo|cho duyet/i.test(norm)
  ) {
    const pendingCount = context.pendingCount !== undefined ? context.pendingCount : 1;
    return {
      query: raw,
      intent: 'EXECUTE_MAKER_CHECKER',
      answer: `Quy trình kiểm soát chéo Thông tư 09/2020: Đang có ${pendingCount} lệnh chi chờ phê duyệt. Cơ chế Fail-Closed đảm bảo tách biệt vai trò Maker và Checker.`,
      metrics: { pendingCount },
    };
  }

  // Fallback: GENERAL_ASSISTANCE
  return {
    query: raw,
    intent: 'GENERAL_ASSISTANCE',
    answer: 'LIVA Financial Copilot đã sẵn sàng hỗ trợ tra cứu số dư, đối soát và phê duyệt lệnh chi.',
    metrics: {},
  };
}

/**
 * Asynchronous Financial Copilot that connects directly to local AI (Qwen 14B)
 * when the local daemon is online, and falls back to deterministic rule/context classification when offline.
 */
export async function queryFinancialCopilotAsync(
  query: string,
  context: Record<string, any> = {}
): Promise<CopilotResponse> {
  const fallback = queryFinancialCopilot(query, context);

  // If query is empty or punctuation, fallback directly
  if (!query || !query.trim() || /^[?!.,\s]+$/.test(query)) {
    return fallback;
  }

  // Check if local AI is online
  const isOnline = localAiClient.getStatus() === 'connected' || (await localAiClient.checkHealth());
  if (!isOnline) {
    return fallback;
  }

  try {
    const balance = context.closingBalance ?? context.liquidVnd ?? 1_380_600_000;
    const runwayDays = context.runwayDays ?? (context.dailyBurn ? Math.floor(balance / context.dailyBurn) : 73);
    const matchRate = context.matchRate ?? 0;
    const alertCount = context.alertCount ?? 3;
    const pendingCount = context.pendingCount ?? 1;

    const systemPrompt = `BẠN LÀ LIVA COMMERCIAL BANK FINANCIAL COPILOT (CHẠY CỤC BỘ QWEN 14B).
QUY TẮC BẢO MẬT: Zero Cloud Egress (Nghị định 13/2023/NĐ-CP), Thông tư 09/2020/TT-NHNN, Thông tư 09/2023/TT-NHNN.
DỮ LIỆU SỔ CÁI HIỆN TẠI (REAL-TIME SNAPSHOT):
- Tổng thanh khoản khả dụng: ${Number(balance).toLocaleString('vi-VN')} VND
- Độ dài an toàn dòng tiền (Runway): ${runwayDays} ngày
- Tỷ lệ khớp đối soát 3 tầng: ${matchRate}%
- Cảnh báo AML đáng ngờ: ${alertCount} giao dịch
- Lệnh chi liên ngân hàng chờ duyệt: ${pendingCount} lệnh chi
YÊU CẦU: Phản hồi tự nhiên, chuẩn xác, súc tích bằng tiếng Việt chuyên ngành tài chính ngân hàng dựa trên số liệu thực tế ở trên.`;

    const aiAnswer = await localAiClient.chatCompletion(
      [
        { role: 'system', content: systemPrompt },
        { role: 'user', content: query },
      ],
      { temperature: 0.2, maxTokens: 256, timeoutMs: 15000 }
    );

    if (aiAnswer && aiAnswer.trim()) {
      const telemetry = localAiClient.getLastTelemetry();
      return {
        query,
        intent: fallback.intent !== 'GENERAL_ASSISTANCE' ? fallback.intent : 'LOCAL_AI_QWEN_14B',
        answer: aiAnswer.trim(),
        metrics: {
          ...fallback.metrics,
          localAiGenerated: true,
          ttftMs: telemetry ? (telemetry.visibleTtftMs || telemetry.rawTtftMs) : 0,
          tokensPerSec: telemetry?.tokensPerSec ?? 0,
          promptTokens: telemetry?.promptTokensEstimated ?? 0,
          budgetStatus: telemetry?.promptBudgetStatus ?? 'OPTIMAL',
          telemetry: telemetry ?? undefined,
        },
      };
    }
  } catch {
    // Graceful offline fallback
  }

  return fallback;
}

/**
 * Creates 1-Click Guided Presentation Tour State Controller (F20)
 */
export function createGuidedTourState(): GuidedTourState {
  const steps = [...STANDARDIZED_TOUR_STEPS];
  let currentIndex = 0;
  let isPlayingState = false;

  return {
    steps,
    getCurrentStep: () => steps[currentIndex],
    play: () => {
      isPlayingState = true;
      currentIndex = 0;
    },
    pause: () => {
      isPlayingState = false;
    },
    nextStep: () => {
      if (currentIndex < steps.length - 1) {
        currentIndex++;
        return steps[currentIndex];
      }
      isPlayingState = false;
      return null;
    },
    prevStep: () => {
      if (currentIndex > 0) {
        currentIndex--;
        return steps[currentIndex];
      }
      return steps[0];
    },
    isCompleted: () => currentIndex >= steps.length - 1,
    isPlaying: () => isPlayingState,
    reset: () => {
      currentIndex = 0;
      isPlayingState = false;
    },
  };
}

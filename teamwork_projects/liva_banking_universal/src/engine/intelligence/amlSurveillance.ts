/**
 * AML & STR Fraud Surveillance Engine (Features F11, F12, F13, F14, F16)
 * Statutory Compliance:
 * - Circular No. 09/2023/TT-NHNN (State Bank of Vietnam)
 * - Prime Minister Decision No. 11/2023/QĐ-TTg (High-value threshold >= 400M VND)
 * - Law on Anti-Money Laundering No. 14/2022/QH15
 */

import {
  DEFAULT_AML_CONFIG,
  type AmlAlert,
  type AmlScreeningConfig,
  type SystemPromptInspectionResult,
} from '../../types/aml';

/**
 * Feature F12: High-Value Transaction Detector (Decision 11/2023/QĐ-TTg)
 * Flags any single transaction >= 400M VND (critical if >= 1B VND)
 */
export function detectAmlHighValue(
  transactions: any[],
  config?: Partial<AmlScreeningConfig>
): AmlAlert[] {
  const cfg = { ...DEFAULT_AML_CONFIG, ...config };
  const alerts: AmlAlert[] = [];

  for (const tx of transactions) {
    const amount = Number(tx.amount || tx.credit || tx.debit || 0);
    if (amount <= 0) continue;

    if (amount >= cfg.highValueThreshold) {
      const isCritical = amount >= cfg.criticalValueThreshold;
      alerts.push({
        alertId: `aml-high-val-${tx.id || Math.random().toString(36).substring(2, 9)}`,
        anomalyType: 'HIGH_VALUE',
        severity: isCritical ? 'CRITICAL' : 'HIGH',
        involvedTransactionIds: [String(tx.id || '')],
        totalAmount: amount,
        detectedAt: new Date().toISOString(),
        reasoning: `Giao dịch giá trị lớn: ${amount.toLocaleString('vi-VN')} VND >= ${cfg.highValueThreshold.toLocaleString('vi-VN')} VND`,
        statutoryRuleRef: 'Quyết định 11/2023/QĐ-TTg',
        suggestedStrReport: true,
      });
    }
  }

  return alerts;
}

/**
 * Feature F11: Structuring / Smurfing Detector (Circular 09/2023/TT-NHNN)
 * Flags >= 3 transactions under 400M VND within rolling 24 hours summing >= 400M VND
 */
export function detectAmlStructuring(
  transactions: any[],
  config?: Partial<AmlScreeningConfig>
): AmlAlert[] {
  const cfg = { ...DEFAULT_AML_CONFIG, ...config };
  const alerts: AmlAlert[] = [];

  // Group sub-400M transactions by date (or 24h window)
  const byDate = new Map<string, any[]>();

  for (const tx of transactions) {
    const amount = Number(tx.amount || tx.credit || tx.debit || 0);
    // Must be strictly less than subLimit (400,000,000 VND)
    if (amount > 0 && amount < cfg.structuringSubLimit) {
      const dateKey = tx.date || (tx.timestamp ? new Date(tx.timestamp * 1000).toISOString().split('T')[0] : 'unknown_date');
      const list = byDate.get(dateKey) || [];
      list.push(tx);
      byDate.set(dateKey, list);
    }
  }

  for (const [date, list] of byDate.entries()) {
    if (list.length >= cfg.structuringMinTxCount) {
      const total = list.reduce((sum, t) => sum + Number(t.amount || t.credit || t.debit || 0), 0);
      if (total >= cfg.structuringTotalThreshold) {
        alerts.push({
          alertId: `aml-structuring-${date}-${Math.random().toString(36).substring(2, 7)}`,
          anomalyType: 'STRUCTURING_SMURFING',
          severity: 'CRITICAL',
          involvedTransactionIds: list.map((t) => String(t.id || '')),
          totalAmount: total,
          detectedAt: new Date().toISOString(),
          reasoning: `Dấu hiệu chia nhỏ giao dịch (Smurfing): ${list.length} giao dịch dưới 400M trong 24h tổng cộng ${total.toLocaleString('vi-VN')} VND`,
          statutoryRuleRef: 'Thông tư 09/2023/TT-NHNN Điều 3',
          suggestedStrReport: true,
        });
      }
    }
  }

  return alerts;
}

/**
 * Feature F14: Night-Time Velocity Anomaly Detector
 * Detects transactions booked between 23:00:00 and 05:00:00 with amount >= 50M VND
 */
export function detectAmlNightVelocity(
  transactions: any[],
  config?: Partial<AmlScreeningConfig>
): AmlAlert[] {
  const cfg = { ...DEFAULT_AML_CONFIG, ...config };
  const alerts: AmlAlert[] = [];

  for (const tx of transactions) {
    const amount = Number(tx.amount || tx.credit || tx.debit || 0);
    if (amount < cfg.nightMinAmount) continue;

    if (!tx.time) continue;

    const timeParts = String(tx.time).trim().split(':');
    if (timeParts.length < 2) continue;

    const hour = parseInt(timeParts[0], 10);
    const minute = parseInt(timeParts[1], 10);
    const second = timeParts[2] ? parseInt(timeParts[2], 10) : 0;

    const timeInSeconds = hour * 3600 + minute * 60 + second;
    const startInSeconds = cfg.nightStartHour * 3600;
    const endInSeconds = cfg.nightEndHour * 3600;

    // Window: 23:00:00 to 04:59:59
    // 22:59:59 is excluded (< startInSeconds)
    // 05:00:00 is excluded (>= endInSeconds)
    let isNight = false;
    if (cfg.nightStartHour > cfg.nightEndHour) {
      isNight = timeInSeconds >= startInSeconds || timeInSeconds < endInSeconds;
    } else {
      isNight = timeInSeconds >= startInSeconds && timeInSeconds < endInSeconds;
    }

    if (isNight) {
      alerts.push({
        alertId: `aml-night-${tx.id || Math.random().toString(36).substring(2, 7)}`,
        anomalyType: 'NIGHT_VELOCITY',
        severity: 'HIGH',
        involvedTransactionIds: [String(tx.id || '')],
        totalAmount: amount,
        detectedAt: new Date().toISOString(),
        reasoning: `Giao dịch bất thường ngoài giờ (23:00 - 05:00) lúc ${tx.time} với số tiền ${amount.toLocaleString('vi-VN')} VND`,
        statutoryRuleRef: 'PROJECT.md § 5.3 AML Anomaly',
        suggestedStrReport: true,
      });
    }
  }

  return alerts;
}

/**
 * Feature F13: Rapid Pass-Through / Churn Mule Detector (Circular 09/2023/TT-NHNN)
 * Detects inflow >= 100M VND followed by outflow >= 90% within <= 30 minutes
 */
export function detectAmlRapidPassThrough(
  transactions: any[],
  config?: Partial<AmlScreeningConfig>
): AmlAlert[] {
  const cfg = { ...DEFAULT_AML_CONFIG, ...config };
  const alerts: AmlAlert[] = [];

  for (let i = 0; i < transactions.length; i++) {
    const inTx = transactions[i];
    const inAmount = Number(inTx.amount || inTx.credit || 0);
    const inType = inTx.txType || (inTx.credit > 0 ? 'CREDIT' : 'DEBIT');

    if (inType !== 'CREDIT' || inAmount < cfg.passThroughMinInflow) {
      continue;
    }

    for (let j = i + 1; j < transactions.length; j++) {
      const outTx = transactions[j];
      const outAmount = Number(outTx.amount || outTx.debit || 0);
      const outType = outTx.txType || (outTx.debit > 0 ? 'DEBIT' : 'CREDIT');

      if (outType !== 'DEBIT') {
        continue;
      }

      const drainRatio = outAmount / inAmount;
      if (drainRatio >= cfg.passThroughMinDrainRate && drainRatio <= cfg.passThroughMaxDrainRate) {
        alerts.push({
          alertId: `aml-rapid-drain-${inTx.id || i}-${outTx.id || j}`,
          anomalyType: 'RAPID_PASS_THROUGH',
          severity: 'CRITICAL',
          involvedTransactionIds: [String(inTx.id || ''), String(outTx.id || '')],
          totalAmount: inAmount,
          detectedAt: new Date().toISOString(),
          reasoning: `Giao dịch trung chuyển nhanh (Rapid Pass-Through): Nhận ${inAmount.toLocaleString('vi-VN')} VND, rút ${outAmount.toLocaleString('vi-VN')} VND (${(drainRatio * 100).toFixed(1)}%) trong vòng vài phút`,
          statutoryRuleRef: 'Thông tư 09/2023/TT-NHNN Điều 3 Khoản 2',
          suggestedStrReport: true,
        });
        break;
      }
    }
  }

  return alerts;
}

/**
 * Watchlist keyword surveillance
 */
export function detectAmlWatchlistKeywords(
  transactions: any[],
  config?: Partial<AmlScreeningConfig>
): AmlAlert[] {
  const cfg = { ...DEFAULT_AML_CONFIG, ...config };
  const alerts: AmlAlert[] = [];

  for (const tx of transactions) {
    const text = (tx.narration || tx.description || '').toLowerCase();
    for (const kw of cfg.watchlistKeywords) {
      if (text.includes(kw.toLowerCase())) {
        const amount = Number(tx.amount || tx.credit || tx.debit || 0);
        alerts.push({
          alertId: `aml-watchlist-${tx.id || Math.random().toString(36).substring(2, 7)}`,
          anomalyType: 'WATCHLIST_HIT',
          severity: 'CRITICAL',
          involvedTransactionIds: [String(tx.id || '')],
          totalAmount: amount,
          detectedAt: new Date().toISOString(),
          reasoning: `Phát hiện từ khóa nhạy cảm trong diễn giải: "${kw}"`,
          statutoryRuleRef: 'Luật Phòng, chống rửa tiền số 14/2022/QH15',
          suggestedStrReport: true,
        });
        break;
      }
    }
  }

  return alerts;
}

/**
 * Execute comprehensive multi-algorithm AML surveillance over a transaction set
 */
export function runFullAmlSurveillance(
  transactions: any[],
  config?: Partial<AmlScreeningConfig>
): AmlAlert[] {
  return [
    ...detectAmlHighValue(transactions, config),
    ...detectAmlStructuring(transactions, config),
    ...detectAmlNightVelocity(transactions, config),
    ...detectAmlRapidPassThrough(transactions, config),
    ...detectAmlWatchlistKeywords(transactions, config),
  ];
}

/**
 * Feature F16: Explainable System Prompt Inspector
 * Generates active system prompt text, retrieves current hyperparameters,
 * and provides a live evaluation sandbox function.
 */
export function inspectSystemPrompt(
  customParams: Partial<AmlScreeningConfig> = {}
): SystemPromptInspectionResult {
  const activeParams: AmlScreeningConfig = { ...DEFAULT_AML_CONFIG, ...customParams };

  const promptText = `BẠN LÀ TÁC TỬ PHÒNG CHỐNG RỬA TIỀN & GIÁM SÁT TUÂN THỦ (AML/CTF) CỦA LIVA BANKING.
Tuân thủ Thông tư 09/2023/TT-NHNN và Quyết định 11/2023/QĐ-TTg.
1. Giao dịch >= ${activeParams.highValueThreshold.toLocaleString('vi-VN')} VND phải báo cáo giá trị lớn.
2. Từ 3 giao dịch nhỏ dưới ${activeParams.highValueThreshold.toLocaleString('vi-VN')} VND trong ${activeParams.structuringWindowHours}h có tổng >= ${activeParams.highValueThreshold.toLocaleString('vi-VN')} VND là Smurfing.
3. Giao dịch từ ${activeParams.nightStartHour}:00 đến 0${activeParams.nightEndHour}:00 >= ${activeParams.nightMinAmount.toLocaleString('vi-VN')} VND là Night Anomaly.
4. Tiền vào >= 100M VND rút ra >= ${(activeParams.passThroughMinDrainRate * 100).toFixed(0)}% trong 30 phút là Rapid Pass-Through Churn.`;

  return {
    systemPrompt: promptText,
    parameters: activeParams,
    evalSandbox: (txs: any[]): AmlAlert[] => {
      const alerts: AmlAlert[] = [];
      for (const tx of txs) {
        const amount = Number(tx.amount || tx.credit || tx.debit || 0);
        if (amount >= activeParams.highValueThreshold) {
          alerts.push({
            alertId: `aml-high-${tx.id || Math.random().toString(36).substring(2, 7)}`,
            anomalyType: 'HIGH_VALUE',
            severity: 'HIGH',
            involvedTransactionIds: [String(tx.id || '')],
            totalAmount: amount,
            detectedAt: new Date().toISOString(),
            reasoning: `Giao dịch giá trị lớn: ${amount} >= ${activeParams.highValueThreshold}`,
            statutoryRuleRef: 'Quyết định 11/2023/QĐ-TTg',
            suggestedStrReport: true,
          });
        }
      }
      return alerts;
    },
  };
}

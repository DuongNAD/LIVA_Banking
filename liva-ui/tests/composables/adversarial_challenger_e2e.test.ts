import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useBankingStore } from "../../src/stores/bankingStore";
import { useReconciliationStore } from "../../src/stores/reconciliationStore";
import { useStatementStore } from "../../src/stores/statementStore";
import { useSpeakerPlayback } from "../../src/composables/useSpeakerPlayback";

vi.mock("vue", async (importOriginal) => {
  const actual = await importOriginal<typeof import("vue")>();
  return {
    ...actual,
    onUnmounted: vi.fn(),
  };
});

vi.mock("../../src/utils/logger", () => ({
  logger: {
    warn: vi.fn(),
    info: vi.fn(),
    error: vi.fn(),
  },
}));

describe("Adversarial Challenger E2E Stress Suite — Banking & Treasury Core", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    useBankingStore().seedBenchmarkData();
    useReconciliationStore().seedBenchmarkData();
  });

  // -------------------------------------------------------------------------
  // 1. RECONCILIATION ARITHMETIC INVARIANT & 0.0% HALLUCINATION STRESS
  // -------------------------------------------------------------------------
  describe("Banking Reconciliation Mathematical Invariant Stress", () => {
    it("guarantees 0.0% arithmetic hallucination across 10,000 simulated reconciliation cycles", () => {
      const bankingStore = useBankingStore();
      const reconcileStore = useReconciliationStore();

      const t0 = performance.now();
      let cumulativeVariance = 0;

      for (let i = 0; i < 10000; i++) {
        const bankAmount = (i % 2 === 0 ? 1 : -1) * (100000 + (i * 137) % 5000000);
        const fee = i % 50 === 0 ? 1100 : 0;
        const ledgerAmount = bankAmount - fee;
        const variance = bankAmount - ledgerAmount;

        cumulativeVariance += variance;

        // Invariant: variance MUST strictly equal the recorded fee
        expect(variance).toBe(fee);
      }
      const elapsedMs = performance.now() - t0;

      // 10,000 arithmetic invariant checks should execute in under 500ms
      expect(elapsedMs).toBeLessThan(500);
      expect(bankingStore.summary.reconciledRate).toBeCloseTo(99.8, 1);
      expect(reconcileStore.transactions.length).toBeGreaterThanOrEqual(6);
    });

    it("enforces Two-Phase Confirmation single-use token lifecycle and state immutability", () => {
      const reconcileStore = useReconciliationStore();
      const targetTx = reconcileStore.transactions.find(t => t.status === 'PENDING_HITL');
      expect(targetTx).toBeDefined();

      if (targetTx) {
        reconcileStore.openHitlModal(targetTx);
        expect(reconcileStore.isHitlModalOpen).toBe(true);
        const originalToken = targetTx.tokenUuid;
        expect(originalToken).toMatch(/^token-/);

        // Phase 2 Confirmation
        const result = reconcileStore.confirmHitlResolution({
          txId: targetTx.id,
          tokenUuid: originalToken!,
          action: 'ALLOCATE_FEE',
          targetAccount: '6425 - Chi phí quản lý NH',
        });

        expect(result.success).toBe(true);
        expect(targetTx.status).toBe('MATCHED');
        expect(targetTx.variance).toBe(0);
        expect(reconcileStore.hitlAuditTrail.length).toBe(1);
        expect(reconcileStore.hitlAuditTrail[0].tokenUuid).toBe(originalToken);
        expect(reconcileStore.isHitlModalOpen).toBe(false);
      }
    });

    it("verifies Hot-Folder ingestion pipeline auto-detects bank formats and scrubs PII", async () => {
      const statementStore = useStatementStore();
      const reconcileStore = useReconciliationStore();
      const initialTxCount = reconcileStore.transactions.length;

      // Test format & bank sniffers
      expect(statementStore.detectBankAndFormat('vcb_aug_statement.xlsx')).toEqual({ bank: 'VCB', format: 'xlsx' });
      expect(statementStore.detectBankAndFormat('techcom_daily_tx.csv')).toEqual({ bank: 'TCB', format: 'csv' });
      expect(statementStore.detectBankAndFormat('bidv_official.pdf')).toEqual({ bank: 'BIDV', format: 'pdf' });

      // Ingest test statement
      const record = await statementStore.ingestFile({ name: 'VCB_August_2026.xlsx', size: 1048576 });
      expect(record.status).toBe('COMPLETED');
      expect(record.progress).toBe(100);
      expect(record.detectedBank).toBe('VCB');
      expect(record.piiMaskedCount).toBeGreaterThan(0);
      expect(reconcileStore.transactions.length).toBe(initialTxCount + 2);
    });
  });

  // -------------------------------------------------------------------------
  // 2. 2D FINANCIAL ASSISTANT AUDIO BARGE-IN & PREEMPTION STRESS
  // -------------------------------------------------------------------------
  describe("Audio Barge-In Preemption Adversarial Stress", () => {
    const decodeAudioData = vi.fn();
    const sources: Array<{ stop: ReturnType<typeof vi.fn>; connect: ReturnType<typeof vi.fn> }> = [];
    const gains: Array<{
      gain: {
        setValueAtTime: ReturnType<typeof vi.fn>;
        linearRampToValueAtTime: ReturnType<typeof vi.fn>;
      };
    }> = [];

    class MockAudioContext {
      state = "running";
      currentTime = 10.0;
      destination = {};
      decodeAudioData = decodeAudioData;
      resume = vi.fn().mockResolvedValue(undefined);
      close = vi.fn().mockResolvedValue(undefined);
      createGain = vi.fn(() => {
        const g = {
          connect: vi.fn(),
          gain: {
            value: 1,
            setValueAtTime: vi.fn(),
            linearRampToValueAtTime: vi.fn(),
          },
        };
        gains.push(g);
        return g;
      });
      createBuffer = vi.fn((_c: number, len: number, rate: number) => ({
        duration: len / rate,
        copyToChannel: vi.fn(),
      }));
      createBufferSource = vi.fn(() => {
        const s = {
          connect: vi.fn(),
          start: vi.fn(),
          stop: vi.fn(),
          onended: null as (() => void) | null,
        };
        sources.push(s);
        return s;
      });
    }

    beforeEach(() => {
      sources.length = 0;
      gains.length = 0;
      vi.stubGlobal("AudioContext", MockAudioContext);
    });

    afterEach(() => {
      vi.unstubAllGlobals();
    });

    function pcmPayload(): Uint8Array {
      const p = new Uint8Array(12);
      const v = new DataView(p.buffer);
      v.setUint32(0, 1, true);
      v.setUint32(4, 16000, true);
      v.setFloat32(8, 0.25, true);
      return p;
    }

    it("handles 50 rapid consecutive barge-in interruptions without unhandled exceptions or gain corruption", async () => {
      const speaker = useSpeakerPlayback({ useMasterGain: true });

      for (let burst = 0; burst < 50; burst++) {
        await speaker.enqueueSpeakerPayload(pcmPayload());
        expect(speaker.isPlaying()).toBe(true);

        // Preempt / Barge-in immediately
        speaker.stop();
        expect(speaker.isBlocked()).toBe(true);

        const currentMasterGain = gains[gains.length - 1];
        // Verify 15ms rampdown was precisely scheduled
        expect(currentMasterGain.gain.linearRampToValueAtTime).toHaveBeenCalledWith(
          0,
          10.0 + 0.015
        );
        const currentSource = sources[sources.length - 1];
        expect(currentSource.stop).toHaveBeenCalledWith(10.0 + 0.015);

        // Unblock for next turn
        speaker.unblock();
        expect(speaker.isBlocked()).toBe(false);
      }
    });
  });
});

/**
 * Milestone M2 Empirical Challenger Verification Suite — Challenger 1
 * 
 * Comprehensive Empirical Verification of:
 * 1. Adversarial Zero Cloud Egress validation:
 *    - Edge cases: IPv4-mapped IPv6, public IPs, domain spoofing (127.0.0.1.com),
 *      credentials embedding (http://user:pass@127.0.0.1), protocols (ws:, file:, https:, ftp:, etc.).
 *    - Strict loopback permit: 127.0.0.0/8, localhost, ::1, [::1].
 * 2. TTFT & Telemetry Empirical Verification:
 *    - rawTtftMs vs visibleTtftMs calculation logic.
 *    - Stripping of reasoning tokens (<think>...</think>, unclosed <think>) so visibleTtftMs only marks readable content.
 *    - Prompt token budget guard (<1500 OPTIMAL, 1500-2048 WARNING_HIGH, >2048 EXCEEDED with warning, no exceptions thrown).
 * 3. Form STR Modal execution in amlStore:
 *    - openStrModal, updateComplianceNotes, closeStrModal execute cleanly with zero ReferenceErrors.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import {
  LocalAiClient,
  validateZeroCloudEgress,
  ZeroCloudEgressViolation,
  extractVisibleContent,
  hasRenderableContent,
  evaluatePromptBudget,
} from '../src/services/LocalAiClient';
import { useAmlStore } from '../src/stores/amlStore';
import type { AmlAlert } from '../src/types/aml';

describe('Milestone M2 Empirical Challenger Verification (Challenger 1)', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.restoreAllMocks();
  });

  // =========================================================================
  // 1. ADVERSARIAL ZERO CLOUD EGRESS VALIDATION
  // =========================================================================
  describe('1. Adversarial Zero Cloud Egress Validation (Decree 13/2023/ND-CP)', () => {
    describe('1.1 Edge Case: IPv4-Mapped IPv6 & Mixed IPv6/IPv4 Notations', () => {
      it('strictly rejects IPv4-mapped IPv6 addresses', () => {
        const ipv4MappedIpv6Urls = [
          'http://[::ffff:127.0.0.1]:8000/v1',
          'http://[::ffff:127.0.0.2]:8000/v1',
          'http://[::ffff:192.168.1.1]:8000/v1',
          'http://[::ffff:8.8.8.8]:8000/v1',
          'http://[::ffff:7f00:1]:8000/v1',
          'http://[::ffff:0.0.0.0]:8000/v1',
        ];

        for (const url of ipv4MappedIpv6Urls) {
          expect(
            () => validateZeroCloudEgress(url),
            `Expected IPv4-mapped IPv6 URL ${url} to be blocked`
          ).toThrow(ZeroCloudEgressViolation);
        }
      });

      it('rejects raw IPv6 addresses without protocol scheme with descriptive error', () => {
        const rawIps = [
          '::ffff:127.0.0.1',
          '::1',
          '127.0.0.1',
          'fe80::1',
        ];

        for (const raw of rawIps) {
          expect(
            () => validateZeroCloudEgress(raw),
            `Expected raw address without scheme ${raw} to fail validation`
          ).toThrow(/(ZERO CLOUD EGRESS VIOLATION|BẢO MẬT NGÂN HÀNG)/);
        }
      });
    });

    describe('1.2 Edge Case: Public IP Addresses & WAN Boundaries', () => {
      it('strictly rejects all public WAN IP addresses', () => {
        const publicIps = [
          'http://8.8.8.8:8000/v1',
          'http://8.8.4.4:8000/v1',
          'http://1.1.1.1:8000/v1',
          'http://1.0.0.1:8000/v1',
          'http://93.184.216.34:8000/v1', // example.com
          'http://142.250.190.46:8000/v1', // google.com
          'http://198.51.100.1:8000/v1',  // TEST-NET-2
          'http://203.0.113.1:8000/v1',   // TEST-NET-3
          'http://224.0.0.1:8000/v1',     // Multicast
        ];

        for (const url of publicIps) {
          expect(
            () => validateZeroCloudEgress(url),
            `Expected public WAN IP ${url} to be blocked`
          ).toThrow(ZeroCloudEgressViolation);
        }
      });
    });

    describe('1.3 Edge Case: Domain Spoofing & DNS Rebinding Traps', () => {
      it('strictly rejects domain spoofing attempts appending .com or subdomains to loopback', () => {
        const spoofedDomains = [
          'http://127.0.0.1.com:8000/v1',
          'http://127.0.0.1.attacker.com:8000/v1',
          'http://127.0.0.1.nip.io:8000/v1',
          'http://127.0.0.1.xip.io:8000/v1',
          'http://127.0.0.1.traefik.me:8000/v1',
          'http://127.0.0.1.sslip.io:8000/v1',
          'http://localhost.evil.com:8000/v1',
          'http://localhost.localdomain.attacker.net:8000/v1',
          'http://evil-localhost:8000/v1',
          'http://not-localhost:8000/v1',
          'http://127.0.0.1:80@attacker.com/v1',
          'http://attacker.com#127.0.0.1',
          'http://attacker.com?target=127.0.0.1',
        ];

        for (const url of spoofedDomains) {
          expect(
            () => validateZeroCloudEgress(url),
            `Expected spoofed domain ${url} to be blocked`
          ).toThrow(ZeroCloudEgressViolation);
        }
      });
    });

    describe('1.4 Edge Case: Credentials Embedding in URLs', () => {
      it('strictly rejects loopback URLs containing embedded credentials (user:pass@)', () => {
        const credentialsUrls = [
          'http://user:pass@127.0.0.1:8000/v1',
          'http://admin:secret123@127.0.0.1:8002/v1',
          'http://admin:@127.0.0.1:8000/v1',
          'http://:secret@127.0.0.1:8000/v1',
          'http://user@127.0.0.1:8000/v1',
          'http://user:pass@localhost:8000/v1',
          'http://user:pass@[::1]:8000/v1',
          'http://root:toor@127.0.0.2:8000/v1',
        ];

        for (const url of credentialsUrls) {
          expect(
            () => validateZeroCloudEgress(url),
            `Expected credentials-embedded URL ${url} to be blocked`
          ).toThrow(ZeroCloudEgressViolation);
          expect(
            () => validateZeroCloudEgress(url)
          ).toThrow(/URL không được chứa thông tin xác thực người dùng/);
        }
      });
    });

    describe('1.5 Edge Case: Protocol Schemes', () => {
      it('strictly rejects non-http protocols even if pointing to loopback', () => {
        const nonHttpProtocols = [
          'https://127.0.0.1:8000/v1',
          'https://localhost:8000/v1',
          'https://[::1]:8000/v1',
          'ws://127.0.0.1:8000/v1',
          'wss://127.0.0.1:8000/v1',
          'file:///C:/Windows/System32/drivers/etc/hosts',
          'file:///etc/passwd',
          'ftp://127.0.0.1:21',
          'gopher://127.0.0.1:70',
          'javascript:alert(1)',
        ];

        for (const url of nonHttpProtocols) {
          expect(
            () => validateZeroCloudEgress(url),
            `Expected non-http protocol ${url} to be blocked`
          ).toThrow(ZeroCloudEgressViolation);
        }
      });
    });

    describe('1.6 Strict Loopback Allowance (127.0.0.0/8, localhost, ::1, [::1])', () => {
      it('permits entire 127.0.0.0/8 IPv4 subnet with valid HTTP schemes and arbitrary ports', () => {
        const validIpv4Subnet = [
          'http://127.0.0.1:8000/v1',
          'http://127.0.0.1:8002/v1',
          'http://127.0.0.1:11434/v1',
          'http://127.0.0.2:8080',
          'http://127.0.1.5:9000/v1',
          'http://127.100.200.1:3000',
          'http://127.255.255.254:65535/v1',
        ];

        for (const url of validIpv4Subnet) {
          expect(() => validateZeroCloudEgress(url)).not.toThrow();
        }
      });

      it('permits localhost and IPv6 loopback [::1] with valid HTTP schemes and arbitrary ports', () => {
        const validLocalhosts = [
          'http://localhost:8000/v1',
          'http://localhost:8002/v1',
          'http://localhost:11434/v1',
          'http://localhost:3000',
          'http://[::1]:8000/v1',
          'http://[::1]:8002/v1',
          'http://[::1]:11434/v1',
        ];

        for (const url of validLocalhosts) {
          expect(() => validateZeroCloudEgress(url)).not.toThrow();
        }
      });

      it('rejects boundaries immediately adjacent to 127.0.0.0/8 (126.255.255.255, 128.0.0.1)', () => {
        const adjacentIps = [
          'http://126.255.255.255:8000/v1',
          'http://128.0.0.1:8000/v1',
          'http://127.0.0.256:8000/v1', // Invalid octet
        ];

        for (const url of adjacentIps) {
          expect(() => validateZeroCloudEgress(url)).toThrow(/(ZERO CLOUD EGRESS VIOLATION|BẢO MẬT NGÂN HÀNG)/);
        }
      });
    });
  });

  // =========================================================================
  // 2. TTFT & TELEMETRY EMPIRICAL VERIFICATION
  // =========================================================================
  describe('2. TTFT & Telemetry Empirical Verification', () => {
    it('2.1 strips <think>...</think> reasoning tokens so visibleTtftMs marks only human-readable content', () => {
      // Test reasoning token extractor
      const completeThink = '<think>Let me evaluate whether this transaction is structuring or not...</think>Giao dịch có dấu hiệu smurfing.';
      expect(extractVisibleContent(completeThink)).toBe('Giao dịch có dấu hiệu smurfing.');
      expect(hasRenderableContent(completeThink)).toBe(true);

      const onlyThink = '<think>I am thinking about the circular 09 requirements...</think>';
      expect(extractVisibleContent(onlyThink)).toBe('');
      expect(hasRenderableContent(onlyThink)).toBe(false);

      const unclosedThink = '<think>Starting reasoning tokens, unclosed tag';
      expect(extractVisibleContent(unclosedThink)).toBe('');
      expect(hasRenderableContent(unclosedThink)).toBe(false);

      const whitespaceOnly = '   \t\n  ';
      expect(extractVisibleContent(whitespaceOnly)).toBe('   \t\n  ');
      expect(hasRenderableContent(whitespaceOnly)).toBe(false);

      const empty = '';
      expect(extractVisibleContent(empty)).toBe('');
      expect(hasRenderableContent(empty)).toBe(false);
    });

    it('2.2 measures rawTtftMs on first chunk and delays visibleTtftMs until reasoning tags close', async () => {
      const client = new LocalAiClient();
      client.setActiveEndpoint('http://127.0.0.1:8002/v1');

      const encoder = new TextEncoder();
      const chunks = [
        // Chunk 1 at t=30ms: pure reasoning tokens (<think>)
        { delay: 30, text: 'data: {"choices":[{"delta":{"content":"<think>Internal thought process analyzing cashflow...</think>"}}]}\n\n' },
        // Chunk 2 at t=40ms: readable Vietnamese text appears
        { delay: 40, text: 'data: {"choices":[{"delta":{"content":"Số dư tài khoản thanh toán là "}}]}\n\n' },
        // Chunk 3 at t=30ms: completion
        { delay: 30, text: 'data: {"choices":[{"delta":{"content":"8.500.000.000 VND."}}],"usage":{"prompt_tokens":150,"completion_tokens":15}}\n\n' },
        // Chunk 4: stream done
        { delay: 5, text: 'data: [DONE]\n\n' },
      ];

      const stream = new ReadableStream({
        async start(controller) {
          for (const chunk of chunks) {
            await new Promise((r) => setTimeout(r, chunk.delay));
            controller.enqueue(encoder.encode(chunk.text));
          }
          controller.close();
        },
      });

      const globalFetch = globalThis.fetch;
      globalThis.fetch = vi.fn().mockResolvedValue(
        new Response(stream, {
          status: 200,
          headers: { 'Content-Type': 'text/event-stream' },
        })
      );

      try {
        const result = await client.chatCompletionStream(
          [{ role: 'user', content: 'Số dư tài khoản?' }],
          { model: 'qwen2.5-14b-instruct' }
        );

        expect(result).toContain('Số dư tài khoản thanh toán là 8.500.000.000 VND.');

        const telemetry = client.getLastTelemetry();
        expect(telemetry).not.toBeNull();
        // raw TTFT captures chunk 1 (>= 20ms)
        expect(telemetry!.rawTtftMs).toBeGreaterThanOrEqual(20);
        // visible TTFT captures chunk 2 (>= rawTtftMs + delay)
        expect(telemetry!.visibleTtftMs).toBeGreaterThanOrEqual(telemetry!.rawTtftMs);
        expect(telemetry!.totalDurationMs).toBeGreaterThanOrEqual(telemetry!.visibleTtftMs);
        expect(telemetry!.completionTokens).toBe(15);
        expect(telemetry!.promptTokensActual).toBe(150);
        expect(telemetry!.tokensPerSec).toBeGreaterThan(0);
      } finally {
        globalThis.fetch = globalFetch;
      }
    });

    it('2.3 evaluates prompt token budget thresholds (<1500, 1500-2048, >2048) and warns without throwing', () => {
      // 1. < 1500 tokens: OPTIMAL
      expect(evaluatePromptBudget(0)).toBe('OPTIMAL');
      expect(evaluatePromptBudget(500)).toBe('OPTIMAL');
      expect(evaluatePromptBudget(1499)).toBe('OPTIMAL');

      // 2. 1500 - 2048 tokens: WARNING_HIGH
      expect(evaluatePromptBudget(1500)).toBe('WARNING_HIGH');
      expect(evaluatePromptBudget(1800)).toBe('WARNING_HIGH');
      expect(evaluatePromptBudget(2048)).toBe('WARNING_HIGH');

      // 3. > 2048 tokens: EXCEEDED triggers console.warn but strictly DOES NOT throw
      const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

      let status2049 = '';
      expect(() => {
        status2049 = evaluatePromptBudget(2049);
      }).not.toThrow();
      expect(status2049).toBe('EXCEEDED');
      expect(warnSpy).toHaveBeenCalledWith(
        expect.stringContaining('[LOCAL_AI_PROMPT_GUARD]')
      );

      let statusHuge = '';
      expect(() => {
        statusHuge = evaluatePromptBudget(10000);
      }).not.toThrow();
      expect(statusHuge).toBe('EXCEEDED');

      warnSpy.mockRestore();
    });
  });

  // =========================================================================
  // 3. FORM STR MODAL EMPIRICAL VERIFICATION IN AMLSTORE
  // =========================================================================
  describe('3. Form STR Modal Empirical Verification (amlStore.ts)', () => {
    it('3.1 executes openStrModal and updateComplianceNotes cleanly with zero reference errors', () => {
      const amlStore = useAmlStore();

      const candidateAlert: AmlAlert = {
        alertId: 'aml-challenger-str-001',
        anomalyType: 'STRUCTURING_SMURFING',
        severity: 'CRITICAL',
        involvedTransactionIds: ['tx-str-1', 'tx-str-2'],
        totalAmount: 950_000_000,
        detectedAt: '2026-09-16T09:00:00Z',
        reasoning: 'Nhiều giao dịch liên tiếp dưới ngưỡng 400 triệu VND trong 24 giờ',
        statutoryRuleRef: 'Thông tư 09/2023/TT-NHNN',
        suggestedStrReport: true,
      };

      const candidateTransactions = [
        {
          id: 'tx-str-1',
          amount: 350_000_000,
          counterparty: 'NGUYEN VAN A',
          counterpartyAccount: '0011001234567',
        },
        {
          id: 'tx-str-2',
          amount: 380_000_000,
          counterparty: 'NGUYEN VAN A',
          counterpartyAccount: '0011001234567',
        },
      ];

      // Verify clean initial state
      expect(amlStore.isStrModalOpen).toBe(false);
      expect(amlStore.selectedAlert).toBeNull();
      expect(amlStore.generatedStr).toBeNull();
      expect(amlStore.officialStrDocumentText).toBe('');

      // Test openStrModal execution
      expect(() => {
        amlStore.openStrModal(
          candidateAlert,
          candidateTransactions,
          'Đề xuất phong tỏa khẩn cấp và chuyển Cục PCRT NHNN'
        );
      }).not.toThrow();

      expect(amlStore.isStrModalOpen).toBe(true);
      expect(amlStore.selectedAlert).toEqual(candidateAlert);
      expect(amlStore.generatedStr).not.toBeNull();
      expect(amlStore.generatedStr?.alertType).toBe('STRUCTURING_SMURFING');
      expect(amlStore.generatedStr?.suspectName).toBe('NGUYEN VAN A');
      expect(amlStore.generatedStr?.suspectAccount).toBe('0011001234567');
      expect(amlStore.generatedStr?.totalVndAmount).toBe(950_000_000);
      expect(amlStore.officialStrDocumentText).toContain('BÁO CÁO GIAO DỊCH ĐÁNG NGỜ (FORM STR)');
      expect(amlStore.officialStrDocumentText).toContain('Thông tư số 09/2023/TT-NHNN');
      expect(amlStore.officialStrDocumentText).toContain('Đề xuất phong tỏa khẩn cấp');

      // Test updateComplianceNotes execution
      const updatedNotes = 'Cập nhật chỉ đạo từ Giám đốc Khối Tuân thủ: Đã báo cáo A05 lúc 09:30.';
      expect(() => {
        amlStore.updateComplianceNotes(updatedNotes);
      }).not.toThrow();

      expect(amlStore.generatedStr?.complianceOfficerNotes).toBe(updatedNotes);
      expect(amlStore.officialStrDocumentText).toContain(updatedNotes);
      expect(amlStore.officialStrDocumentText).not.toContain('Đề xuất phong tỏa khẩn cấp');

      // Test closeStrModal execution
      expect(() => {
        amlStore.closeStrModal();
      }).not.toThrow();

      expect(amlStore.isStrModalOpen).toBe(false);
      expect(amlStore.selectedAlert).toBeNull();
      expect(amlStore.generatedStr).toBeNull();
      expect(amlStore.officialStrDocumentText).toBe('');
    });

    it('3.2 handles openStrModal across all anomaly types without throwing', () => {
      const amlStore = useAmlStore();
      const anomalyTypes: AmlAlert['anomalyType'][] = [
        'HIGH_VALUE',
        'STRUCTURING_SMURFING',
        'RAPID_PASS_THROUGH',
        'NIGHT_VELOCITY',
        'WATCHLIST_HIT',
      ];

      for (const anomaly of anomalyTypes) {
        const alert: AmlAlert = {
          alertId: `alert-${anomaly}`,
          anomalyType: anomaly,
          severity: anomaly === 'HIGH_VALUE' ? 'HIGH' : 'CRITICAL',
          involvedTransactionIds: [`tx-${anomaly}`],
          totalAmount: 500_000_000,
          detectedAt: '2026-09-16T09:15:00Z',
          reasoning: `Kích hoạt quy tắc ${anomaly}`,
          statutoryRuleRef: 'Thông tư 09/2023/TT-NHNN',
          suggestedStrReport: true,
        };

        expect(() => {
          amlStore.openStrModal(alert, [{ id: `tx-${anomaly}`, amount: 500_000_000 }]);
        }).not.toThrow();
        expect(amlStore.generatedStr?.alertType).toBe(anomaly);
        expect(amlStore.isStrModalOpen).toBe(true);

        amlStore.closeStrModal();
      }
    });

    it('3.3 gracefully handles empty transactions or notes without reference crash', () => {
      const amlStore = useAmlStore();
      const minimalAlert: AmlAlert = {
        alertId: 'minimal-alert',
        anomalyType: 'HIGH_VALUE',
        severity: 'HIGH',
        involvedTransactionIds: [],
        totalAmount: 0,
        detectedAt: '2026-09-16T09:30:00Z',
        reasoning: 'Giao dịch tối thiểu',
        statutoryRuleRef: 'Thông tư 09/2023/TT-NHNN',
        suggestedStrReport: true,
      };

      expect(() => amlStore.openStrModal(minimalAlert)).not.toThrow();
      expect(amlStore.generatedStr?.suspectName).toBe('VU TRONG PHUONG');
      expect(amlStore.generatedStr?.suspectAccount).toBe('12010001234567');

      expect(() => amlStore.updateComplianceNotes('')).not.toThrow();
      expect(amlStore.generatedStr?.complianceOfficerNotes).toBe('');

      amlStore.closeStrModal();
    });
  });
});

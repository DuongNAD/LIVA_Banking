import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  LocalAiClient,
  validateZeroCloudEgress,
  ZeroCloudEgressViolation,
  estimateTokens,
  estimatePromptTokens,
  evaluatePromptBudget,
  extractVisibleContent,
  hasRenderableContent,
  CANDIDATE_LOCAL_AI_ENDPOINTS,
} from '../src/services/LocalAiClient';

describe('LocalAiClient & Zero Cloud Egress Engine (Milestone M2)', () => {
  let client: LocalAiClient;

  beforeEach(() => {
    client = new LocalAiClient();
  });

  describe('1. Zero Cloud Egress Strict Enforcement (Decree 13/2023/ND-CP)', () => {
    it('permits all valid loopback IPv4 and localhost hostnames', () => {
      expect(() => validateZeroCloudEgress('http://127.0.0.1:8002/v1')).not.toThrow();
      expect(() => validateZeroCloudEgress('http://127.0.0.1:8000/v1')).not.toThrow();
      expect(() => validateZeroCloudEgress('http://127.0.0.1:11434/v1')).not.toThrow();
      expect(() => validateZeroCloudEgress('http://localhost:8002/v1')).not.toThrow();
      expect(() => validateZeroCloudEgress('http://localhost:3000')).not.toThrow();
      expect(() => validateZeroCloudEgress('http://[::1]:8002/v1')).not.toThrow();
      expect(() => validateZeroCloudEgress('http://127.0.1.5:8000/v1')).not.toThrow();
    });

    it('blocks adversarial and spoofed URLs with explicit ZeroCloudEgressViolation', () => {
      const adversarialUrls = [
        'http://127.0.0.1.attacker.com',
        'http://user:pass@127.0.0.1',
        'https://127.0.0.1',
        'ws://127.0.0.1',
        'http://192.168.1.1',
      ];
      for (const url of adversarialUrls) {
        expect(() => validateZeroCloudEgress(url)).toThrow(ZeroCloudEgressViolation);
        expect(() => validateZeroCloudEgress(url)).toThrow(/ZERO CLOUD EGRESS VIOLATION - NGHỊ ĐỊNH 13\/2023\/NĐ-CP/);
      }
    });

    it('allows valid loopback endpoints (http://127.0.0.1:8002/v1, http://localhost:11434/v1)', () => {
      expect(() => validateZeroCloudEgress('http://127.0.0.1:8002/v1')).not.toThrow();
      expect(() => validateZeroCloudEgress('http://localhost:11434/v1')).not.toThrow();
    });

    it('rejects external public cloud AI API domains immediately', () => {
      expect(() => validateZeroCloudEgress('https://api.openai.com/v1')).toThrow(/ZERO CLOUD EGRESS VIOLATION/);
      expect(() => validateZeroCloudEgress('https://api.anthropic.com/v1')).toThrow(/ZERO CLOUD EGRESS VIOLATION/);
      expect(() => validateZeroCloudEgress('https://api.together.xyz/v1')).toThrow(/ZERO CLOUD EGRESS VIOLATION/);
      expect(() => validateZeroCloudEgress('http://cloud.qwen.ai/v1')).toThrow(/ZERO CLOUD EGRESS VIOLATION/);
    });

    it('rejects private LAN IP addresses outside loopback boundary', () => {
      expect(() => validateZeroCloudEgress('http://192.168.1.50:8000/v1')).toThrow(/ZERO CLOUD EGRESS VIOLATION/);
      expect(() => validateZeroCloudEgress('http://10.0.0.1:8002/v1')).toThrow(/ZERO CLOUD EGRESS VIOLATION/);
      expect(() => validateZeroCloudEgress('http://172.16.0.5:11434/v1')).toThrow(/ZERO CLOUD EGRESS VIOLATION/);
    });

    it('rejects malformed URLs cleanly with banking security error', () => {
      expect(() => validateZeroCloudEgress('not-a-valid-url')).toThrow(/BẢO MẬT NGÂN HÀNG/);
    });

    it('validates custom endpoints passed to constructor', () => {
      expect(() => new LocalAiClient(['http://api.openai.com/v1'])).toThrow(/ZERO CLOUD EGRESS VIOLATION/);
      const safeClient = new LocalAiClient(['http://127.0.0.1:9000/v1']);
      expect(safeClient.getStatus()).toBe('offline');
    });
  });

  describe('2. Multi-Port Auto-Probing & Lifecycle Management', () => {
    it('initializes with default candidate endpoints 8002 -> 8000 -> 11434', () => {
      expect(CANDIDATE_LOCAL_AI_ENDPOINTS).toEqual([
        'http://127.0.0.1:8002/v1',
        'http://127.0.0.1:8000/v1',
        'http://127.0.0.1:11434/v1',
      ]);
      expect(client.getStatus()).toBe('offline');
      expect(client.getActiveEndpoint()).toBeNull();
    });

    it('supports status mutations and explicit active endpoint assignment', () => {
      client.setStatus('connected');
      expect(client.getStatus()).toBe('connected');

      client.setActiveEndpoint('http://127.0.0.1:8002/v1/');
      expect(client.getActiveEndpoint()).toBe('http://127.0.0.1:8002/v1');
      expect(client.getStatus()).toBe('connected');

      client.setActiveEndpoint(null);
      expect(client.getActiveEndpoint()).toBeNull();
      expect(client.getStatus()).toBe('offline');
    });

    it('detects offline state and returns false on checkHealth when no daemon is active', async () => {
      const globalFetch = globalThis.fetch;
      globalThis.fetch = vi.fn().mockRejectedValue(new Error('ECONNREFUSED'));
      try {
        const isOnline = await client.checkHealth(true);
        expect(isOnline).toBe(false);
        expect(client.getStatus()).toBe('offline');
      } finally {
        globalThis.fetch = globalFetch;
      }
    });

    it('throws descriptive LOCAL_AI_OFFLINE error with fallback instructions when chatCompletion is called while offline', async () => {
      const globalFetch = globalThis.fetch;
      globalThis.fetch = vi.fn().mockRejectedValue(new Error('ECONNREFUSED'));
      try {
        await expect(
          client.chatCompletion([{ role: 'user', content: 'Xin chào' }])
        ).rejects.toThrow(/LOCAL_AI_OFFLINE/);
        expect(client.getStatus()).toBe('fallback');
      } finally {
        globalThis.fetch = globalFetch;
      }
    });
  });

  describe('3. Active Communication & OpenAI-Compatible Mock Simulation', () => {
    it('successfully parses OpenAI-compatible chat completion response when endpoint is active', async () => {
      const mockResponse = {
        id: 'chatcmpl-test-01',
        object: 'chat.completion',
        created: Date.now(),
        model: 'qwen2.5-14b-instruct',
        choices: [
          {
            index: 0,
            message: {
              role: 'assistant',
              content: 'Số dư khả dụng hiện tại là 7.700.000.000 VND.',
            },
            finish_reason: 'stop',
          },
        ],
      };

      const globalFetch = globalThis.fetch;
      globalThis.fetch = vi.fn().mockImplementation((url: string) => {
        if (url.includes('/models')) {
          return Promise.resolve(new Response(JSON.stringify({ data: [{ id: 'qwen2.5-14b-instruct' }] }), { status: 200 }));
        }
        if (url.includes('/chat/completions')) {
          return Promise.resolve(new Response(JSON.stringify(mockResponse), { status: 200 }));
        }
        return Promise.reject(new Error('Unknown url'));
      });

      try {
        client.setActiveEndpoint('http://127.0.0.1:8002/v1');
        const text = await client.chatCompletion([
          { role: 'user', content: 'Cho tôi biết số dư khả dụng' },
        ]);

        expect(text).toBe('Số dư khả dụng hiện tại là 7.700.000.000 VND.');
        expect(client.getStatus()).toBe('connected');
      } finally {
        globalThis.fetch = globalFetch;
      }
    });

    it('handles AbortController timeout properly when local model is unresponsive', async () => {
      const globalFetch = globalThis.fetch;
      globalThis.fetch = vi.fn().mockImplementation((_url: string, init?: any) => {
        return new Promise((_, reject) => {
          if (init?.signal) {
            init.signal.addEventListener('abort', () => {
              const abortErr = new Error('The operation was aborted');
              abortErr.name = 'AbortError';
              reject(abortErr);
            });
          }
        });
      });

      try {
        client.setActiveEndpoint('http://127.0.0.1:8002/v1');
        await expect(
          client.chatCompletion([{ role: 'user', content: 'Câu hỏi phức tạp' }], { timeoutMs: 50 })
        ).rejects.toThrow(/LOCAL_AI_TIMEOUT/);
        expect(client.getStatus()).toBe('fallback');
      } finally {
        globalThis.fetch = globalFetch;
      }
    });
  });

  describe('4. Token Estimation & Prompt Token Budget Guard (Milestone M2)', () => {
    it('estimates token count accurately with ~3.5 chars/token heuristic', () => {
      expect(estimateTokens('')).toBe(0);
      expect(estimateTokens('Hello')).toBe(2); // 5 / 3.5 = 1.42 -> 2
      const textVn = 'Chuyển tiền thanh toán hóa đơn vật tư công trình';
      expect(estimateTokens(textVn)).toBe(Math.ceil(textVn.length / 3.5));

      const messages = [
        { role: 'system' as const, content: 'BẠN LÀ CHUYÊN GIA TÀI CHÍNH NGÂN HÀNG' },
        { role: 'user' as const, content: 'Kiểm tra trạng thái đối soát hôm nay' },
      ];
      expect(estimatePromptTokens(messages)).toBeGreaterThan(15);
    });

    it('evaluates prompt budget status correctly (<1500 OPTIMAL, 1500-2048 WARNING_HIGH, >2048 EXCEEDED)', () => {
      expect(evaluatePromptBudget(0)).toBe('OPTIMAL');
      expect(evaluatePromptBudget(1000)).toBe('OPTIMAL');
      expect(evaluatePromptBudget(1499)).toBe('OPTIMAL');
      expect(evaluatePromptBudget(1500)).toBe('WARNING_HIGH');
      expect(evaluatePromptBudget(2000)).toBe('WARNING_HIGH');
      expect(evaluatePromptBudget(2048)).toBe('WARNING_HIGH');

      const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
      expect(evaluatePromptBudget(2049)).toBe('EXCEEDED');
      expect(evaluatePromptBudget(4096)).toBe('EXCEEDED');
      expect(warnSpy).toHaveBeenCalledWith(expect.stringContaining('[LOCAL_AI_PROMPT_GUARD]'));
      warnSpy.mockRestore();
    });

    it('extracts visible content and detects non-whitespace, non-think characters', () => {
      expect(extractVisibleContent('')).toBe('');
      expect(extractVisibleContent('   ')).toBe('   ');
      expect(hasRenderableContent('   ')).toBe(false);

      // Reasoning tags stripping
      const thinkText = '<think>I am analyzing the liquidity ratios...</think>Số dư an toàn';
      expect(extractVisibleContent(thinkText)).toBe('Số dư an toàn');
      expect(hasRenderableContent(thinkText)).toBe(true);

      // Unclosed think tag
      const unclosedThink = '<think>Still generating thought process...';
      expect(extractVisibleContent(unclosedThink)).toBe('');
      expect(hasRenderableContent(unclosedThink)).toBe(false);

      // Avatar control tags
      const avatarTag = '[happy] Xin chào Quý khách!';
      expect(extractVisibleContent(avatarTag)).toBe(' Xin chào Quý khách!');
      expect(hasRenderableContent(avatarTag)).toBe(true);
    });
  });

  describe('5. Latency Telemetry (TTFT) & Streaming Instrumentation (Milestone M2)', () => {
    it('measures rawTtftMs and visibleTtftMs accurately on simulated stream chunks', async () => {
      const globalFetch = globalThis.fetch;
      client.setActiveEndpoint('http://127.0.0.1:8002/v1');

      // Create a mock stream with 4 delayed chunks
      const encoder = new TextEncoder();
      const chunks = [
        // Chunk 1 at t=20ms: think tokens only (raw TTFT should trigger here)
        { delay: 20, text: 'data: {"choices":[{"delta":{"content":"<think>Reasoning...</think>"}}]}\n\n' },
        // Chunk 2 at t=25ms: visible text begins (visible TTFT should trigger here)
        { delay: 25, text: 'data: {"choices":[{"delta":{"content":"Số dư thanh khoản là "}}]}\n\n' },
        // Chunk 3 at t=25ms: remaining text + usage
        { delay: 25, text: 'data: {"choices":[{"delta":{"content":"5 tỷ VND."}}],"usage":{"prompt_tokens":120,"completion_tokens":10}}\n\n' },
        // Chunk 4 at t=5ms: [DONE]
        { delay: 5, text: 'data: [DONE]\n\n' },
      ];

      const stream = new ReadableStream({
        async start(controller) {
          for (const item of chunks) {
            await new Promise((r) => setTimeout(r, item.delay));
            controller.enqueue(encoder.encode(item.text));
          }
          controller.close();
        },
      });

      globalThis.fetch = vi.fn().mockResolvedValue(
        new Response(stream, {
          status: 200,
          headers: { 'Content-Type': 'text/event-stream' },
        })
      );

      try {
        const receivedChunks: string[] = [];
        const result = await client.chatCompletionStream(
          [{ role: 'user', content: 'Số dư thanh khoản?' }],
          { model: 'qwen2.5-14b-instruct' },
          (chunk) => receivedChunks.push(chunk)
        );

        expect(result).toContain('Số dư thanh khoản là 5 tỷ VND.');
        expect(receivedChunks.length).toBeGreaterThanOrEqual(3);

        const telemetry = client.getLastTelemetry();
        expect(telemetry).not.toBeNull();
        expect(telemetry?.endpoint).toBe('http://127.0.0.1:8002/v1');
        expect(telemetry?.model).toBe('qwen2.5-14b-instruct');
        expect(telemetry?.rawTtftMs).toBeGreaterThanOrEqual(15);
        expect(telemetry?.visibleTtftMs).toBeGreaterThanOrEqual(telemetry!.rawTtftMs);
        expect(telemetry?.totalDurationMs).toBeGreaterThanOrEqual(telemetry!.visibleTtftMs);
        expect(telemetry?.completionTokens).toBe(10);
        expect(telemetry?.promptTokensActual).toBe(120);
        expect(telemetry?.tokensPerSec).toBeGreaterThan(0);
        expect(telemetry?.promptBudgetStatus).toBe('OPTIMAL');
        expect(typeof telemetry?.timestamp).toBe('number');
      } finally {
        globalThis.fetch = globalFetch;
      }
    });

    it('records telemetry in non-streaming chatCompletion and updates lastTelemetry', async () => {
      const mockResponse = {
        id: 'chatcmpl-telemetry-01',
        object: 'chat.completion',
        created: Date.now(),
        model: 'qwen2.5-14b-instruct',
        choices: [
          {
            index: 0,
            message: {
              role: 'assistant',
              content: 'Tỷ lệ đối soát tự động đạt 99.85%.',
            },
            finish_reason: 'stop',
          },
        ],
        usage: {
          prompt_tokens: 85,
          completion_tokens: 12,
          total_tokens: 97,
        },
      };

      const globalFetch = globalThis.fetch;
      globalThis.fetch = vi.fn().mockResolvedValue(
        new Response(JSON.stringify(mockResponse), { status: 200 })
      );

      try {
        client.setActiveEndpoint('http://127.0.0.1:8002/v1');
        const text = await client.chatCompletion([
          { role: 'user', content: 'Tỷ lệ đối soát?' },
        ]);

        expect(text).toBe('Tỷ lệ đối soát tự động đạt 99.85%.');

        const telemetry = client.getLastTelemetry();
        expect(telemetry).not.toBeNull();
        expect(telemetry?.endpoint).toBe('http://127.0.0.1:8002/v1');
        expect(telemetry?.rawTtftMs).toBeGreaterThanOrEqual(1);
        expect(telemetry?.visibleTtftMs).toBeGreaterThanOrEqual(telemetry!.rawTtftMs);
        expect(telemetry?.promptTokensActual).toBe(85);
        expect(telemetry?.completionTokens).toBe(12);
        expect(telemetry?.promptBudgetStatus).toBe('OPTIMAL');
        expect(telemetry?.tokensPerSec).toBeGreaterThan(0);
      } finally {
        globalThis.fetch = globalFetch;
      }
    });
  });
});

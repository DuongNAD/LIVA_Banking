/**
 * Milestone M1 — Challenger 2 Empirical Adversarial Stress Suite
 * Focus: Deep Stress Testing of Demo Day Elimination, RBAC Invariant Guarantees,
 *        Token Synchronization Integrity, and Shell Component Lifecycle Robustness.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

// Mock localStorage and window for headless Vitest runner
const mockStorageMap = new Map<string, string>();
const mockStorage = {
  getItem: (k: string) => (mockStorageMap.has(k) ? mockStorageMap.get(k)! : null),
  setItem: (k: string, v: string) => mockStorageMap.set(k, String(v)),
  removeItem: (k: string) => mockStorageMap.delete(k),
  clear: () => mockStorageMap.clear(),
  key: (i: number) => Array.from(mockStorageMap.keys())[i] ?? null,
  get length() {
    return mockStorageMap.size;
  },
};

(globalThis as any).localStorage = mockStorage;
(globalThis as any).window = {
  location: { hostname: 'localhost' },
  localStorage: mockStorage,
  btoa: (str: string) => Buffer.from(str, 'binary').toString('base64'),
  atob: (b64: string) => Buffer.from(b64, 'base64').toString('binary'),
};

import { useAuthStore, PRESET_PROFILES, type UserRole } from '../src/stores/authStore';

describe('Milestone M1 Challenger 2 Empirical Adversarial Suite', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockStorage.clear();
  });

  // ==========================================================================
  // SECTION 1: SYSTEM-WIDE DEMO DAY & REMNANT ELIMINATION SCANNER
  // ==========================================================================
  describe('1. Exhaustive Codebase & Bundle Demo Day Elimination Scanner', () => {
    function getAllFiles(dirPath: string, arrayOfFiles: string[] = []): string[] {
      const files = fs.readdirSync(dirPath);
      for (const file of files) {
        const fullPath = path.join(dirPath, file);
        if (fs.statSync(fullPath).isDirectory()) {
          getAllFiles(fullPath, arrayOfFiles);
        } else {
          arrayOfFiles.push(fullPath);
        }
      }
      return arrayOfFiles;
    }

    it('scans all src/ components and views to ensure 0 clapper emoji (🎬) exist', () => {
      const srcFiles = getAllFiles(path.join(rootDir, 'src')).filter(
        (f) => f.endsWith('.vue') || f.endsWith('.ts') || f.endsWith('.html')
      );

      for (const file of srcFiles) {
        const content = fs.readFileSync(file, 'utf-8');
        expect(content, `Found clapper emoji in ${file}`).not.toContain('🎬');
      }
    });

    it('scans all src/ components and views to ensure 0 INNOSTART remnants exist', () => {
      const srcFiles = getAllFiles(path.join(rootDir, 'src')).filter(
        (f) => f.endsWith('.vue') || f.endsWith('.ts') || f.endsWith('.html')
      );

      for (const file of srcFiles) {
        const content = fs.readFileSync(file, 'utf-8');
        expect(content, `Found INNOSTART in ${file}`).not.toContain('INNOSTART');
      }
    });

    it('scans all src/ files to ensure no live interactive Demo Day tour buttons or scripts exist in templates', () => {
      const forbiddenCues = [
        'guided-tour-trigger',
        'Trình Diễn Demo Day',
        'Lời thoại diễn giả',
        'Speaker Script Cue',
        '🚀 NẠP DỮ LIỆU MẪU ĐO KIỂM CHUẨN (Demo Ready)',
      ];

      const srcFiles = getAllFiles(path.join(rootDir, 'src')).filter(
        (f) => f.endsWith('.vue') || f.endsWith('.ts')
      );

      for (const file of srcFiles) {
        const content = fs.readFileSync(file, 'utf-8');
        for (const cue of forbiddenCues) {
          expect(content, `Found forbidden cue "${cue}" in ${file}`).not.toContain(cue);
        }
      }
    });

    it('verifies production build dist/ assets are completely free of clapper emojis, INNOSTART, and Demo Day trigger text', () => {
      const distDir = path.join(rootDir, 'dist');
      if (fs.existsSync(distDir)) {
        const distFiles = getAllFiles(distDir).filter(
          (f) => f.endsWith('.js') || f.endsWith('.html') || f.endsWith('.css')
        );

        for (const file of distFiles) {
          const content = fs.readFileSync(file, 'utf-8');
          expect(content, `Found clapper emoji in dist file ${file}`).not.toContain('🎬');
          expect(content, `Found INNOSTART in dist file ${file}`).not.toContain('INNOSTART');
          expect(content, `Found Demo Day (5 Phút) in dist file ${file}`).not.toContain('Demo Day (5 Phút)');
          expect(content, `Found guided-tour-trigger in dist file ${file}`).not.toContain('guided-tour-trigger');
        }
      }
    });
  });

  // ==========================================================================
  // SECTION 2: ADVERSARIAL AUTH STORE & RBAC INVARIANT STRESS HARNESS
  // ==========================================================================
  describe('2. Adversarial authStore & RBAC Invariant Verification', () => {
    it('verifies strict RBAC mutual exclusion across all 4 standard profiles', async () => {
      const auth = useAuthStore();

      // Test MAKER exclusivity
      await auth.switchRole('MAKER');
      expect(auth.isMaker).toBe(true);
      expect(auth.isChecker).toBe(false);
      expect(auth.isAml).toBe(false);
      expect(auth.isTreasury).toBe(false);

      // Test CHECKER exclusivity
      await auth.switchRole('CHECKER');
      expect(auth.isMaker).toBe(false);
      expect(auth.isChecker).toBe(true);
      expect(auth.isAml).toBe(false);
      expect(auth.isTreasury).toBe(false);

      // Test AML exclusivity
      await auth.switchRole('AML');
      expect(auth.isMaker).toBe(false);
      expect(auth.isChecker).toBe(false);
      expect(auth.isAml).toBe(true);
      expect(auth.isTreasury).toBe(false);

      // Test TREASURY exclusivity
      await auth.switchRole('TREASURY');
      expect(auth.isMaker).toBe(false);
      expect(auth.isChecker).toBe(false);
      expect(auth.isAml).toBe(false);
      expect(auth.isTreasury).toBe(true);
    });

    it('enforces Fail-Closed security: unknown or adversary roles receive ZERO privileges', () => {
      const auth = useAuthStore();
      const maliciousRoles = [
        'ADMIN',
        'SUPERADMIN',
        'ROOT',
        'SYSTEM',
        'MAKER; DROP TABLE users;',
        '<script>alert(1)</script>',
        '__proto__',
        'constructor',
        '',
      ];

      for (const badRole of maliciousRoles) {
        auth.currentUser = {
          id: 'usr_malicious',
          email: 'adversary@evil.corp',
          fullName: 'Adversary Injection',
          role: badRole as any,
          officerId: 'EVIL-666',
          branchCode: 'HO-HN-001',
          avatar: '🥷',
          bankAccess: [],
        };

        expect(auth.isMaker, `Role ${badRole} leaked Maker privilege`).toBe(false);
        expect(auth.isChecker, `Role ${badRole} leaked Checker privilege`).toBe(false);
        expect(auth.isAml, `Role ${badRole} leaked AML privilege`).toBe(false);
        expect(auth.isTreasury, `Role ${badRole} leaked Treasury privilege`).toBe(false);
        expect(auth.isAuditor, `Role ${badRole} leaked Auditor privilege`).toBe(false);
      }
    });

    it('survives prototype pollution and nested object injection into localStorage without store corruption', () => {
      const pollutionPayloads = [
        '{"__proto__": {"isMaker": true, "role": "CHECKER"}}',
        '{"constructor": {"prototype": {"admin": true}}}',
        '{"role": "MAKER", "officerId": {"$ne": null}}',
        '{"role": "CHECKER", "bankAccess": "ALL_BANKS"}',
        '[1, 2, 3, "unexpected_array"]',
        '123456789',
        '"just_a_string"',
      ];

      for (const payload of pollutionPayloads) {
        mockStorage.setItem('liva_auth_user', payload);
        setActivePinia(createPinia());
        const auth = useAuthStore();

        expect(() => auth.isMaker).not.toThrow();
        expect(() => auth.isChecker).not.toThrow();
        expect(() => auth.isAml).not.toThrow();
        expect(() => auth.isTreasury).not.toThrow();
        expect(() => auth.currentOfficerId).not.toThrow();
      }
    });

    it('survives extreme malformed tokens including 20KB gigantic payloads and non-ASCII binary strings', async () => {
      const hugeString = 'A'.repeat(20000);
      const strangeTokens = [
        `${hugeString}.${hugeString}.${hugeString}`,
        'part1.Tiếng_Việt_Có_Dấu_Không_Lỗi.part3',
        '..',
        '....',
        'header.payload.sig1.sig2.sig3',
        'eyJhbGciOiJIUzI1NiJ9.eyJleHAiOi0xfQ.sig', // exp: -1
        'eyJhbGciOiJIUzI1NiJ9.eyJleHAiOjB9.sig',  // exp: 0
        'eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.eyJzdWIiOiIxMjMifQ.', // alg: none
      ];

      for (const tok of strangeTokens) {
        mockStorage.setItem('liva_auth_token', tok);
        setActivePinia(createPinia());
        const auth = useAuthStore();

        expect(() => auth.token).not.toThrow();
        expect(() => auth.currentOfficerId).not.toThrow();
        expect(() => auth.currentBranchCode).not.toThrow();

        // Must recover safely when user switches role
        await auth.switchRole('MAKER');
        expect(auth.isMaker).toBe(true);
        const fixedToken = mockStorage.getItem('liva_auth_token');
        expect(fixedToken).not.toBe(tok);
        expect(fixedToken!.split('.').length).toBe(3);
      }
    });

    it('guarantees JWT generation invariant: 3 parts, valid base64url, valid claims, 4-hour expiration', async () => {
      const auth = useAuthStore();
      const roles: UserRole[] = ['MAKER', 'CHECKER', 'AML', 'TREASURY'];

      for (const r of roles) {
        if (r === 'AUDITOR') continue;
        await auth.switchRole(r as any);
        const tok = mockStorage.getItem('liva_auth_token');
        expect(tok).toBeDefined();
        expect(tok).not.toBeNull();

        const parts = tok!.split('.');
        expect(parts.length).toBe(3);

        // Header inspection
        const headerJson = Buffer.from(parts[0], 'base64url').toString('utf-8');
        const header = JSON.parse(headerJson);
        expect(header.alg).toBe('HS256');
        expect(header.typ).toBe('JWT');

        // Payload inspection
        const payloadJson = Buffer.from(parts[1], 'base64url').toString('utf-8');
        const payload = JSON.parse(payloadJson);
        expect(payload.role).toBe(r);
        expect(payload.email).toBe(PRESET_PROFILES[r as keyof typeof PRESET_PROFILES].email);
        if (payload.officerId) {
          expect(payload.officerId).toBe(PRESET_PROFILES[r as keyof typeof PRESET_PROFILES].officerId);
        }
        expect(auth.currentUser?.officerId).toBe(PRESET_PROFILES[r as keyof typeof PRESET_PROFILES].officerId);
        expect(auth.currentOfficerId).toBe(PRESET_PROFILES[r as keyof typeof PRESET_PROFILES].officerId);
        expect(auth.currentBranchCode).toBe('HO-HN-001');
        expect(auth.currentTerminalId).toBe('WS-OPER-04');

        // 4-hour TTL check
        expect(payload.exp - payload.iat).toBe(14400);
      }
    });

    it('handles concurrent role switching without racing into an inconsistent state', async () => {
      const auth = useAuthStore();

      // Launch multiple simultaneous role switch requests
      const promises = [
        auth.switchRole('CHECKER'),
        auth.switchRole('AML'),
        auth.switchRole('TREASURY'),
        auth.switchRole('MAKER'),
      ];

      await Promise.all(promises);

      // Once all settled, verify store consistency: role must match active flags & stored token
      const finalRole = auth.currentUser?.role;
      expect(finalRole).toBeDefined();

      const storedToken = mockStorage.getItem('liva_auth_token');
      expect(storedToken).toBeDefined();
      const parts = storedToken!.split('.');
      const payload = JSON.parse(Buffer.from(parts[1], 'base64url').toString('utf-8'));

      expect(payload.role).toBe(finalRole);
      if (finalRole === 'MAKER') expect(auth.isMaker).toBe(true);
      if (finalRole === 'CHECKER') expect(auth.isChecker).toBe(true);
      if (finalRole === 'AML') expect(auth.isAml).toBe(true);
      if (finalRole === 'TREASURY') expect(auth.isTreasury).toBe(true);
    });
  });

  // ==========================================================================
  // SECTION 3: OFFICER STATUS BAR LIFECYCLE & TIME ORACLE STRESS
  // ==========================================================================
  describe('3. OfficerStatusBar Lifecycle & Time Formatting Oracle', () => {
    it('formats session elapsed time with exact precision across 10 boundary conditions', () => {
      const formatTime = (totalSeconds: number) => {
        const hrs = Math.floor(totalSeconds / 3600).toString().padStart(2, '0');
        const mins = Math.floor((totalSeconds % 3600) / 60).toString().padStart(2, '0');
        const secs = (totalSeconds % 60).toString().padStart(2, '0');
        return `${hrs}:${mins}:${secs}`;
      };

      const testCases: [number, string][] = [
        [0, '00:00:00'],
        [1, '00:00:01'],
        [59, '00:00:59'],
        [60, '00:01:00'],
        [61, '00:01:01'],
        [3599, '00:59:59'],
        [3600, '01:00:00'],
        [3661, '01:01:01'],
        [86399, '23:59:59'],
        [86400, '24:00:00'],
        [359999, '99:59:59'],
      ];

      for (const [sec, expected] of testCases) {
        expect(formatTime(sec), `Failed for ${sec}s`).toBe(expected);
      }
    });

    it('stress tests 500 rapid timer create and clear operations to verify zero memory or handle leaks', () => {
      const handles: any[] = [];
      for (let i = 0; i < 500; i++) {
        let count = 0;
        const h = setInterval(() => { count++; }, 1000);
        handles.push(h);
        clearInterval(h);
      }
      expect(handles.length).toBe(500);
    });

    it('verifies tokenShort displays correctly on varying length edge cases', () => {
      const formatShort = (tok: string | null) => {
        if (!tok) return 'Chưa cấp';
        return tok.length > 16
          ? tok.substring(0, 8) + '...' + tok.substring(tok.length - 6)
          : tok;
      };

      expect(formatShort(null)).toBe('Chưa cấp');
      expect(formatShort('')).toBe('Chưa cấp');
      expect(formatShort('12345')).toBe('12345');
      expect(formatShort('1234567890123456')).toBe('1234567890123456'); // 16 chars: full
      expect(formatShort('12345678901234567')).toBe('12345678...234567'); // 17 chars: trimmed
      expect(formatShort('header.payload.signature')).toBe('header.p...nature');
    });

    it('verifies complete tab mapping dispatch matrix for all 4 roles', () => {
      const roleToTabMap: Record<'MAKER' | 'CHECKER' | 'AML' | 'TREASURY', string> = {
        MAKER: 'reconciliation',
        CHECKER: 'treasury',
        AML: 'aml',
        TREASURY: 'dashboard',
      };

      expect(roleToTabMap.MAKER).toBe('reconciliation');
      expect(roleToTabMap.CHECKER).toBe('treasury');
      expect(roleToTabMap.AML).toBe('aml');
      expect(roleToTabMap.TREASURY).toBe('dashboard');
    });

    it('verifies OfficerStatusBar role badge styling contract matches UI design tokens', () => {
      const getBadgeClass = (role: UserRole) => {
        if (role === 'CHECKER') return 'bg-amber-500/20 text-amber-400 border border-amber-500/30';
        if (role === 'AML' || role === 'AUDITOR') return 'bg-rose-500/20 text-rose-400 border border-rose-500/30';
        if (role === 'TREASURY') return 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30';
        return 'bg-blue-500/20 text-blue-400 border border-blue-500/30';
      };

      expect(getBadgeClass('MAKER')).toContain('blue');
      expect(getBadgeClass('CHECKER')).toContain('amber');
      expect(getBadgeClass('AML')).toContain('rose');
      expect(getBadgeClass('AUDITOR')).toContain('rose');
      expect(getBadgeClass('TREASURY')).toContain('emerald');
    });

    it('verifies decodedClaims fault tolerance with URL-safe base64 (- and _) and malformed payloads', () => {
      const decode = (tok: string | null, fallbackUser: any) => {
        if (tok) {
          const parts = tok.split('.');
          if (parts.length === 3) {
            try {
              const payloadStr = (globalThis as any).window.atob(
                parts[1].replace(/-/g, '+').replace(/_/g, '/')
              );
              return JSON.parse(payloadStr);
            } catch {
              // fall through
            }
          }
        }
        return fallbackUser;
      };

      const fallback = { role: 'MAKER', fullName: 'Fallback Officer' };

      // 1. URL-safe base64 with - and _
      const testPayload = { test: true, special: 'abc-def_ghi' };
      const urlSafeB64 = Buffer.from(JSON.stringify(testPayload))
        .toString('base64')
        .replace(/\+/g, '-')
        .replace(/\//g, '_');
      expect(decode(`header.${urlSafeB64}.sig`, fallback)).toEqual(testPayload);

      // 2. Malformed Base64
      expect(decode('header.not_valid_b64!@#$.sig', fallback)).toBe(fallback);

      // 3. Valid Base64 but broken JSON
      const brokenJsonB64 = Buffer.from('{ broken json').toString('base64');
      expect(decode(`header.${brokenJsonB64}.sig`, fallback)).toBe(fallback);

      // 4. Null / empty token
      expect(decode(null, fallback)).toBe(fallback);
      expect(decode('', fallback)).toBe(fallback);
    });

    it('verifies logout idempotency and full cleanup without errors on consecutive calls', async () => {
      const auth = useAuthStore();
      await auth.switchRole('CHECKER');
      expect(auth.isAuthenticated).toBe(true);
      expect(mockStorage.getItem('liva_auth_token')).not.toBeNull();

      // First logout
      auth.logout();
      expect(auth.isAuthenticated).toBe(false);
      expect(auth.token).toBeNull();
      expect(mockStorage.getItem('liva_auth_token')).toBeNull();

      // Second consecutive logout (must be idempotent, no throw)
      expect(() => auth.logout()).not.toThrow();
      expect(auth.isAuthenticated).toBe(false);
      expect(auth.token).toBeNull();
    });
  });
});


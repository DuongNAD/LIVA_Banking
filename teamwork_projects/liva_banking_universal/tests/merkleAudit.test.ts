import { describe, it, expect } from 'vitest';
import * as nodeCrypto from 'node:crypto';
import {
  sha256,
  hmacSha256,
  computeMerkleLeaf,
  buildMerkleTree,
  generateMerkleProof,
  verifyMerkleProof,
  ForwardAuditLedger,
  GENESIS_LEDGER_HASH,
} from '../src/engine/treasury/merkleAudit';

describe('Feature F18: Cryptographic Merkle Audit Engine & Forward Hash-Chained Ledger', () => {
  describe('1. Cryptographic Hash Primitives (FIPS 180-4 SHA-256 & RFC 2104 HMAC)', () => {
    it('computes standard SHA-256 matching NIST test vectors', () => {
      // NIST Vector 1: Empty string
      expect(sha256('')).toBe('e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855');

      // NIST Vector 2: "abc"
      expect(sha256('abc')).toBe('ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');

      // NIST Vector 3: 448-bit edge case ("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq")
      const str448 = 'abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq';
      const expected448 = nodeCrypto.createHash('sha256').update(str448).digest('hex');
      expect(sha256(str448)).toBe(expected448);
    });

    it('produces identical output to node:crypto for Vietnamese banking strings', () => {
      const inputs = [
        'Lệnh chi số 12345 thanh toán Masan 550.000.000 VND',
        'Nghị định 13/2023/NĐ-CP bảo vệ dữ liệu cá nhân ngân hàng',
        'Thông tư 09/2020/TT-NHNN Điều 16 & 18 kiểm soát chéo Maker-Checker',
        'a'.repeat(1000),
      ];

      for (const input of inputs) {
        const expected = nodeCrypto.createHash('sha256').update(input).digest('hex');
        expect(sha256(input)).toBe(expected);
      }
    });

    it('computes RFC 2104 HMAC-SHA256 matching node:crypto', () => {
      const key = 'SECRET_BANKING_KEY_2026';
      const message = 'vch-1001|550000000|maker_01|checker_cfo|2026-08-15T10:00:00Z';

      const expectedHmac = nodeCrypto.createHmac('sha256', key).update(message).digest('hex');
      expect(hmacSha256(key, message)).toBe(expectedHmac);
    });
  });

  describe('2. Canonical Merkle Leaf Hashing', () => {
    it('computes 64-character lowercase hex leaf hash for approved voucher', () => {
      const voucher = {
        voucherId: 'vch-001',
        makerId: 'maker_1',
        checkerId: 'checker_cfo',
        amountVnd: 100_000_000,
        status: 'APPROVED',
        approvedAt: '2026-08-15T10:00:00Z',
      };

      const leaf = computeMerkleLeaf(voucher);
      expect(leaf).toHaveLength(64);
      expect(leaf).toBe(leaf.toLowerCase());

      const expectedContent = 'vch-001|maker_1|checker_cfo|100000000|APPROVED|2026-08-15T10:00:00Z';
      expect(leaf).toBe(sha256(expectedContent));
    });

    it('diverges leaf hash upon tampering with amount by 1 VND', () => {
      const v1 = { voucherId: 'v1', makerId: 'm1', amountVnd: 500_000_000, status: 'APPROVED' };
      const v2 = { voucherId: 'v1', makerId: 'm1', amountVnd: 500_000_001, status: 'APPROVED' };

      expect(computeMerkleLeaf(v1)).not.toBe(computeMerkleLeaf(v2));
    });

    it('diverges leaf hash upon tampering with voucher status', () => {
      const v1 = { voucherId: 'v1', makerId: 'm1', amountVnd: 500_000_000, status: 'PENDING_APPROVAL' };
      const v2 = { voucherId: 'v1', makerId: 'm1', amountVnd: 500_000_000, status: 'APPROVED' };

      expect(computeMerkleLeaf(v1)).not.toBe(computeMerkleLeaf(v2));
    });
  });

  describe('3. Binary Merkle Tree Construction', () => {
    it('returns deterministic root for empty leaves', () => {
      const tree = buildMerkleTree([]);
      expect(tree.root).toBe(sha256('EMPTY_TREE'));
      expect(tree.leaves).toHaveLength(0);
    });

    it('returns the leaf itself as root for single-leaf tree', () => {
      const leaf = sha256('SINGLE_LEAF');
      const tree = buildMerkleTree([leaf]);
      expect(tree.root).toBe(leaf);
      expect(tree.leaves).toEqual([leaf]);
    });

    it('builds tree with 2 leaves combining left and right', () => {
      const l1 = sha256('LEAF_1');
      const l2 = sha256('LEAF_2');
      const tree = buildMerkleTree([l1, l2]);

      const expectedRoot = sha256(l1 + l2);
      expect(tree.root).toBe(expectedRoot);
    });

    it('builds tree with odd number of leaves (3 leaves) by duplicating the last leaf', () => {
      const l1 = sha256('L1');
      const l2 = sha256('L2');
      const l3 = sha256('L3');

      const tree = buildMerkleTree([l1, l2, l3]);

      // Level 1: [sha256(L1+L2), sha256(L3+L3)]
      const p1 = sha256(l1 + l2);
      const p2 = sha256(l3 + l3);
      const expectedRoot = sha256(p1 + p2);

      expect(tree.root).toBe(expectedRoot);
    });

    it('builds deterministic root for 16 leaves', () => {
      const leaves = Array.from({ length: 16 }, (_, i) => sha256(`VOUCHER_TX_${i}`));
      const tree = buildMerkleTree(leaves);
      expect(tree.root).toHaveLength(64);
      expect(tree.leaves).toHaveLength(16);
    });

    it('guarantees tamper-evidence: altering 1 character in any leaf alters Merkle root', () => {
      const l1 = sha256('VOUCHER_MASAN_550M');
      const l2 = sha256('VOUCHER_FPT_85M');
      const l3 = sha256('VOUCHER_ANPHAT_145M');

      const originalTree = buildMerkleTree([l1, l2, l3]);

      // Tamper leaf 2
      const tamperedL2 = sha256('VOUCHER_FPT_85M_TAMPERED');
      const tamperedTree = buildMerkleTree([l1, tamperedL2, l3]);

      expect(originalTree.root).not.toBe(tamperedTree.root);
    });
  });

  describe('4. O(log N) Cryptographic Merkle Inclusion Proofs', () => {
    it('generates and verifies inclusion proof for all leaves in a 4-leaf tree', () => {
      const leaves = [sha256('leaf_0'), sha256('leaf_1'), sha256('leaf_2'), sha256('leaf_3')];
      const tree = buildMerkleTree(leaves);

      for (let i = 0; i < leaves.length; i++) {
        const proof = generateMerkleProof(leaves, i);
        expect(proof.leaf).toBe(leaves[i]);
        expect(proof.root).toBe(tree.root);
        expect(proof.proof).toHaveLength(2); // log2(4) = 2 steps

        const isValid = verifyMerkleProof(proof.leaf, proof.proof, proof.root);
        expect(isValid).toBe(true);
      }
    });

    it('generates and verifies inclusion proof for odd-count tree (5 leaves)', () => {
      const leaves = Array.from({ length: 5 }, (_, i) => sha256(`item_${i}`));
      const tree = buildMerkleTree(leaves);

      for (let i = 0; i < leaves.length; i++) {
        const proof = generateMerkleProof(leaves, i);
        const isValid = verifyMerkleProof(proof.leaf, proof.proof, tree.root);
        expect(isValid).toBe(true);
      }
    });

    it('verifies inclusion proof for single-leaf tree', () => {
      const leaf = sha256('lonely_leaf');
      const tree = buildMerkleTree([leaf]);

      const proof = generateMerkleProof([leaf], 0);
      expect(proof.proof).toHaveLength(0);
      expect(verifyMerkleProof(leaf, proof.proof, tree.root)).toBe(true);
    });

    it('rejects inclusion proof if leaf data is tampered', () => {
      const leaves = [sha256('leaf_A'), sha256('leaf_B'), sha256('leaf_C')];
      const tree = buildMerkleTree(leaves);

      const proof = generateMerkleProof(leaves, 1);
      const fakeLeaf = sha256('leaf_B_TAMPERED');

      const isValid = verifyMerkleProof(fakeLeaf, proof.proof, tree.root);
      expect(isValid).toBe(false);
    });

    it('rejects inclusion proof if root is wrong or spoofed', () => {
      const leaves = [sha256('leaf_A'), sha256('leaf_B')];
      const proof = generateMerkleProof(leaves, 0);

      const fakeRoot = sha256('FAKE_ROOT');
      expect(verifyMerkleProof(proof.leaf, proof.proof, fakeRoot)).toBe(false);
    });

    it('fails closed when targetIndex is out of bounds', () => {
      const leaves = [sha256('A'), sha256('B')];
      expect(() => generateMerkleProof(leaves, -1)).toThrow(/out of bounds/);
      expect(() => generateMerkleProof(leaves, 2)).toThrow(/out of bounds/);
      expect(() => generateMerkleProof([], 0)).toThrow(/empty/);
    });
  });

  describe('5. Forward Hash-Chained Audit Ledger', () => {
    it('initializes with known genesis block hash and appends chained blocks', () => {
      const ledger = new ForwardAuditLedger();
      expect(ledger.getEntries()).toHaveLength(0);
      expect(ledger.getLatestHash()).toBe(GENESIS_LEDGER_HASH);

      const e1 = ledger.append('maker_01', 'VOUCHER_CREATED', { id: 'v1', amount: 50_000_000 });
      expect(e1.entryIndex).toBe(1);
      expect(e1.prevHash).toBe(GENESIS_LEDGER_HASH);
      expect(e1.currentHash).toHaveLength(64);
      expect(ledger.getLatestHash()).toBe(e1.currentHash);

      const e2 = ledger.append('checker_01', 'VOUCHER_APPROVED', { id: 'v1' });
      expect(e2.entryIndex).toBe(2);
      expect(e2.prevHash).toBe(e1.currentHash);
      expect(e2.currentHash).toHaveLength(64);
      expect(ledger.getLatestHash()).toBe(e2.currentHash);
    });

    it('verifies integrity of untampered ledger blocks', () => {
      const ledger = new ForwardAuditLedger();
      ledger.append('system', 'INIT', {});
      ledger.append('maker_1', 'VOUCHER_DRAFT', { amount: 100 });
      ledger.append('checker_1', 'VOUCHER_APPROVE', { amount: 100 });

      const check = ledger.verifyIntegrity();
      expect(check.isValid).toBe(true);
      expect(check.brokenAt).toBeUndefined();
    });

    it('detects tampering if any block payload is modified post-hoc', () => {
      const ledger = new ForwardAuditLedger();
      ledger.append('maker_1', 'VOUCHER_DRAFT', { amount: 100_000_000 });
      ledger.append('checker_1', 'VOUCHER_APPROVE', { amount: 100_000_000 });

      // Maliciously tamper block #1 payload
      const entries = (ledger as any).entries;
      entries[0].payload = { amount: 999_999_999 };

      const check = ledger.verifyIntegrity();
      expect(check.isValid).toBe(false);
      expect(check.brokenAt).toBe(1);
      expect(check.details).toContain('Block #1 currentHash mismatch');
    });

    it('computes Merkle root over all audit chain block hashes', () => {
      const ledger = new ForwardAuditLedger();
      ledger.append('maker_1', 'VOUCHER_DRAFT', { amount: 100 });
      ledger.append('checker_1', 'VOUCHER_APPROVE', { amount: 100 });

      const root = ledger.computeMerkleRoot();
      expect(root).toHaveLength(64);
      expect(root).toBe(root.toLowerCase());
    });
  });
});

/**
 * Cryptographic Merkle Audit Engine & Forward Hash-Chained Ledger
 * Adhering to Circular 09/2020/TT-NHNN & Decree 13/2023/ND-CP Zero Data Egress
 *
 * Implements:
 * 1. FIPS 180-4 Standard SHA-256 and RFC 2104 HMAC-SHA256 (Pure TypeScript, zero dependencies, client & server safe)
 * 2. Binary Merkle Tree computation with odd-leaf duplicate pairing
 * 3. O(log N) Cryptographic Inclusion Proof generation & verification
 * 4. Forward hash-chained tamper-evident audit ledger (H_k = SHA256(H_{k-1} || index || ts || actor || event || payload))
 */

import type {
  PaymentVoucher,
  MerkleProof,
  MerkleProofStep,
  MerkleTreeResult,
  AuditLogEntry,
} from '../../types/treasury';

// ============================================================================
// PURE TYPESCRIPT SHA-256 IMPLEMENTATION (FIPS 180-4)
// ============================================================================

const K = new Uint32Array([
  0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
  0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
  0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
  0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
  0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
  0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
  0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
  0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
]);

function rotr(x: number, n: number): number {
  return (x >>> n) | (x << (32 - n));
}

function ch(x: number, y: number, z: number): number {
  return (x & y) ^ (~x & z);
}

function maj(x: number, y: number, z: number): number {
  return (x & y) ^ (x & z) ^ (y & z);
}

function sigma0(x: number): number {
  return rotr(x, 2) ^ rotr(x, 13) ^ rotr(x, 22);
}

function sigma1(x: number): number {
  return rotr(x, 6) ^ rotr(x, 11) ^ rotr(x, 25);
}

function gamma0(x: number): number {
  return rotr(x, 7) ^ rotr(x, 18) ^ (x >>> 3);
}

function gamma1(x: number): number {
  return rotr(x, 17) ^ rotr(x, 19) ^ (x >>> 10);
}

function utf8Encode(str: string): Uint8Array {
  if (typeof TextEncoder !== 'undefined') {
    return new TextEncoder().encode(str);
  }
  const bytes: number[] = [];
  for (let i = 0; i < str.length; i++) {
    let c = str.charCodeAt(i);
    if (c < 0x80) {
      bytes.push(c);
    } else if (c < 0x800) {
      bytes.push(0xc0 | (c >> 6), 0x80 | (c & 0x3f));
    } else if (c < 0xd800 || c >= 0xe000) {
      bytes.push(0xe0 | (c >> 12), 0x80 | ((c >> 6) & 0x3f), 0x80 | (c & 0x3f));
    } else {
      i++;
      c = 0x10000 + (((c & 0x3ff) << 10) | (str.charCodeAt(i) & 0x3ff));
      bytes.push(
        0xf0 | (c >> 18),
        0x80 | ((c >> 12) & 0x3f),
        0x80 | ((c >> 6) & 0x3f),
        0x80 | (c & 0x3f)
      );
    }
  }
  return new Uint8Array(bytes);
}

export function sha256Bytes(data: Uint8Array): Uint8Array {
  const byteLen = data.length;
  const bitLen = byteLen * 8;

  // Pre-processing: Padding (message + 0x80 + zeroes + 64-bit length)
  const padLen = (56 - ((byteLen + 1) % 64) + 64) % 64;
  const totalLen = byteLen + 1 + padLen + 8;
  const padded = new Uint8Array(totalLen);
  padded.set(data);
  padded[byteLen] = 0x80;

  // Append length in bits (big-endian 64-bit integer)
  const view = new DataView(padded.buffer, padded.byteOffset, padded.byteLength);
  view.setUint32(totalLen - 4, bitLen >>> 0, false);
  view.setUint32(totalLen - 8, Math.floor(bitLen / 0x100000000), false);

  // Initial hash values
  let h0 = 0x6a09e667;
  let h1 = 0xbb67ae85;
  let h2 = 0x3c6ef372;
  let h3 = 0xa54ff53a;
  let h4 = 0x510e527f;
  let h5 = 0x9b05688c;
  let h6 = 0x1f83d9ab;
  let h7 = 0x5be0cd19;

  const w = new Uint32Array(64);

  // Process 512-bit (64-byte) blocks
  for (let offset = 0; offset < totalLen; offset += 64) {
    for (let i = 0; i < 16; i++) {
      w[i] = view.getUint32(offset + i * 4, false);
    }
    for (let i = 16; i < 64; i++) {
      w[i] = (gamma1(w[i - 2]) + w[i - 7] + gamma0(w[i - 15]) + w[i - 16]) | 0;
    }

    let a = h0;
    let b = h1;
    let c = h2;
    let d = h3;
    let e = h4;
    let f = h5;
    let g = h6;
    let h = h7;

    for (let i = 0; i < 64; i++) {
      const t1 = (h + sigma1(e) + ch(e, f, g) + K[i] + w[i]) | 0;
      const t2 = (sigma0(a) + maj(a, b, c)) | 0;
      h = g;
      g = f;
      f = e;
      e = (d + t1) | 0;
      d = c;
      c = b;
      b = a;
      a = (t1 + t2) | 0;
    }

    h0 = (h0 + a) | 0;
    h1 = (h1 + b) | 0;
    h2 = (h2 + c) | 0;
    h3 = (h3 + d) | 0;
    h4 = (h4 + e) | 0;
    h5 = (h5 + f) | 0;
    h6 = (h6 + g) | 0;
    h7 = (h7 + h) | 0;
  }

  const out = new Uint8Array(32);
  const outView = new DataView(out.buffer);
  outView.setUint32(0, h0, false);
  outView.setUint32(4, h1, false);
  outView.setUint32(8, h2, false);
  outView.setUint32(12, h3, false);
  outView.setUint32(16, h4, false);
  outView.setUint32(20, h5, false);
  outView.setUint32(24, h6, false);
  outView.setUint32(28, h7, false);
  return out;
}

export function sha256(data: string): string {
  const bytes = utf8Encode(data);
  const hash = sha256Bytes(bytes);
  let hex = '';
  for (let i = 0; i < hash.length; i++) {
    hex += hash[i].toString(16).padStart(2, '0');
  }
  return hex.toLowerCase();
}

/**
 * Standard RFC 2104 HMAC-SHA256 Implementation
 */
export function hmacSha256(keyStr: string, messageStr: string): string {
  let key = utf8Encode(keyStr);
  const msg = utf8Encode(messageStr);

  if (key.length > 64) {
    key = sha256Bytes(key);
  }

  const paddedKey = new Uint8Array(64);
  paddedKey.set(key);

  const oKeyPad = new Uint8Array(64);
  const iKeyPad = new Uint8Array(64);

  for (let i = 0; i < 64; i++) {
    oKeyPad[i] = paddedKey[i] ^ 0x5c;
    iKeyPad[i] = paddedKey[i] ^ 0x36;
  }

  const inner = new Uint8Array(64 + msg.length);
  inner.set(iKeyPad, 0);
  inner.set(msg, 64);
  const innerHash = sha256Bytes(inner);

  const outer = new Uint8Array(64 + 32);
  outer.set(oKeyPad, 0);
  outer.set(innerHash, 64);
  const outerHash = sha256Bytes(outer);

  let hex = '';
  for (let i = 0; i < outerHash.length; i++) {
    hex += outerHash[i].toString(16).padStart(2, '0');
  }
  return hex.toLowerCase();
}

// ============================================================================
// MERKLE TREE LEAF & TREE COMPUTATION
// ============================================================================

/**
 * Computes canonical Merkle leaf hash for a payment voucher.
 * Consistent with E2E test harness specification:
 * content = `${voucherId}|${makerId}|${checkerId}|${amountVnd}|${status}|${approvedAt || createdAt}`
 */
export function computeMerkleLeaf(voucher: Partial<PaymentVoucher> | any): string {
  const content = `${voucher.voucherId}|${voucher.makerId}|${voucher.checkerId || ''}|${voucher.amountVnd}|${voucher.status}|${voucher.approvedAt || voucher.createdAt}`;
  return sha256(content).toLowerCase();
}

/**
 * Builds binary Merkle tree from an array of leaf hashes.
 * Rules:
 * - Empty leaves: returns deterministic safe hash for 'EMPTY_TREE'.
 * - 1 leaf: returns root = leaf.
 * - Multi-leaf: pairs (0,1), (2,3), etc.
 * - Odd leaf: duplicate paired with itself.
 */
export function buildMerkleTree(leafHashes: string[]): MerkleTreeResult {
  if (!leafHashes || leafHashes.length === 0) {
    return {
      root: sha256('EMPTY_TREE').toLowerCase(),
      leaves: [],
    };
  }

  if (leafHashes.length === 1) {
    return {
      root: leafHashes[0].toLowerCase(),
      leaves: leafHashes.map((l) => l.toLowerCase()),
    };
  }

  const normalizedLeaves = leafHashes.map((l) => l.toLowerCase());
  let currentLevel = [...normalizedLeaves];

  while (currentLevel.length > 1) {
    const nextLevel: string[] = [];
    for (let i = 0; i < currentLevel.length; i += 2) {
      if (i + 1 < currentLevel.length) {
        const combined = sha256(currentLevel[i] + currentLevel[i + 1]).toLowerCase();
        nextLevel.push(combined);
      } else {
        // Odd leaf: duplicate paired with itself
        const combined = sha256(currentLevel[i] + currentLevel[i]).toLowerCase();
        nextLevel.push(combined);
      }
    }
    currentLevel = nextLevel;
  }

  return {
    root: currentLevel[0].toLowerCase(),
    leaves: normalizedLeaves,
  };
}

/**
 * Generates an O(log N) cryptographic Merkle audit inclusion proof
 * for a target leaf at the specified index.
 */
export function generateMerkleProof(leafHashes: string[], targetIndex: number): MerkleProof {
  if (!leafHashes || leafHashes.length === 0) {
    throw new Error('Cannot generate Merkle proof for empty leaves');
  }
  if (targetIndex < 0 || targetIndex >= leafHashes.length) {
    throw new Error(`Target index ${targetIndex} out of bounds [0, ${leafHashes.length - 1}]`);
  }

  const normalizedLeaves = leafHashes.map((l) => l.toLowerCase());
  const tree = buildMerkleTree(normalizedLeaves);
  const targetLeaf = normalizedLeaves[targetIndex];

  if (normalizedLeaves.length === 1) {
    return {
      leaf: targetLeaf,
      root: tree.root,
      index: targetIndex,
      proof: [],
    };
  }

  const proof: MerkleProofStep[] = [];
  let currentLevel = [...normalizedLeaves];
  let currentIndex = targetIndex;

  while (currentLevel.length > 1) {
    const isEven = currentIndex % 2 === 0;
    const siblingIndex = isEven ? currentIndex + 1 : currentIndex - 1;

    if (isEven) {
      // Sibling is on the right
      if (siblingIndex < currentLevel.length) {
        proof.push({ position: 'right', hash: currentLevel[siblingIndex] });
      } else {
        // Odd leaf duplicated itself
        proof.push({ position: 'right', hash: currentLevel[currentIndex] });
      }
    } else {
      // Sibling is on the left
      proof.push({ position: 'left', hash: currentLevel[siblingIndex] });
    }

    // Advance to parent level
    const nextLevel: string[] = [];
    for (let i = 0; i < currentLevel.length; i += 2) {
      if (i + 1 < currentLevel.length) {
        nextLevel.push(sha256(currentLevel[i] + currentLevel[i + 1]).toLowerCase());
      } else {
        nextLevel.push(sha256(currentLevel[i] + currentLevel[i]).toLowerCase());
      }
    }

    currentIndex = Math.floor(currentIndex / 2);
    currentLevel = nextLevel;
  }

  return {
    leaf: targetLeaf,
    root: tree.root,
    index: targetIndex,
    proof,
  };
}

/**
 * Verifies an O(log N) Merkle audit inclusion proof.
 * Returns true if recomputed root matches the expected Merkle root.
 */
export function verifyMerkleProof(leaf: string, proof: MerkleProofStep[], root: string): boolean {
  let currentHash = leaf.toLowerCase();

  for (const step of proof) {
    const stepHash = step.hash.toLowerCase();
    if (step.position === 'left') {
      currentHash = sha256(stepHash + currentHash).toLowerCase();
    } else {
      currentHash = sha256(currentHash + stepHash).toLowerCase();
    }
  }

  return currentHash === root.toLowerCase();
}

// ============================================================================
// FORWARD HASH-CHAINED AUDIT LEDGER (TAMPER-EVIDENT)
// ============================================================================

export const GENESIS_LEDGER_HASH = sha256('GENESIS_LEDGER_BLOCK_LIVA_2026').toLowerCase();

/**
 * Forward Hash-Chained Audit Ledger
 * Guarantees immutable, tamper-evident audit logging for all banking operations.
 * Formula: H_k = SHA256(H_{k-1} || entryIndex || timestamp || actor || event || payload)
 */
export class ForwardAuditLedger {
  private entries: AuditLogEntry[] = [];
  private genesisHash: string;

  constructor(genesisHash: string = GENESIS_LEDGER_HASH) {
    this.genesisHash = genesisHash;
  }

  public append(actor: string, event: string, payload: any): AuditLogEntry {
    const entryIndex = this.entries.length + 1;
    const timestamp = new Date().toISOString();
    const prevHash = this.entries.length === 0 ? this.genesisHash : this.entries[this.entries.length - 1].currentHash;
    const payloadStr = typeof payload === 'string' ? payload : JSON.stringify(payload);

    const blockContent = `${prevHash}|${entryIndex}|${timestamp}|${actor}|${event}|${payloadStr}`;
    const currentHash = sha256(blockContent).toLowerCase();

    const entry: AuditLogEntry = {
      entryIndex,
      timestamp,
      actor,
      event,
      payload,
      prevHash,
      currentHash,
    };

    this.entries.push(entry);
    return entry;
  }

  public verifyIntegrity(): { isValid: boolean; brokenAt?: number; details?: string } {
    let expectedPrevHash = this.genesisHash;

    for (let i = 0; i < this.entries.length; i++) {
      const entry = this.entries[i];

      if (entry.prevHash !== expectedPrevHash) {
        return {
          isValid: false,
          brokenAt: entry.entryIndex,
          details: `Block #${entry.entryIndex} prevHash mismatch: expected ${expectedPrevHash}, got ${entry.prevHash}`,
        };
      }

      const payloadStr = typeof entry.payload === 'string' ? entry.payload : JSON.stringify(entry.payload);
      const recomputedContent = `${entry.prevHash}|${entry.entryIndex}|${entry.timestamp}|${entry.actor}|${entry.event}|${payloadStr}`;
      const recomputedHash = sha256(recomputedContent).toLowerCase();

      if (entry.currentHash !== recomputedHash) {
        return {
          isValid: false,
          brokenAt: entry.entryIndex,
          details: `Block #${entry.entryIndex} currentHash mismatch (data tampered): expected ${recomputedHash}, got ${entry.currentHash}`,
        };
      }

      expectedPrevHash = entry.currentHash;
    }

    return { isValid: true };
  }

  public getEntries(): AuditLogEntry[] {
    return [...this.entries];
  }

  public getLatestHash(): string {
    return this.entries.length === 0
      ? this.genesisHash
      : this.entries[this.entries.length - 1].currentHash;
  }

  public computeMerkleRoot(): string {
    const hashes = this.entries.map((e) => e.currentHash);
    return buildMerkleTree(hashes).root;
  }

  public clear(): void {
    this.entries = [];
  }
}

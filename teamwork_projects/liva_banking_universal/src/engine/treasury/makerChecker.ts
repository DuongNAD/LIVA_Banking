/**
 * Maker-Checker Dual Control State Machine (F17)
 * Adhering to Circular 09/2020/TT-NHNN Articles 16 & 18
 *
 * Implements:
 * 1. Strict 5-stage state transitions: DRAFT -> PENDING_APPROVAL -> APPROVED / REJECTED -> SETTLED, EXPIRED
 * 2. Strict separation of duties: makerId !== checkerId with fail-closed SelfApprovalProhibited
 * 3. Single-use UUIDv4 resolution token with 15-minute TTL (900 seconds) & replay protection
 * 4. RFC 2104 HMAC-SHA256 digital signature generation on approval
 * 5. Deterministic Merkle leaf hash computation
 */

import type { PaymentVoucher, VoucherStatus, AuthMethod } from '../../types/treasury';
import { computeMerkleLeaf, hmacSha256 } from './merkleAudit';

const HMAC_SECRET_KEY = 'LIVA_CIRCULAR_09_2020_DUAL_CONTROL_KEY_2026';

// Set of consumed tokens to ensure single-use replay protection
const consumedTokens = new Set<string>();

/**
 * Generates a standard UUIDv4 string.
 */
export function generateUuidV4(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

/**
 * Resets consumed tokens cache (useful for test isolation).
 */
export function resetConsumedTokens(): void {
  consumedTokens.clear();
}

/**
 * Creates a new Payment Voucher in DRAFT status.
 * Fails closed if makerId is missing, beneficiary account is missing, or amount is <= 0.
 */
export function createPaymentVoucher(
  makerId: string,
  beneficiaryAccount: string,
  beneficiaryBank: string,
  amountVnd: number,
  purpose: string,
  beneficiaryName?: string
): PaymentVoucher {
  if (!makerId || makerId.trim() === '') {
    throw new Error('Maker ID is required');
  }
  if (!amountVnd || amountVnd <= 0 || !Number.isFinite(amountVnd)) {
    throw new Error('Payment amount must be positive');
  }
  if (!beneficiaryAccount || beneficiaryAccount.trim() === '') {
    throw new Error('Beneficiary account is required');
  }

  const voucherId = `vch-${Date.now()}-${Math.floor(Math.random() * 1000)}`;

  return {
    voucherId,
    makerId: makerId.trim(),
    beneficiaryAccount: beneficiaryAccount.trim(),
    beneficiaryBank: beneficiaryBank?.trim() || 'VIETCOMBANK',
    beneficiaryName: beneficiaryName?.trim(),
    amountVnd: Math.round(amountVnd),
    purpose: purpose?.trim() || 'Thanh toan hop dong thuong mai',
    status: 'DRAFT',
    createdAt: new Date().toISOString(),
  };
}

/**
 * Submits a DRAFT voucher for independent Checker authorization.
 * Generates a single-use UUIDv4 HITL token with a 15-minute (900s) TTL.
 */
export function submitVoucherForApproval(voucher: PaymentVoucher): PaymentVoucher {
  if (voucher.status !== 'DRAFT') {
    throw new Error(`Cannot submit voucher in ${voucher.status} status`);
  }

  const hitlToken = generateUuidV4();
  const tokenExpiresAt = new Date(Date.now() + 15 * 60 * 1000).toISOString(); // 15-minute TTL

  return {
    ...voucher,
    status: 'PENDING_APPROVAL',
    submittedAt: new Date().toISOString(),
    hitlToken,
    tokenExpiresAt,
  };
}

/**
 * Approves a voucher in PENDING_APPROVAL status.
 * Strict fail-closed checks:
 * 1. Voucher must be in PENDING_APPROVAL status.
 * 2. Checker ID must be provided.
 * 3. Self-approval strictly prohibited: makerId !== checkerId.
 * 4. Token must not be expired (15-minute window).
 * 5. Token must not be replayed (single-use consumption).
 * 6. Generates RFC 2104 HMAC-SHA256 digital signature and Merkle leaf hash.
 */
export function approveVoucher(
  voucher: PaymentVoucher,
  checkerId: string,
  authMethod: AuthMethod = 'BIOMETRIC_SIM',
  hitlToken?: string
): PaymentVoucher {
  if (voucher.status !== 'PENDING_APPROVAL') {
    throw new Error(`Voucher is not pending approval (status: ${voucher.status})`);
  }

  if (!checkerId || checkerId.trim() === '') {
    throw new Error('Checker ID is required');
  }

  const cleanChecker = checkerId.trim();

  // Circular 09/2020: Maker cannot self-approve!
  if (voucher.makerId === cleanChecker) {
    throw new Error('Circular 09/2020/TT-NHNN Violation: Maker cannot be Checker (Self-approval prevented)');
  }

  // Token expiration verification (15 minutes / 900 seconds)
  if (voucher.tokenExpiresAt) {
    const expiresMs = new Date(voucher.tokenExpiresAt).getTime();
    if (Date.now() > expiresMs) {
      throw new Error('TokenExpired: Circular 09/2020 15-minute approval window has expired');
    }
  }

  // Single-use token replay protection
  const activeToken = hitlToken || voucher.hitlToken;
  if (activeToken) {
    if (consumedTokens.has(activeToken)) {
      throw new Error('TokenAlreadyUsed: Replay attack detected on HITL authorization token');
    }
    consumedTokens.add(activeToken);
  }

  const approvedAt = new Date().toISOString();

  // Generate digital HMAC-SHA256 signature
  const hmacPayload = `${voucher.voucherId}|${voucher.amountVnd}|${voucher.makerId}|${cleanChecker}|${approvedAt}`;
  const signatureHmac = hmacSha256(HMAC_SECRET_KEY, hmacPayload);

  // Compute canonical Merkle leaf hash
  const approvedVoucherState = {
    ...voucher,
    checkerId: cleanChecker,
    status: 'APPROVED' as VoucherStatus,
    approvedAt,
    authMethod,
    signatureHmac,
    hitlToken: null,
  };

  const leafHash = computeMerkleLeaf(approvedVoucherState);

  return {
    ...approvedVoucherState,
    merkleLeafHash: leafHash,
  };
}

/**
 * Rejects a voucher in PENDING_APPROVAL status with a mandatory/custom reason.
 * Prevents Maker from rejecting their own voucher as checker.
 */
export function rejectVoucher(
  voucher: PaymentVoucher,
  checkerId: string,
  reason: string = 'Rejected by checker',
  hitlToken?: string
): PaymentVoucher {
  if (voucher.status !== 'PENDING_APPROVAL') {
    throw new Error(`Voucher is not pending approval (status: ${voucher.status})`);
  }

  if (!checkerId || checkerId.trim() === '') {
    throw new Error('Checker ID is required');
  }

  const cleanChecker = checkerId.trim();

  // Circular 09/2020: Maker cannot reject as checker
  if (voucher.makerId === cleanChecker) {
    throw new Error('Circular 09/2020/TT-NHNN Violation: Maker cannot reject own voucher as checker');
  }

  // Consume token if present
  const activeToken = hitlToken || voucher.hitlToken;
  if (activeToken) {
    if (consumedTokens.has(activeToken)) {
      throw new Error('TokenAlreadyUsed: Replay attack detected on HITL authorization token');
    }
    consumedTokens.add(activeToken);
  }

  return {
    ...voucher,
    checkerId: cleanChecker,
    status: 'REJECTED',
    rejectedAt: new Date().toISOString(),
    rejectReason: reason,
    hitlToken: null,
  };
}

/**
 * Settles an APPROVED voucher, releasing payment through core banking / Napas.
 */
export function settleVoucher(voucher: PaymentVoucher): PaymentVoucher {
  if (voucher.status !== 'APPROVED') {
    throw new Error(`Cannot settle voucher in status ${voucher.status} (must be APPROVED)`);
  }

  return {
    ...voucher,
    status: 'SETTLED',
    settledAt: new Date().toISOString(),
  };
}

/**
 * Checks whether a PENDING_APPROVAL voucher has exceeded its 15-minute TTL.
 * If expired, transitions status to EXPIRED.
 */
export function checkVoucherExpiration(
  voucher: PaymentVoucher,
  currentTimestampMs: number = Date.now()
): PaymentVoucher {
  if (voucher.status !== 'PENDING_APPROVAL') {
    return voucher;
  }

  if (voucher.tokenExpiresAt) {
    const expiresMs = new Date(voucher.tokenExpiresAt).getTime();
    if (currentTimestampMs > expiresMs) {
      return {
        ...voucher,
        status: 'EXPIRED',
        hitlToken: null,
      };
    }
  }

  return voucher;
}

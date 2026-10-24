/**
 * LIVA Banking Universal — Centralized Backend API Server
 * Provides JWT Authentication, Multi-Machine Data Sync, and Centralized Treasury State
 * Pure Node.js Standard Library (Zero External Dependencies)
 */

import http from 'node:http';
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { DatabaseSync } from 'node:sqlite';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const PORT = process.env.PORT || 3001;

// ============================================================================
// 1. JWT Secret Initialization & Cryptographic Ephemeral Fallback
// ============================================================================
let JWT_SECRET = process.env.JWT_SECRET;
let isEphemeralJwtSecret = false;

if (!JWT_SECRET || typeof JWT_SECRET !== 'string' || JWT_SECRET.trim() === '') {
  // Production Fail-Closed Safety Guard
  if (process.env.NODE_ENV === 'production') {
    console.error(
      '\x1b[31m%s\x1b[0m',
      '[CRITICAL SECURITY ERROR] Refusing to start LIVA Banking Universal Server in production mode without explicit JWT_SECRET!'
    );
    console.error(
      'Please provision a cryptographically secure JWT_SECRET (min 256-bit entropy) via environment variables or secret vault.'
    );
    process.exit(1);
  }

  // Development & Test Mode: Cryptographically Secure Ephemeral Fallback
  isEphemeralJwtSecret = true;
  JWT_SECRET = crypto.randomBytes(32).toString('hex');

  console.warn(
    '\x1b[33m%s\x1b[0m',
    '[SECURITY WARNING] process.env.JWT_SECRET is not configured! ' +
      'A cryptographically secure dynamic ephemeral secret has been generated for this session. ' +
      'Issued tokens will become invalid upon server restart. ' +
      'For persistent multi-workstation sessions, set JWT_SECRET in your .env or environment.'
  );
}

// ==========================================
// 1. JWT Implementation (HMAC-SHA256)
// ==========================================
function base64UrlEncode(str) {
  return Buffer.from(str).toString('base64url');
}

function base64UrlDecode(str) {
  return Buffer.from(str, 'base64url').toString('utf-8');
}

function signJwt(payload, expiresInSeconds = 14400) {
  const header = { alg: 'HS256', typ: 'JWT' };
  const now = Math.floor(Date.now() / 1000);
  const fullPayload = {
    ...payload,
    iat: now,
    exp: now + expiresInSeconds,
  };

  const headerB64 = base64UrlEncode(JSON.stringify(header));
  const payloadB64 = base64UrlEncode(JSON.stringify(fullPayload));
  const data = `${headerB64}.${payloadB64}`;

  const signature = crypto
    .createHmac('sha256', JWT_SECRET)
    .update(data)
    .digest('base64url');

  return `${data}.${signature}`;
}

function verifyJwt(token) {
  if (!token || typeof token !== 'string') return null;
  const parts = token.split('.');
  if (parts.length !== 3) return null;

  const [headerB64, payloadB64, signature] = parts;
  const data = `${headerB64}.${payloadB64}`;

  const expectedSignature = crypto
    .createHmac('sha256', JWT_SECRET)
    .update(data)
    .digest('base64url');

  const sigBuf = Buffer.from(signature);
  const expectedSigBuf = Buffer.from(expectedSignature);
  if (sigBuf.length !== expectedSigBuf.length || !crypto.timingSafeEqual(sigBuf, expectedSigBuf)) {
    return null;
  }

  try {
    const payload = JSON.parse(base64UrlDecode(payloadB64));
    const now = Math.floor(Date.now() / 1000);
    if (payload.exp !== undefined && payload.exp < now) {
      return null; // Token expired
    }
    return payload;
  } catch (err) {
    return null;
  }
}

// ==========================================
// 2. Pre-seeded Users (RBAC) & SSE Channels
// ==========================================
const USERS = [
  {
    id: 'usr_maker_01',
    username: 'maker_nam',
    email: 'maker@livabanking.vn',
    password: 'LivaMaker@2026',
    altPasswords: ['maker123'],
    aliases: ['maker_nam', 'opr-77092', 'opr77092', 'maker@livabanking.vn', 'maker_nam@livabanking.vn', 'maker'],
    fullName: 'Nguyễn Văn Kế Toán',
    role: 'MAKER',
    officerId: 'OPR-77092',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Cán Bộ Vận Hành & Đối Soát (Maker)',
    avatar: '👤',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
  {
    id: 'usr_maker_02',
    username: 'maker_mai',
    email: 'maker_mai@livabanking.vn',
    password: 'LivaMaker@2026',
    altPasswords: ['maker123'],
    aliases: ['maker_mai', 'opr-77093', 'opr77093'],
    fullName: 'Lê Phương Mai',
    role: 'MAKER',
    officerId: 'OPR-77093',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Kế toán viên Thanh toán (Maker)',
    avatar: '👤',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
  {
    id: 'usr_checker_01',
    username: 'checker_tri',
    email: 'checker@livabanking.vn',
    password: 'LivaChecker@2026',
    altPasswords: ['checker123'],
    aliases: ['checker_tri', 'sup-88214', 'sup88214', 'checker@livabanking.vn', 'checker_tri@livabanking.vn', 'checker'],
    fullName: 'Trần Thị Giám Đốc',
    role: 'CHECKER',
    officerId: 'SUP-88214',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Kiểm Soát Viên Phê Duyệt (Checker)',
    avatar: '🛡️',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
  {
    id: 'usr_checker_02',
    username: 'checker_huong',
    email: 'checker_huong@livabanking.vn',
    password: 'LivaChecker@2026',
    altPasswords: ['checker123'],
    aliases: ['checker_huong', 'sup-88215', 'sup88215'],
    fullName: 'Đỗ Lan Hương',
    role: 'CHECKER',
    officerId: 'SUP-88215',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Phó phòng Kế toán / Kiểm Soát Viên (Checker)',
    avatar: '🛡️',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
  {
    id: 'usr_aml_01',
    username: 'auditor_lan',
    email: 'aml@livabanking.vn',
    password: 'LivaAudit@2026',
    altPasswords: ['aml123', 'auditor123'],
    aliases: ['auditor_lan', 'aml_lan', 'cmp-99015', 'cmp99015', 'aml@livabanking.vn', 'auditor@livabanking.vn', 'aml', 'auditor'],
    fullName: 'Lê Hoàng Thanh Tra',
    role: 'AML',
    officerId: 'CMP-99015',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Cán Bộ Giám Sát Tuân Thủ & PCRT',
    avatar: '⚖️',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
  {
    id: 'usr_treasury_01',
    username: 'cfo_hoang',
    email: 'treasury@livabanking.vn',
    password: 'LivaCfo@2026',
    altPasswords: ['treasury123'],
    aliases: ['cfo_hoang', 'trz-55038', 'trz55038', 'treasury@livabanking.vn', 'treasury'],
    fullName: 'Đặng Đình Bảo',
    role: 'TREASURY',
    officerId: 'TRZ-55038',
    branchCode: 'HO-HN-001',
    branchName: 'Hội Sở Chính Hà Nội',
    terminalId: 'WS-OPER-04',
    title: 'Cán Bộ Quản Trị Thanh Khoản & Vốn',
    avatar: '🏦',
    bankAccess: ['CITAD', 'NAPAS', 'BILATERAL', 'SWIFT', 'VCB', 'TCB', 'BIDV'],
  },
];

// Connected SSE clients for real-time synchronization
const sseClients = new Set();

function broadcastEvent(event, data) {
  const payload = `event: ${event}\ndata: ${JSON.stringify(data)}\n\n`;
  for (const client of sseClients) {
    try {
      client.write(payload);
    } catch (err) {
      sseClients.delete(client);
    }
  }
}

// ==========================================
// 3. Centralized Shared Database (SQLite + In-Memory Fallback)
// ==========================================
const SQLITE_DB_PATH = path.resolve(__dirname, '..', '..', '..', 'data', 'liva_banking.db');
const SEED_SQL_PATH = path.resolve(__dirname, '..', '..', '..', 'data', 'liva_banking_seed.sql');

let sqliteDb = null;
try {
  if (fs.existsSync(SQLITE_DB_PATH)) {
    sqliteDb = new DatabaseSync(SQLITE_DB_PATH);
  }
} catch (err) {
  // Graceful fallback if SQLite is inaccessible
}

let DB = {
  accounts: [
    {
      id: 'acc_vcb_01',
      bankCode: 'VCB',
      channelCode: 'CITAD',
      bankName: 'Ngân hàng TMCP Ngoại thương Việt Nam (Vietcombank)',
      channelName: 'Kênh Thanh toán Điện tử Liên ngân hàng CITAD (NHNN)',
      clearingAuthority: 'Ngân hàng Nhà nước Việt Nam (Sở Giao dịch NHNN)',
      accountNumber: '0071001234567',
      accountName: 'CONG TY TNHH LIVA SOLUTIONS',
      balance: 25000000000,
      balanceVnd: 25000000000,
      availableBalanceVnd: 25000000000,
      currency: 'VND',
      inflows: 1710000000,
      inflowMonthVnd: 1710000000,
      outflows: 514500000,
      outflowMonthVnd: 514500000,
      minReserveVnd: 1000000000,
      brandColor: '#005a3c',
      status: 'ACTIVE',
      lastReconciledAt: '2026-08-31T17:00:00Z',
      clearingMechanism: 'RTGS Thanh toán Điện tử Liên ngân hàng IBPS/CITAD',
      cutoffTime: '16:30 (Phiên chiều NHNN)',
    },
    {
      id: 'acc_tcb_01',
      bankCode: 'TCB',
      channelCode: 'NAPAS',
      bankName: 'Ngân hàng TMCP Kỹ thương Việt Nam (Techcombank)',
      channelName: 'Kênh Chuyển mạch Tài chính & Bù trừ Điện tử NAPAS 24/7',
      clearingAuthority: 'Công ty Cổ phần Thanh toán Quốc gia Việt Nam (NAPAS)',
      accountNumber: '19034567890123',
      accountName: 'CONG TY TNHH LIVA SOLUTIONS',
      balance: 15000000000,
      balanceVnd: 15000000000,
      availableBalanceVnd: 15000000000,
      currency: 'VND',
      inflows: 800000000,
      inflowMonthVnd: 800000000,
      outflows: 2200000,
      outflowMonthVnd: 2200000,
      minReserveVnd: 500000000,
      brandColor: '#0284c7',
      status: 'ACTIVE',
      lastReconciledAt: '2026-08-31T17:00:00Z',
      clearingMechanism: 'Bù trừ Tự động ACH & Chuyển tiền nhanh 24/7 (VietQR)',
      cutoffTime: '24/7/365 (Thời gian thực)',
    },
    {
      id: 'acc_bidv_01',
      bankCode: 'BIDV',
      channelCode: 'BILATERAL',
      bankName: 'Ngân hàng TMCP Đầu tư & Phát triển VN (BIDV)',
      channelName: 'Kênh Thanh toán Song phương & Tài khoản Vostro / Nostro',
      clearingAuthority: 'Hệ thống Ngân hàng Đại lý & Định chế Tài chính Thành viên',
      accountNumber: '12410001234567',
      accountName: 'CONG TY TNHH LIVA SOLUTIONS',
      balance: 486130000,
      balanceVnd: 486130000,
      availableBalanceVnd: 486130000,
      currency: 'VND',
      inflows: 420000000,
      inflowMonthVnd: 420000000,
      outflows: 419500000,
      outflowMonthVnd: 419500000,
      minReserveVnd: 300000000,
      brandColor: '#d97706',
      status: 'ACTIVE',
      lastReconciledAt: '2026-08-31T17:00:00Z',
      clearingMechanism: 'Kết nối Kênh Thanh toán Song phương Trực tiếp',
      cutoffTime: '17:00 (Hàng ngày)',
    },
    {
      id: 'acc_swift_01',
      bankCode: 'SWIFT',
      channelCode: 'SWIFT',
      bankName: 'Hiệp hội Viễn thông Tài chính Quốc tế (SWIFT Alliance)',
      channelName: 'Kênh Chuyển tiền & Điện báo Quốc tế SWIFT',
      clearingAuthority: 'Hiệp hội Viễn thông Tài chính Liên ngân hàng Quốc tế (SWIFT)',
      accountNumber: 'SWIFT-NOSTRO-USD-EUR',
      accountName: 'CONG TY TNHH LIVA SOLUTIONS',
      balance: 420000000,
      balanceVnd: 420000000,
      availableBalanceVnd: 420000000,
      currency: 'VND',
      inflows: 350000000,
      inflowMonthVnd: 350000000,
      outflows: 180000000,
      outflowMonthVnd: 180000000,
      minReserveVnd: 200000000,
      brandColor: '#7c3aed',
      status: 'ACTIVE',
      lastReconciledAt: '2026-08-31T17:00:00Z',
      clearingMechanism: 'Điện báo ISO 20022 MX (pacs.008/009)',
      cutoffTime: '23:00 (Theo giờ chuẩn GMT)',
    },
  ],
  vouchers: [
    {
      id: 'vch_init_01',
      voucherNumber: 'VCH-2026-08-001',
      sourceAccount: '0071001234567 (VCB)',
      targetAccount: '020012345678',
      targetBeneficiary: 'CONG TY CP THIET BI MAY BOM DONG LUC',
      targetBank: 'BIDV',
      amount: 450000000,
      currency: 'VND',
      description: 'Thanh toan may bom cong nghiep theo HD-88',
      purpose: 'VẬT TƯ & THIẾT BỊ',
      status: 'PENDING_APPROVAL',
      makerId: 'usr_maker_01',
      makerName: 'Nguyễn Văn Kế Toán (Maker)',
      createdAt: new Date().toISOString(),
      approvedAt: null,
      checkerId: null,
      checkerName: null,
      merkleLeafHash: null,
    },
  ],
  merkleLedger: {
    genesisHash: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    currentRootHash: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    leaves: [],
    history: [],
  },
  reconciliationSummary: {
    totalRecords: 0,
    matchedCount: 0,
    unmatchedCount: 0,
    matchRate: 0.0,
    lastReconciledAt: null,
  },
  amlAlerts: [],
};

// Helper: flexible account resolution
function findAccount(sourceIdent) {
  if (!sourceIdent && DB.accounts.length > 0) return DB.accounts[0];
  const ident = String(sourceIdent).trim().toLowerCase();
  let acc = DB.accounts.find((a) => a.id && a.id.toLowerCase() === ident);
  if (acc) return acc;
  acc = DB.accounts.find((a) => a.accountNumber && a.accountNumber.toLowerCase() === ident);
  if (acc) return acc;
  acc = DB.accounts.find(
    (a) => (a.bankCode && a.bankCode.toLowerCase() === ident) ||
           (a.channelCode && a.channelCode.toLowerCase() === ident)
  );
  if (acc) return acc;
  acc = DB.accounts.find(
    (a) => (a.accountNumber && ident.includes(a.accountNumber.toLowerCase())) ||
           (a.bankCode && ident.includes(a.bankCode.toLowerCase())) ||
           (a.channelCode && ident.includes(a.channelCode.toLowerCase())) ||
           (a.id && ident.includes(a.id.toLowerCase()))
  );
  if (acc) return acc;
  acc = DB.accounts.find(
    (a) => (a.channelCode && ident.includes(a.channelCode.toLowerCase())) ||
           (a.bankCode && ident.includes(a.bankCode.toLowerCase()))
  );
  return acc || DB.accounts[0] || null;
}

// Helper: generate full real-time system state snapshot
function createSystemSnapshot() {
  return {
    accounts: DB.accounts,
    vouchers: DB.vouchers,
    merkleRoot: DB.merkleLedger.currentRootHash,
    merkleLedger: {
      currentRootHash: DB.merkleLedger.currentRootHash,
      leavesCount: DB.merkleLedger.leaves.length,
      historyCount: DB.merkleLedger.history.length,
    },
    reconciliationSummary: DB.reconciliationSummary,
    metrics: {
      totalLiquidityVnd: DB.accounts.reduce(
        (sum, acc) => sum + (acc.balance !== undefined ? acc.balance : acc.balanceVnd || 0),
        0
      ),
      pendingVouchersCount: DB.vouchers.filter((v) => v.status === 'PENDING_APPROVAL').length,
      approvedVouchersCount: DB.vouchers.filter((v) => v.status === 'APPROVED').length,
      reconciliationMatchRate: DB.reconciliationSummary?.matchRate ?? 0,
      activeAmlAlertsCount: DB.amlAlerts ? DB.amlAlerts.length : 0,
    },
    dailyBurnVnd: Math.round(
      DB.accounts.reduce((sum, acc) => sum + (acc.outflowMonthVnd || acc.outflows || 0), 0) / 30
    ),
    timestamp: new Date().toISOString(),
  };
}

// ==========================================
// 4. HTTP Server & Route Handler
// ==========================================
const server = http.createServer((req, res) => {
  // CORS Headers
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization');

  if (req.method === 'OPTIONS') {
    res.writeHead(204);
    res.end();
    return;
  }

  const url = new URL(req.url, `http://${req.headers.host}`);
  const pathname = url.pathname;

  // ------------------------------------------
  // Route: SSE Real-Time Sync Channel
  // ------------------------------------------
  if (req.method === 'GET' && pathname === '/api/sync/events') {
    res.writeHead(200, {
      'Content-Type': 'text/event-stream',
      'Cache-Control': 'no-cache',
      'Connection': 'keep-alive',
      'Access-Control-Allow-Origin': '*',
    });

    res.write(': connected\n\n');

    // Transmit immediate State Snapshot to newly connected client
    const snapshot = createSystemSnapshot();
    res.write(`event: state:snapshot\ndata: ${JSON.stringify(snapshot)}\n\n`);

    sseClients.add(res);

    const keepAliveTimer = setInterval(() => {
      try {
        res.write(': keepalive\n\n');
      } catch (err) {
        clearInterval(keepAliveTimer);
        sseClients.delete(res);
      }
    }, 15000);

    req.on('close', () => {
      clearInterval(keepAliveTimer);
      sseClients.delete(res);
    });
    return;
  }

  // Body Parsing Helper
  let rawBody = '';
  req.on('data', (chunk) => {
    rawBody += chunk;
  });

  req.on('end', () => {
    let body = {};
    if (rawBody) {
      try {
        body = JSON.parse(rawBody);
      } catch (e) {
        // ignore parse error
      }
    }

    // Helper Send JSON
    const sendJson = (statusCode, data) => {
      res.writeHead(statusCode, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify(data));
    };

    // Helper Extract Auth User
    const getAuthUser = () => {
      const authHeader = req.headers['authorization'];
      if (!authHeader || !authHeader.startsWith('Bearer ')) return null;
      const token = authHeader.split(' ')[1];
      return verifyJwt(token);
    };

    // ------------------------------------------
    // Route 1: Health Check
    // ------------------------------------------
    if (req.method === 'GET' && (pathname === '/api/health' || pathname === '/health')) {
      return sendJson(200, {
        status: 'OK',
        service: 'LIVA Banking Universal Centralized API Server',
        timestamp: new Date().toISOString(),
        activeUsersCount: USERS.length,
        vouchersCount: DB.vouchers.length,
        merkleRoot: DB.merkleLedger.currentRootHash,
      });
    }

    // ------------------------------------------
    // Route 2: Auth Login
    // ------------------------------------------
    if (req.method === 'POST' && pathname === '/api/auth/login') {
      const { email, password, role: preferredRole } = body;
      const inputIdent = (email || '').trim().toLowerCase();

      const user = USERS.find((u) =>
        u.email.toLowerCase() === inputIdent ||
        (u.username && u.username.toLowerCase() === inputIdent) ||
        u.officerId.toLowerCase() === inputIdent ||
        (u.aliases && u.aliases.some((a) => a.toLowerCase() === inputIdent))
      );

      if (!user) {
        return sendJson(401, {
          success: false,
          error: 'Email hoặc mật khẩu không chính xác.',
        });
      }

      const validPasswords = [user.password, ...(user.altPasswords || [])];
      if (!validPasswords.includes(password)) {
        return sendJson(401, {
          success: false,
          error: 'Email hoặc mật khẩu không chính xác.',
        });
      }

      // Enforce Segregation of Duties (SoD - Thông tư 09/2020/TT-NHNN)
      if (preferredRole) {
        const isAmlAuditorMatch =
          (preferredRole === 'AML' && user.role === 'AUDITOR') ||
          (preferredRole === 'AUDITOR' && user.role === 'AML');

        if (user.role !== preferredRole && !isAmlAuditorMatch) {
          const roleLabels = {
            MAKER: 'Kế toán (Maker)',
            CHECKER: 'Kiểm soát (Checker)',
            AML: 'Giám sát AML',
            AUDITOR: 'Kiểm toán (Auditor)',
            TREASURY: 'Quản trị Vốn (Treasury)',
          };
          const userRoleLabel = roleLabels[user.role] || user.role;
          const requestedRoleLabel = roleLabels[preferredRole] || preferredRole;

          return sendJson(403, {
            success: false,
            error: `Vi phạm Tách bạch trách nhiệm (SoD - Thông tư 09/2020/TT-NHNN): Cán bộ ${user.fullName} được định danh vai trò [${userRoleLabel}], không có thẩm quyền truy cập phân hệ [${requestedRoleLabel}].`,
          });
        }
      }

      const token = signJwt({
        sub: user.id,
        id: user.id,
        email: user.email,
        name: user.fullName,
        fullName: user.fullName,
        role: user.role,
        officerId: user.officerId,
        branchCode: user.branchCode,
        branchName: user.branchName,
        terminalId: user.terminalId,
        title: user.title,
        avatar: user.avatar,
        bankAccess: user.bankAccess,
      });

      return sendJson(200, {
        success: true,
        token,
        user: {
          id: user.id,
          email: user.email,
          fullName: user.fullName,
          name: user.fullName,
          role: user.role,
          officerId: user.officerId,
          branchCode: user.branchCode,
          branchName: user.branchName,
          terminalId: user.terminalId,
          title: user.title,
          avatar: user.avatar,
          bankAccess: user.bankAccess,
        },
      });
    }

    // ------------------------------------------
    // Route 3: Auth Me (Verify Token)
    // ------------------------------------------
    if (req.method === 'GET' && pathname === '/api/auth/me') {
      const authUser = getAuthUser();
      if (!authUser) {
        return sendJson(401, { success: false, error: 'Token không hợp lệ hoặc đã hết hạn.' });
      }
      return sendJson(200, { success: true, user: authUser });
    }

    // ------------------------------------------
    // Route 4: Get Bank Accounts
    // ------------------------------------------
    if (req.method === 'GET' && pathname === '/api/banking/accounts') {
      const authUser = getAuthUser();
      if (!authUser) {
        return sendJson(401, { success: false, error: 'Yêu cầu đăng nhập để xem tài khoản.' });
      }
      return sendJson(200, { success: true, accounts: DB.accounts });
    }

    // ------------------------------------------
    // Route 5: Get Vouchers
    // ------------------------------------------
    if (req.method === 'GET' && pathname === '/api/treasury/vouchers') {
      const authUser = getAuthUser();
      if (!authUser) {
        return sendJson(401, { success: false, error: 'Yêu cầu đăng nhập.' });
      }
      return sendJson(200, {
        success: true,
        vouchers: DB.vouchers,
        pendingCount: DB.vouchers.filter((v) => v.status === 'PENDING_APPROVAL').length,
      });
    }

    // ------------------------------------------
    // Route 6: Create Voucher (Maker / Treasury / Checker)
    // ------------------------------------------
    if (req.method === 'POST' && pathname === '/api/treasury/vouchers') {
      const authUser = getAuthUser();
      if (!authUser) return sendJson(401, { success: false, error: 'Chưa đăng nhập.' });
      if (authUser.role !== 'MAKER' && authUser.role !== 'CHECKER' && authUser.role !== 'TREASURY') {
        return sendJson(403, { success: false, error: 'Chỉ Maker/Treasury mới có quyền tạo lệnh chi.' });
      }

      const rawAmount = body.amount !== undefined ? body.amount : body.amountVnd;
      const amount = Number(rawAmount);
      if (!Number.isFinite(amount) || amount <= 0) {
        return sendJson(400, {
          success: false,
          error: 'Số tiền lệnh chi phải là số dương hợp lệ.',
        });
      }

      const voucherId = body.id || body.voucherId || `vch_${Date.now()}`;
      const newVoucher = {
        id: voucherId,
        voucherId: voucherId,
        voucherNumber: body.voucherNumber || `VCH-2026-${String(DB.vouchers.length + 1).padStart(3, '0')}`,
        sourceAccount: body.sourceAccount || '0071001234567 (VCB)',
        targetAccount: body.targetAccount || body.beneficiaryAccount || '020012345678',
        beneficiaryAccount: body.beneficiaryAccount || body.targetAccount || '020012345678',
        targetBeneficiary: body.targetBeneficiary || body.beneficiaryName || 'Đơn vị thụ hưởng',
        beneficiaryName: body.beneficiaryName || body.targetBeneficiary || 'Đơn vị thụ hưởng',
        targetBank: body.targetBank || body.beneficiaryBank || 'CITAD_SBV',
        beneficiaryBank: body.beneficiaryBank || body.targetBank || 'CITAD_SBV',
        amount: amount,
        amountVnd: amount,
        currency: 'VND',
        description: body.description || body.purpose || 'Chi thanh toán liên ngân hàng',
        purpose: body.purpose || body.description || 'Chi thanh toán liên ngân hàng',
        status: 'PENDING_APPROVAL',
        makerId: authUser.id || authUser.sub,
        makerName: authUser.name || authUser.fullName,
        createdAt: new Date().toISOString(),
        approvedAt: null,
        checkerId: null,
        checkerName: null,
        merkleLeafHash: null,
        hitlToken: body.hitlToken || null,
      };

      DB.vouchers.unshift(newVoucher);
      broadcastEvent('voucher:created', { voucher: newVoucher });

      return sendJson(201, {
        success: true,
        message: 'Lệnh chi đã được tạo và gửi lên Server chờ Checker phê duyệt.',
        voucher: newVoucher,
      });
    }

    // ------------------------------------------
    // Route 7: Approve Voucher (Checker Only)
    // ------------------------------------------
    if (req.method === 'POST' && pathname.startsWith('/api/treasury/vouchers/') && pathname.endsWith('/approve')) {
      const authUser = getAuthUser();
      if (!authUser) return sendJson(401, { success: false, error: 'Chưa đăng nhập.' });
      if (authUser.role !== 'CHECKER') {
        return sendJson(403, {
          success: false,
          error: 'Quy tắc Maker-Checker (TT 09/2020): Chỉ Giám đốc (Checker) mới có quyền phê duyệt.',
        });
      }

      const parts = pathname.split('/');
      const voucherId = parts[parts.length - 2];
      const voucher = DB.vouchers.find((v) => v.id === voucherId || v.voucherId === voucherId);

      if (!voucher) {
        return sendJson(404, { success: false, error: 'Không tìm thấy lệnh chi.' });
      }

      const currentUserId = authUser.id || authUser.sub;
      if (voucher.makerId === currentUserId) {
        return sendJson(403, {
          success: false,
          error: 'Kiểm soát viên không được tự phê duyệt lệnh chi do chính mình tạo lập (Thông tư 09/2020/TT-NHNN).',
        });
      }

      if (voucher.status !== 'PENDING_APPROVAL') {
        return sendJson(400, { success: false, error: 'Lệnh chi không ở trạng thái chờ phê duyệt.' });
      }

      // Solvency check & account deduction (Circular 09/2020 & Accounting Invariant)
      const voucherAmount = Math.round(Number(voucher.amount !== undefined ? voucher.amount : voucher.amountVnd || 0));
      if (!Number.isFinite(voucherAmount) || voucherAmount <= 0) {
        return sendJson(400, { success: false, error: 'Số tiền lệnh chi không hợp lệ.' });
      }

      const sourceAccount = findAccount(voucher.sourceAccount);
      if (!sourceAccount) {
        return sendJson(400, { success: false, error: 'Không tìm thấy tài khoản nguồn trích nợ.' });
      }

      const currentBalance = Math.round(Number(sourceAccount.balance !== undefined ? sourceAccount.balance : sourceAccount.balanceVnd || 0));
      if (currentBalance < voucherAmount) {
        return sendJson(400, {
          success: false,
          error: `Số dư khả dụng không đủ để phê duyệt lệnh chi. Số dư hiện tại: ${currentBalance.toLocaleString('vi-VN')} VND, Số tiền lệnh: ${voucherAmount.toLocaleString('vi-VN')} VND.`,
        });
      }

      // Deduct balance and update outflows
      const newBalance = currentBalance - voucherAmount;
      sourceAccount.balance = newBalance;
      sourceAccount.balanceVnd = newBalance;
      if (sourceAccount.availableBalanceVnd !== undefined) {
        sourceAccount.availableBalanceVnd = newBalance;
      }
      sourceAccount.outflows = (sourceAccount.outflows || 0) + voucherAmount;
      if (sourceAccount.outflowMonthVnd !== undefined) {
        sourceAccount.outflowMonthVnd = (sourceAccount.outflowMonthVnd || 0) + voucherAmount;
      }

      // Compute Cryptographic Merkle Leaf Hash
      const approvedTime = new Date().toISOString();
      const leafPayload = `${voucher.voucherId || voucher.id}|${voucher.makerId || 'usr_maker_01'}|${authUser.id || authUser.sub}|${voucherAmount}|APPROVED|${approvedTime}`;
      const leafHash = crypto.createHash('sha256').update(leafPayload).digest('hex');

      // Update Voucher State
      voucher.status = 'APPROVED';
      voucher.approvedAt = approvedTime;
      voucher.checkerId = authUser.id || authUser.sub;
      voucher.checkerName = authUser.name || authUser.fullName;
      voucher.merkleLeafHash = leafHash;

      // Update Merkle Ledger
      DB.merkleLedger.leaves.push(leafHash);
      const combinedHash = crypto
        .createHash('sha256')
        .update(DB.merkleLedger.currentRootHash + leafHash)
        .digest('hex');
      DB.merkleLedger.currentRootHash = combinedHash;
      DB.merkleLedger.history.push({
        voucherId: voucher.id || voucher.voucherId,
        leafHash,
        rootHash: combinedHash,
        timestamp: voucher.approvedAt,
        approvedBy: voucher.checkerName,
        sourceAccount: sourceAccount.accountNumber || sourceAccount.id,
        deductedAmount: voucherAmount,
        remainingBalance: newBalance,
      });

      // Dual SSE broadcast: voucher:approved and account:updated
      broadcastEvent('voucher:approved', {
        voucher,
        voucherId: voucher.voucherId || voucher.id,
        checkerId: voucher.checkerId,
        merkleHash: leafHash,
        merkleRoot: combinedHash,
      });

      broadcastEvent('account:updated', {
        accountId: sourceAccount.id,
        bankCode: sourceAccount.bankCode,
        channelCode: sourceAccount.channelCode,
        accountNumber: sourceAccount.accountNumber,
        balance: newBalance,
        balanceVnd: newBalance,
        availableBalanceVnd: newBalance,
        outflows: sourceAccount.outflows,
        outflowMonthVnd: sourceAccount.outflowMonthVnd,
        deductedAmount: voucherAmount,
        voucherId: voucher.voucherId || voucher.id,
        lastUpdated: voucher.approvedAt,
      });

      return sendJson(200, {
        success: true,
        message: 'Lệnh chi đã được phê duyệt và ghi nhận vào Merkle Audit Ledger bất biến.',
        voucher,
        merkleRoot: combinedHash,
        sourceAccount: {
          id: sourceAccount.id,
          bankCode: sourceAccount.bankCode,
          channelCode: sourceAccount.channelCode,
          accountNumber: sourceAccount.accountNumber,
          balance: newBalance,
          balanceVnd: newBalance,
        },
      });
    }

    // ------------------------------------------
    // Route 8: Reject Voucher (Checker Only)
    // ------------------------------------------
    if (req.method === 'POST' && pathname.startsWith('/api/treasury/vouchers/') && pathname.endsWith('/reject')) {
      const authUser = getAuthUser();
      if (!authUser) return sendJson(401, { success: false, error: 'Chưa đăng nhập.' });
      if (authUser.role !== 'CHECKER') {
        return sendJson(403, { success: false, error: 'Chỉ Checker mới có quyền từ chối lệnh chi.' });
      }

      const parts = pathname.split('/');
      const voucherId = parts[parts.length - 2];
      const voucher = DB.vouchers.find((v) => v.id === voucherId || v.voucherId === voucherId);

      if (!voucher) return sendJson(404, { success: false, error: 'Không tìm thấy lệnh chi.' });

      if (voucher.status !== 'PENDING_APPROVAL') {
        return sendJson(400, { success: false, error: 'Lệnh chi không ở trạng thái chờ phê duyệt.' });
      }

      voucher.status = 'REJECTED';
      voucher.approvedAt = new Date().toISOString();
      voucher.checkerId = authUser.id || authUser.sub;
      voucher.checkerName = authUser.name || authUser.fullName;
      voucher.rejectionReason = body.reason || body.remarks || 'Không đạt điều kiện thẩm định thanh toán.';

      broadcastEvent('voucher:rejected', {
        voucher,
        voucherId: voucher.voucherId || voucher.id,
        checkerId: voucher.checkerId,
        remarks: voucher.rejectionReason,
        reason: voucher.rejectionReason,
      });

      return sendJson(200, {
        success: true,
        message: 'Lệnh chi đã bị từ chối.',
        voucher,
      });
    }

    // ------------------------------------------
    // Route 9: Merkle Audit Ledger
    // ------------------------------------------
    if (req.method === 'GET' && pathname === '/api/treasury/merkle-ledger') {
      return sendJson(200, {
        success: true,
        ledger: DB.merkleLedger,
      });
    }

    // ------------------------------------------
    // Route 10: Live Reconciliation Summary
    // ------------------------------------------
    if (req.method === 'GET' && pathname === '/api/reconciliation/summary') {
      return sendJson(200, { success: true, summary: DB.reconciliationSummary });
    }

    if (req.method === 'POST' && pathname === '/api/reconciliation/summary') {
      const authUser = getAuthUser();
      if (!authUser) return sendJson(401, { success: false, error: 'Chưa đăng nhập.' });

      const { totalRecords = 0, matchedCount = 0, unmatchedCount = 0 } = body;
      const matchRate = totalRecords > 0 ? Math.round((matchedCount / totalRecords) * 10000) / 100 : 0;

      DB.reconciliationSummary = {
        totalRecords: Number(totalRecords),
        matchedCount: Number(matchedCount),
        unmatchedCount: Number(unmatchedCount),
        matchRate,
        lastReconciledAt: new Date().toISOString(),
      };

      broadcastEvent('reconciliation:summary', DB.reconciliationSummary);

      return sendJson(200, { success: true, summary: DB.reconciliationSummary });
    }

    // ------------------------------------------
    // Route 11: Live AML Alerts
    // ------------------------------------------
    if (req.method === 'GET' && pathname === '/api/aml/alerts') {
      return sendJson(200, { success: true, alerts: DB.amlAlerts });
    }

    if (req.method === 'POST' && pathname === '/api/aml/alerts') {
      const authUser = getAuthUser();
      if (!authUser) return sendJson(401, { success: false, error: 'Chưa đăng nhập.' });

      const alertId = body.alertId || body.id || `alt_${Date.now()}`;
      const newAlert = {
        alertId,
        id: alertId,
        ruleId: body.ruleId || 'STR-CIRCULAR-09',
        severity: body.severity || 'HIGH',
        triggerType: body.triggerType || body.type || 'SUSPICIOUS_TRANSACTION',
        message: body.message || body.description || 'Cảnh báo giao dịch đáng ngờ',
        details: body.details || {},
        status: 'OPEN',
        createdAt: new Date().toISOString(),
      };

      DB.amlAlerts.unshift(newAlert);
      broadcastEvent('aml:alert', newAlert);

      return sendJson(201, { success: true, alert: newAlert });
    }

    // ------------------------------------------
    // Route 12: SQL Database Statistics
    // ------------------------------------------
    if (req.method === 'GET' && pathname === '/api/database/stats') {
      if (sqliteDb) {
        const totalStmts = sqliteDb.prepare('SELECT count(*) as c FROM bank_statements').get()?.c || 0;
        const totalInvs = sqliteDb.prepare('SELECT count(*) as c FROM invoices').get()?.c || 0;
        const totalVouchers = sqliteDb.prepare('SELECT count(*) as c FROM payment_vouchers').get()?.c || 0;
        const totalAlerts = sqliteDb.prepare('SELECT count(*) as c FROM aml_alerts').get()?.c || 0;
        const totalLiquidity = sqliteDb.prepare('SELECT sum(balance_vnd) as s FROM accounts').get()?.s || 0;
        return sendJson(200, {
          success: true,
          mode: 'SQLITE_PERSISTENT',
          databasePath: SQLITE_DB_PATH,
          stats: {
            accountsCount: 5,
            statementsCount: totalStmts,
            invoicesCount: totalInvs,
            vouchersCount: totalVouchers,
            amlAlertsCount: totalAlerts,
            totalLiquidityVnd: totalLiquidity,
          },
        });
      }
      return sendJson(200, {
        success: true,
        mode: 'IN_MEMORY',
        stats: {
          accountsCount: DB.accounts.length,
          statementsCount: 0,
          invoicesCount: 0,
          vouchersCount: DB.vouchers.length,
          amlAlertsCount: DB.amlAlerts.length,
          totalLiquidityVnd: DB.accounts.reduce((acc, a) => acc + (a.balance || 0), 0),
        },
      });
    }

    // ------------------------------------------
    // Route 13: SQL Statements Query
    // ------------------------------------------
    if (req.method === 'GET' && pathname === '/api/database/statements') {
      if (sqliteDb) {
        const limit = Number(url.searchParams?.get('limit') || 200);
        const bank = url.searchParams?.get('bank');
        let query = 'SELECT * FROM bank_statements';
        const params = [];
        if (bank && bank !== 'ALL') {
          query += ' WHERE bank_code = ?';
          params.push(bank);
        }
        query += ' ORDER BY booking_date DESC LIMIT ?';
        params.push(limit);
        const rows = sqliteDb.prepare(query).all(...params);
        return sendJson(200, { success: true, count: rows.length, statements: rows });
      }
      return sendJson(200, { success: true, count: 0, statements: [] });
    }

    // ------------------------------------------
    // Route 14: SQL Invoices Query
    // ------------------------------------------
    if (req.method === 'GET' && pathname === '/api/database/invoices') {
      if (sqliteDb) {
        const limit = Number(url.searchParams?.get('limit') || 100);
        const rows = sqliteDb.prepare('SELECT * FROM invoices LIMIT ?').all(limit);
        return sendJson(200, { success: true, count: rows.length, invoices: rows });
      }
      return sendJson(200, { success: true, count: 0, invoices: [] });
    }

    // ------------------------------------------
    // Route 15: SQL Accounts Query
    // ------------------------------------------
    if (req.method === 'GET' && pathname === '/api/database/accounts') {
      if (sqliteDb) {
        const rows = sqliteDb.prepare('SELECT * FROM accounts').all();
        return sendJson(200, { success: true, count: rows.length, accounts: rows });
      }
      return sendJson(200, { success: true, count: DB.accounts.length, accounts: DB.accounts });
    }

    // ------------------------------------------
    // Route 16: SQL Dump Download
    // ------------------------------------------
    if (req.method === 'GET' && pathname === '/api/database/sql-dump') {
      if (fs.existsSync(SEED_SQL_PATH)) {
        const sqlContent = fs.readFileSync(SEED_SQL_PATH, 'utf-8');
        res.writeHead(200, {
          'Content-Type': 'text/plain; charset=utf-8',
          'Content-Disposition': 'attachment; filename="liva_banking_seed.sql"',
        });
        return res.end(sqlContent);
      }
      return sendJson(404, { success: false, error: 'Chưa khởi tạo tệp seed SQL.' });
    }

    // ------------------------------------------
    // 404 Not Found
    // ------------------------------------------
    return sendJson(404, { error: `Endpoint ${pathname} không tồn tại.` });
  });
});

// Safe server close wrapper to terminate active SSE streams and prevent hanging test runners
const origServerClose = server.close.bind(server);
server.close = function (cb) {
  for (const client of sseClients) {
    try {
      client.end();
      client.destroy();
    } catch {}
  }
  sseClients.clear();
  if (typeof server.closeAllConnections === 'function') {
    server.closeAllConnections();
  }
  return origServerClose(cb);
};

// Direct execution check to prevent port conflicts in test runners
const isDirectRun = Boolean(
  process.argv[1] && (
    fileURLToPath(import.meta.url) === path.resolve(process.argv[1]) ||
    process.argv[1].endsWith('server.mjs')
  )
);

if (isDirectRun || process.env.RUN_SERVER === 'true') {
  server.listen(PORT, () => {
    console.log(`[LIVA Backend Server] Online at http://localhost:${PORT}`);
    console.log(`[LIVA Backend Server] JWT Auth ready. Presets: maker@livabanking.vn, checker@livabanking.vn, aml@livabanking.vn, treasury@livabanking.vn`);
    console.log(`[LIVA Backend Server] SSE Real-time Sync active at /api/sync/events`);
  });
}

export {
  server,
  USERS,
  DB,
  signJwt,
  verifyJwt,
  broadcastEvent,
  sseClients,
  PORT,
  base64UrlEncode,
  base64UrlDecode,
  JWT_SECRET,
  isEphemeralJwtSecret,
};
export default server;

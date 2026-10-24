import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import fs from 'node:fs';
import path from 'node:path';
import { useReconciliationStore } from '../src/stores/reconciliationStore';
import { AML_SANDBOX_PRESETS } from '../src/stores/amlStore';
import sqlDatabaseSeed from '../src/data/sqlDatabaseSeed.json';

describe('SQL Database Seeding & Ingestion Suite (Zero Hardcoding)', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('verifies SQLite file, schema, and ANSI SQL dump artifacts exist and are non-empty', () => {
    const rootDir = path.resolve(__dirname, '../../..');
    const dbPath = path.join(rootDir, 'data', 'liva_banking.db');
    const schemaPath = path.join(rootDir, 'data', 'schema.sql');
    const seedSqlPath = path.join(rootDir, 'data', 'liva_banking_seed.sql');

    expect(fs.existsSync(dbPath)).toBe(true);
    expect(fs.existsSync(schemaPath)).toBe(true);
    expect(fs.existsSync(seedSqlPath)).toBe(true);

    const dbStats = fs.statSync(dbPath);
    const schemaStats = fs.statSync(schemaPath);
    const seedSqlStats = fs.statSync(seedSqlPath);

    expect(dbStats.size).toBeGreaterThan(50_000); // SQLite DB > 50KB
    expect(schemaStats.size).toBeGreaterThan(1_000); // Schema SQL
    expect(seedSqlStats.size).toBeGreaterThan(100_000); // SQL dump > 100KB
  });

  it('validates sqlDatabaseSeed structure and table counts', () => {
    expect(sqlDatabaseSeed.accounts.length).toBe(5);
    expect(sqlDatabaseSeed.bankStatements.length).toBe(160);
    expect(sqlDatabaseSeed.invoices.length).toBe(60);
    expect(sqlDatabaseSeed.paymentVouchers.length).toBe(20);
    expect(sqlDatabaseSeed.amlAlerts.length).toBe(10);
    expect(sqlDatabaseSeed.stats.totalLiquidityVnd).toBeGreaterThan(100_000_000_000);
  });

  it('proves zero float drift: exact mathematical balance invariant across all 5 accounts', () => {
    const totalOpening = sqlDatabaseSeed.accounts.reduce(
      (sum, a) => sum + Number(a.opening_balance_vnd),
      0
    );
    const totalCredit = sqlDatabaseSeed.bankStatements.reduce(
      (sum, s) => sum + Number(s.credit_amount),
      0
    );
    const totalDebit = sqlDatabaseSeed.bankStatements.reduce(
      (sum, s) => sum + Number(s.debit_amount),
      0
    );
    const totalClosing = sqlDatabaseSeed.accounts.reduce(
      (sum, a) => sum + Number(a.balance_vnd),
      0
    );

    expect(totalOpening).toBe(80_000_000_000);
    expect(totalCredit).toBeGreaterThan(0);
    expect(totalDebit).toBeGreaterThan(0);
    expect(totalClosing).toBe(sqlDatabaseSeed.stats.totalLiquidityVnd);

    // Exact balance equation: Closing = Opening + Credit - Debit
    const calculatedClosing = totalOpening + totalCredit - totalDebit;
    expect(calculatedClosing).toBe(totalClosing);
    expect(totalClosing - calculatedClosing).toBe(0);
  });

  it('loads SQL database dataset into reconciliationStore and verifies full 3-tier matching', () => {
    const store = useReconciliationStore();
    store.loadFromSqlDatabase();

    expect(store.bankTransactions.length).toBe(160);
    expect(store.ledgerEntries.length).toBe(60);
    expect(store.activeDatasetName).toContain('Cơ Sở Dữ Liệu SQL Ngân Hàng');
    expect(store.balanceInvariantReport?.isBalanced).toBe(true);
    expect(store.balanceInvariantReport?.discrepancy).toBe(0);

    // Summary checks
    expect(store.reconciliationSummary).not.toBeNull();
    const summary = store.reconciliationSummary!;
    expect(summary.totalBankTransactions).toBe(160);
    expect(summary.totalLedgerEntries).toBe(60);
    expect(summary.tier1Count).toBeGreaterThanOrEqual(20);
    expect(summary.tier2Count).toBeGreaterThanOrEqual(1);
    expect(summary.tier3Count).toBeGreaterThanOrEqual(1);
    expect(summary.hitlQuarantineCount).toBeGreaterThanOrEqual(1);
    expect(summary.totalMatchedAmount).toBeGreaterThan(10_000_000_000);
  });

  it('verifies amlStore has SQL_DATABASE preset with diverse transactions', () => {
    const sqlPreset = AML_SANDBOX_PRESETS.find((p) => p.id === 'SQL_DATABASE');
    expect(sqlPreset).toBeDefined();
    expect(sqlPreset!.transactions.length).toBeGreaterThan(0);
    expect(sqlPreset!.name).toContain('Cơ Sở Dữ Liệu SQL');
  });
});

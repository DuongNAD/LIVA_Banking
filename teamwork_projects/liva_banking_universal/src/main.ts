/**
 * LIVA Banking Universal Entry Point
 * Milestone 1: Universal Statement Ingestion & 3-Tier Reconciliation Engine
 * Milestone 2: Local AI Intelligence, AML/STR Surveillance & System Prompt Inspector
 * Milestone 3: Treasury Maker-Checker Dual Control & Interactive Demo Presentation
 * Milestone 4: Standalone Web Harness Scaffolding & Zero-Backend Integration (F21)
 */

import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';

// Master App & Views
export { default as App } from './App.vue';
export { default as BankingDashboardView } from './views/BankingDashboardView.vue';
export { default as ReconciliationWorkbenchView } from './views/ReconciliationWorkbenchView.vue';
export { default as ComplianceAmlView } from './views/ComplianceAmlView.vue';
export { default as TreasuryPaymentView } from './views/TreasuryPaymentView.vue';

// Milestone 1: Ingestion & Reconciliation
export { parseExcelFile, parseCsvOrTsv, parsePastedTable, parseLedgerCsv } from './engine/ingestion/universalParser';
export { runReconciliationEngine } from './engine/reconciliation/reconciliationEngine';
export { verifyBalanceInvariants } from './engine/reconciliation/balanceValidator';

// Milestone 2: Intelligence & NLP
export {
  normalizeVietnameseIntent,
  removeVietnameseAccents,
  matchRemarksToLedger,
  VIETNAMESE_BANKING_ABBREVIATIONS,
} from './engine/intelligence/vietnameseNlp';

// Milestone 2: Wire Fee Disentanglement
export {
  disentangleWireFee,
  checkWireFeeDiscrepancy,
  KNOWN_VIETNAMESE_WIRE_FEES,
} from './engine/intelligence/feeExtractor';

// Milestone 2: AML & STR Surveillance
export {
  detectAmlHighValue,
  detectAmlStructuring,
  detectAmlNightVelocity,
  detectAmlRapidPassThrough,
  detectAmlWatchlistKeywords,
  runFullAmlSurveillance,
  inspectSystemPrompt,
} from './engine/intelligence/amlSurveillance';

// Milestone 2: Statutory Form STR
export {
  generateFormStr,
  formatOfficialStrDocument,
} from './engine/intelligence/strGenerator';

// Milestone 3: Treasury Maker-Checker Dual Control (F17)
export {
  createPaymentVoucher,
  submitVoucherForApproval,
  approveVoucher,
  rejectVoucher,
  settleVoucher,
  checkVoucherExpiration,
  generateUuidV4,
  resetConsumedTokens,
} from './engine/treasury/makerChecker';

// Milestone 3: Cryptographic Merkle Audit & Forward Hash Ledger (F18)
export {
  computeMerkleLeaf,
  buildMerkleTree,
  generateMerkleProof,
  verifyMerkleProof,
  sha256,
  hmacSha256,
  ForwardAuditLedger,
  GENESIS_LEDGER_HASH,
} from './engine/treasury/merkleAudit';

// Milestone 3: Financial Copilot & Guided Presentation Tour (F19 & F20)
export {
  queryFinancialCopilot,
  createGuidedTourState,
  STANDARDIZED_TOUR_STEPS,
} from './engine/treasury/copilotEngine';

// Stores
export { useAmlStore, AML_SANDBOX_PRESETS } from './stores/amlStore';
export { useTreasuryStore } from './stores/treasuryStore';
export { useBankingStore } from './stores/bankingStore';
export { useReconciliationStore } from './stores/reconciliationStore';

// UI Components
export { default as MakerCheckerModal } from './components/treasury/MakerCheckerModal.vue';
export { default as MerkleProofCard } from './components/treasury/MerkleProofCard.vue';
export { default as FinancialCopilotDrawer } from './components/copilot/FinancialCopilotDrawer.vue';
export { default as GuidedTourOverlay } from './components/copilot/GuidedTourOverlay.vue';
export { default as AmlAlertList } from './components/aml/AmlAlertList.vue';
export { default as StrReportModal } from './components/aml/StrReportModal.vue';
export { default as SystemPromptInspector } from './components/aml/SystemPromptInspector.vue';

// Types
export * from './types/banking';
export * from './types/reconciliation';
export * from './types/aml';
export * from './types/treasury';

// Browser SPA Mounting
if (typeof document !== 'undefined') {
  const rootElement = document.getElementById('app');
  if (rootElement) {
    const pinia = createPinia();
    const app = createApp(App);
    app.use(pinia);
    app.mount(rootElement);
  }
}

console.log('LIVA Banking Universal Ingestion, Reconciliation, AML Intelligence & Treasury Maker-Checker Engine Initialized.');

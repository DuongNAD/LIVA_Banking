//! Scaled property-based fuzzing suite for `liva-compliance`.
//!
//! Tests invariants across arbitrary inputs:
//! 1. Jaro-Winkler bounded output in [0, 10_000] bps.
//! 2. RFC 6962 Binary Merkle Tree inclusion proofs hold for arbitrary leaf counts.
//! 3. 4-Tier threshold matrix completeness (no panics, valid classification for all u64).

use liva_compliance::aml::integer_jaro_winkler;
use liva_compliance::audit::BinaryMerkleTree;
use liva_compliance::maker_checker::{ApprovalMode, MakerCheckerEngine};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn prop_integer_jaro_winkler_bounded(s1 in "\\PC{0,30}", s2 in "\\PC{0,30}") {
        let score = integer_jaro_winkler(&s1, &s2);
        prop_assert!(score <= 10_000, "Score {} exceeds 10,000 bps", score);
        if s1 == s2 && !s1.is_empty() {
            prop_assert_eq!(score, 10_000);
        }
    }

    #[test]
    fn prop_merkle_tree_arbitrary_leaves(leaf_count in 1usize..32usize) {
        let leaves: Vec<Vec<u8>> = (0..leaf_count)
            .map(|i| format!("PROPTEST-LEAF-{}", i).into_bytes())
            .collect();

        let tree = BinaryMerkleTree::from_raw_leaves(&leaves);
        prop_assert_eq!(tree.total_leaves(), leaf_count);

        for i in 0..leaf_count {
            let proof = tree.generate_inclusion_proof(i).unwrap();
            prop_assert!(proof.verify(&leaves[i]), "Proof failed for leaf {}", i);
            let forged = format!("FORGED-{}", i).into_bytes();
            prop_assert!(!proof.verify(&forged), "Forged proof succeeded unexpectedly");
        }
    }

    #[test]
    fn prop_tier_classification_all_u64(amount in any::<u64>()) {
        let engine = MakerCheckerEngine::with_default_policy();
        let tier = engine.classify_tier(amount);
        prop_assert!(tier.is_ok(), "Amount {} failed to classify into a tier", amount);

        let t = tier.unwrap();
        match t.tier_level {
            1 => {
                prop_assert!(amount < 10_000_000);
                prop_assert_eq!(t.approval_mode, ApprovalMode::Auto);
            }
            2 => {
                prop_assert!(amount >= 10_000_000 && amount < 100_000_000);
                prop_assert_eq!(t.approval_mode, ApprovalMode::SingleChecker);
            }
            3 => {
                prop_assert!(amount >= 100_000_000 && amount < 500_000_000);
                prop_assert_eq!(t.approval_mode, ApprovalMode::SeniorChecker);
            }
            4 => {
                prop_assert!(amount >= 500_000_000);
                prop_assert_eq!(t.approval_mode, ApprovalMode::DualChecker);
            }
            other => panic!("Unexpected tier level {}", other),
        }
    }
}

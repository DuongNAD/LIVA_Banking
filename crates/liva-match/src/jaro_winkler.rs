#![allow(clippy::float_arithmetic)]

//! Deterministic Jaro-Winkler String Similarity Metric.
//!
//! Includes Vietnamese diacritics normalization for party names and token set overlap.

use std::collections::HashSet;
use crate::text_cleaner::{
    normalize_vietnamese_text, strip_bank_narration_noise, strip_corporate_legal_noise,
};

/// Calculates the Jaro similarity between two string slices.
pub fn jaro_similarity(s1: &str, s2: &str) -> f64 {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let len1 = s1_chars.len();
    let len2 = s2_chars.len();

    if len1 == 0 && len2 == 0 {
        return 1.0;
    }
    if len1 == 0 || len2 == 0 {
        return 0.0;
    }

    let match_distance = (len1.max(len2) / 2).saturating_sub(1);

    let mut s1_matches = vec![false; len1];
    let mut s2_matches = vec![false; len2];

    let mut matches: usize = 0;

    for i in 0..len1 {
        let start = i.saturating_sub(match_distance);
        let end = (i + match_distance + 1).min(len2);

        for j in start..end {
            if s2_matches[j] {
                continue;
            }
            if s1_chars[i] == s2_chars[j] {
                s1_matches[i] = true;
                s2_matches[j] = true;
                matches += 1;
                break;
            }
        }
    }

    if matches == 0 {
        return 0.0;
    }

    let mut transpositions: usize = 0;
    let mut k = 0;
    for i in 0..len1 {
        if !s1_matches[i] {
            continue;
        }
        while !s2_matches[k] {
            k += 1;
        }
        if s1_chars[i] != s2_chars[k] {
            transpositions += 1;
        }
        k += 1;
    }

    let m = matches as f64;
    let t = (transpositions / 2) as f64;

    ((m / len1 as f64) + (m / len2 as f64) + ((m - t) / m)) / 3.0
}

/// Calculates the Jaro-Winkler similarity score (between 0.0 and 1.0).
/// Uses prefix scale p = 0.1, up to max 4 matching prefix characters.
pub fn jaro_winkler(s1: &str, s2: &str) -> f64 {
    let jaro = jaro_similarity(s1, s2);
    if jaro < 0.7 {
        return jaro;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let prefix_limit = 4.min(s1_chars.len().min(s2_chars.len()));
    let mut prefix_len = 0;

    for i in 0..prefix_limit {
        if s1_chars[i] == s2_chars[i] {
            prefix_len += 1;
        } else {
            break;
        }
    }

    let p = 0.1;
    jaro + (prefix_len as f64 * p * (1.0 - jaro))
}

/// Compares two Vietnamese party names using Jaro-Winkler and token set containment
/// after stripping bank narration noise and corporate legal noise.
pub fn compare_party_names(name1: &str, name2: &str) -> f64 {
    let clean1 = strip_bank_narration_noise(name1);
    let clean2 = strip_bank_narration_noise(name2);
    let n1 = normalize_vietnamese_text(&clean1);
    let n2 = normalize_vietnamese_text(&clean2);

    if n1 == n2 {
        return 1.0;
    }
    if n1.is_empty() || n2.is_empty() {
        return 0.0;
    }

    // Extract core distinctive business tokens by stripping corporate legal noise
    let (tokens1, core1) = strip_corporate_legal_noise(&n1);
    let (tokens2, core2) = strip_corporate_legal_noise(&n2);

    // Direct match on core distinctive names
    if core1 == core2 {
        return 1.0;
    }

    // Token set overlap on core tokens
    let set1: HashSet<&str> = tokens1.iter().map(|s| s.as_str()).collect();
    let set2: HashSet<&str> = tokens2.iter().map(|s| s.as_str()).collect();
    let intersection = set1.intersection(&set2).count();
    let min_len = set1.len().min(set2.len());
    let max_len = set1.len().max(set2.len());

    let token_containment = if min_len > 0 {
        intersection as f64 / min_len as f64
    } else {
        0.0
    };

    // If all core tokens of the shorter name are present in the longer name
    if min_len > 0 && intersection == min_len {
        let ratio = min_len as f64 / max_len as f64;
        return (0.88 + 0.12 * ratio).min(1.0);
    }

    // Core string containment (e.g. "abc" inside "abc group" or narration)
    if !core1.is_empty() && !core2.is_empty() {
        let min_char_len = core1.len().min(core2.len());
        let max_char_len = core1.len().max(core2.len());
        if min_char_len >= 3 && (core1.contains(&core2) || core2.contains(&core1)) {
            let ratio = min_char_len as f64 / max_char_len as f64;
            return (0.88 + 0.12 * ratio).min(1.0);
        }
    }

    if token_containment >= 0.80 {
        return (0.88 + 0.12 * token_containment).min(1.0);
    }

    // Jaro-Winkler similarity computed on stripped core tokens
    let core_jw = jaro_winkler(&core1, &core2);

    // If stripped core tokens have zero token overlap:
    // Enforce low party similarity (< 0.70) so FuzzyMatcher will NOT auto-approve.
    if intersection == 0 {
        return core_jw.min(0.65);
    }

    let has_distinctive_token = tokens1
        .iter()
        .any(|t| t.len() >= 4 && set2.contains(t.as_str()));

    let mut score = core_jw.max(token_containment * 0.85);
    if has_distinctive_token && score < 0.75 {
        score = score.max(0.70 + 0.15 * token_containment);
    }
    score
}

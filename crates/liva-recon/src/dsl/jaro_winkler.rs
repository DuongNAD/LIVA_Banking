//! Pure Integer Jaro-Winkler Distance Implementation
//!
//! Computes string similarity in basis points ($0 \dots 10,000$ bps, where $10,000 \equiv 100.00\%$)
//! using micro-precision integer arithmetic ($1,000,000$ scale).
//!
//! Strictly adheres to `#![deny(clippy::float_arithmetic)]` with ZERO floating-point types (`f32`/`f64`).

/// Calculates pure integer Jaro similarity in basis points ($0 \dots 10,000$).
pub fn integer_jaro(s1: &str, s2: &str) -> u32 {
    let chars1: Vec<char> = s1.chars().collect();
    let chars2: Vec<char> = s2.chars().collect();

    let len1 = chars1.len();
    let len2 = chars2.len();

    if len1 == 0 && len2 == 0 {
        return 10_000;
    }
    if len1 == 0 || len2 == 0 {
        return 0;
    }
    if chars1 == chars2 {
        return 10_000;
    }

    let match_window = (len1.max(len2) / 2).saturating_sub(1);

    let mut s1_matches = vec![false; len1];
    let mut s2_matches = vec![false; len2];
    let mut matches = 0usize;

    for i in 0..len1 {
        let start = i.saturating_sub(match_window);
        let end = (i + match_window + 1).min(len2);

        for j in start..end {
            if !s2_matches[j] && chars1[i] == chars2[j] {
                s1_matches[i] = true;
                s2_matches[j] = true;
                matches += 1;
                break;
            }
        }
    }

    if matches == 0 {
        return 0;
    }

    // Identify transpositions
    let mut s1_matched_chars = Vec::with_capacity(matches);
    for (i, &matched) in s1_matches.iter().enumerate() {
        if matched {
            s1_matched_chars.push(chars1[i]);
        }
    }

    let mut s2_matched_chars = Vec::with_capacity(matches);
    for (j, &matched) in s2_matches.iter().enumerate() {
        if matched {
            s2_matched_chars.push(chars2[j]);
        }
    }

    let mut half_transpositions = 0usize;
    for k in 0..matches {
        if s1_matched_chars[k] != s2_matched_chars[k] {
            half_transpositions += 1;
        }
    }
    let transpositions = half_transpositions / 2;

    // Micro-precision integer scaling (scale = 1_000_000)
    // term1 = m / |s1|
    // term2 = m / |s2|
    // term3 = (m - t) / m
    let m_u64 = matches as u64;
    let term1 = (m_u64 * 1_000_000) / (len1 as u64);
    let term2 = (m_u64 * 1_000_000) / (len2 as u64);
    let term3 = ((matches.saturating_sub(transpositions)) as u64 * 1_000_000) / m_u64;

    let avg_micros = (term1 + term2 + term3) / 3;

    // Convert micros (0..1_000_000) to basis points (0..10_000) with half-up rounding
    ((avg_micros + 50) / 100).min(10_000) as u32
}

/// Calculates pure integer Jaro-Winkler similarity in basis points ($0 \dots 10,000$).
///
/// If Jaro similarity is $\ge 7,000$ bps (0.70), applies a prefix bonus of up to 4 initial characters:
/// $$\Delta_{\text{bps}} = \frac{L \times (10,000 - J_{\text{bps}}) + 5}{10}$$
/// where $L = \min(4, \text{common\_prefix})$.
pub fn integer_jaro_winkler(s1: &str, s2: &str) -> u32 {
    let chars1: Vec<char> = s1.chars().collect();
    let chars2: Vec<char> = s2.chars().collect();

    let len1 = chars1.len();
    let len2 = chars2.len();

    if len1 == 0 && len2 == 0 {
        return 10_000;
    }
    if len1 == 0 || len2 == 0 {
        return 0;
    }
    if chars1 == chars2 {
        return 10_000;
    }

    let j_bps = integer_jaro(s1, s2);

    // Common prefix up to 4 characters
    let mut prefix_len = 0usize;
    for (c1, c2) in chars1.iter().zip(chars2.iter()).take(4) {
        if c1 == c2 {
            prefix_len += 1;
        } else {
            break;
        }
    }

    if j_bps >= 7_000 && prefix_len > 0 {
        // Boost factor p = 0.1 (in basis points: L * 0.1 * (10_000 - j_bps))
        let delta_bps = (prefix_len as u32 * (10_000 - j_bps) + 5) / 10;
        (j_bps + delta_bps).min(10_000)
    } else {
        j_bps
    }
}

/// Pure integer square root for u128.
pub fn integer_sqrt_u128(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    if n <= 3 {
        return 1;
    }
    let mut x0 = n / 2;
    let mut x1 = (x0 + n / x0) / 2;
    while x1 < x0 {
        x0 = x1;
        x1 = (x0 + n / x0) / 2;
    }
    x0
}

/// Pure integer square root for u64.
pub fn integer_sqrt_u64(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    if n <= 3 {
        return 1;
    }
    let mut x0 = n / 2;
    let mut x1 = (x0 + n / x0) / 2;
    while x1 < x0 {
        x0 = x1;
        x1 = (x0 + n / x0) / 2;
    }
    x0
}

/// Extracts initials from whitespace-separated words in uppercase.
pub fn extract_initials(phrase: &str) -> String {
    phrase
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .collect::<String>()
        .to_ascii_uppercase()
}

/// Known Vietnamese corporate abbreviations and acronym pairings.
const KNOWN_ACRONYM_PAIRS: &[(&str, &str)] = &[
    ("HAGL", "HOANG ANH GIA LAI"),
    ("VCS", "VINACOMIN"),
    ("VCS", "THAN KHOANG SAN VINACOMIN"),
    ("MB", "MAY BOM"),
    ("HD", "HAI DANG"),
    ("AP", "AN PHAT"),
    ("TL", "THANG LONG"),
    ("DV", "DAI VIET"),
    ("HB", "HOA BINH"),
    ("SH", "SONG HONG"),
    ("TV", "TRI VIET"),
    ("TCB", "TECHCOMBANK"),
    ("VCB", "VIETCOMBANK"),
    ("BIDV", "DAU TU VA PHAT TRIEN"),
    ("CTG", "VIETINBANK"),
];

/// Detects whether an acronym pattern exists between party name and narration.
pub fn is_acronym_pattern(party_str: &str, narration_str: &str) -> bool {
    let p_upper = party_str.to_ascii_uppercase();
    let n_upper = narration_str.to_ascii_uppercase();

    // 1. Direct initials acronym check (e.g. "HOANG ANH GIA LAI" -> "HAGL")
    let initials = extract_initials(&p_upper);
    if initials.len() >= 2 && n_upper.split_whitespace().any(|w| w == initials) {
        return true;
    }

    // 2. Sub-phrase initials (e.g. "HAI DANG" in "MAY BOM HAI DANG" -> "HD")
    let words: Vec<&str> = p_upper.split_whitespace().collect();
    if words.len() >= 2 {
        for len in 2..=words.len().min(4) {
            for window in words.windows(len) {
                let sub_init: String = window.iter().filter_map(|w| w.chars().next()).collect();
                if sub_init.len() >= 2 && n_upper.split_whitespace().any(|w| w == sub_init) {
                    return true;
                }
            }
        }
    }

    // 3. Known acronym pairs
    for &(acronym, full_name) in KNOWN_ACRONYM_PAIRS {
        if (p_upper.contains(full_name) || full_name.contains(&p_upper))
            && n_upper.split_whitespace().any(|w| w == acronym)
        {
            return true;
        }
        if (n_upper.contains(full_name) || full_name.contains(&n_upper))
            && p_upper.split_whitespace().any(|w| w == acronym)
        {
            return true;
        }
    }

    // 4. Reverse: narration initials match party word (e.g. party is "HAGL")
    let narr_initials = extract_initials(&n_upper);
    if narr_initials.len() >= 2 && p_upper.split_whitespace().any(|w| w == narr_initials) {
        return true;
    }

    false
}

/// Evaluates semantic similarity between party and narration in integer basis points (0..10,000 bps).
///
/// Combines:
/// 1. Acronym recognition (initials, known corporate aliases).
/// 2. Core significant domain word coverage (discounting banking transfer boilerplate).
/// 3. Token cosine / dot product similarity using integer micro-precision arithmetic.
pub fn compute_semantic_similarity_bps(party_str: &str, narration_str: &str) -> u32 {
    let p_upper = party_str.to_ascii_uppercase();
    let n_upper = narration_str.to_ascii_uppercase();

    if p_upper.is_empty() || n_upper.is_empty() {
        return 0;
    }
    if p_upper == n_upper {
        return 10_000;
    }

    // 1. Acronym evaluation
    let initials = extract_initials(&p_upper);
    let mut acronym_sim = 0u32;
    if initials.len() >= 2 && n_upper.split_whitespace().any(|w| w == initials) {
        acronym_sim = 9_500; // 95% for direct initials match (e.g. HAGL)
    }

    for &(acronym, full_name) in KNOWN_ACRONYM_PAIRS {
        if (p_upper.contains(full_name) || full_name.contains(&p_upper))
            && n_upper.split_whitespace().any(|w| w == acronym)
        {
            acronym_sim = acronym_sim.max(9_200);
        }
    }

    // Sub-phrase initials
    let p_words_all: Vec<&str> = p_upper.split_whitespace().collect();
    if p_words_all.len() >= 2 {
        for len in 2..=p_words_all.len().min(4) {
            for window in p_words_all.windows(len) {
                let sub_init: String = window.iter().filter_map(|w| w.chars().next()).collect();
                if sub_init.len() >= 2 && n_upper.split_whitespace().any(|w| w == sub_init) {
                    acronym_sim = acronym_sim.max(8_500);
                }
            }
        }
    }

    // 2. Filter banking boilerplate and extract core domain words
    const BANK_FILLER: &[&str] = &[
        "CK", "TT", "TIEN", "CHUYEN", "KHOAN", "THANH", "TOAN", "NAPAS", "VIETQR",
        "IBFT", "DA", "TRU", "PHI", "DON", "HANG", "THEO", "XIN", "CAM", "ON",
        "CONG", "TY", "TNHH", "CP", "CO", "PHAN", "MTV", "TAP", "DOAN", "CTY",
    ];

    let p_core_words: Vec<&str> = p_upper
        .split_whitespace()
        .filter(|w| w.len() > 1 && !BANK_FILLER.contains(w))
        .collect();

    let n_core_words: Vec<&str> = n_upper
        .split_whitespace()
        .filter(|w| w.len() > 1 && !BANK_FILLER.contains(w))
        .collect();

    let mut core_coverage_bps = 0u32;
    if !p_core_words.is_empty() {
        let mut covered = 0usize;
        for &pw in &p_core_words {
            // Direct word containment, substring, or acronym match
            if n_core_words.iter().any(|&nw| nw == pw || nw.contains(pw))
                || (initials.len() >= 2 && n_upper.split_whitespace().any(|w| w == initials))
            {
                covered += 1;
            }
        }

        let ratio = (covered as u64 * 10_000) / (p_core_words.len() as u64);
        core_coverage_bps = if covered == p_core_words.len() && covered >= 1 {
            // 100% of core domain words present (e.g. "MAY BOM" in "ck tien may bom hd 88")
            9_200.max(ratio as u32)
        } else {
            ratio as u32
        };
    }

    // 3. Integer Token Cosine / Dot Product Similarity
    let cosine_bps = integer_token_cosine_similarity(&p_upper, &n_upper);

    // Composite semantic similarity is the maximum of acronym, core coverage, and token cosine
    let combined_semantic = acronym_sim.max(core_coverage_bps).max(cosine_bps);
    combined_semantic.min(10_000)
}

/// Calculates token cosine similarity in integer basis points (0..10,000 bps)
/// using dot products of word frequency and character n-gram vectors.
pub fn integer_token_cosine_similarity(s1: &str, s2: &str) -> u32 {
    if s1.is_empty() || s2.is_empty() {
        return 0;
    }
    if s1 == s2 {
        return 10_000;
    }

    let mut tokens1: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    let mut tokens2: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    // Add word tokens with weight 10
    for w in s1.split_whitespace() {
        *tokens1.entry(w.to_string()).or_insert(0) += 10;
    }
    for w in s2.split_whitespace() {
        *tokens2.entry(w.to_string()).or_insert(0) += 10;
    }

    // Add character 2-grams with weight 3 for subword overlap
    let chars1: Vec<char> = s1.chars().filter(|c| !c.is_whitespace()).collect();
    for window in chars1.windows(2) {
        let gram: String = window.iter().collect();
        *tokens1.entry(gram).or_insert(0) += 3;
    }

    let chars2: Vec<char> = s2.chars().filter(|c| !c.is_whitespace()).collect();
    for window in chars2.windows(2) {
        let gram: String = window.iter().collect();
        *tokens2.entry(gram).or_insert(0) += 3;
    }

    // Compute dot product and norms using u128
    let mut dot: u128 = 0;
    let mut norm1: u128 = 0;
    let mut norm2: u128 = 0;

    for (k, &v1) in &tokens1 {
        norm1 += (v1 as u128) * (v1 as u128);
        if let Some(&v2) = tokens2.get(k) {
            dot += (v1 as u128) * (v2 as u128);
        }
    }

    for &v2 in tokens2.values() {
        norm2 += (v2 as u128) * (v2 as u128);
    }

    if norm1 == 0 || norm2 == 0 || dot == 0 {
        return 0;
    }

    let product = norm1.saturating_mul(norm2);
    let root = integer_sqrt_u128(product);

    if root == 0 {
        return 0;
    }

    let bps = (dot.saturating_mul(10_000)) / root;
    (bps as u32).min(10_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_and_empty() {
        assert_eq!(integer_jaro_winkler("", ""), 10_000);
        assert_eq!(integer_jaro_winkler("TEST", "TEST"), 10_000);
        assert_eq!(integer_jaro_winkler("CONG TY AN PHAT", "CONG TY AN PHAT"), 10_000);

        assert_eq!(integer_jaro_winkler("ABC", ""), 0);
        assert_eq!(integer_jaro_winkler("", "ABC"), 0);
    }

    #[test]
    fn test_completely_different() {
        assert_eq!(integer_jaro_winkler("ABCDE", "FGHIJ"), 0);
        assert_eq!(integer_jaro_winkler("12345", "67890"), 0);
    }

    #[test]
    fn test_martha_marhta_benchmark() {
        // MARTHA vs MARHTA: Float Jaro = 0.9444, JW = 0.9611
        let j = integer_jaro("MARTHA", "MARHTA");
        assert_eq!(j, 9_444);

        let jw = integer_jaro_winkler("MARTHA", "MARHTA");
        assert_eq!(jw, 9_611); // 96.11%
    }

    #[test]
    fn test_dwayne_duane_benchmark() {
        // DWAYNE vs DUANE: Float Jaro = 0.8222, JW = 0.8400
        let j = integer_jaro("DWAYNE", "DUANE");
        assert_eq!(j, 8_222);

        let jw = integer_jaro_winkler("DWAYNE", "DUANE");
        assert_eq!(jw, 8_400); // 84.00%
    }

    #[test]
    fn test_vietnamese_unicode_names() {
        let score = integer_jaro_winkler("CÔNG TY AN PHÁT", "CÔNG TY CỔ PHẦN AN PHÁT");
        assert!(score >= 8_000, "Expected >= 8000 bps, got {}", score);

        let diff = integer_jaro_winkler("NGUYỄN VĂN AN", "TRẦN THỊ BÌNH");
        assert!(diff < 5_000, "Expected low score for different names, got {}", diff);
    }

    #[test]
    fn test_integer_sqrt() {
        assert_eq!(integer_sqrt_u64(0), 0);
        assert_eq!(integer_sqrt_u64(1), 1);
        assert_eq!(integer_sqrt_u64(4), 2);
        assert_eq!(integer_sqrt_u64(9), 3);
        assert_eq!(integer_sqrt_u64(100), 10);
        assert_eq!(integer_sqrt_u64(144), 12);
        assert_eq!(integer_sqrt_u64(10_000), 100);

        assert_eq!(integer_sqrt_u128(0), 0);
        assert_eq!(integer_sqrt_u128(1), 1);
        assert_eq!(integer_sqrt_u128(1_000_000_000_000), 1_000_000);
        assert_eq!(integer_sqrt_u128(100_000_000_000_000_000_000_000_000), 10_000_000_000_000);
    }

    #[test]
    fn test_acronym_detection_hagl() {
        assert!(is_acronym_pattern("HOANG ANH GIA LAI", "THANH TOAN HAGL"));
        assert!(is_acronym_pattern("CONG TY CP HOANG ANH GIA LAI", "HAGL"));
        assert_eq!(extract_initials("HOANG ANH GIA LAI"), "HAGL");

        let sim = compute_semantic_similarity_bps("HOANG ANH GIA LAI", "THANH TOAN HAGL");
        assert!(sim >= 8_200, "Expected semantic similarity >= 8200 bps, got {}", sim);
    }

    #[test]
    fn test_acronym_detection_vcs_vinacomin() {
        assert!(is_acronym_pattern("VINACOMIN", "CHUYEN TIEN VCS"));
        assert!(is_acronym_pattern("THAN KHOANG SAN VINACOMIN", "VCS"));

        let sim = compute_semantic_similarity_bps("VINACOMIN", "CHUYEN TIEN VCS");
        assert!(sim >= 8_200, "Expected semantic similarity >= 8200 bps, got {}", sim);
    }

    #[test]
    fn test_abbreviation_may_bom_hd() {
        let sim = compute_semantic_similarity_bps("MAY BOM", "ck tien may bom hd 88");
        assert!(sim >= 8_200, "Expected semantic similarity >= 8200 bps, got {}", sim);

        let sim2 = compute_semantic_similarity_bps("CONG TY TNHH MAY BOM", "ck tien may bom hd 88");
        assert!(sim2 >= 8_200, "Expected semantic similarity >= 8200 bps, got {}", sim2);
    }

    #[test]
    fn test_token_cosine_similarity() {
        let sim = integer_token_cosine_similarity("CONG TY AN PHAT", "CONG TY AN PHAT");
        assert_eq!(sim, 10_000);

        let diff = integer_token_cosine_similarity("CONG TY AN PHAT", "TAP DOAN DUC LONG");
        assert!(diff < 3_000, "Expected low token cosine for different companies, got {}", diff);
    }
}

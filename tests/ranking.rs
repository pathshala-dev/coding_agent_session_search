use coding_agent_search::search::query::{MatchType, SearchHit};

// Utility: reproduce ranking blend used in the TUI without touching tui.rs
fn blended_score(hit: &SearchHit, max_created: i64, alpha: f32) -> f32 {
    let recency = if max_created > 0 {
        hit.created_at.unwrap_or(0) as f32 / max_created as f32
    } else {
        0.0
    };
    hit.score * hit.match_type.quality_factor() + alpha * recency
}

#[test]
fn exact_hits_rank_above_wildcards_at_equal_recency_and_score() {
    let max_created = 2_000_000;
    let alpha = 0.4; // Balanced mode in TUI

    let exact = SearchHit {
        title: "t".into(),
        snippet: "s".into(),
        content: "c".into(),
        score: 1.0,
        source_path: "p".into(),
        agent: "a".into(),
        workspace: "w".into(),
        created_at: Some(max_created),
        line_number: None,
        match_type: MatchType::Exact,
    };

    let prefix = SearchHit {
        match_type: MatchType::Prefix,
        ..exact.clone()
    };
    let suffix = SearchHit {
        match_type: MatchType::Suffix,
        ..exact.clone()
    };
    let substring = SearchHit {
        match_type: MatchType::Substring,
        ..exact.clone()
    };
    let implicit = SearchHit {
        match_type: MatchType::ImplicitWildcard,
        ..exact.clone()
    };

    let exact_score = blended_score(&exact, max_created, alpha);
    let prefix_score = blended_score(&prefix, max_created, alpha);
    let suffix_score = blended_score(&suffix, max_created, alpha);
    let substring_score = blended_score(&substring, max_created, alpha);
    let implicit_score = blended_score(&implicit, max_created, alpha);

    assert!(exact_score > prefix_score);
    assert!(prefix_score > suffix_score);
    assert!(suffix_score > substring_score);
    assert!(substring_score > implicit_score);
}

#[test]
fn recency_boost_can_outweigh_quality_when_far_newer() {
    // Two hits: older exact vs newer suffix wildcard.
    // Using RecentHeavy alpha so recency clearly outranks quality penalty.
    let alpha = 1.0; // RecentHeavy mode

    let older_exact = SearchHit {
        title: "old".into(),
        snippet: "s".into(),
        content: "c".into(),
        score: 1.0,
        source_path: "p1".into(),
        agent: "a".into(),
        workspace: "w".into(),
        created_at: Some(1_000_000),
        line_number: None,
        match_type: MatchType::Exact,
    };

    let newer_suffix = SearchHit {
        title: "new".into(),
        snippet: "s".into(),
        content: "c".into(),
        score: 1.0,
        source_path: "p2".into(),
        agent: "a".into(),
        workspace: "w".into(),
        created_at: Some(2_000_000),
        line_number: None,
        match_type: MatchType::Suffix, // quality factor 0.8 vs 1.0
    };

    let max_created = newer_suffix.created_at.unwrap();
    let older_score = blended_score(&older_exact, max_created, alpha);
    let newer_score = blended_score(&newer_suffix, max_created, alpha);

    assert!(
        newer_score > older_score,
        "recency boost should let much newer suffix beat older exact: {newer_score} > {older_score}"
    );
}

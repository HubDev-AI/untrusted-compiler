use sec4_core::{
    build_security_map, parse_source, render_security_audit_text, run_security_audit,
    summarize_history_window, AuditSeverity, Policy,
};
use std::collections::HashMap;
use std::path::Path;

fn base_report() -> sec4_core::AuditReport {
    let source = r#"
fn main() -> Int {
  1
}
"#;
    let program = parse_source(Path::new("main.ut"), source).expect("source should parse");
    let policy = Policy::default();
    let map = build_security_map(&program, &policy);
    run_security_audit(&policy, &map)
}

#[test]
fn history_window_summary_returns_none_for_empty_reports() {
    assert!(
        summarize_history_window(&[], 1).is_none(),
        "empty report history should not produce a summary"
    );
}

#[test]
fn history_window_summary_computes_rollups_and_latest_deltas() {
    let mut oldest = base_report();
    oldest.policy.hash = "pol_old".to_string();
    oldest.build.time_ms = 1000;
    oldest.summary.risk_score = 4;
    oldest.summary.highest_severity = AuditSeverity::MEDIUM;
    oldest.summary.finding_counts = HashMap::from([
        ("LOW".to_string(), 1),
        ("MEDIUM".to_string(), 1),
        ("HIGH".to_string(), 0),
        ("CRITICAL".to_string(), 0),
    ]);

    let mut latest = base_report();
    latest.policy.hash = "pol_new".to_string();
    latest.build.time_ms = 2000;
    latest.summary.risk_score = 11;
    latest.summary.highest_severity = AuditSeverity::HIGH;
    latest.summary.finding_counts = HashMap::from([
        ("LOW".to_string(), 2),
        ("MEDIUM".to_string(), 0),
        ("HIGH".to_string(), 1),
        ("CRITICAL".to_string(), 0),
    ]);

    let summary = summarize_history_window(&[oldest, latest], 5)
        .expect("non-empty history should produce summary");
    assert_eq!(summary.window, 5);
    assert_eq!(summary.reports, 2);
    assert_eq!(summary.oldest_risk_score, 4);
    assert_eq!(summary.latest_risk_score, 11);
    assert_eq!(summary.min_risk_score, 4);
    assert_eq!(summary.max_risk_score, 11);
    assert_eq!(summary.risk_score_delta, 7);
    assert!((summary.average_risk_score - 7.5).abs() < f64::EPSILON);
    assert_eq!(summary.oldest_time_ms, 1000);
    assert_eq!(summary.latest_time_ms, 2000);
    assert_eq!(summary.oldest_policy_hash, "pol_old");
    assert_eq!(summary.latest_policy_hash, "pol_new");
    assert_eq!(summary.highest_severity_seen, "HIGH");
    assert_eq!(
        summary.severity_rollup.get("LOW").copied(),
        Some(3),
        "rollup should sum counts across reports"
    );
    assert_eq!(
        summary.severity_rollup.get("HIGH").copied(),
        Some(1),
        "rollup should include high findings seen in window"
    );
    assert_eq!(
        summary.severity_latest_delta.get("LOW").copied(),
        Some(1),
        "latest delta should compare latest minus oldest counts"
    );
    assert_eq!(
        summary.severity_latest_delta.get("MEDIUM").copied(),
        Some(-1),
        "latest delta should capture reductions too"
    );
}

#[test]
fn text_render_includes_history_window_when_present() {
    let report = base_report();
    let summary = summarize_history_window(std::slice::from_ref(&report), 1)
        .expect("single-report history window should be summarized");

    let mut with_history = report.clone();
    with_history.history_window = Some(summary);
    let rendered = render_security_audit_text(&with_history);
    assert!(
        rendered.contains("HistoryWindow:"),
        "text audit renderer should include history window section when present"
    );
    assert!(
        rendered.contains("reports=1/1"),
        "history window line should include report/window counts"
    );
}

use coding_agent_search::connectors::cline::ClineConnector;
use coding_agent_search::connectors::{Connector, ScanContext};
use std::path::PathBuf;

#[test]
fn cline_parses_fixture_task() {
    let fixture_root = PathBuf::from("tests/fixtures/cline");
    let conn = ClineConnector::new();
    let ctx = ScanContext {
        data_root: fixture_root.clone(),
        since_ts: None,
    };
    let convs = conn.scan(&ctx).expect("scan");
    assert_eq!(convs.len(), 1);
    let c = &convs[0];
    assert_eq!(c.title.as_deref(), Some("Cline fixture task"));
    assert_eq!(c.messages.len(), 3);
    assert!(c.messages.iter().any(|m| m.content.contains("API reply")));
}

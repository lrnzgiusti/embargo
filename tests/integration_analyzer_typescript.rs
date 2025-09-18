use embargo::core::CodebaseAnalyzer;
use embargo::formatters::LLMOptimizedFormatter;
use std::path::PathBuf;

#[test]
fn analyzer_on_typescript_app_directory() {
    let root: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test_apps")
        .join("typescript_app");

    let mut analyzer = CodebaseAnalyzer::new();
    let graph = analyzer.analyze(&root, &["typescript"]).unwrap();
    assert!(graph.node_count() > 0);

    let tmp = tempfile::NamedTempFile::new().unwrap();
    let out = tmp.path().to_path_buf();
    LLMOptimizedFormatter::new()
        .format_to_file(&graph, &out)
        .unwrap();
    let s = std::fs::read_to_string(&out).unwrap();
    assert!(s.contains("# CODE_GRAPH"));
}

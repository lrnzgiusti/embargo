use embargo::core::CodebaseAnalyzer;
use embargo::formatters::LLMOptimizedFormatter;
use std::fs;

#[test]
fn analyzer_end_to_end_on_small_rust_file() {
    // Create a tiny rust project in temp dir
    let dir = tempfile::TempDir::new().unwrap();
    let src = dir.path().join("mini.rs");
    fs::write(&src, "fn hello() { println!(\"hi\"); }\n").unwrap();

    let mut analyzer = CodebaseAnalyzer::new();
    let graph = analyzer.analyze(dir.path(), &["rust"]).unwrap();

    // Expect at least one node (function) and possibly module/import nodes
    assert!(graph.node_count() >= 1);

    let out = dir.path().join("out.md");
    LLMOptimizedFormatter::new()
        .format_to_file(&graph, &out)
        .unwrap();
    let s = fs::read_to_string(&out).unwrap();

    assert!(s.contains("# CODE_GRAPH"));
    assert!(s.contains("NODES:"));
    assert!(s.contains("EDGES:"));
}

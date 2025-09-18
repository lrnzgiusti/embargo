use embargo::core::graph::{GraphBuilder, Node, NodeType};
use embargo::formatters::LLMOptimizedFormatter;
use std::path::PathBuf;

fn node(path: &str, id: &str, name: &str, ty: NodeType, line: usize) -> Node {
    Node::new(
        id.to_string(),
        name.to_string(),
        ty,
        PathBuf::from(path),
        line,
        "rust".to_string(),
    )
}

// Helper reserved for future block-specific snapshotting.
#[allow(dead_code)]
fn _extract_block<'a>(s: &'a str, header: &str) -> Option<&'a str> {
    let start = s.find(header)?;
    let rest = &s[start..];
    if let Some(end) = rest[start..].find("\n\n") {
        Some(&rest[..start + end + 2])
    } else {
        Some(rest)
    }
}

#[test]
fn llm_formatter_files_and_directory_tree_are_stable() {
    let mut gb = GraphBuilder::new();
    // Two files in sorted path order
    let m = node("proj/src/a.rs", "M", "mod_a", NodeType::Module, 1);
    let f = node("proj/src/a.rs", "F", "foo", NodeType::Function, 10);
    let v = node("proj/src/b.rs", "V", "x", NodeType::Variable, 2);
    gb.add_node(m);
    gb.add_node(f);
    gb.add_node(v);
    let graph = gb.build();

    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_path_buf();
    // Force hierarchical mode (no semantic clusters) to snapshot FILES block
    LLMOptimizedFormatter::new()
        .with_hierarchical(true)
        .with_compressed_ids(true)
        .with_semantic_clustering(false)
        .format_to_file(&graph, &path)
        .unwrap();
    let out = std::fs::read_to_string(&path).unwrap();

    // Header counts are deterministic
    assert!(out.contains("# CODE_GRAPH"));
    assert!(out.contains("NODES:3 EDGES:0"));

    // Snapshot FILES block
    let files_idx = out.find("## FILES\n").expect("FILES section present");
    let after = &out[files_idx..];
    // take lines until the first blank line after listing
    let stop = after.find("\n\n").unwrap();
    let files_block = &after[..stop + 2];
    let expected = "## FILES\nU0: proj/src/a.rs\nU1: proj/src/b.rs\n\n";
    assert_eq!(files_block, expected);

    // Snapshot DIRECTORY_TREE minimal structure
    let tree_idx = out
        .find("## DIRECTORY_TREE\n")
        .expect("DIRECTORY_TREE present");
    let tree_after = &out[tree_idx..];
    // It prints ROOT and then an empty tree line and a blank line
    let expected_tree_prefix = "## DIRECTORY_TREE\nROOT: proj/src/\n\n";
    assert!(tree_after.starts_with(expected_tree_prefix));
}

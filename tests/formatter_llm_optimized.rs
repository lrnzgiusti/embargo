use embargo::core::graph::{Edge, EdgeType, GraphBuilder, Node, NodeType};
use embargo::formatters::LLMOptimizedFormatter;
use std::path::PathBuf;

fn node(id: &str, name: &str, ty: NodeType) -> Node {
    Node::new(
        id.to_string(),
        name.to_string(),
        ty,
        PathBuf::from("/tmp/mod.rs"),
        10,
        "rust".to_string(),
    )
}

#[test]
fn llm_optimized_contains_headers_and_counts() {
    let mut gb = GraphBuilder::new();
    let m = node("M", "mod_m", NodeType::Module);
    let f = node("F", "foo", NodeType::Function);
    gb.add_node(m.clone());
    gb.add_node(f.clone());
    gb.add_edge(Edge::new(EdgeType::Contains, m.id.clone(), f.id.clone()));
    let graph = gb.build();

    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_path_buf();

    let fmt = LLMOptimizedFormatter::new();
    fmt.format_to_file(&graph, &path).unwrap();
    let s = std::fs::read_to_string(&path).unwrap();

    assert!(s.contains("# EMBARGO: LLM-Optimized Codebase Dependency Graph"));
    assert!(s.contains("# CODE_GRAPH"));
    assert!(s.contains("NODES:2 EDGES:1"));
    assert!(s.contains("## DIRECTORY_TREE"));
    assert!(s.contains("## ARCHITECTURAL_CLUSTERS"));
    assert!(s.contains("## DEPENDENCY_PATTERNS"));
}

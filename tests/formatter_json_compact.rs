use embargo::core::graph::{Edge, EdgeType, GraphBuilder, Node, NodeType};
use embargo::formatters::JsonCompactFormatter;
use serde_json::Value;
use std::path::PathBuf;

fn node(id: &str, name: &str, ty: NodeType) -> Node {
    Node::new(
        id.to_string(),
        name.to_string(),
        ty,
        PathBuf::from("/tmp/file.rs"),
        1,
        "rust".to_string(),
    )
}

#[test]
fn json_compact_formatter_outputs_valid_json() {
    let mut gb = GraphBuilder::new();
    let a = node("A", "mod_a", NodeType::Module);
    let b = node("B", "func_b", NodeType::Function);
    let c = node("C", "var_c", NodeType::Variable);
    gb.add_node(a.clone());
    gb.add_node(b.clone());
    gb.add_node(c.clone());
    gb.add_edge(Edge::new(EdgeType::Call, b.id.clone(), c.id.clone()));

    let graph = gb.build();
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().with_extension("json");

    let fmt = JsonCompactFormatter::new();
    fmt.format_to_file(&graph, &path).unwrap();

    let data = std::fs::read_to_string(&path).unwrap();
    let v: Value = serde_json::from_str(&data).unwrap();

    assert_eq!(v["meta"]["nodes"].as_u64().unwrap() as usize, 3);
    assert_eq!(v["meta"]["edges"].as_u64().unwrap() as usize, 1);
    assert!(v["files"].is_array());
    assert!(v["nodes"].is_array());
    assert!(v["edges"].is_array());

    // Edge is [src_id, tgt_id, type_code], where Call => 1
    let edge = &v["edges"][0];
    assert_eq!(edge[2].as_u64().unwrap(), 1);
}

use embargo::core::graph::{Edge, EdgeType, GraphBuilder, Node, NodeType};
use embargo::formatters::JsonCompactFormatter;
use serde_json::{json, Value};
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

#[test]
fn json_compact_snapshot_small_graph() {
    let mut gb = GraphBuilder::new();
    let a = node("proj/src/a.rs", "A", "foo", NodeType::Function, 1);
    let b = node("proj/src/b.rs", "B", "bar", NodeType::Function, 2);
    let a_id = a.id.clone();
    let b_id = b.id.clone();
    gb.add_node(a);
    gb.add_node(b);
    // foo -> bar
    gb.add_edge(Edge::new(EdgeType::Call, a_id, b_id));
    let graph = gb.build();

    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().with_extension("json");
    JsonCompactFormatter::new()
        .format_to_file(&graph, &path)
        .unwrap();
    let s = std::fs::read_to_string(&path).unwrap();
    let v: Value = serde_json::from_str(&s).unwrap();

    // Build expected structurally (order-sensitive) and compare values
    // file ids are assigned in first-pass encounter order: a.rs -> 0, b.rs -> 1
    let expected = json!({
        "meta": {"nodes": 2, "edges": 1, "format": "compact"},
        "files": ["proj/src/a.rs", "proj/src/b.rs"],
        "nodes": [
            {"n":"foo","t":2,"f":0,"l":1},
            {"n":"bar","t":2,"f":1,"l":2}
        ],
        "edges": [[0,1,1]]
    });
    assert_eq!(v, expected);
}

use embargo::core::{
    graph::{Edge, EdgeType, GraphBuilder, Node, NodeType},
    DependencyGraph,
};
use std::path::PathBuf;

fn make_node(id: &str, name: &str, ty: NodeType) -> Node {
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
fn graph_builder_adds_nodes_and_edges() {
    let mut gb = GraphBuilder::new();

    let n1 = make_node("id:module:m", "m", NodeType::Module);
    let n2 = make_node("id:function:f", "f", NodeType::Function);
    let n3 = make_node("id:variable:v", "v", NodeType::Variable);

    gb.add_node(n1.clone());
    gb.add_node(n2.clone());
    gb.add_node(n3.clone());

    let e1 = Edge::new(EdgeType::Contains, n1.id.clone(), n2.id.clone());
    let e2 = Edge::new(EdgeType::Uses, n2.id.clone(), n3.id.clone());

    assert!(gb.add_edge(e1).is_some());
    assert!(gb.add_edge(e2).is_some());

    let graph: DependencyGraph = gb.build();
    assert_eq!(graph.node_count(), 3);
    assert_eq!(graph.edge_count(), 2);
}

#[test]
fn add_edge_returns_none_when_missing_nodes() {
    let mut gb = GraphBuilder::new();
    let n1 = make_node("A", "a", NodeType::Function);
    gb.add_node(n1.clone());

    // target not present
    let e = Edge::new(EdgeType::Call, n1.id.clone(), "missing".to_string());
    assert!(gb.add_edge(e).is_none());
}

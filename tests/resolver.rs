use embargo::core::resolver::{CallSite, CallType, FunctionResolver};
use embargo::core::{graph::Node, EdgeType, NodeType};
use std::path::PathBuf;

fn func(id: &str, name: &str) -> Node {
    Node::new(
        id.to_string(),
        name.to_string(),
        NodeType::Function,
        PathBuf::from("/tmp/mod.rs"),
        1,
        "rust".to_string(),
    )
}

#[test]
fn resolver_simple_call_matches_function() {
    let nodes = vec![
        func("id:function:foo:1", "foo"),
        func("id:function:bar:2", "bar"),
    ];

    let mut resolver = FunctionResolver::new();
    resolver.build_indexes(&nodes).unwrap();

    let call = CallSite {
        caller_id: nodes[0].id.clone(),
        called_name: "bar".to_string(),
        call_type: CallType::SimpleCall,
        context: None,
        line_number: 42,
    };

    let edges = resolver.resolve_calls(&[call]);
    assert_eq!(edges.len(), 1);
    let e = &edges[0];
    assert_eq!(e.edge_type, EdgeType::Call);
    assert_eq!(e.source_id, nodes[0].id);
    assert_eq!(e.target_id, nodes[1].id);
}

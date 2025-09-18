use embargo::core::{EdgeType, NodeType};
use embargo::parsers::rust::RustParser;
use embargo::parsers::LanguageParser;
use std::fs;

#[test]
fn rust_parser_extracts_core_entities() {
    let dir = tempfile::TempDir::new().unwrap();
    let file = dir.path().join("sample.rs");
    let code = r#"
        mod m {}
        use std::fmt;

        struct Point { x: i32 }

        trait T { fn t(&self); }

        impl T for Point { fn t(&self) {} }

        fn foo(a: i32) { let _ = a; println!("{}", a); }
    "#;
    fs::write(&file, code).unwrap();

    let parser = RustParser::new().unwrap();
    let result = parser.parse_file(&file).unwrap();

    assert!(!result.nodes.is_empty());

    // Expect at least one module, one function, one class, one interface
    assert!(result.nodes.iter().any(|n| n.node_type == NodeType::Module));
    assert!(result
        .nodes
        .iter()
        .any(|n| n.node_type == NodeType::Function && n.name == "foo"));
    assert!(result
        .nodes
        .iter()
        .any(|n| n.node_type == NodeType::Class && n.name == "Point"));
    assert!(result
        .nodes
        .iter()
        .any(|n| n.node_type == NodeType::Interface && n.name == "T"));

    // Contains edges: struct -> field or trait -> method
    assert!(result
        .edges
        .iter()
        .any(|e| e.edge_type == EdgeType::Contains));

    // Call sites exist (extracted generically)
    assert!(result
        .call_sites
        .as_ref()
        .map(|v| !v.is_empty())
        .unwrap_or(false));
}

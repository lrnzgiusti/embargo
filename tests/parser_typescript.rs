use embargo::core::{EdgeType, NodeType};
use embargo::parsers::typescript::TypeScriptParser;
use embargo::parsers::LanguageParser;
use std::fs;

#[test]
fn typescript_parser_extracts_imports_classes_interfaces_and_functions() {
    let dir = tempfile::TempDir::new().unwrap();
    let file = dir.path().join("sample.ts");
    let code = r#"
import { X } from './x';

interface I { a: number }

class C implements I {
  a: number = 1;
  constructor() {}
  m(p: string) { return p; }
}

function f(n: number): number { return n; }
const g = (x: number) => x + 1;
"#;
    fs::write(&file, code).unwrap();

    let parser = TypeScriptParser::new().unwrap();
    let result = parser.parse_file(&file).unwrap();

    assert!(result.nodes.iter().any(|n| n.node_type == NodeType::Module)); // import
    assert!(result
        .nodes
        .iter()
        .any(|n| n.node_type == NodeType::Interface && n.name == "I"));
    assert!(result
        .nodes
        .iter()
        .any(|n| n.node_type == NodeType::Class && n.name == "C"));
    assert!(result
        .nodes
        .iter()
        .any(|n| n.node_type == NodeType::Function && n.name == "m"));
    assert!(result
        .nodes
        .iter()
        .any(|n| n.node_type == NodeType::Function && n.name == "f"));
    assert!(result
        .nodes
        .iter()
        .any(|n| n.node_type == NodeType::Function && n.name == "g"));

    // inheritance/implements or contains edges should be present
    assert!(result
        .edges
        .iter()
        .any(|e| matches!(e.edge_type, EdgeType::Implements | EdgeType::Contains)));

    assert!(result.call_sites.as_ref().is_some());
}

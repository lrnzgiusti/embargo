use embargo::core::{EdgeType, NodeType};
use embargo::parsers::python::PythonParser;
use embargo::parsers::LanguageParser;
use std::fs;

#[test]
fn python_parser_extracts_imports_classes_functions_and_calls() {
    let dir = tempfile::TempDir::new().unwrap();
    let file = dir.path().join("sample.py");
    let code = r#"
import os

class A(Base):
    """Doc for A"""
    def m(self, x):
        return helper(x)

def helper(v):
    return v
"#;
    fs::write(&file, code).unwrap();

    let parser = PythonParser::new().unwrap();
    let result = parser.parse_file(&file).unwrap();

    assert!(result.nodes.iter().any(|n| n.node_type == NodeType::Module)); // import
    assert!(result
        .nodes
        .iter()
        .any(|n| n.node_type == NodeType::Class && n.name == "A"));
    assert!(result
        .nodes
        .iter()
        .any(|n| n.node_type == NodeType::Function && n.name == "m"));
    assert!(result
        .nodes
        .iter()
        .any(|n| n.node_type == NodeType::Function && n.name == "helper"));

    // class contains method edge
    assert!(result
        .edges
        .iter()
        .any(|e| e.edge_type == EdgeType::Contains));

    // callsites should be detected
    assert!(result
        .call_sites
        .as_ref()
        .map(|cs| !cs.is_empty())
        .unwrap_or(false));
}

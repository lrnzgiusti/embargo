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

#[test]
fn python_parser_extracts_inheritance() {
    let dir = tempfile::TempDir::new().unwrap();
    let file = dir.path().join("inheritance.py");
    let code = r#"
class Base:
    pass

class Child(Base):
    pass

class GrandChild(Child):
    pass
"#;
    fs::write(&file, code).unwrap();

    let parser = PythonParser::new().unwrap();
    let result = parser.parse_file(&file).unwrap();

    // Should have 3 classes
    let classes: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| n.node_type == NodeType::Class)
        .collect();
    assert_eq!(classes.len(), 3);

    // Should have inheritance edges
    let inheritance_edges: Vec<_> = result
        .edges
        .iter()
        .filter(|e| e.edge_type == EdgeType::Inheritance)
        .collect();
    assert_eq!(inheritance_edges.len(), 2); // Child->Base, GrandChild->Child
}

#[test]
fn python_parser_extracts_nested_functions() {
    let dir = tempfile::TempDir::new().unwrap();
    let file = dir.path().join("nested.py");
    let code = r#"
def outer():
    def inner():
        def deeply_nested():
            pass
        return deeply_nested
    return inner
"#;
    fs::write(&file, code).unwrap();

    let parser = PythonParser::new().unwrap();
    let result = parser.parse_file(&file).unwrap();

    // Should have 3 functions
    let functions: Vec<_> = result
        .nodes
        .iter()
        .filter(|n| n.node_type == NodeType::Function)
        .collect();
    assert_eq!(functions.len(), 3);

    // Should have outer, inner, deeply_nested
    assert!(functions.iter().any(|f| f.name == "outer"));
    assert!(functions.iter().any(|f| f.name == "inner"));
    assert!(functions.iter().any(|f| f.name == "deeply_nested"));

    // Should have containment edges for nested functions
    let contains_edges: Vec<_> = result
        .edges
        .iter()
        .filter(|e| e.edge_type == EdgeType::Contains)
        .collect();
    assert_eq!(contains_edges.len(), 2); // outer->inner, inner->deeply_nested
}

#[test]
fn python_parser_extracts_function_signatures_with_types() {
    let dir = tempfile::TempDir::new().unwrap();
    let file = dir.path().join("typed.py");
    let code = r#"
from typing import List, Optional

def process(data: List[str], count: int = 10) -> Optional[str]:
    pass

class Service:
    def handle(self, request: dict, timeout: float = 5.0) -> bool:
        pass
"#;
    fs::write(&file, code).unwrap();

    let parser = PythonParser::new().unwrap();
    let result = parser.parse_file(&file).unwrap();

    // Check process function signature
    let process_fn = result
        .nodes
        .iter()
        .find(|n| n.name == "process")
        .expect("process function should exist");
    assert!(process_fn.signature.is_some());
    let sig = process_fn.signature.as_ref().unwrap();
    assert!(sig.contains("data: List[str]"));
    assert!(sig.contains("count: int = 10"));

    // Check handle method signature
    let handle_fn = result
        .nodes
        .iter()
        .find(|n| n.name == "handle")
        .expect("handle method should exist");
    assert!(handle_fn.signature.is_some());
    let sig = handle_fn.signature.as_ref().unwrap();
    assert!(sig.contains("self"));
    assert!(sig.contains("request: dict"));
}

#[test]
fn python_parser_extracts_visibility() {
    let dir = tempfile::TempDir::new().unwrap();
    let file = dir.path().join("visibility.py");
    let code = r#"
class Example:
    def public_method(self):
        pass
    
    def _protected_method(self):
        pass
    
    def __private_method(self):
        pass
    
    def __init__(self):
        pass
    
    def __str__(self):
        pass
"#;
    fs::write(&file, code).unwrap();

    let parser = PythonParser::new().unwrap();
    let result = parser.parse_file(&file).unwrap();

    // Check visibility is set correctly
    let public_fn = result
        .nodes
        .iter()
        .find(|n| n.name == "public_method")
        .unwrap();
    assert_eq!(public_fn.visibility.as_deref(), Some("public"));

    let protected_fn = result
        .nodes
        .iter()
        .find(|n| n.name == "_protected_method")
        .unwrap();
    assert_eq!(protected_fn.visibility.as_deref(), Some("protected"));

    let private_fn = result
        .nodes
        .iter()
        .find(|n| n.name == "__private_method")
        .unwrap();
    assert_eq!(private_fn.visibility.as_deref(), Some("private"));

    let init_fn = result.nodes.iter().find(|n| n.name == "__init__").unwrap();
    assert_eq!(init_fn.visibility.as_deref(), Some("dunder"));

    let str_fn = result.nodes.iter().find(|n| n.name == "__str__").unwrap();
    assert_eq!(str_fn.visibility.as_deref(), Some("dunder"));
}

#[test]
fn python_parser_extracts_docstrings() {
    let dir = tempfile::TempDir::new().unwrap();
    let file = dir.path().join("docstrings.py");
    let code = r#"
class MyClass:
    """This is a class docstring."""
    
    def my_method(self):
        """This is a method docstring."""
        pass

def my_function():
    """This is a function docstring."""
    pass
"#;
    fs::write(&file, code).unwrap();

    let parser = PythonParser::new().unwrap();
    let result = parser.parse_file(&file).unwrap();

    let my_class = result
        .nodes
        .iter()
        .find(|n| n.name == "MyClass")
        .unwrap();
    assert!(my_class.docstring.is_some());
    assert!(my_class
        .docstring
        .as_ref()
        .unwrap()
        .contains("class docstring"));

    let my_method = result
        .nodes
        .iter()
        .find(|n| n.name == "my_method")
        .unwrap();
    assert!(my_method.docstring.is_some());
    assert!(my_method
        .docstring
        .as_ref()
        .unwrap()
        .contains("method docstring"));

    let my_function = result
        .nodes
        .iter()
        .find(|n| n.name == "my_function")
        .unwrap();
    assert!(my_function.docstring.is_some());
    assert!(my_function
        .docstring
        .as_ref()
        .unwrap()
        .contains("function docstring"));
}

#[test]
fn python_parser_handles_multiple_inheritance() {
    let dir = tempfile::TempDir::new().unwrap();
    let file = dir.path().join("multi_inherit.py");
    let code = r#"
class Mixin1:
    pass

class Mixin2:
    pass

class Combined(Mixin1, Mixin2):
    pass
"#;
    fs::write(&file, code).unwrap();

    let parser = PythonParser::new().unwrap();
    let result = parser.parse_file(&file).unwrap();

    // Combined should have 2 inheritance edges
    let combined_id = result
        .nodes
        .iter()
        .find(|n| n.name == "Combined")
        .map(|n| &n.id)
        .unwrap();

    let inheritance_from_combined: Vec<_> = result
        .edges
        .iter()
        .filter(|e| e.edge_type == EdgeType::Inheritance && e.source_id == *combined_id)
        .collect();

    assert_eq!(inheritance_from_combined.len(), 2);
}

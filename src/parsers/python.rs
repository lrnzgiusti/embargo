use anyhow::Result;
use std::path::Path;
use tree_sitter::Node as TSNode;

use super::common::{
    extract_docstring, extract_text, find_child_by_kind, generate_node_id, TreeSitterParser,
};
use super::{LanguageParser, ParseResult};
use crate::core::{CallSite, CallSiteExtractor, Edge, EdgeType, Node, NodeType};

pub struct PythonParser {
    #[allow(dead_code)]
    parser: TreeSitterParser,
}

impl PythonParser {
    pub fn new() -> Result<Self> {
        let language = tree_sitter_python::language();
        let parser = TreeSitterParser::new(language)?;
        Ok(Self { parser })
    }

    fn extract_imports(
        &self,
        root: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            match child.kind() {
                "import_statement" | "import_from_statement" => {
                    self.process_import(&child, source, file_path, nodes, edges);
                }
                _ => {}
            }
        }
    }

    fn process_import(
        &self,
        import_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        _edges: &mut Vec<Edge>,
    ) {
        let import_text = extract_text(import_node, source);
        let line_number = import_node.start_position().row + 1;

        let module_id = generate_node_id(file_path, "import", import_text, line_number);
        let import_node = Node::new(
            module_id.clone(),
            import_text.to_string(),
            NodeType::Module,
            file_path.to_path_buf(),
            line_number,
            "python".to_string(),
        );

        nodes.push(import_node);
    }

    fn extract_classes(
        &self,
        root: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            if child.kind() == "class_definition" {
                self.process_class(&child, source, file_path, nodes, edges);
            }
        }
    }

    fn process_class(
        &self,
        class_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(class_node, "identifier") {
            let class_name = extract_text(&name_node, source);
            let line_number = class_node.start_position().row + 1;
            let class_id = generate_node_id(file_path, "class", class_name, line_number);

            let mut class_node_obj = Node::new(
                class_id.clone(),
                class_name.to_string(),
                NodeType::Class,
                file_path.to_path_buf(),
                line_number,
                "python".to_string(),
            );

            if let Some(docstring) = extract_docstring(class_node, source) {
                class_node_obj = class_node_obj.with_docstring(docstring);
            }

            if let Some(argument_list) = find_child_by_kind(class_node, "argument_list") {
                for arg in argument_list.children(&mut argument_list.walk()) {
                    if arg.kind() == "identifier" {
                        let parent_class = extract_text(&arg, source);
                        let parent_id = format!("external:class:{}:0", parent_class);
                        let inheritance_edge =
                            Edge::new(EdgeType::Inheritance, class_id.clone(), parent_id);
                        edges.push(inheritance_edge);
                    }
                }
            }

            nodes.push(class_node_obj);

            self.extract_class_methods(class_node, source, file_path, &class_id, nodes, edges);
        }
    }

    fn extract_class_methods(
        &self,
        class_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        class_id: &str,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(class_body) = find_child_by_kind(class_node, "block") {
            for child in class_body.children(&mut class_body.walk()) {
                if child.kind() == "function_definition" {
                    self.process_method(&child, source, file_path, Some(class_id), nodes, edges);
                }
            }
        }
    }

    fn extract_functions(
        &self,
        root: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            if child.kind() == "function_definition" {
                self.process_method(&child, source, file_path, None, nodes, edges);
            }
        }
    }

    fn process_method(
        &self,
        func_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        class_id: Option<&str>,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(func_node, "identifier") {
            let func_name = extract_text(&name_node, source);
            let line_number = func_node.start_position().row + 1;
            let func_id = generate_node_id(file_path, "function", func_name, line_number);

            let mut signature = func_name.to_string();
            if let Some(params) = find_child_by_kind(func_node, "parameters") {
                signature = format!("{}({})", func_name, extract_text(&params, source));
            }

            let mut func_node_obj = Node::new(
                func_id.clone(),
                func_name.to_string(),
                NodeType::Function,
                file_path.to_path_buf(),
                line_number,
                "python".to_string(),
            )
            .with_signature(signature);

            if let Some(docstring) = extract_docstring(func_node, source) {
                func_node_obj = func_node_obj.with_docstring(docstring);
            }

            nodes.push(func_node_obj);

            if let Some(class_id) = class_id {
                let contains_edge =
                    Edge::new(EdgeType::Contains, class_id.to_string(), func_id.clone());
                edges.push(contains_edge);
            }

            // Note: Function calls are now extracted separately via extract_call_sites
            // self.extract_function_calls_legacy(func_node, source, file_path, &func_id, edges);
        }
    }

    #[allow(dead_code)]
    fn extract_function_calls_legacy(
        &self,
        func_node: &TSNode,
        source: &[u8],
        _file_path: &Path,
        func_id: &str,
        edges: &mut Vec<Edge>,
    ) {
        // Legacy implementation - kept for compatibility but will be replaced by CallSiteExtractor
        self.traverse_for_calls_legacy(func_node, source, func_id, edges);
    }

    #[allow(dead_code)]
    fn traverse_for_calls_legacy(
        &self,
        node: &TSNode,
        source: &[u8],
        caller_id: &str,
        edges: &mut Vec<Edge>,
    ) {
        if node.kind() == "call" {
            if let Some(function_node) = node.child(0) {
                let called_function = extract_text(&function_node, source);
                if !called_function.is_empty() {
                    let called_id = format!("external:function:{}:0", called_function);
                    let call_edge = Edge::new(EdgeType::Call, caller_id.to_string(), called_id);
                    edges.push(call_edge);
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_for_calls_legacy(&child, source, caller_id, edges);
        }
    }

    /// Extract call sites using the new optimized CallSiteExtractor
    fn extract_call_sites(
        &self,
        root_node: &TSNode,
        source: &[u8],
        file_path: &Path,
    ) -> Vec<CallSite> {
        let mut extractor = CallSiteExtractor::new();
        extractor.extract_from_ast(root_node, source, file_path)
    }
}

impl LanguageParser for PythonParser {
    fn parse_file(&self, file_path: &Path) -> Result<ParseResult> {
        let mut parser = TreeSitterParser::new(tree_sitter_python::language())?;
        let tree = parser.parse_file(file_path)?;
        let source = parser.get_source(file_path)?;
        let source_bytes = source.as_bytes();

        let root_node = tree.root_node();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        self.extract_imports(&root_node, source_bytes, file_path, &mut nodes, &mut edges);
        self.extract_classes(&root_node, source_bytes, file_path, &mut nodes, &mut edges);
        self.extract_functions(&root_node, source_bytes, file_path, &mut nodes, &mut edges);

        // Extract call sites using the new system
        let call_sites = self.extract_call_sites(&root_node, source_bytes, file_path);

        Ok(ParseResult {
            nodes,
            edges,
            call_sites: Some(call_sites),
        })
    }

    fn language_name(&self) -> &str {
        "python"
    }
}

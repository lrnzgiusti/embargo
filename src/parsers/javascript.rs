use anyhow::Result;
use std::path::Path;
use tree_sitter::Node as TSNode;

use super::common::{
    extract_text, find_child_by_kind, find_children_by_kind, generate_node_id, TreeSitterParser,
};
use super::{LanguageParser, ParseResult};
use crate::core::{CallSite, CallSiteExtractor, Edge, EdgeType, Node, NodeType};

pub struct JavaScriptParser {
    #[allow(dead_code)]
    parser: TreeSitterParser,
}

impl JavaScriptParser {
    pub fn new() -> Result<Self> {
        let language = tree_sitter_javascript::language();
        let parser = TreeSitterParser::new(language)?;
        Ok(Self { parser })
    }

    fn extract_imports(
        &self,
        root: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
    ) {
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            match child.kind() {
                "import_statement" => {
                    self.process_import(&child, source, file_path, nodes);
                }
                "variable_declaration" => {
                    // Check for require() statements
                    self.check_require_statement(&child, source, file_path, nodes);
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
    ) {
        let import_text = extract_text(import_node, source);
        let line_number = import_node.start_position().row + 1;

        let module_id = generate_node_id(file_path, "import", &import_text, line_number);
        let import_node_obj = Node::new(
            module_id,
            import_text.to_string(),
            NodeType::Module,
            file_path.to_path_buf(),
            line_number,
            "javascript".to_string(),
        );

        nodes.push(import_node_obj);
    }

    fn check_require_statement(
        &self,
        var_decl: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
    ) {
        // Look for patterns like: const foo = require('bar')
        for declarator in find_children_by_kind(var_decl, "variable_declarator") {
            if let Some(init_node) = find_child_by_kind(&declarator, "call_expression") {
                if let Some(function_node) = init_node.child(0) {
                    let function_name = extract_text(&function_node, source);
                    if function_name == "require" {
                        let require_text = extract_text(&init_node, source);
                        let line_number = var_decl.start_position().row + 1;

                        let module_id =
                            generate_node_id(file_path, "require", &require_text, line_number);
                        let require_node_obj = Node::new(
                            module_id,
                            require_text.to_string(),
                            NodeType::Module,
                            file_path.to_path_buf(),
                            line_number,
                            "javascript".to_string(),
                        );

                        nodes.push(require_node_obj);
                    }
                }
            }
        }
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
            if child.kind() == "class_declaration" {
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
            let class_id = generate_node_id(file_path, "class", &class_name, line_number);

            let class_node_obj = Node::new(
                class_id.clone(),
                class_name.to_string(),
                NodeType::Class,
                file_path.to_path_buf(),
                line_number,
                "javascript".to_string(),
            );

            // Handle inheritance (extends)
            if let Some(class_heritage) = find_child_by_kind(class_node, "class_heritage") {
                if let Some(identifier) = find_child_by_kind(&class_heritage, "identifier") {
                    let parent_class = extract_text(&identifier, source);
                    let parent_id = format!("external:class:{}:0", parent_class);
                    let inheritance_edge =
                        Edge::new(EdgeType::Inheritance, class_id.clone(), parent_id);
                    edges.push(inheritance_edge);
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
        if let Some(class_body) = find_child_by_kind(class_node, "class_body") {
            for child in class_body.children(&mut class_body.walk()) {
                match child.kind() {
                    "method_definition" => {
                        self.process_method(
                            &child,
                            source,
                            file_path,
                            Some(class_id),
                            nodes,
                            edges,
                        );
                    }
                    "field_definition" => {
                        if let Some(property_name) =
                            find_child_by_kind(&child, "property_identifier")
                        {
                            let field_name = extract_text(&property_name, source);
                            let line_number = child.start_position().row + 1;
                            let field_id =
                                generate_node_id(file_path, "variable", &field_name, line_number);

                            let field_node = Node::new(
                                field_id.clone(),
                                field_name.to_string(),
                                NodeType::Variable,
                                file_path.to_path_buf(),
                                line_number,
                                "javascript".to_string(),
                            );

                            nodes.push(field_node);

                            let contains_edge =
                                Edge::new(EdgeType::Contains, class_id.to_string(), field_id);
                            edges.push(contains_edge);
                        }
                    }
                    _ => {}
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
            match child.kind() {
                "function_declaration" => {
                    self.process_function(&child, source, file_path, None, nodes, edges);
                }
                "variable_declaration" => {
                    // Check for arrow functions and function expressions
                    for declarator in find_children_by_kind(&child, "variable_declarator") {
                        if let Some(init) = find_child_by_kind(&declarator, "arrow_function") {
                            self.process_arrow_function(
                                &declarator,
                                &init,
                                source,
                                file_path,
                                nodes,
                                edges,
                            );
                        } else if let Some(init) = find_child_by_kind(&declarator, "function") {
                            self.process_function_expression(
                                &declarator,
                                &init,
                                source,
                                file_path,
                                nodes,
                                edges,
                            );
                        }
                    }
                }
                "expression_statement" => {
                    // Check for prototype methods like Object.prototype.method = function() {}
                    self.check_prototype_method(&child, source, file_path, nodes, edges);
                }
                _ => {}
            }
        }
    }

    fn process_function(
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
            let func_id = generate_node_id(file_path, "function", &func_name, line_number);

            let mut signature = func_name.to_string();
            if let Some(params) = find_child_by_kind(func_node, "formal_parameters") {
                signature = format!("{}({})", func_name, extract_text(&params, source));
            }

            let func_node_obj = Node::new(
                func_id.clone(),
                func_name.to_string(),
                NodeType::Function,
                file_path.to_path_buf(),
                line_number,
                "javascript".to_string(),
            )
            .with_signature(signature);

            nodes.push(func_node_obj);

            if let Some(class_id) = class_id {
                let contains_edge = Edge::new(EdgeType::Contains, class_id.to_string(), func_id);
                edges.push(contains_edge);
            }
        }
    }

    fn process_method(
        &self,
        method_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        class_id: Option<&str>,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(method_node, "property_identifier") {
            let method_name = extract_text(&name_node, source);
            let line_number = method_node.start_position().row + 1;
            let method_id = generate_node_id(file_path, "function", &method_name, line_number);

            let mut signature = method_name.to_string();
            if let Some(params) = find_child_by_kind(method_node, "formal_parameters") {
                signature = format!("{}({})", method_name, extract_text(&params, source));
            }

            let method_node_obj = Node::new(
                method_id.clone(),
                method_name.to_string(),
                NodeType::Function,
                file_path.to_path_buf(),
                line_number,
                "javascript".to_string(),
            )
            .with_signature(signature);

            nodes.push(method_node_obj);

            if let Some(class_id) = class_id {
                let contains_edge = Edge::new(EdgeType::Contains, class_id.to_string(), method_id);
                edges.push(contains_edge);
            }
        }
    }

    fn process_arrow_function(
        &self,
        declarator: &TSNode,
        _arrow_func: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        _edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(declarator, "identifier") {
            let func_name = extract_text(&name_node, source);
            let line_number = declarator.start_position().row + 1;
            let func_id = generate_node_id(file_path, "function", &func_name, line_number);

            let func_node_obj = Node::new(
                func_id,
                func_name.to_string(),
                NodeType::Function,
                file_path.to_path_buf(),
                line_number,
                "javascript".to_string(),
            );

            nodes.push(func_node_obj);
        }
    }

    fn process_function_expression(
        &self,
        declarator: &TSNode,
        _func_expr: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        _edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(declarator, "identifier") {
            let func_name = extract_text(&name_node, source);
            let line_number = declarator.start_position().row + 1;
            let func_id = generate_node_id(file_path, "function", &func_name, line_number);

            let func_node_obj = Node::new(
                func_id,
                func_name.to_string(),
                NodeType::Function,
                file_path.to_path_buf(),
                line_number,
                "javascript".to_string(),
            );

            nodes.push(func_node_obj);
        }
    }

    fn check_prototype_method(
        &self,
        expr_stmt: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(assignment) = find_child_by_kind(expr_stmt, "assignment_expression") {
            if let Some(left_side) = assignment.child(0) {
                let left_text = extract_text(&left_side, source);
                if left_text.contains(".prototype.") {
                    // This looks like a prototype method assignment
                    if let Some(right_side) = assignment.child(2) {
                        if right_side.kind() == "function" {
                            // Extract method name from prototype assignment
                            let parts: Vec<&str> = left_text.split('.').collect();
                            if let Some(method_name) = parts.last() {
                                let line_number = expr_stmt.start_position().row + 1;
                                let method_id = generate_node_id(
                                    file_path,
                                    "function",
                                    method_name,
                                    line_number,
                                );

                                let method_node_obj = Node::new(
                                    method_id.clone(),
                                    method_name.to_string(),
                                    NodeType::Function,
                                    file_path.to_path_buf(),
                                    line_number,
                                    "javascript".to_string(),
                                );

                                nodes.push(method_node_obj);

                                // If we can determine the class, add a contains edge
                                if parts.len() >= 2 {
                                    let class_name = parts[0];
                                    let class_id = format!("external:class:{}:0", class_name);
                                    let contains_edge =
                                        Edge::new(EdgeType::Contains, class_id, method_id);
                                    edges.push(contains_edge);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn extract_object_methods(
        &self,
        root: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        _edges: &mut Vec<Edge>,
    ) {
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            if child.kind() == "variable_declaration" {
                for declarator in find_children_by_kind(&child, "variable_declarator") {
                    if let Some(object_expr) = find_child_by_kind(&declarator, "object") {
                        self.extract_methods_from_object(&object_expr, source, file_path, nodes);
                    }
                }
            }
        }
    }

    fn extract_methods_from_object(
        &self,
        object_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
    ) {
        for child in object_node.children(&mut object_node.walk()) {
            if child.kind() == "pair" {
                if let Some(key_node) = child.child(0) {
                    if let Some(value_node) = child.child(2) {
                        if value_node.kind() == "function" {
                            let method_name = extract_text(&key_node, source);
                            let line_number = child.start_position().row + 1;
                            let method_id =
                                generate_node_id(file_path, "function", &method_name, line_number);

                            let method_node_obj = Node::new(
                                method_id,
                                method_name.to_string(),
                                NodeType::Function,
                                file_path.to_path_buf(),
                                line_number,
                                "javascript".to_string(),
                            );

                            nodes.push(method_node_obj);
                        }
                    }
                }
            }
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

impl LanguageParser for JavaScriptParser {
    fn parse_file(&self, file_path: &Path) -> Result<ParseResult> {
        let mut parser = TreeSitterParser::new(tree_sitter_javascript::language())?;
        let tree = parser.parse_file(file_path)?;
        let source = parser.get_source(file_path)?;
        let source_bytes = source.as_bytes();

        let root_node = tree.root_node();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        self.extract_imports(&root_node, source_bytes, file_path, &mut nodes);
        self.extract_classes(&root_node, source_bytes, file_path, &mut nodes, &mut edges);
        self.extract_functions(&root_node, source_bytes, file_path, &mut nodes, &mut edges);
        self.extract_object_methods(&root_node, source_bytes, file_path, &mut nodes, &mut edges);

        // Extract call sites using the new system
        let call_sites = self.extract_call_sites(&root_node, source_bytes, file_path);

        Ok(ParseResult {
            nodes,
            edges,
            call_sites: Some(call_sites),
        })
    }

    fn language_name(&self) -> &str {
        "javascript"
    }
}

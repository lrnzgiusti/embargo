use anyhow::Result;
use std::path::Path;
use tree_sitter::Node as TSNode;

use super::common::{extract_text, find_child_by_kind, generate_node_id, TreeSitterParser};
use super::{LanguageParser, ParseResult};
use crate::core::{CallSite, CallSiteExtractor, Edge, EdgeType, Node, NodeType};

pub struct CppParser {
    #[allow(dead_code)]
    parser: TreeSitterParser,
}

impl CppParser {
    pub fn new() -> Result<Self> {
        let language = tree_sitter_cpp::language();
        let parser = TreeSitterParser::new(language)?;
        Ok(Self { parser })
    }

    fn extract_includes(
        &self,
        root: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            if child.kind() == "preproc_include" {
                self.process_include(&child, source, file_path, nodes, edges);
            }
        }
    }

    fn process_include(
        &self,
        include_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        _edges: &mut Vec<Edge>,
    ) {
        let include_text = extract_text(include_node, source);
        let line_number = include_node.start_position().row + 1;

        let module_id = generate_node_id(file_path, "include", include_text, line_number);
        let include_node_obj = Node::new(
            module_id.clone(),
            include_text.to_string(),
            NodeType::Module,
            file_path.to_path_buf(),
            line_number,
            "cpp".to_string(),
        );

        nodes.push(include_node_obj);
    }

    fn extract_namespaces(
        &self,
        root: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            if child.kind() == "namespace_definition" {
                self.process_namespace(&child, source, file_path, nodes, edges);
            }
        }
    }

    fn process_namespace(
        &self,
        namespace_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(namespace_node, "identifier") {
            let namespace_name = extract_text(&name_node, source);
            let line_number = namespace_node.start_position().row + 1;
            let namespace_id =
                generate_node_id(file_path, "namespace", namespace_name, line_number);

            let namespace_node_obj = Node::new(
                namespace_id.clone(),
                namespace_name.to_string(),
                NodeType::Module,
                file_path.to_path_buf(),
                line_number,
                "cpp".to_string(),
            );

            nodes.push(namespace_node_obj);

            // Process contents of namespace
            if let Some(declaration_list) = find_child_by_kind(namespace_node, "declaration_list") {
                self.extract_from_declaration_list(
                    &declaration_list,
                    source,
                    file_path,
                    &namespace_id,
                    nodes,
                    edges,
                );
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
        self.extract_from_declaration_list(root, source, file_path, "", nodes, edges);
    }

    fn extract_from_declaration_list(
        &self,
        parent_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        parent_id: &str,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        let mut cursor = parent_node.walk();

        for child in parent_node.children(&mut cursor) {
            match child.kind() {
                "class_specifier" | "struct_specifier" => {
                    self.process_class_or_struct(
                        &child, source, file_path, parent_id, nodes, edges,
                    );
                }
                "function_definition" => {
                    self.process_function(&child, source, file_path, parent_id, nodes, edges);
                }
                "template_declaration" => {
                    self.process_template(&child, source, file_path, parent_id, nodes, edges);
                }
                "namespace_definition" => {
                    self.process_namespace(&child, source, file_path, nodes, edges);
                }
                _ => {}
            }
        }
    }

    fn process_class_or_struct(
        &self,
        class_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        parent_id: &str,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(class_node, "type_identifier") {
            let class_name = extract_text(&name_node, source);
            let line_number = class_node.start_position().row + 1;
            let class_id = generate_node_id(file_path, "class", class_name, line_number);

            let node_type = NodeType::Class;

            let class_node_obj = Node::new(
                class_id.clone(),
                class_name.to_string(),
                node_type,
                file_path.to_path_buf(),
                line_number,
                "cpp".to_string(),
            );

            // Handle inheritance
            if let Some(base_class_clause) = find_child_by_kind(class_node, "base_class_clause") {
                for base_class in base_class_clause.children(&mut base_class_clause.walk()) {
                    if base_class.kind() == "base_class" {
                        if let Some(base_name_node) =
                            find_child_by_kind(&base_class, "type_identifier")
                        {
                            let parent_class = extract_text(&base_name_node, source);
                            let parent_id = format!("external:class:{}:0", parent_class);
                            let inheritance_edge =
                                Edge::new(EdgeType::Inheritance, class_id.clone(), parent_id);
                            edges.push(inheritance_edge);
                        }
                    }
                }
            }

            // Add containment edge if this class is inside a namespace
            if !parent_id.is_empty() {
                let containment_edge =
                    Edge::new(EdgeType::Contains, parent_id.to_string(), class_id.clone());
                edges.push(containment_edge);
            }

            nodes.push(class_node_obj);

            // Extract class members
            if let Some(field_declaration_list) =
                find_child_by_kind(class_node, "field_declaration_list")
            {
                self.extract_class_members(
                    &field_declaration_list,
                    source,
                    file_path,
                    &class_id,
                    nodes,
                    edges,
                );
            }
        }
    }

    fn extract_class_members(
        &self,
        field_list: &TSNode,
        source: &[u8],
        file_path: &Path,
        class_id: &str,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        let mut cursor = field_list.walk();

        for child in field_list.children(&mut cursor) {
            match child.kind() {
                "function_definition" => {
                    self.process_method(&child, source, file_path, class_id, nodes, edges);
                }
                "declaration" => {
                    // Handle method declarations, constructors, destructors
                    if let Some(declarator) = find_child_by_kind(&child, "function_declarator") {
                        self.process_method_declaration(
                            &child,
                            &declarator,
                            source,
                            file_path,
                            class_id,
                            nodes,
                            edges,
                        );
                    }
                }
                "field_declaration" => {
                    self.process_field(&child, source, file_path, class_id, nodes, edges);
                }
                "template_declaration" => {
                    self.process_template(&child, source, file_path, class_id, nodes, edges);
                }
                _ => {}
            }
        }
    }

    fn process_method(
        &self,
        method_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        class_id: &str,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(declarator) = find_child_by_kind(method_node, "function_declarator") {
            if let Some(name_node) = find_child_by_kind(&declarator, "identifier") {
                let method_name = extract_text(&name_node, source);
                let line_number = method_node.start_position().row + 1;
                let method_id = generate_node_id(file_path, "method", method_name, line_number);

                let method_node_obj = Node::new(
                    method_id.clone(),
                    method_name.to_string(),
                    NodeType::Function,
                    file_path.to_path_buf(),
                    line_number,
                    "cpp".to_string(),
                );

                nodes.push(method_node_obj);

                // Add containment edge
                let containment_edge =
                    Edge::new(EdgeType::Contains, class_id.to_string(), method_id.clone());
                edges.push(containment_edge);

                // Note: Function calls are now extracted separately via extract_call_sites
                // Legacy function call extraction would go here but is replaced by CallSiteExtractor
            }
        }
    }

    fn process_method_declaration(
        &self,
        decl_node: &TSNode,
        declarator: &TSNode,
        source: &[u8],
        file_path: &Path,
        class_id: &str,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(declarator, "identifier") {
            let method_name = extract_text(&name_node, source);
            let line_number = decl_node.start_position().row + 1;
            let method_id = generate_node_id(file_path, "method", method_name, line_number);

            let method_node_obj = Node::new(
                method_id.clone(),
                method_name.to_string(),
                NodeType::Function,
                file_path.to_path_buf(),
                line_number,
                "cpp".to_string(),
            );

            nodes.push(method_node_obj);

            // Add containment edge
            let containment_edge = Edge::new(EdgeType::Contains, class_id.to_string(), method_id);
            edges.push(containment_edge);
        }
    }

    fn process_field(
        &self,
        field_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        class_id: &str,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(declarator) = find_child_by_kind(field_node, "field_declarator") {
            if let Some(name_node) = find_child_by_kind(&declarator, "field_identifier") {
                let field_name = extract_text(&name_node, source);
                let line_number = field_node.start_position().row + 1;
                let field_id = generate_node_id(file_path, "field", field_name, line_number);

                let field_node_obj = Node::new(
                    field_id.clone(),
                    field_name.to_string(),
                    NodeType::Variable,
                    file_path.to_path_buf(),
                    line_number,
                    "cpp".to_string(),
                );

                nodes.push(field_node_obj);

                // Add containment edge
                let containment_edge =
                    Edge::new(EdgeType::Contains, class_id.to_string(), field_id);
                edges.push(containment_edge);
            }
        }
    }

    fn process_function(
        &self,
        func_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        parent_id: &str,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(declarator) = find_child_by_kind(func_node, "function_declarator") {
            if let Some(name_node) = find_child_by_kind(&declarator, "identifier") {
                let func_name = extract_text(&name_node, source);
                let line_number = func_node.start_position().row + 1;
                let func_id = generate_node_id(file_path, "function", func_name, line_number);

                let func_node_obj = Node::new(
                    func_id.clone(),
                    func_name.to_string(),
                    NodeType::Function,
                    file_path.to_path_buf(),
                    line_number,
                    "cpp".to_string(),
                );

                nodes.push(func_node_obj);

                // Add containment edge if inside namespace
                if !parent_id.is_empty() {
                    let containment_edge =
                        Edge::new(EdgeType::Contains, parent_id.to_string(), func_id.clone());
                    edges.push(containment_edge);
                }

                // Note: Function calls are now extracted separately via extract_call_sites
                // Legacy function call extraction would go here but is replaced by CallSiteExtractor
            }
        }
    }

    fn process_template(
        &self,
        template_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        parent_id: &str,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        // Process the template declaration content
        for child in template_node.children(&mut template_node.walk()) {
            match child.kind() {
                "class_specifier" | "struct_specifier" => {
                    self.process_class_or_struct(
                        &child, source, file_path, parent_id, nodes, edges,
                    );
                }
                "function_definition" => {
                    self.process_function(&child, source, file_path, parent_id, nodes, edges);
                }
                _ => {}
            }
        }
    }

    // Legacy function call extraction methods (kept for compatibility but replaced by CallSiteExtractor)
    #[allow(dead_code)]
    fn extract_function_calls_legacy(
        &self,
        body_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        caller_id: &str,
        edges: &mut Vec<Edge>,
    ) {
        self.traverse_for_calls_legacy(body_node, source, file_path, caller_id, edges);
    }

    #[allow(dead_code)]
    fn traverse_for_calls_legacy(
        &self,
        node: &TSNode,
        source: &[u8],
        _file_path: &Path,
        caller_id: &str,
        edges: &mut Vec<Edge>,
    ) {
        match node.kind() {
            "call_expression" => {
                self.process_function_call_legacy(node, source, caller_id, edges);
            }
            _ => {
                // Recursively traverse child nodes
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.traverse_for_calls_legacy(&child, source, _file_path, caller_id, edges);
                }
            }
        }
    }

    #[allow(dead_code)]
    fn process_function_call_legacy(
        &self,
        call_node: &TSNode,
        source: &[u8],
        caller_id: &str,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(function_node) = call_node.child(0) {
            let function_name = match function_node.kind() {
                "identifier" => extract_text(&function_node, source),
                "field_expression" => {
                    // Handle member function calls like obj.method()
                    if let Some(field_node) = find_child_by_kind(&function_node, "field_identifier")
                    {
                        extract_text(&field_node, source)
                    } else {
                        extract_text(&function_node, source)
                    }
                }
                "qualified_identifier" => {
                    // Handle qualified calls like namespace::function()
                    extract_text(&function_node, source)
                }
                _ => extract_text(&function_node, source),
            };

            if !function_name.is_empty() {
                let callee_id = format!("external:function:{}:0", function_name);
                let call_edge = Edge::new(EdgeType::Call, caller_id.to_string(), callee_id);
                edges.push(call_edge);
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

    fn extract_using_declarations(
        &self,
        root: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        _edges: &mut Vec<Edge>,
    ) {
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            if child.kind() == "using_declaration" {
                let using_text = extract_text(&child, source);
                let line_number = child.start_position().row + 1;

                let using_id = generate_node_id(file_path, "using", using_text, line_number);
                let using_node = Node::new(
                    using_id,
                    using_text.to_string(),
                    NodeType::Module,
                    file_path.to_path_buf(),
                    line_number,
                    "cpp".to_string(),
                );

                nodes.push(using_node);
            }
        }
    }
}

impl LanguageParser for CppParser {
    fn parse_file(&self, file_path: &Path) -> Result<ParseResult> {
        let mut parser = TreeSitterParser::new(tree_sitter_cpp::language())?;
        let tree = parser.parse_file(file_path)?;
        let source = parser.get_source(file_path)?;
        let source_bytes = source.as_bytes();

        let root = tree.root_node();

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Extract different types of C++ constructs
        self.extract_includes(&root, source_bytes, file_path, &mut nodes, &mut edges);
        self.extract_using_declarations(&root, source_bytes, file_path, &mut nodes, &mut edges);
        self.extract_namespaces(&root, source_bytes, file_path, &mut nodes, &mut edges);
        self.extract_classes(&root, source_bytes, file_path, &mut nodes, &mut edges);

        // Extract global functions and templates
        self.extract_from_declaration_list(
            &root,
            source_bytes,
            file_path,
            "",
            &mut nodes,
            &mut edges,
        );

        // Extract call sites using the new system
        let call_sites = self.extract_call_sites(&root, source_bytes, file_path);

        Ok(ParseResult {
            nodes,
            edges,
            call_sites: Some(call_sites),
        })
    }

    fn language_name(&self) -> &str {
        "cpp"
    }
}

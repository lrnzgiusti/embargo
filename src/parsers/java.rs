use anyhow::Result;
use std::path::Path;
use tree_sitter::Node as TSNode;

use super::common::{
    extract_docstring, extract_text, find_child_by_kind, generate_node_id, TreeSitterParser,
};
use super::{LanguageParser, ParseResult};
use crate::core::{CallSite, CallSiteExtractor, Edge, EdgeType, Node, NodeType};

pub struct JavaParser {
    #[allow(dead_code)]
    parser: TreeSitterParser,
}

impl JavaParser {
    pub fn new() -> Result<Self> {
        let language = tree_sitter_java::language();
        let parser = TreeSitterParser::new(language)?;
        Ok(Self { parser })
    }

    fn extract_package(
        &self,
        root: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
    ) {
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            if child.kind() == "package_declaration" {
                if let Some(name_node) = find_child_by_kind(&child, "scoped_identifier") {
                    let package_name = extract_text(&name_node, source);
                    let line_number = child.start_position().row + 1;
                    let package_id =
                        generate_node_id(file_path, "package", &package_name, line_number);

                    let package_node = Node::new(
                        package_id,
                        package_name.to_string(),
                        NodeType::Module,
                        file_path.to_path_buf(),
                        line_number,
                        "java".to_string(),
                    );

                    nodes.push(package_node);
                }
                break; // Only one package declaration per file
            }
        }
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
            if child.kind() == "import_declaration" {
                self.process_import(&child, source, file_path, nodes);
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
            "java".to_string(),
        );

        nodes.push(import_node_obj);
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
            match child.kind() {
                "class_declaration" => {
                    self.process_class(&child, source, file_path, nodes, edges);
                }
                "enum_declaration" => {
                    self.process_enum(&child, source, file_path, nodes, edges);
                }
                _ => {}
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

            let mut class_node_obj = Node::new(
                class_id.clone(),
                class_name.to_string(),
                NodeType::Class,
                file_path.to_path_buf(),
                line_number,
                "java".to_string(),
            );

            // Extract docstring/comments
            if let Some(docstring) = extract_docstring(class_node, source) {
                class_node_obj = class_node_obj.with_docstring(docstring);
            }

            // Handle inheritance (extends)
            if let Some(superclass) = find_child_by_kind(class_node, "superclass") {
                if let Some(type_node) = find_child_by_kind(&superclass, "type_identifier") {
                    let parent_class = extract_text(&type_node, source);
                    let parent_id = format!("external:class:{}:0", parent_class);
                    let inheritance_edge =
                        Edge::new(EdgeType::Inheritance, class_id.clone(), parent_id);
                    edges.push(inheritance_edge);
                }
            }

            // Handle interfaces (implements)
            if let Some(super_interfaces) = find_child_by_kind(class_node, "super_interfaces") {
                for interface_list in super_interfaces.children(&mut super_interfaces.walk()) {
                    if interface_list.kind() == "interface_type_list" {
                        for interface in interface_list.children(&mut interface_list.walk()) {
                            if interface.kind() == "type_identifier" {
                                let interface_name = extract_text(&interface, source);
                                let interface_id =
                                    format!("external:interface:{}:0", interface_name);
                                let implements_edge =
                                    Edge::new(EdgeType::Implements, class_id.clone(), interface_id);
                                edges.push(implements_edge);
                            }
                        }
                    }
                }
            }

            nodes.push(class_node_obj);

            // Extract class members
            self.extract_class_members(class_node, source, file_path, &class_id, nodes, edges);
        }
    }

    fn process_enum(
        &self,
        enum_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(enum_node, "identifier") {
            let enum_name = extract_text(&name_node, source);
            let line_number = enum_node.start_position().row + 1;
            let enum_id = generate_node_id(file_path, "enum", &enum_name, line_number);

            let enum_node_obj = Node::new(
                enum_id.clone(),
                enum_name.to_string(),
                NodeType::Class, // Treating enums as classes for simplicity
                file_path.to_path_buf(),
                line_number,
                "java".to_string(),
            );

            nodes.push(enum_node_obj);

            // Extract enum constants
            if let Some(enum_body) = find_child_by_kind(enum_node, "enum_body") {
                for child in enum_body.children(&mut enum_body.walk()) {
                    if child.kind() == "enum_constant" {
                        if let Some(constant_name_node) = find_child_by_kind(&child, "identifier") {
                            let constant_name = extract_text(&constant_name_node, source);
                            let constant_line = child.start_position().row + 1;
                            let constant_id = generate_node_id(
                                file_path,
                                "variable",
                                &constant_name,
                                constant_line,
                            );

                            let constant_node = Node::new(
                                constant_id.clone(),
                                constant_name.to_string(),
                                NodeType::Variable,
                                file_path.to_path_buf(),
                                constant_line,
                                "java".to_string(),
                            )
                            .with_visibility("public".to_string());

                            nodes.push(constant_node);

                            let contains_edge =
                                Edge::new(EdgeType::Contains, enum_id.clone(), constant_id);
                            edges.push(contains_edge);
                        }
                    }
                }
            }
        }
    }

    fn extract_class_members(
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
                    "method_declaration" | "constructor_declaration" => {
                        self.process_method(
                            &child,
                            source,
                            file_path,
                            Some(class_id),
                            nodes,
                            edges,
                        );
                    }
                    "field_declaration" => {
                        self.process_field(&child, source, file_path, class_id, nodes, edges);
                    }
                    "class_declaration" => {
                        // Inner class
                        self.process_class(&child, source, file_path, nodes, edges);
                    }
                    _ => {}
                }
            }
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
        // Find variable declarator
        if let Some(variable_declarator) = find_child_by_kind(field_node, "variable_declarator") {
            if let Some(name_node) = find_child_by_kind(&variable_declarator, "identifier") {
                let field_name = extract_text(&name_node, source);
                let line_number = field_node.start_position().row + 1;
                let field_id = generate_node_id(file_path, "variable", &field_name, line_number);

                let mut visibility = "package".to_string(); // Default visibility

                // Check modifiers
                for child in field_node.children(&mut field_node.walk()) {
                    if child.kind() == "modifiers" {
                        let modifiers_text = extract_text(&child, source);
                        if modifiers_text.contains("public") {
                            visibility = "public".to_string();
                        } else if modifiers_text.contains("private") {
                            visibility = "private".to_string();
                        } else if modifiers_text.contains("protected") {
                            visibility = "protected".to_string();
                        }
                        break;
                    }
                }

                let field_node_obj = Node::new(
                    field_id.clone(),
                    field_name.to_string(),
                    NodeType::Variable,
                    file_path.to_path_buf(),
                    line_number,
                    "java".to_string(),
                )
                .with_visibility(visibility);

                nodes.push(field_node_obj);

                let contains_edge = Edge::new(EdgeType::Contains, class_id.to_string(), field_id);
                edges.push(contains_edge);
            }
        }
    }

    fn extract_interfaces(
        &self,
        root: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            if child.kind() == "interface_declaration" {
                self.process_interface(&child, source, file_path, nodes, edges);
            }
        }
    }

    fn process_interface(
        &self,
        interface_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(interface_node, "identifier") {
            let interface_name = extract_text(&name_node, source);
            let line_number = interface_node.start_position().row + 1;
            let interface_id =
                generate_node_id(file_path, "interface", &interface_name, line_number);

            let interface_node_obj = Node::new(
                interface_id.clone(),
                interface_name.to_string(),
                NodeType::Interface,
                file_path.to_path_buf(),
                line_number,
                "java".to_string(),
            );

            nodes.push(interface_node_obj);

            // Extract interface methods
            if let Some(interface_body) = find_child_by_kind(interface_node, "interface_body") {
                for child in interface_body.children(&mut interface_body.walk()) {
                    if child.kind() == "method_declaration" {
                        self.process_method(
                            &child,
                            source,
                            file_path,
                            Some(&interface_id),
                            nodes,
                            edges,
                        );
                    }
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
            if child.kind() == "method_declaration" {
                self.process_method(&child, source, file_path, None, nodes, edges);
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
        if let Some(name_node) = find_child_by_kind(method_node, "identifier") {
            let method_name = extract_text(&name_node, source);
            let line_number = method_node.start_position().row + 1;
            let method_id = generate_node_id(file_path, "function", &method_name, line_number);

            let mut signature = method_name.to_string();
            if let Some(params) = find_child_by_kind(method_node, "formal_parameters") {
                signature = format!("{}({})", method_name, extract_text(&params, source));
            }

            let mut visibility = "package".to_string(); // Default visibility

            // Check modifiers
            for child in method_node.children(&mut method_node.walk()) {
                if child.kind() == "modifiers" {
                    let modifiers_text = extract_text(&child, source);
                    if modifiers_text.contains("public") {
                        visibility = "public".to_string();
                    } else if modifiers_text.contains("private") {
                        visibility = "private".to_string();
                    } else if modifiers_text.contains("protected") {
                        visibility = "protected".to_string();
                    }
                    break;
                }
            }

            let mut method_node_obj = Node::new(
                method_id.clone(),
                method_name.to_string(),
                NodeType::Function,
                file_path.to_path_buf(),
                line_number,
                "java".to_string(),
            )
            .with_signature(signature)
            .with_visibility(visibility);

            if let Some(docstring) = extract_docstring(method_node, source) {
                method_node_obj = method_node_obj.with_docstring(docstring);
            }

            nodes.push(method_node_obj);

            if let Some(class_id) = class_id {
                let contains_edge = Edge::new(EdgeType::Contains, class_id.to_string(), method_id);
                edges.push(contains_edge);
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

impl LanguageParser for JavaParser {
    fn parse_file(&self, file_path: &Path) -> Result<ParseResult> {
        let mut parser = TreeSitterParser::new(tree_sitter_java::language())?;
        let tree = parser.parse_file(file_path)?;
        let source = parser.get_source(file_path)?;
        let source_bytes = source.as_bytes();

        let root_node = tree.root_node();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        self.extract_package(&root_node, source_bytes, file_path, &mut nodes);
        self.extract_imports(&root_node, source_bytes, file_path, &mut nodes);
        self.extract_classes(&root_node, source_bytes, file_path, &mut nodes, &mut edges);
        self.extract_interfaces(&root_node, source_bytes, file_path, &mut nodes, &mut edges);
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
        "java"
    }
}

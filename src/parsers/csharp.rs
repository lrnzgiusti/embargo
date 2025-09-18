use anyhow::Result;
use std::path::Path;
use tree_sitter::Node as TSNode;

use super::common::{
    extract_docstring, extract_text, find_child_by_kind, find_children_by_kind, generate_node_id,
    TreeSitterParser,
};
use super::{LanguageParser, ParseResult};
use crate::core::{CallSite, CallSiteExtractor, Edge, EdgeType, Node, NodeType};

pub struct CSharpParser {
    #[allow(dead_code)]
    parser: TreeSitterParser,
}

impl CSharpParser {
    pub fn new() -> Result<Self> {
        let language = tree_sitter_c_sharp::language();
        let parser = TreeSitterParser::new(language)?;
        Ok(Self { parser })
    }

    fn extract_using_directives(
        &self,
        root: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
    ) {
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            if child.kind() == "using_directive" {
                let using_text = extract_text(&child, source);
                let line_number = child.start_position().row + 1;

                let module_id = generate_node_id(file_path, "using", &using_text, line_number);
                let using_node = Node::new(
                    module_id,
                    using_text.to_string(),
                    NodeType::Module,
                    file_path.to_path_buf(),
                    line_number,
                    "csharp".to_string(),
                );

                nodes.push(using_node);
            }
        }
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
            if child.kind() == "namespace_declaration" {
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
                generate_node_id(file_path, "namespace", &namespace_name, line_number);

            let namespace_node_obj = Node::new(
                namespace_id.clone(),
                namespace_name.to_string(),
                NodeType::Module,
                file_path.to_path_buf(),
                line_number,
                "csharp".to_string(),
            );

            nodes.push(namespace_node_obj);

            // Extract members of the namespace
            if let Some(declaration_list) = find_child_by_kind(namespace_node, "declaration_list") {
                self.extract_namespace_members(
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

    fn extract_namespace_members(
        &self,
        declaration_list: &TSNode,
        source: &[u8],
        file_path: &Path,
        namespace_id: &str,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        for child in declaration_list.children(&mut declaration_list.walk()) {
            match child.kind() {
                "class_declaration" => {
                    self.process_class(&child, source, file_path, Some(namespace_id), nodes, edges);
                }
                "interface_declaration" => {
                    self.process_interface(
                        &child,
                        source,
                        file_path,
                        Some(namespace_id),
                        nodes,
                        edges,
                    );
                }
                "struct_declaration" => {
                    self.process_struct(
                        &child,
                        source,
                        file_path,
                        Some(namespace_id),
                        nodes,
                        edges,
                    );
                }
                "enum_declaration" => {
                    self.process_enum(&child, source, file_path, Some(namespace_id), nodes, edges);
                }
                _ => {}
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
                self.process_class(&child, source, file_path, None, nodes, edges);
            }
        }
    }

    fn process_class(
        &self,
        class_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        namespace_id: Option<&str>,
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
                "csharp".to_string(),
            );

            // Extract docstring/comments
            if let Some(docstring) = extract_docstring(class_node, source) {
                class_node_obj = class_node_obj.with_docstring(docstring);
            }

            // Handle inheritance and interfaces
            if let Some(base_list) = find_child_by_kind(class_node, "base_list") {
                for base_type in base_list.children(&mut base_list.walk()) {
                    if base_type.kind() == "identifier" || base_type.kind() == "generic_name" {
                        let base_name = extract_text(&base_type, source);
                        let base_id = format!("external:class:{}:0", base_name);

                        // For simplicity, treating all base types as inheritance
                        // In a more sophisticated parser, we'd distinguish between classes and interfaces
                        let inheritance_edge =
                            Edge::new(EdgeType::Inheritance, class_id.clone(), base_id);
                        edges.push(inheritance_edge);
                    }
                }
            }

            nodes.push(class_node_obj);

            if let Some(namespace_id) = namespace_id {
                let contains_edge = Edge::new(
                    EdgeType::Contains,
                    namespace_id.to_string(),
                    class_id.clone(),
                );
                edges.push(contains_edge);
            }

            // Extract class members
            self.extract_class_members(class_node, source, file_path, &class_id, nodes, edges);
        }
    }

    fn process_struct(
        &self,
        struct_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        namespace_id: Option<&str>,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(struct_node, "identifier") {
            let struct_name = extract_text(&name_node, source);
            let line_number = struct_node.start_position().row + 1;
            let struct_id = generate_node_id(file_path, "struct", &struct_name, line_number);

            let struct_node_obj = Node::new(
                struct_id.clone(),
                struct_name.to_string(),
                NodeType::Class, // Using Class type for structs
                file_path.to_path_buf(),
                line_number,
                "csharp".to_string(),
            );

            nodes.push(struct_node_obj);

            if let Some(namespace_id) = namespace_id {
                let contains_edge = Edge::new(
                    EdgeType::Contains,
                    namespace_id.to_string(),
                    struct_id.clone(),
                );
                edges.push(contains_edge);
            }

            // Extract struct members
            self.extract_class_members(struct_node, source, file_path, &struct_id, nodes, edges);
        }
    }

    fn process_enum(
        &self,
        enum_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        namespace_id: Option<&str>,
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
                NodeType::Class, // Using Class type for enums
                file_path.to_path_buf(),
                line_number,
                "csharp".to_string(),
            );

            nodes.push(enum_node_obj);

            if let Some(namespace_id) = namespace_id {
                let contains_edge = Edge::new(
                    EdgeType::Contains,
                    namespace_id.to_string(),
                    enum_id.clone(),
                );
                edges.push(contains_edge);
            }

            // Extract enum members
            if let Some(enum_member_declaration_list) =
                find_child_by_kind(enum_node, "enum_member_declaration_list")
            {
                for enum_member in
                    enum_member_declaration_list.children(&mut enum_member_declaration_list.walk())
                {
                    if enum_member.kind() == "enum_member_declaration" {
                        if let Some(identifier) = find_child_by_kind(&enum_member, "identifier") {
                            let member_name = extract_text(&identifier, source);
                            let member_line = enum_member.start_position().row + 1;
                            let member_id =
                                generate_node_id(file_path, "variable", &member_name, member_line);

                            let member_node = Node::new(
                                member_id.clone(),
                                member_name.to_string(),
                                NodeType::Variable,
                                file_path.to_path_buf(),
                                member_line,
                                "csharp".to_string(),
                            )
                            .with_visibility("public".to_string());

                            nodes.push(member_node);

                            let contains_edge =
                                Edge::new(EdgeType::Contains, enum_id.clone(), member_id);
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
        if let Some(declaration_list) = find_child_by_kind(class_node, "declaration_list") {
            for child in declaration_list.children(&mut declaration_list.walk()) {
                match child.kind() {
                    "method_declaration" => {
                        self.process_method(
                            &child,
                            source,
                            file_path,
                            Some(class_id),
                            nodes,
                            edges,
                        );
                    }
                    "constructor_declaration" => {
                        self.process_constructor(
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
                    "property_declaration" => {
                        self.process_property(&child, source, file_path, class_id, nodes, edges);
                    }
                    "event_declaration" => {
                        self.process_event(&child, source, file_path, class_id, nodes, edges);
                    }
                    _ => {}
                }
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
            if let Some(param_list) = find_child_by_kind(method_node, "parameter_list") {
                signature = format!("{}({})", method_name, extract_text(&param_list, source));
            }

            let visibility = self.extract_visibility_modifier(method_node, source);

            let mut method_node_obj = Node::new(
                method_id.clone(),
                method_name.to_string(),
                NodeType::Function,
                file_path.to_path_buf(),
                line_number,
                "csharp".to_string(),
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

    fn process_constructor(
        &self,
        constructor_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        class_id: Option<&str>,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(constructor_node, "identifier") {
            let constructor_name = extract_text(&name_node, source);
            let line_number = constructor_node.start_position().row + 1;
            let constructor_id =
                generate_node_id(file_path, "function", &constructor_name, line_number);

            let mut signature = constructor_name.to_string();
            if let Some(param_list) = find_child_by_kind(constructor_node, "parameter_list") {
                signature = format!(
                    "{}({})",
                    constructor_name,
                    extract_text(&param_list, source)
                );
            }

            let visibility = self.extract_visibility_modifier(constructor_node, source);

            let constructor_node_obj = Node::new(
                constructor_id.clone(),
                constructor_name.to_string(),
                NodeType::Function,
                file_path.to_path_buf(),
                line_number,
                "csharp".to_string(),
            )
            .with_signature(signature)
            .with_visibility(visibility);

            nodes.push(constructor_node_obj);

            if let Some(class_id) = class_id {
                let contains_edge =
                    Edge::new(EdgeType::Contains, class_id.to_string(), constructor_id);
                edges.push(contains_edge);
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
        if let Some(variable_declaration) = find_child_by_kind(field_node, "variable_declaration") {
            for variable_declarator in
                find_children_by_kind(&variable_declaration, "variable_declarator")
            {
                if let Some(identifier) = find_child_by_kind(&variable_declarator, "identifier") {
                    let field_name = extract_text(&identifier, source);
                    let line_number = field_node.start_position().row + 1;
                    let field_id =
                        generate_node_id(file_path, "variable", &field_name, line_number);

                    let visibility = self.extract_visibility_modifier(field_node, source);

                    let field_node_obj = Node::new(
                        field_id.clone(),
                        field_name.to_string(),
                        NodeType::Variable,
                        file_path.to_path_buf(),
                        line_number,
                        "csharp".to_string(),
                    )
                    .with_visibility(visibility);

                    nodes.push(field_node_obj);

                    let contains_edge =
                        Edge::new(EdgeType::Contains, class_id.to_string(), field_id);
                    edges.push(contains_edge);
                }
            }
        }
    }

    fn process_property(
        &self,
        property_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        class_id: &str,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(identifier) = find_child_by_kind(property_node, "identifier") {
            let property_name = extract_text(&identifier, source);
            let line_number = property_node.start_position().row + 1;
            let property_id = generate_node_id(file_path, "property", &property_name, line_number);

            let visibility = self.extract_visibility_modifier(property_node, source);

            let property_node_obj = Node::new(
                property_id.clone(),
                property_name.to_string(),
                NodeType::Variable, // Using Variable type for properties
                file_path.to_path_buf(),
                line_number,
                "csharp".to_string(),
            )
            .with_visibility(visibility);

            nodes.push(property_node_obj);

            let contains_edge = Edge::new(EdgeType::Contains, class_id.to_string(), property_id);
            edges.push(contains_edge);
        }
    }

    fn process_event(
        &self,
        event_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        class_id: &str,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(variable_declaration) = find_child_by_kind(event_node, "variable_declaration") {
            for variable_declarator in
                find_children_by_kind(&variable_declaration, "variable_declarator")
            {
                if let Some(identifier) = find_child_by_kind(&variable_declarator, "identifier") {
                    let event_name = extract_text(&identifier, source);
                    let line_number = event_node.start_position().row + 1;
                    let event_id = generate_node_id(file_path, "event", &event_name, line_number);

                    let visibility = self.extract_visibility_modifier(event_node, source);

                    let event_node_obj = Node::new(
                        event_id.clone(),
                        event_name.to_string(),
                        NodeType::Variable, // Using Variable type for events
                        file_path.to_path_buf(),
                        line_number,
                        "csharp".to_string(),
                    )
                    .with_visibility(visibility);

                    nodes.push(event_node_obj);

                    let contains_edge =
                        Edge::new(EdgeType::Contains, class_id.to_string(), event_id);
                    edges.push(contains_edge);
                }
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
                self.process_interface(&child, source, file_path, None, nodes, edges);
            }
        }
    }

    fn process_interface(
        &self,
        interface_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        namespace_id: Option<&str>,
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
                "csharp".to_string(),
            );

            nodes.push(interface_node_obj);

            if let Some(namespace_id) = namespace_id {
                let contains_edge = Edge::new(
                    EdgeType::Contains,
                    namespace_id.to_string(),
                    interface_id.clone(),
                );
                edges.push(contains_edge);
            }

            // Extract interface members
            if let Some(declaration_list) = find_child_by_kind(interface_node, "declaration_list") {
                for child in declaration_list.children(&mut declaration_list.walk()) {
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

    fn extract_visibility_modifier(&self, node: &TSNode, source: &[u8]) -> String {
        for child in node.children(&mut node.walk()) {
            if child.kind() == "modifier" {
                let modifier_text = extract_text(&child, source);
                if modifier_text.contains("public") {
                    return "public".to_string();
                } else if modifier_text.contains("private") {
                    return "private".to_string();
                } else if modifier_text.contains("protected") {
                    return "protected".to_string();
                } else if modifier_text.contains("internal") {
                    return "internal".to_string();
                }
            }
        }
        "internal".to_string() // Default C# visibility
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

impl LanguageParser for CSharpParser {
    fn parse_file(&self, file_path: &Path) -> Result<ParseResult> {
        let mut parser = TreeSitterParser::new(tree_sitter_c_sharp::language())?;
        let tree = parser.parse_file(file_path)?;
        let source = parser.get_source(file_path)?;
        let source_bytes = source.as_bytes();

        let root_node = tree.root_node();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        self.extract_using_directives(&root_node, source_bytes, file_path, &mut nodes);
        self.extract_namespaces(&root_node, source_bytes, file_path, &mut nodes, &mut edges);
        self.extract_classes(&root_node, source_bytes, file_path, &mut nodes, &mut edges);
        self.extract_interfaces(&root_node, source_bytes, file_path, &mut nodes, &mut edges);

        // Extract call sites using the new system
        let call_sites = self.extract_call_sites(&root_node, source_bytes, file_path);

        Ok(ParseResult {
            nodes,
            edges,
            call_sites: Some(call_sites),
        })
    }

    fn language_name(&self) -> &str {
        "csharp"
    }
}

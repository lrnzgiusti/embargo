use anyhow::Result;
use std::path::Path;
use tree_sitter::Node as TSNode;

use super::common::{
    extract_docstring, extract_text, find_child_by_kind, find_children_by_kind, generate_node_id,
    TreeSitterParser,
};
use super::{LanguageParser, ParseResult};
use crate::core::{CallSite, CallSiteExtractor, Edge, EdgeType, Node, NodeType};

pub struct GoParser {
    #[allow(dead_code)]
    parser: TreeSitterParser,
}

impl GoParser {
    pub fn new() -> Result<Self> {
        let language = tree_sitter_go::language();
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
            if child.kind() == "package_clause" {
                if let Some(package_identifier) = find_child_by_kind(&child, "package_identifier") {
                    let package_name = extract_text(&package_identifier, source);
                    let line_number = child.start_position().row + 1;
                    let package_id =
                        generate_node_id(file_path, "package", &package_name, line_number);

                    let package_node = Node::new(
                        package_id,
                        package_name.to_string(),
                        NodeType::Module,
                        file_path.to_path_buf(),
                        line_number,
                        "go".to_string(),
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
            match child.kind() {
                "import_declaration" => {
                    self.process_import(&child, source, file_path, nodes);
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
        // Handle single import or import group
        if let Some(import_spec_list) = find_child_by_kind(import_node, "import_spec_list") {
            // Multiple imports in a group
            for import_spec in import_spec_list.children(&mut import_spec_list.walk()) {
                if import_spec.kind() == "import_spec" {
                    self.process_single_import(&import_spec, source, file_path, nodes);
                }
            }
        } else if let Some(import_spec) = find_child_by_kind(import_node, "import_spec") {
            // Single import
            self.process_single_import(&import_spec, source, file_path, nodes);
        }
    }

    fn process_single_import(
        &self,
        import_spec: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
    ) {
        let import_text = extract_text(import_spec, source);
        let line_number = import_spec.start_position().row + 1;

        let module_id = generate_node_id(file_path, "import", &import_text, line_number);
        let import_node_obj = Node::new(
            module_id,
            import_text.to_string(),
            NodeType::Module,
            file_path.to_path_buf(),
            line_number,
            "go".to_string(),
        );

        nodes.push(import_node_obj);
    }

    fn extract_types(
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
                "type_declaration" => {
                    self.process_type_declaration(&child, source, file_path, nodes, edges);
                }
                _ => {}
            }
        }
    }

    fn process_type_declaration(
        &self,
        type_decl: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        // Handle type specs within type declaration
        if let Some(type_spec_list) = find_child_by_kind(type_decl, "type_spec_list") {
            for type_spec in type_spec_list.children(&mut type_spec_list.walk()) {
                if type_spec.kind() == "type_spec" {
                    self.process_type_spec(&type_spec, source, file_path, nodes, edges);
                }
            }
        } else if let Some(type_spec) = find_child_by_kind(type_decl, "type_spec") {
            self.process_type_spec(&type_spec, source, file_path, nodes, edges);
        }
    }

    fn process_type_spec(
        &self,
        type_spec: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(type_identifier) = find_child_by_kind(type_spec, "type_identifier") {
            let type_name = extract_text(&type_identifier, source);
            let line_number = type_spec.start_position().row + 1;

            // Determine what kind of type this is
            if let Some(type_node) = type_spec.child(2) {
                // Third child is the type definition
                match type_node.kind() {
                    "struct_type" => {
                        self.process_struct_type(
                            &type_identifier,
                            &type_node,
                            source,
                            file_path,
                            nodes,
                            edges,
                        );
                    }
                    "interface_type" => {
                        self.process_interface_type(
                            &type_identifier,
                            &type_node,
                            source,
                            file_path,
                            nodes,
                            edges,
                        );
                    }
                    _ => {
                        // Generic type alias
                        let type_id = generate_node_id(file_path, "type", &type_name, line_number);
                        let type_node_obj = Node::new(
                            type_id,
                            type_name.to_string(),
                            NodeType::Class, // Using Class for custom types
                            file_path.to_path_buf(),
                            line_number,
                            "go".to_string(),
                        );
                        nodes.push(type_node_obj);
                    }
                }
            }
        }
    }

    fn process_struct_type(
        &self,
        name_node: &TSNode,
        struct_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        let struct_name = extract_text(name_node, source);
        let line_number = struct_node.start_position().row + 1;
        let struct_id = generate_node_id(file_path, "struct", &struct_name, line_number);

        let struct_node_obj = Node::new(
            struct_id.clone(),
            struct_name.to_string(),
            NodeType::Class,
            file_path.to_path_buf(),
            line_number,
            "go".to_string(),
        );

        nodes.push(struct_node_obj);

        // Extract struct fields
        if let Some(field_declaration_list) =
            find_child_by_kind(struct_node, "field_declaration_list")
        {
            for field_decl in field_declaration_list.children(&mut field_declaration_list.walk()) {
                if field_decl.kind() == "field_declaration" {
                    self.process_struct_field(
                        &field_decl,
                        source,
                        file_path,
                        &struct_id,
                        nodes,
                        edges,
                    );
                }
            }
        }
    }

    fn process_struct_field(
        &self,
        field_decl: &TSNode,
        source: &[u8],
        file_path: &Path,
        struct_id: &str,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        // Fields can have multiple field identifiers
        for field_identifier in find_children_by_kind(field_decl, "field_identifier") {
            let field_name = extract_text(&field_identifier, source);
            let line_number = field_decl.start_position().row + 1;
            let field_id = generate_node_id(file_path, "field", &field_name, line_number);

            let field_node_obj = Node::new(
                field_id.clone(),
                field_name.to_string(),
                NodeType::Variable,
                file_path.to_path_buf(),
                line_number,
                "go".to_string(),
            )
            .with_visibility("public".to_string()); // Go fields are public if capitalized

            nodes.push(field_node_obj);

            let contains_edge = Edge::new(EdgeType::Contains, struct_id.to_string(), field_id);
            edges.push(contains_edge);
        }
    }

    fn process_interface_type(
        &self,
        name_node: &TSNode,
        interface_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        let interface_name = extract_text(name_node, source);
        let line_number = interface_node.start_position().row + 1;
        let interface_id = generate_node_id(file_path, "interface", &interface_name, line_number);

        let interface_node_obj = Node::new(
            interface_id.clone(),
            interface_name.to_string(),
            NodeType::Interface,
            file_path.to_path_buf(),
            line_number,
            "go".to_string(),
        );

        nodes.push(interface_node_obj);

        // Extract interface methods
        if let Some(method_spec_list) = find_child_by_kind(interface_node, "method_spec_list") {
            for method_spec in method_spec_list.children(&mut method_spec_list.walk()) {
                if method_spec.kind() == "method_spec" {
                    if let Some(field_identifier) =
                        find_child_by_kind(&method_spec, "field_identifier")
                    {
                        let method_name = extract_text(&field_identifier, source);
                        let method_line = method_spec.start_position().row + 1;
                        let method_id =
                            generate_node_id(file_path, "function", &method_name, method_line);

                        let method_node_obj = Node::new(
                            method_id.clone(),
                            method_name.to_string(),
                            NodeType::Function,
                            file_path.to_path_buf(),
                            method_line,
                            "go".to_string(),
                        );

                        nodes.push(method_node_obj);

                        let contains_edge =
                            Edge::new(EdgeType::Contains, interface_id.clone(), method_id);
                        edges.push(contains_edge);
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
            if child.kind() == "function_declaration" {
                self.process_function(&child, source, file_path, nodes, edges);
            } else if child.kind() == "method_declaration" {
                self.process_method(&child, source, file_path, nodes, edges);
            }
        }
    }

    fn process_function(
        &self,
        func_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        _edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(func_node, "identifier") {
            let func_name = extract_text(&name_node, source);
            let line_number = func_node.start_position().row + 1;
            let func_id = generate_node_id(file_path, "function", &func_name, line_number);

            let mut signature = func_name.to_string();
            if let Some(param_list) = find_child_by_kind(func_node, "parameter_list") {
                signature = format!("{}({})", func_name, extract_text(&param_list, source));
            }

            let mut func_node_obj = Node::new(
                func_id,
                func_name.to_string(),
                NodeType::Function,
                file_path.to_path_buf(),
                line_number,
                "go".to_string(),
            )
            .with_signature(signature);

            if let Some(docstring) = extract_docstring(func_node, source) {
                func_node_obj = func_node_obj.with_docstring(docstring);
            }

            nodes.push(func_node_obj);
        }
    }

    fn process_method(
        &self,
        method_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(method_node, "field_identifier") {
            let method_name = extract_text(&name_node, source);
            let line_number = method_node.start_position().row + 1;
            let method_id = generate_node_id(file_path, "function", &method_name, line_number);

            let mut signature = method_name.to_string();
            if let Some(param_list) = find_child_by_kind(method_node, "parameter_list") {
                signature = format!("{}({})", method_name, extract_text(&param_list, source));
            }

            let mut method_node_obj = Node::new(
                method_id.clone(),
                method_name.to_string(),
                NodeType::Function,
                file_path.to_path_buf(),
                line_number,
                "go".to_string(),
            )
            .with_signature(signature);

            if let Some(docstring) = extract_docstring(method_node, source) {
                method_node_obj = method_node_obj.with_docstring(docstring);
            }

            nodes.push(method_node_obj);

            // Extract receiver type and create edge
            if let Some(receiver) = find_child_by_kind(method_node, "parameter_list") {
                // The first parameter_list is the receiver for methods
                if let Some(param_decl) = receiver.child(1) {
                    // Skip opening parenthesis
                    if param_decl.kind() == "parameter_declaration" {
                        // Look for type identifier in the receiver
                        if let Some(type_id) = find_child_by_kind(&param_decl, "type_identifier") {
                            let receiver_type = extract_text(&type_id, source);
                            let receiver_type_id = format!("external:struct:{}:0", receiver_type);
                            let contains_edge =
                                Edge::new(EdgeType::Contains, receiver_type_id, method_id);
                            edges.push(contains_edge);
                        } else if let Some(pointer_type) =
                            find_child_by_kind(&param_decl, "pointer_type")
                        {
                            if let Some(type_id) =
                                find_child_by_kind(&pointer_type, "type_identifier")
                            {
                                let receiver_type = extract_text(&type_id, source);
                                let receiver_type_id =
                                    format!("external:struct:{}:0", receiver_type);
                                let contains_edge =
                                    Edge::new(EdgeType::Contains, receiver_type_id, method_id);
                                edges.push(contains_edge);
                            }
                        }
                    }
                }
            }
        }
    }

    fn extract_variables(
        &self,
        root: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
    ) {
        let mut cursor = root.walk();

        for child in root.children(&mut cursor) {
            if child.kind() == "var_declaration" {
                self.process_var_declaration(&child, source, file_path, nodes);
            } else if child.kind() == "const_declaration" {
                self.process_const_declaration(&child, source, file_path, nodes);
            }
        }
    }

    fn process_var_declaration(
        &self,
        var_decl: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
    ) {
        if let Some(var_spec_list) = find_child_by_kind(var_decl, "var_spec_list") {
            for var_spec in var_spec_list.children(&mut var_spec_list.walk()) {
                if var_spec.kind() == "var_spec" {
                    self.process_var_spec(&var_spec, source, file_path, nodes);
                }
            }
        } else if let Some(var_spec) = find_child_by_kind(var_decl, "var_spec") {
            self.process_var_spec(&var_spec, source, file_path, nodes);
        }
    }

    fn process_const_declaration(
        &self,
        const_decl: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
    ) {
        if let Some(const_spec_list) = find_child_by_kind(const_decl, "const_spec_list") {
            for const_spec in const_spec_list.children(&mut const_spec_list.walk()) {
                if const_spec.kind() == "const_spec" {
                    self.process_const_spec(&const_spec, source, file_path, nodes);
                }
            }
        } else if let Some(const_spec) = find_child_by_kind(const_decl, "const_spec") {
            self.process_const_spec(&const_spec, source, file_path, nodes);
        }
    }

    fn process_var_spec(
        &self,
        var_spec: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
    ) {
        for identifier in find_children_by_kind(var_spec, "identifier") {
            let var_name = extract_text(&identifier, source);
            let line_number = var_spec.start_position().row + 1;
            let var_id = generate_node_id(file_path, "variable", &var_name, line_number);

            let var_node_obj = Node::new(
                var_id,
                var_name.to_string(),
                NodeType::Variable,
                file_path.to_path_buf(),
                line_number,
                "go".to_string(),
            );

            nodes.push(var_node_obj);
        }
    }

    fn process_const_spec(
        &self,
        const_spec: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
    ) {
        for identifier in find_children_by_kind(const_spec, "identifier") {
            let const_name = extract_text(&identifier, source);
            let line_number = const_spec.start_position().row + 1;
            let const_id = generate_node_id(file_path, "variable", &const_name, line_number);

            let const_node_obj = Node::new(
                const_id,
                const_name.to_string(),
                NodeType::Variable,
                file_path.to_path_buf(),
                line_number,
                "go".to_string(),
            )
            .with_visibility("public".to_string()); // Constants are typically public if capitalized

            nodes.push(const_node_obj);
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

impl LanguageParser for GoParser {
    fn parse_file(&self, file_path: &Path) -> Result<ParseResult> {
        let mut parser = TreeSitterParser::new(tree_sitter_go::language())?;
        let tree = parser.parse_file(file_path)?;
        let source = parser.get_source(file_path)?;
        let source_bytes = source.as_bytes();

        let root_node = tree.root_node();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        self.extract_package(&root_node, source_bytes, file_path, &mut nodes);
        self.extract_imports(&root_node, source_bytes, file_path, &mut nodes);
        self.extract_types(&root_node, source_bytes, file_path, &mut nodes, &mut edges);
        self.extract_functions(&root_node, source_bytes, file_path, &mut nodes, &mut edges);
        self.extract_variables(&root_node, source_bytes, file_path, &mut nodes);

        // Extract call sites using the new system
        let call_sites = self.extract_call_sites(&root_node, source_bytes, file_path);

        Ok(ParseResult {
            nodes,
            edges,
            call_sites: Some(call_sites),
        })
    }

    fn language_name(&self) -> &str {
        "go"
    }
}

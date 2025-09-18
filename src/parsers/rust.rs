use anyhow::Result;
use std::path::Path;
use tree_sitter::Node as TSNode;

use super::common::{
    extract_docstring, extract_text, find_child_by_kind, find_children_by_kind, generate_node_id,
    TreeSitterParser,
};
use super::{LanguageParser, ParseResult};
use crate::core::{CallSite, CallSiteExtractor, Edge, EdgeType, Node, NodeType};

pub struct RustParser {
    #[allow(dead_code)]
    parser: TreeSitterParser,
}

impl RustParser {
    pub fn new() -> Result<Self> {
        let language = tree_sitter_rust::language();
        let parser = TreeSitterParser::new(language)?;
        Ok(Self { parser })
    }

    /// Extract complete function signature including visibility, generics, parameters, and return type
    fn extract_complete_function_signature(
        &self,
        func_node: &TSNode,
        source: &[u8],
        func_name: &str,
    ) -> String {
        let mut signature_parts = Vec::new();

        // Extract visibility (pub, pub(crate), etc.)
        if let Some(visibility) = self.extract_visibility(func_node, source) {
            signature_parts.push(visibility);
        }

        // Extract function keyword
        signature_parts.push("fn".to_string());

        // Extract function name
        signature_parts.push(func_name.to_string());

        // Extract generic parameters
        if let Some(generics) = self.extract_generics(func_node, source) {
            signature_parts.push(generics);
        }

        // Extract parameters
        let params = if let Some(params_node) = find_child_by_kind(func_node, "parameters") {
            extract_text(&params_node, source).to_string()
        } else {
            "()".to_string()
        };
        signature_parts.push(params);

        // Extract return type
        if let Some(return_type) = self.extract_return_type(func_node, source) {
            signature_parts.push(return_type);
        }

        signature_parts.join(" ")
    }

    /// Extract visibility modifier
    fn extract_visibility(&self, func_node: &TSNode, source: &[u8]) -> Option<String> {
        // Look for visibility modifiers in the function declaration
        let mut cursor = func_node.walk();
        for child in func_node.children(&mut cursor) {
            match child.kind() {
                "visibility_modifier" => {
                    return Some(extract_text(&child, source).to_string());
                }
                "function_item" => {
                    // Check parent for visibility
                    if let Some(parent) = child.parent() {
                        return self.extract_visibility(&parent, source);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Extract generic parameters
    fn extract_generics(&self, func_node: &TSNode, source: &[u8]) -> Option<String> {
        if let Some(generics_node) = find_child_by_kind(func_node, "type_parameters") {
            Some(extract_text(&generics_node, source).to_string())
        } else {
            None
        }
    }

    /// Extract return type
    fn extract_return_type(&self, func_node: &TSNode, source: &[u8]) -> Option<String> {
        if let Some(return_type_node) = find_child_by_kind(func_node, "type_annotation") {
            Some(extract_text(&return_type_node, source).to_string())
        } else {
            None
        }
    }

    fn extract_modules(
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
                "mod_item" => {
                    self.process_module(&child, source, file_path, nodes, edges);
                }
                "use_declaration" => {
                    self.process_use_declaration(&child, source, file_path, nodes, edges);
                }
                _ => {}
            }
        }
    }

    fn process_module(
        &self,
        mod_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        _edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(mod_node, "identifier") {
            let mod_name = extract_text(&name_node, source);
            let line_number = mod_node.start_position().row + 1;

            let module_id = generate_node_id(file_path, "module", mod_name, line_number);
            let module_node = Node::new(
                module_id.clone(),
                mod_name.to_string(),
                NodeType::Module,
                file_path.to_path_buf(),
                line_number,
                "rust".to_string(),
            );

            nodes.push(module_node);
        }
    }

    fn process_use_declaration(
        &self,
        use_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        _edges: &mut Vec<Edge>,
    ) {
        let use_text = extract_text(use_node, source);
        let line_number = use_node.start_position().row + 1;

        let import_id = generate_node_id(file_path, "import", &use_text, line_number);
        let import_node = Node::new(
            import_id.clone(),
            use_text.to_string(),
            NodeType::Module,
            file_path.to_path_buf(),
            line_number,
            "rust".to_string(),
        );

        nodes.push(import_node);
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
            if child.kind() == "function_item" {
                self.process_function(&child, source, file_path, nodes, edges);
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

            // Extract complete function signature
            let signature = self.extract_complete_function_signature(func_node, source, func_name);

            // Extract documentation if available
            let documentation = extract_docstring(func_node, source);

            let func_id = generate_node_id(file_path, "function", func_name, line_number);
            let func_node_obj = Node::new(
                func_id.clone(),
                func_name.to_string(),
                NodeType::Function,
                file_path.to_path_buf(),
                line_number,
                "rust".to_string(),
            )
            .with_signature(signature)
            .with_docstring(documentation.unwrap_or_default());

            nodes.push(func_node_obj);
        }
    }

    fn extract_structs(
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
                "struct_item" => {
                    self.process_struct(&child, source, file_path, nodes, edges);
                }
                "enum_item" => {
                    self.process_enum(&child, source, file_path, nodes, edges);
                }
                "trait_item" => {
                    self.process_trait(&child, source, file_path, nodes, edges);
                }
                "impl_item" => {
                    self.process_impl(&child, source, file_path, nodes, edges);
                }
                _ => {}
            }
        }
    }

    fn process_struct(
        &self,
        struct_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(struct_node, "type_identifier") {
            let struct_name = extract_text(&name_node, source);
            let line_number = struct_node.start_position().row + 1;

            let documentation = extract_docstring(struct_node, source);

            let struct_id = generate_node_id(file_path, "struct", struct_name, line_number);
            let struct_node_obj = Node::new(
                struct_id.clone(),
                struct_name.to_string(),
                NodeType::Class,
                file_path.to_path_buf(),
                line_number,
                "rust".to_string(),
            )
            .with_docstring(documentation.unwrap_or_default());

            nodes.push(struct_node_obj);

            // Extract struct fields
            if let Some(field_list) = find_child_by_kind(struct_node, "field_declaration_list") {
                self.extract_struct_fields(
                    &field_list,
                    source,
                    file_path,
                    &struct_id,
                    nodes,
                    edges,
                );
            }
        }
    }

    fn extract_struct_fields(
        &self,
        field_list: &TSNode,
        source: &[u8],
        file_path: &Path,
        struct_id: &str,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        let field_nodes = find_children_by_kind(field_list, "field_declaration");

        for field_node in field_nodes {
            if let Some(name_node) = find_child_by_kind(&field_node, "field_identifier") {
                let field_name = extract_text(&name_node, source);
                let line_number = field_node.start_position().row + 1;

                let field_id = generate_node_id(file_path, "field", field_name, line_number);
                let field_node_obj = Node::new(
                    field_id.clone(),
                    field_name.to_string(),
                    NodeType::Variable,
                    file_path.to_path_buf(),
                    line_number,
                    "rust".to_string(),
                );

                nodes.push(field_node_obj);

                // Create edge from struct to field
                let edge = Edge::new(EdgeType::Contains, struct_id.to_string(), field_id);
                edges.push(edge);
            }
        }
    }

    fn process_enum(
        &self,
        enum_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        _edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(enum_node, "type_identifier") {
            let enum_name = extract_text(&name_node, source);
            let line_number = enum_node.start_position().row + 1;

            let documentation = extract_docstring(enum_node, source);

            let enum_id = generate_node_id(file_path, "enum", enum_name, line_number);
            let enum_node_obj = Node::new(
                enum_id.clone(),
                enum_name.to_string(),
                NodeType::Class,
                file_path.to_path_buf(),
                line_number,
                "rust".to_string(),
            )
            .with_docstring(documentation.unwrap_or_default());

            nodes.push(enum_node_obj);
        }
    }

    fn process_trait(
        &self,
        trait_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(trait_node, "type_identifier") {
            let trait_name = extract_text(&name_node, source);
            let line_number = trait_node.start_position().row + 1;

            let documentation = extract_docstring(trait_node, source);

            let trait_id = generate_node_id(file_path, "trait", trait_name, line_number);
            let trait_node_obj = Node::new(
                trait_id.clone(),
                trait_name.to_string(),
                NodeType::Interface,
                file_path.to_path_buf(),
                line_number,
                "rust".to_string(),
            )
            .with_docstring(documentation.unwrap_or_default());

            nodes.push(trait_node_obj);

            // Extract trait methods
            if let Some(declaration_list) = find_child_by_kind(trait_node, "declaration_list") {
                self.extract_trait_methods(
                    &declaration_list,
                    source,
                    file_path,
                    &trait_id,
                    nodes,
                    edges,
                );
            }
        }
    }

    fn extract_trait_methods(
        &self,
        declaration_list: &TSNode,
        source: &[u8],
        file_path: &Path,
        trait_id: &str,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        let function_nodes = find_children_by_kind(declaration_list, "function_signature_item");

        for func_node in function_nodes {
            if let Some(name_node) = find_child_by_kind(&func_node, "identifier") {
                let method_name = extract_text(&name_node, source);
                let line_number = func_node.start_position().row + 1;

                let method_id = generate_node_id(file_path, "method", method_name, line_number);
                let method_node_obj = Node::new(
                    method_id.clone(),
                    method_name.to_string(),
                    NodeType::Function,
                    file_path.to_path_buf(),
                    line_number,
                    "rust".to_string(),
                );

                nodes.push(method_node_obj);

                // Create edge from trait to method
                let edge = Edge::new(EdgeType::Contains, trait_id.to_string(), method_id);
                edges.push(edge);
            }
        }
    }

    fn process_impl(
        &self,
        impl_node: &TSNode,
        source: &[u8],
        file_path: &Path,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
    ) {
        // Extract impl blocks for structs/enums/traits
        if let Some(type_node) = find_child_by_kind(impl_node, "type_identifier") {
            let type_name = extract_text(&type_node, source);

            // Extract methods in impl block
            if let Some(declaration_list) = find_child_by_kind(impl_node, "declaration_list") {
                self.extract_impl_methods(
                    &declaration_list,
                    source,
                    file_path,
                    &type_name,
                    nodes,
                    edges,
                );
            }
        }
    }

    fn extract_impl_methods(
        &self,
        declaration_list: &TSNode,
        source: &[u8],
        file_path: &Path,
        type_name: &str,
        nodes: &mut Vec<Node>,
        _edges: &mut Vec<Edge>,
    ) {
        let function_nodes = find_children_by_kind(declaration_list, "function_item");

        for func_node in function_nodes {
            if let Some(name_node) = find_child_by_kind(&func_node, "identifier") {
                let method_name = extract_text(&name_node, source);
                let line_number = func_node.start_position().row + 1;

                // Extract complete function signature
                let full_signature =
                    self.extract_complete_function_signature(&func_node, source, method_name);
                let signature = format!("{}::{}", type_name, full_signature);

                let documentation = extract_docstring(&func_node, source);

                let method_id = generate_node_id(
                    file_path,
                    "method",
                    &format!("{}::{}", type_name, method_name),
                    line_number,
                );
                let method_node_obj = Node::new(
                    method_id.clone(),
                    method_name.to_string(),
                    NodeType::Function,
                    file_path.to_path_buf(),
                    line_number,
                    "rust".to_string(),
                )
                .with_signature(signature)
                .with_docstring(documentation.unwrap_or_default());

                nodes.push(method_node_obj);

                // Try to create edge from type to method (if type exists in nodes)
                // This would require a more sophisticated approach to track type definitions
            }
        }
    }

    fn extract_call_sites(&self, root: &TSNode, source: &[u8], file_path: &Path) -> Vec<CallSite> {
        let mut extractor = CallSiteExtractor::new();
        extractor.extract_from_ast(root, source, file_path)
    }
}

impl LanguageParser for RustParser {
    fn parse_file(&self, file_path: &Path) -> Result<ParseResult> {
        let source = std::fs::read(file_path)?;
        let mut parser = TreeSitterParser::new(tree_sitter_rust::language())?;
        let tree = parser.parse_file(file_path)?;
        let root = tree.root_node();

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Extract different types of nodes
        self.extract_modules(&root, &source, file_path, &mut nodes, &mut edges);
        self.extract_functions(&root, &source, file_path, &mut nodes, &mut edges);
        self.extract_structs(&root, &source, file_path, &mut nodes, &mut edges);

        // Extract function call sites for advanced resolution
        let call_sites = self.extract_call_sites(&root, &source, file_path);

        Ok(ParseResult {
            nodes,
            edges,
            call_sites: Some(call_sites),
        })
    }

    #[allow(dead_code)]
    fn language_name(&self) -> &str {
        "rust"
    }
}

use anyhow::Result;
use std::path::Path;
use tree_sitter::Node as TSNode;

use super::common::{extract_text, find_child_by_kind, generate_node_id, TreeSitterParser};
use super::{LanguageParser, ParseResult};
use crate::core::{CallSite, CallSiteExtractor, Edge, EdgeType, Node, NodeType};

pub struct TypeScriptParser {
    #[allow(dead_code)]
    parser: TreeSitterParser,
}

impl TypeScriptParser {
    pub fn new() -> Result<Self> {
        let language = tree_sitter_typescript::language_typescript();
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
                "import_statement" => {
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
        let import_node_obj = Node::new(
            module_id.clone(),
            import_text.to_string(),
            NodeType::Module,
            file_path.to_path_buf(),
            line_number,
            "typescript".to_string(),
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
        if let Some(name_node) = find_child_by_kind(class_node, "type_identifier") {
            let class_name = extract_text(&name_node, source);
            let line_number = class_node.start_position().row + 1;
            let class_id = generate_node_id(file_path, "class", class_name, line_number);

            let class_node_obj = Node::new(
                class_id.clone(),
                class_name.to_string(),
                NodeType::Class,
                file_path.to_path_buf(),
                line_number,
                "typescript".to_string(),
            );

            if let Some(class_heritage) = find_child_by_kind(class_node, "class_heritage") {
                for heritage_clause in class_heritage.children(&mut class_heritage.walk()) {
                    if heritage_clause.kind() == "extends_clause" {
                        if let Some(parent_type) =
                            find_child_by_kind(&heritage_clause, "type_identifier")
                        {
                            let parent_class = extract_text(&parent_type, source);
                            let parent_id = format!("external:class:{}:0", parent_class);
                            let inheritance_edge =
                                Edge::new(EdgeType::Inheritance, class_id.clone(), parent_id);
                            edges.push(inheritance_edge);
                        }
                    } else if heritage_clause.kind() == "implements_clause" {
                        if let Some(interface_type) =
                            find_child_by_kind(&heritage_clause, "type_identifier")
                        {
                            let interface_name = extract_text(&interface_type, source);
                            let interface_id = format!("external:interface:{}:0", interface_name);
                            let implements_edge =
                                Edge::new(EdgeType::Implements, class_id.clone(), interface_id);
                            edges.push(implements_edge);
                        }
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
                    "public_field_definition" | "private_field_definition" => {
                        if let Some(name_node) = find_child_by_kind(&child, "property_identifier") {
                            let field_name = extract_text(&name_node, source);
                            let line_number = child.start_position().row + 1;
                            let field_id =
                                generate_node_id(file_path, "variable", field_name, line_number);

                            let field_node = Node::new(
                                field_id.clone(),
                                field_name.to_string(),
                                NodeType::Variable,
                                file_path.to_path_buf(),
                                line_number,
                                "typescript".to_string(),
                            )
                            .with_visibility(
                                if child.kind() == "private_field_definition" {
                                    "private"
                                } else {
                                    "public"
                                }
                                .to_string(),
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
        _edges: &mut Vec<Edge>,
    ) {
        if let Some(name_node) = find_child_by_kind(interface_node, "type_identifier") {
            let interface_name = extract_text(&name_node, source);
            let line_number = interface_node.start_position().row + 1;
            let interface_id =
                generate_node_id(file_path, "interface", interface_name, line_number);

            let interface_node_obj = Node::new(
                interface_id.clone(),
                interface_name.to_string(),
                NodeType::Interface,
                file_path.to_path_buf(),
                line_number,
                "typescript".to_string(),
            );

            nodes.push(interface_node_obj);
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
                // Handle const/let declarations which have direct variable_declarator children
                "lexical_declaration" | "variable_statement" => {
                    // First try to find nested variable_declaration (for var statements)
                    if let Some(var_decl) = find_child_by_kind(&child, "variable_declaration") {
                        for declarator in
                            Self::collect_descendants_by_kind(&var_decl, "variable_declarator")
                        {
                            if let Some(val) = declarator.child_by_field_name("value") {
                                if val.kind() == "arrow_function" {
                                    self.process_arrow_function(
                                        &declarator,
                                        &val,
                                        source,
                                        file_path,
                                        nodes,
                                        edges,
                                    );
                                    continue;
                                }
                            }
                            if let Some(init) = find_child_by_kind(&declarator, "arrow_function") {
                                self.process_arrow_function(
                                    &declarator,
                                    &init,
                                    source,
                                    file_path,
                                    nodes,
                                    edges,
                                );
                            }
                        }
                    } else {
                        // Handle lexical_declaration with direct variable_declarator children (const/let)
                        for declarator in
                            Self::collect_descendants_by_kind(&child, "variable_declarator")
                        {
                            if let Some(val) = declarator.child_by_field_name("value") {
                                if val.kind() == "arrow_function" {
                                    self.process_arrow_function(
                                        &declarator,
                                        &val,
                                        source,
                                        file_path,
                                        nodes,
                                        edges,
                                    );
                                    continue;
                                }
                            }
                            if let Some(init) = find_child_by_kind(&declarator, "arrow_function") {
                                self.process_arrow_function(
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
                }
                "variable_declaration" => {
                    for declarator in
                        Self::collect_descendants_by_kind(&child, "variable_declarator")
                    {
                        // Prefer field-based lookup: value == arrow_function
                        if let Some(val) = declarator.child_by_field_name("value") {
                            if val.kind() == "arrow_function" {
                                self.process_arrow_function(
                                    &declarator,
                                    &val,
                                    source,
                                    file_path,
                                    nodes,
                                    edges,
                                );
                                continue;
                            }
                        }
                        // Fallback: direct child match
                        if let Some(init) = find_child_by_kind(&declarator, "arrow_function") {
                            self.process_arrow_function(
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
                _ => {}
            }
        }
    }

    /// Recursively collect descendants by kind for robust traversal
    fn collect_descendants_by_kind<'a>(node: &'a TSNode<'a>, kind: &str) -> Vec<TSNode<'a>> {
        let mut results = Vec::new();
        let mut stack: Vec<TSNode<'a>> = Vec::new();
        stack.push(*node);

        while let Some(n) = stack.pop() {
            let mut cursor = n.walk();
            for child in n.children(&mut cursor) {
                if child.kind() == kind {
                    results.push(child);
                }
                stack.push(child);
            }
        }
        results
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
            let func_id = generate_node_id(file_path, "function", func_name, line_number);

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
                "typescript".to_string(),
            )
            .with_signature(signature);

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
            let method_id = generate_node_id(file_path, "function", method_name, line_number);

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
                "typescript".to_string(),
            )
            .with_signature(signature);

            nodes.push(method_node_obj);

            if let Some(class_id) = class_id {
                let contains_edge =
                    Edge::new(EdgeType::Contains, class_id.to_string(), method_id.clone());
                edges.push(contains_edge);
            }

            // Note: Function calls are now extracted separately via extract_call_sites
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
        // Prefer field-based name extraction for robustness
        let name_node_opt = declarator
            .child_by_field_name("name")
            .or_else(|| find_child_by_kind(declarator, "identifier"));

        if let Some(name_node) = name_node_opt {
            let func_name = extract_text(&name_node, source);
            let line_number = declarator.start_position().row + 1;
            let func_id = generate_node_id(file_path, "function", func_name, line_number);

            let func_node_obj = Node::new(
                func_id.clone(),
                func_name.to_string(),
                NodeType::Function,
                file_path.to_path_buf(),
                line_number,
                "typescript".to_string(),
            );

            nodes.push(func_node_obj);

            // Note: Function calls are now extracted separately via extract_call_sites
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

impl LanguageParser for TypeScriptParser {
    fn parse_file(&self, file_path: &Path) -> Result<ParseResult> {
        let mut parser = TreeSitterParser::new(tree_sitter_typescript::language_typescript())?;
        let tree = parser.parse_file(file_path)?;
        let source = parser.get_source(file_path)?;
        let source_bytes = source.as_bytes();

        let root_node = tree.root_node();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        self.extract_imports(&root_node, source_bytes, file_path, &mut nodes, &mut edges);
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
        "typescript"
    }
}

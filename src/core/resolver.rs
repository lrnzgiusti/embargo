use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::{Edge, EdgeType, Node, NodeType};

/// Fast hash-based function call resolver that maps function calls to their definitions
#[derive(Debug, Clone)]
pub struct FunctionResolver {
    /// Hash map for O(1) function name lookup
    function_index: HashMap<u64, Vec<FunctionEntry>>,

    /// Method resolution for class.method calls
    method_index: HashMap<u64, Vec<MethodEntry>>,

    /// Import mapping for qualified names (module.function)
    import_mapping: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct FunctionEntry {
    pub node_id: String,
    pub name: String,
    #[allow(dead_code)]
    pub file_path: PathBuf,
    #[allow(dead_code)]
    pub line_number: usize,
    #[allow(dead_code)]
    pub signature: Option<String>,
    #[allow(dead_code)]
    pub class_context: Option<String>,
    #[allow(dead_code)]
    pub module_context: String,
}

#[derive(Debug, Clone)]
pub struct MethodEntry {
    #[allow(dead_code)]
    pub node_id: String,
    pub name: String,
    #[allow(dead_code)]
    pub class_name: String,
    #[allow(dead_code)]
    pub file_path: PathBuf,
    #[allow(dead_code)]
    pub line_number: usize,
    #[allow(dead_code)]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CallSite {
    pub caller_id: String,
    pub called_name: String,
    pub call_type: CallType,
    pub context: Option<String>,
    pub line_number: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CallType {
    SimpleCall,    // function_name()
    MethodCall,    // obj.method()
    QualifiedCall, // module.function()
    #[allow(dead_code)]
    AttributeCall, // obj.attr.method()
    DynamicCall,   // var_name() where var_name is computed
    ConstructorCall, // new ClassName() or ClassName()
}

impl FunctionResolver {
    pub fn new() -> Self {
        Self {
            function_index: HashMap::new(),
            method_index: HashMap::new(),
            import_mapping: HashMap::new(),
        }
    }

    /// Build indexes from all parsed nodes for fast lookup
    pub fn build_indexes(&mut self, nodes: &[Node]) -> Result<()> {
        // Pre-calculate capacity to avoid rehashing
        let estimated_functions = nodes.len() / 4; // Rough estimate

        // Clear and reserve capacity
        self.function_index.clear();
        self.function_index.reserve(estimated_functions);
        self.method_index.clear();
        self.method_index.reserve(estimated_functions);
        self.import_mapping.clear();

        // Build function and method indexes in parallel with better allocation
        let function_nodes: Vec<_> = nodes
            .par_iter()
            .filter(|node| matches!(node.node_type, NodeType::Function))
            .collect();

        let (functions, methods): (Vec<_>, Vec<_>) = function_nodes
            .par_iter()
            .map(|node| self.create_function_entry(node))
            .partition(|entry| entry.is_function());

        // Build function index
        for entry in functions {
            if let FunctionOrMethod::Function(func) = entry {
                let hash = Self::compute_hash(&func.name);
                self.function_index
                    .entry(hash)
                    .or_insert_with(Vec::new)
                    .push(func);
            }
        }

        // Build method index
        for entry in methods {
            if let FunctionOrMethod::Method(method) = entry {
                let hash = Self::compute_hash(&method.name);
                self.method_index
                    .entry(hash)
                    .or_insert_with(Vec::new)
                    .push(method);
            }
        }

        // Build import mapping
        self.build_import_mapping(nodes)?;

        Ok(())
    }

    /// Resolve function calls to their definitions and create edges
    pub fn resolve_calls(&self, call_sites: &[CallSite]) -> Vec<Edge> {
        call_sites
            .par_iter()
            .filter_map(|call_site| self.resolve_single_call(call_site))
            .collect()
    }

    /// Resolve a single function call with multiple strategies
    #[allow(dead_code)]
    fn resolve_single_call(&self, call_site: &CallSite) -> Option<Edge> {
        match call_site.call_type {
            CallType::SimpleCall => self.resolve_simple_call(call_site),
            CallType::MethodCall => self.resolve_method_call(call_site),
            CallType::QualifiedCall => self.resolve_qualified_call(call_site),
            CallType::AttributeCall => self.resolve_attribute_call(call_site),
            CallType::DynamicCall => self.resolve_dynamic_call(call_site),
            CallType::ConstructorCall => self.resolve_constructor_call(call_site),
        }
    }

    #[allow(dead_code)]
    fn resolve_simple_call(&self, call_site: &CallSite) -> Option<Edge> {
        let hash = Self::compute_hash(&call_site.called_name);

        // Try exact match first
        if let Some(candidates) = self.function_index.get(&hash) {
            // Prefer functions in the same file/module
            let best_candidate = self.select_best_candidate(candidates, call_site)?;

            return Some(
                Edge::new(
                    EdgeType::Call,
                    call_site.caller_id.clone(),
                    best_candidate.node_id.clone(),
                )
                .with_context(format!("line:{}", call_site.line_number)),
            );
        }

        // Try fuzzy matching for typos/variations
        self.fuzzy_resolve_function(call_site)
    }

    #[allow(dead_code)]
    fn resolve_method_call(&self, call_site: &CallSite) -> Option<Edge> {
        let method_name = self.extract_method_name(&call_site.called_name)?;
        let hash = Self::compute_hash(&method_name);

        if let Some(candidates) = self.method_index.get(&hash) {
            // Try to determine the class context from the call site
            let class_context = self.infer_class_context(call_site);
            let best_candidate = self.select_best_method_candidate(candidates, &class_context)?;

            return Some(
                Edge::new(
                    EdgeType::Call,
                    call_site.caller_id.clone(),
                    best_candidate.node_id.clone(),
                )
                .with_context(format!("method_call:line:{}", call_site.line_number)),
            );
        }

        None
    }

    #[allow(dead_code)]
    fn resolve_qualified_call(&self, call_site: &CallSite) -> Option<Edge> {
        let parts: Vec<&str> = call_site.called_name.split('.').collect();
        if parts.len() < 2 {
            return self.resolve_simple_call(call_site);
        }

        let module_name = parts[..parts.len() - 1].join(".");
        let function_name = parts[parts.len() - 1];

        // Check import mapping first
        if let Some(resolved_module) = self.import_mapping.get(&module_name) {
            let full_name = format!("{}.{}", resolved_module, function_name);
            return self.resolve_by_full_name(&full_name, call_site);
        }

        // Try direct module resolution
        self.resolve_by_module_and_function(&module_name, function_name, call_site)
    }

    #[allow(dead_code)]
    fn resolve_attribute_call(&self, call_site: &CallSite) -> Option<Edge> {
        // For complex attribute chains like obj.attr.method()
        // We need to trace through the attribute access chain
        let parts: Vec<&str> = call_site.called_name.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        let method_name = parts[parts.len() - 1];
        let hash = Self::compute_hash(&method_name);

        if let Some(candidates) = self.method_index.get(&hash) {
            // Use heuristics to select the most likely candidate
            let best_candidate = self.select_attribute_candidate(candidates, call_site)?;

            return Some(
                Edge::new(
                    EdgeType::Call,
                    call_site.caller_id.clone(),
                    best_candidate.node_id.clone(),
                )
                .with_context(format!("attribute_call:line:{}", call_site.line_number)),
            );
        }

        None
    }

    #[allow(dead_code)]
    fn resolve_dynamic_call(&self, call_site: &CallSite) -> Option<Edge> {
        // For dynamic calls where the function name is computed at runtime
        // We can try pattern matching or maintain a list of likely candidates

        // Try common dynamic call patterns
        if call_site.called_name.contains("getattr") || call_site.called_name.contains("__call__") {
            return self.resolve_dynamic_patterns(call_site);
        }

        // If we have context about the variable, try to resolve it
        if let Some(context) = &call_site.context {
            return self.resolve_with_context(call_site, context);
        }

        None
    }

    #[allow(dead_code)]
    fn resolve_constructor_call(&self, call_site: &CallSite) -> Option<Edge> {
        // For constructor calls like "new ClassName()" or direct instantiation
        // Try to resolve to the class constructor or the class itself

        let class_name = &call_site.called_name;

        // First try to find a class with this name
        let hash = Self::compute_hash(class_name);

        // Look for constructor methods in our function index
        if let Some(candidates) = self.function_index.get(&hash) {
            for candidate in candidates {
                // Look for constructors, init methods, or the class name itself
                if &candidate.name == class_name
                    || candidate.name == "__init__"
                    || candidate.name == "constructor"
                    || candidate
                        .class_context
                        .as_ref()
                        .map(|c| c == class_name)
                        .unwrap_or(false)
                {
                    return Some(Edge::new(
                        EdgeType::Call,
                        call_site.caller_id.clone(),
                        candidate.node_id.clone(),
                    ));
                }
            }
        }

        // If no specific constructor found, create an external class reference
        Some(Edge::new(
            EdgeType::Call,
            call_site.caller_id.clone(),
            format!("external:class:{}:0", class_name),
        ))
    }

    /// Compute stable hash for function names with optimized hashing
    fn compute_hash(name: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        name.hash(&mut hasher);
        hasher.finish()
    }

    /// Select the best candidate from multiple matches using heuristics
    #[allow(dead_code)]
    fn select_best_candidate<'a>(
        &self,
        candidates: &'a [FunctionEntry],
        call_site: &CallSite,
    ) -> Option<&'a FunctionEntry> {
        if candidates.is_empty() {
            return None;
        }

        if candidates.len() == 1 {
            return Some(&candidates[0]);
        }

        // Fast scoring system for candidate selection - avoid string operations
        let context_ref = call_site.context.as_ref();
        let mut best_candidate = &candidates[0];
        let mut best_score = 0;

        for candidate in candidates {
            let mut score = 0;

            // Prefer same file (fast path check)
            if let Some(ctx) = context_ref {
                if ctx.contains(&*candidate.file_path.to_string_lossy()) {
                    score += 100;
                }
                // Prefer same module
                if ctx.contains(&candidate.module_context) {
                    score += 50;
                }
            }

            // Prefer functions without class context for simple calls
            if candidate.class_context.is_none() {
                score += 25;
            }

            // Prefer exact name matches (in case of hash collisions)
            if candidate.name == call_site.called_name {
                score += 200;
            }

            if score > best_score {
                best_score = score;
                best_candidate = candidate;
            }
        }

        Some(best_candidate)
    }

    /// Fuzzy matching for function names (handles typos, case differences)
    #[allow(dead_code)]
    fn fuzzy_resolve_function(&self, call_site: &CallSite) -> Option<Edge> {
        let target = call_site.called_name.to_lowercase();
        let mut best_match: Option<(&FunctionEntry, usize)> = None;

        // Only check if the name is reasonably similar (Levenshtein distance)
        for candidates in self.function_index.values() {
            for candidate in candidates {
                let distance = self.levenshtein_distance(&target, &candidate.name.to_lowercase());

                // Only consider matches with distance <= 2 for reasonable-length names
                if distance <= 2 && candidate.name.len() > 3 {
                    if best_match.is_none() || distance < best_match.unwrap().1 {
                        best_match = Some((candidate, distance));
                    }
                }
            }
        }

        if let Some((candidate, _distance)) = best_match {
            return Some(
                Edge::new(
                    EdgeType::Call,
                    call_site.caller_id.clone(),
                    candidate.node_id.clone(),
                )
                .with_context(format!("fuzzy_match:line:{}", call_site.line_number)),
            );
        }

        None
    }

    /// Simple Levenshtein distance calculation
    #[allow(dead_code)]
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        // Initialize first row and column
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                    0
                } else {
                    1
                };

                matrix[i][j] = std::cmp::min(
                    std::cmp::min(
                        matrix[i - 1][j] + 1, // deletion
                        matrix[i][j - 1] + 1, // insertion
                    ),
                    matrix[i - 1][j - 1] + cost, // substitution
                );
            }
        }

        matrix[len1][len2]
    }

    // Helper methods for different resolution strategies
    fn create_function_entry(&self, node: &Node) -> FunctionOrMethod {
        // Determine if this is a method (has class context) or function
        let class_context = self.extract_class_from_id(&node.id);

        if let Some(class_name) = class_context {
            FunctionOrMethod::Method(MethodEntry {
                node_id: node.id.clone(),
                name: node.name.clone(),
                class_name,
                file_path: node.file_path.clone(),
                line_number: node.line_number,
                signature: node.signature.clone(),
            })
        } else {
            FunctionOrMethod::Function(FunctionEntry {
                node_id: node.id.clone(),
                name: node.name.clone(),
                file_path: node.file_path.clone(),
                line_number: node.line_number,
                signature: node.signature.clone(),
                class_context: None,
                module_context: self.extract_module_from_path(&node.file_path),
            })
        }
    }

    fn extract_class_from_id(&self, node_id: &str) -> Option<String> {
        // Parse node IDs to extract class context
        // Example: "file.py:class:MyClass:42" -> Some("MyClass")
        let parts: Vec<&str> = node_id.split(':').collect();
        if parts.len() >= 3 && parts[1] == "class" {
            return Some(parts[2].to_string());
        }
        None
    }

    fn extract_module_from_path(&self, file_path: &Path) -> String {
        file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    fn build_import_mapping(&mut self, nodes: &[Node]) -> Result<()> {
        for node in nodes {
            if node.node_type == NodeType::Module {
                // Parse import statements to build module mapping
                // This is language-specific and would need refinement
                if node.name.contains("import") {
                    self.parse_import_statement(&node.name);
                }
            }
        }
        Ok(())
    }

    fn parse_import_statement(&mut self, import_stmt: &str) {
        // Simple import parsing - would need language-specific refinement
        if import_stmt.contains("from") && import_stmt.contains("import") {
            // Handle "from module import function"
            if let Some(parts) = self.parse_from_import(import_stmt) {
                for (alias, original) in parts {
                    self.import_mapping.insert(alias, original);
                }
            }
        }
    }

    fn parse_from_import(&self, _stmt: &str) -> Option<Vec<(String, String)>> {
        // Simplified parsing - production code would use proper AST analysis
        None // TODO: Implement proper import parsing
    }

    // Additional helper methods would go here...
    #[allow(dead_code)]
    fn extract_method_name(&self, called_name: &str) -> Option<String> {
        called_name.split('.').last().map(|s| s.to_string())
    }

    #[allow(dead_code)]
    fn infer_class_context(&self, _call_site: &CallSite) -> Option<String> {
        // Would analyze the calling context to determine likely class
        None // TODO: Implement class inference
    }

    #[allow(dead_code)]
    fn select_best_method_candidate<'a>(
        &self,
        candidates: &'a [MethodEntry],
        _class_context: &Option<String>,
    ) -> Option<&'a MethodEntry> {
        candidates.first() // Simplified selection
    }

    #[allow(dead_code)]
    fn resolve_by_full_name(&self, _full_name: &str, _call_site: &CallSite) -> Option<Edge> {
        None // TODO: Implement full name resolution
    }

    #[allow(dead_code)]
    fn resolve_by_module_and_function(
        &self,
        _module: &str,
        _function: &str,
        _call_site: &CallSite,
    ) -> Option<Edge> {
        None // TODO: Implement module-based resolution
    }

    #[allow(dead_code)]
    fn select_attribute_candidate<'a>(
        &self,
        candidates: &'a [MethodEntry],
        _call_site: &CallSite,
    ) -> Option<&'a MethodEntry> {
        candidates.first() // Simplified selection
    }

    #[allow(dead_code)]
    fn resolve_dynamic_patterns(&self, _call_site: &CallSite) -> Option<Edge> {
        None // TODO: Implement dynamic pattern resolution
    }

    #[allow(dead_code)]
    fn resolve_with_context(&self, _call_site: &CallSite, _context: &str) -> Option<Edge> {
        None // TODO: Implement context-based resolution
    }
}

#[derive(Debug, Clone)]
enum FunctionOrMethod {
    Function(FunctionEntry),
    Method(MethodEntry),
}

impl FunctionOrMethod {
    fn is_function(&self) -> bool {
        matches!(self, FunctionOrMethod::Function(_))
    }
}

/// Optimized call site extractor that identifies function calls during AST traversal
pub struct CallSiteExtractor {
    call_sites: Vec<CallSite>,
    current_function: Option<String>,
    current_function_line: Option<usize>,
    current_file: Option<String>,
}

impl CallSiteExtractor {
    pub fn new() -> Self {
        Self {
            call_sites: Vec::new(),
            current_function: None,
            current_function_line: None,
            current_file: None,
        }
    }

    pub fn extract_from_ast(
        &mut self,
        root: &tree_sitter::Node,
        source: &[u8],
        file_path: &std::path::Path,
    ) -> Vec<CallSite> {
        self.call_sites.clear();
        self.current_file = Some(
            file_path
                .to_string_lossy()
                .replace('/', "_")
                .replace('\\', "_"),
        );
        self.traverse_ast(root, source);
        std::mem::take(&mut self.call_sites)
    }

    fn traverse_ast(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        // Track current function context for different languages
        if self.is_function_node(node) {
            if let Some((func_name, line_num)) = self.extract_function_info(node, source) {
                self.current_function = Some(func_name);
                self.current_function_line = Some(line_num);
            }
        }

        // Extract call sites (including class instantiations)
        if self.is_call_node(node) {
            if let Some(call_site) = self.extract_call_site(node, source) {
                self.call_sites.push(call_site);
            }
        }

        // Recursively process children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_ast(&child, source);
        }

        // Clear function context when exiting function
        if self.is_function_node(node) {
            self.current_function = None;
            self.current_function_line = None;
        }
    }

    fn is_function_node(&self, node: &tree_sitter::Node) -> bool {
        matches!(
            node.kind(),
            "function_definition" |        // Python/C++
            "function_declaration" |       // TypeScript/JavaScript
            "method_definition" |          // TypeScript/JavaScript
            "constructor_declaration" |    // C++
            "destructor_declaration" |     // C++
            "function_item" // Rust
        )
    }

    fn extract_function_info(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
    ) -> Option<(String, usize)> {
        let line_num = node.start_position().row + 1;

        // Try different ways to extract function name based on node type
        let func_name = if let Some(name_node) = node.child_by_field_name("name") {
            // Python, TypeScript
            self.extract_text(&name_node, source).to_string()
        } else {
            // C++ and other patterns - look for identifier nodes
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                match child.kind() {
                    "identifier" | "function_declarator" => {
                        // For C++, function_declarator contains the function name
                        if child.kind() == "function_declarator" {
                            // Look for identifier inside function_declarator
                            let mut inner_cursor = child.walk();
                            for inner_child in child.children(&mut inner_cursor) {
                                if inner_child.kind() == "identifier" {
                                    return Some((
                                        self.extract_text(&inner_child, source).to_string(),
                                        line_num,
                                    ));
                                }
                            }
                        } else {
                            return Some((self.extract_text(&child, source).to_string(), line_num));
                        }
                    }
                    "property_identifier" => {
                        // TypeScript method names
                        return Some((self.extract_text(&child, source).to_string(), line_num));
                    }
                    _ => continue,
                }
            }
            return None;
        };

        if func_name.is_empty() {
            None
        } else {
            Some((func_name, line_num))
        }
    }

    fn is_call_node(&self, node: &tree_sitter::Node) -> bool {
        matches!(
            node.kind(),
            "call" |                    // Python
            "call_expression" |         // TypeScript/JavaScript/C++/Rust
            "new_expression" |          // C++ class instantiation
            "constructor_call" |        // C++ constructor calls
            "macro_invocation" // Rust macro calls (like println!)
        )
    }

    fn extract_call_site(&self, node: &tree_sitter::Node, source: &[u8]) -> Option<CallSite> {
        let (called_name, call_type) = self.extract_called_function_info(node, source)?;

        if called_name.is_empty() {
            return None;
        }

        // Build proper caller ID that matches the node ID format used in the parsers
        let caller_id = if let Some(ref current_func) = self.current_function {
            // Build a proper node ID that matches the exact format used by generate_node_id
            // Format: "file_path_with_underscores:type:name:line_number"
            format!(
                "{}:function:{}:{}",
                self.current_file.as_deref().unwrap_or("unknown"),
                current_func,
                self.current_function_line.unwrap_or(0)
            )
        } else {
            "module_level".to_string()
        };

        Some(CallSite {
            caller_id,
            called_name,
            call_type,
            context: Some(format!("ast_node:{}", node.kind())),
            line_number: node.start_position().row + 1,
        })
    }

    fn extract_called_function_info(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
    ) -> Option<(String, CallType)> {
        match node.kind() {
            "call" | "call_expression" => {
                // Regular function calls
                let function_node = node.child(0)?;
                let called_name = self.extract_function_name_from_node(&function_node, source);
                let call_type = self.determine_call_type(&called_name, &function_node);
                Some((called_name, call_type))
            }
            "macro_invocation" => {
                // Rust macro calls like println!, vec!, etc.
                if let Some(name_node) = node.child(0) {
                    let macro_name = self.extract_text(&name_node, source).to_string();
                    Some((macro_name, CallType::SimpleCall))
                } else {
                    None
                }
            }
            "new_expression" => {
                // C++ class instantiation: new ClassName(args)
                // Look for the type being instantiated
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    match child.kind() {
                        "type_identifier" | "identifier" => {
                            let class_name = self.extract_text(&child, source).to_string();
                            return Some((class_name, CallType::ConstructorCall));
                        }
                        "qualified_identifier" => {
                            // Handle namespaced classes like std::string
                            let qualified_name = self.extract_text(&child, source).to_string();
                            return Some((qualified_name, CallType::ConstructorCall));
                        }
                        _ => continue,
                    }
                }
                None
            }
            "constructor_call" => {
                // Direct constructor calls
                if let Some(name_node) = node.child(0) {
                    let name = self.extract_text(&name_node, source).to_string();
                    Some((name, CallType::ConstructorCall))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn extract_function_name_from_node(
        &self,
        function_node: &tree_sitter::Node,
        source: &[u8],
    ) -> String {
        match function_node.kind() {
            "identifier" => {
                // Simple function call: func()
                self.extract_text(function_node, source).to_string()
            }
            "attribute" => {
                // Python attribute access: obj.method() or self.method() or super().method()
                // Extract the method name (rightmost identifier)
                let full_text = self.extract_text(function_node, source);
                
                // Handle special cases
                if full_text.starts_with("self.") {
                    // self.method() - extract method name
                    return full_text[5..].to_string();
                }
                if full_text.starts_with("cls.") {
                    // cls.method() - class method call
                    return full_text[4..].to_string();
                }
                if full_text.starts_with("super().") {
                    // super().method() - parent method call
                    return full_text[8..].to_string();
                }
                
                // For other attribute access like module.func or obj.method
                // Look for the attribute identifier
                let mut cursor = function_node.walk();
                for child in function_node.children(&mut cursor) {
                    if child.kind() == "identifier" && child.start_byte() > function_node.start_byte() {
                        // This is the attribute name (not the object)
                        let attr = self.extract_text(&child, source);
                        if !attr.is_empty() {
                            return attr.to_string();
                        }
                    }
                }
                
                // Fallback: return the full attribute chain
                full_text.to_string()
            }
            "field_expression" => {
                // Member function call: obj.method() (Rust, C++, JS)
                // Extract the method name (rightmost part)
                let mut cursor = function_node.walk();
                for child in function_node.children(&mut cursor) {
                    if child.kind() == "field_identifier" {
                        return self.extract_text(&child, source).to_string();
                    }
                }
                self.extract_text(function_node, source).to_string()
            }
            "qualified_identifier" => {
                // Qualified call: namespace::function() or Class::method()
                self.extract_text(function_node, source).to_string()
            }
            "scoped_identifier" => {
                // Rust scoped calls: std::println, crate::module::function
                self.extract_text(function_node, source).to_string()
            }
            "generic_function" => {
                // Rust generic function calls: Vec::<i32>::new()
                if let Some(name_node) = function_node.child_by_field_name("function") {
                    self.extract_function_name_from_node(&name_node, source)
                } else {
                    self.extract_text(function_node, source).to_string()
                }
            }
            "subscript_expression" => {
                // Function pointer calls: func_ptr()
                self.extract_text(function_node, source).to_string()
            }
            "call" => {
                // Nested call: super().method() where super() is itself a call
                // This handles patterns like super().__init__()
                let full_text = self.extract_text(function_node, source);
                if full_text.contains("super()") {
                    return full_text.to_string();
                }
                // For other nested calls, extract the outer function
                if let Some(inner) = function_node.child(0) {
                    return self.extract_function_name_from_node(&inner, source);
                }
                full_text.to_string()
            }
            _ => {
                // Fallback: extract full text
                self.extract_text(function_node, source).to_string()
            }
        }
    }

    fn determine_call_type(
        &self,
        called_name: &str,
        function_node: &tree_sitter::Node,
    ) -> CallType {
        match function_node.kind() {
            "attribute" => {
                // Python attribute access
                if called_name.starts_with("self.") || called_name.starts_with("cls.") {
                    CallType::MethodCall
                } else if called_name.starts_with("super()") || called_name.contains("super()") {
                    CallType::MethodCall
                } else if called_name.contains('.') {
                    CallType::QualifiedCall // module.function style
                } else {
                    CallType::MethodCall
                }
            }
            "field_expression" => CallType::MethodCall, // obj.method()
            "qualified_identifier" => CallType::QualifiedCall, // namespace::func() or Class::method()
            "scoped_identifier" => CallType::QualifiedCall, // Rust std::println, crate::module::function
            "generic_function" => CallType::QualifiedCall,  // Rust Vec::<i32>::new()
            "identifier" => {
                // Check if it looks like a class instantiation (PascalCase)
                if !called_name.is_empty() {
                    let first_char = called_name.chars().next().unwrap();
                    if first_char.is_uppercase() && !called_name.contains('_') {
                        return CallType::ConstructorCall;
                    }
                }
                if called_name.contains("::") {
                    CallType::QualifiedCall // C++/Rust scope resolution
                } else if called_name.contains('.') {
                    CallType::MethodCall // Python/JS style
                } else {
                    CallType::SimpleCall // Simple function call
                }
            }
            "call" => {
                // Nested call like super().__init__()
                if called_name.contains("super()") {
                    CallType::MethodCall
                } else {
                    CallType::DynamicCall
                }
            }
            _ => CallType::DynamicCall, // Function pointers, etc.
        }
    }

    fn extract_text<'a>(&self, node: &tree_sitter::Node, source: &'a [u8]) -> &'a str {
        std::str::from_utf8(&source[node.byte_range()]).unwrap_or("")
    }
}

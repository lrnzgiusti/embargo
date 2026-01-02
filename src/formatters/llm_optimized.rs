//! LLM-optimized output formatter.
//!
//! Generates compact, token-efficient dependency graphs with semantic clustering
//! and behavioral notation designed for AI code analysis.
//!
//! ## Output Structure
//!
//! - **DIRECTORY_TREE**: Hierarchical file organization with semantic prefixes
//! - **ARCHITECTURAL_CLUSTERS**: Code grouped by functional purpose
//! - **DEPENDENCY_PATTERNS**: Cross-module relationship analysis
//!
//! ## Behavioral Notation
//!
//! - `function()[ENTRY]` - Public API entry point
//! - `function()[HOT]` - Performance-critical function
//! - `function()->{calls}` - Immediate function calls

use anyhow::Result;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use super::llm_language::{DefaultLanguageAdapter, LlmLanguageAdapter};
use crate::core::{DependencyGraph, Edge, Node, NodeType};

/// Output verbosity level for LLM-optimized format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputVerbosity {
    /// Compact core only - minimal tokens, no interpretation key
    Compact,
    /// Core + interpretation key (default)
    #[default]
    Standard,
    /// Full output with all sections including dependency patterns
    Verbose,
}

/// LLM-optimized formatter that minimizes tokens while maximizing structural understanding.
pub struct LLMOptimizedFormatter {
    /// Whether to include detailed metadata (false for token efficiency)
    include_metadata: bool,
    /// Whether to use hierarchical grouping (true for better LLM understanding)
    use_hierarchical: bool,
    /// Whether to compress identifiers (true for token efficiency)
    compress_ids: bool,
    /// Whether to use semantic clustering (groups by architectural domains)
    use_semantic_clustering: bool,
    /// Whether to use advanced DAG compression (pattern-based edge compression)
    use_advanced_dag: bool,
    /// Language-specific adapter for formatting semantics
    language_adapter: Box<dyn LlmLanguageAdapter>,
    /// Output verbosity level
    verbosity: OutputVerbosity,
}

impl LLMOptimizedFormatter {
    /// Creates a new formatter with default settings.
    pub fn new() -> Self {
        Self {
            include_metadata: true,
            use_hierarchical: true,
            compress_ids: true,
            use_semantic_clustering: true,
            use_advanced_dag: true,
            language_adapter: Box::new(DefaultLanguageAdapter::new()),
            verbosity: OutputVerbosity::default(),
        }
    }

    /// Sets the output verbosity level.
    pub fn with_verbosity(mut self, verbosity: OutputVerbosity) -> Self {
        self.verbosity = verbosity;
        self
    }

    #[allow(dead_code)]
    pub fn with_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    pub fn with_hierarchical(mut self, hierarchical: bool) -> Self {
        self.use_hierarchical = hierarchical;
        self
    }

    pub fn with_compressed_ids(mut self, compress: bool) -> Self {
        self.compress_ids = compress;
        self
    }

    #[allow(dead_code)]
    pub fn with_semantic_clustering(mut self, cluster: bool) -> Self {
        self.use_semantic_clustering = cluster;
        self
    }

    #[allow(dead_code)]
    pub fn with_advanced_dag(mut self, advanced: bool) -> Self {
        self.use_advanced_dag = advanced;
        self
    }

    /// Set a custom language adapter
    pub fn with_language_adapter(mut self, adapter: Box<dyn LlmLanguageAdapter>) -> Self {
        self.language_adapter = adapter;
        self
    }

    /// Convenience: Python-tuned formatter
    pub fn for_python() -> Self {
        let adapter = Box::new(crate::formatters::PythonLanguageAdapter::new());
        Self::new().with_language_adapter(adapter)
    }

    pub fn format_to_file(&self, graph: &DependencyGraph, output_path: &Path) -> Result<()> {
        let formatted_content = self.format_graph(graph)?;
        fs::write(output_path, formatted_content)?;
        Ok(())
    }

    fn format_graph(&self, graph: &DependencyGraph) -> Result<String> {
        let mut output = String::with_capacity(8192);

        // Interpretation key only for Standard and Verbose modes
        if self.verbosity != OutputVerbosity::Compact {
            self.add_interpretation_key(&mut output);
        }

        // Compact header
        output.push_str("# CODE_GRAPH\n");
        output.push_str(&format!(
            "NODES:{} EDGES:{}\n\n",
            graph.node_count(),
            graph.edge_count()
        ));

        // Build node collections efficiently
        let node_indices: Vec<NodeIndex> = graph.node_indices().collect();
        let mut by_type: HashMap<NodeType, Vec<(NodeIndex, &Node)>> = HashMap::new();

        for &idx in &node_indices {
            if let Some(node) = graph.node_weight(idx) {
                by_type.entry(node.node_type).or_default().push((idx, node));
            }
        }

        // Generate advanced data structures for optimization
        let directory_tree = self.build_directory_tree(&by_type);
        let semantic_clusters = if self.use_semantic_clustering {
            self.build_semantic_clusters(&by_type)
        } else {
            HashMap::new()
        };
        let file_map = self.build_enhanced_file_map(&directory_tree);

        if self.use_semantic_clustering && !semantic_clusters.is_empty() {
            self.format_with_clusters(&mut output, &semantic_clusters, &directory_tree, graph)?;
        } else if self.use_hierarchical {
            self.format_hierarchical(&mut output, &by_type, &file_map, graph)?;
        } else {
            self.format_flat(&mut output, &by_type, &file_map, graph)?;
        }

        // Dependency patterns only for Verbose mode
        if self.verbosity == OutputVerbosity::Verbose {
            if self.use_advanced_dag {
                output.push('\n');
                self.format_advanced_dependencies(&mut output, graph, &semantic_clusters);
            } else {
                self.format_dependency_summary(&mut output, graph);
            }
        }

        Ok(output)
    }

    fn format_hierarchical(
        &self,
        output: &mut String,
        by_type: &HashMap<NodeType, Vec<(NodeIndex, &Node)>>,
        file_map: &HashMap<String, String>,
        graph: &DependencyGraph,
    ) -> Result<()> {
        // Directory tree header
        let directory_tree = self.build_directory_tree(by_type);
        output.push_str("## DIRECTORY_TREE\n");
        output.push_str(&format!("ROOT: {}\n", directory_tree.common_prefix));
        output.push_str(&directory_tree.format_tree());

        // File header for compression
        if self.compress_ids && !file_map.is_empty() {
            output.push_str("## FILES\n");
            let mut files: Vec<_> = file_map.iter().collect();
            files.sort_by_key(|(_, id)| *id);
            for (path, id) in files {
                output.push_str(&format!("{}: {}\n", id, path));
            }
            output.push('\n');
        }

        // Process types in dependency order: modules -> classes -> interfaces -> functions -> variables
        let type_order = [
            NodeType::Module,
            NodeType::Class,
            NodeType::Interface,
            NodeType::Function,
            NodeType::Variable,
            NodeType::Enum,
        ];

        for node_type in type_order {
            if let Some(nodes) = by_type.get(&node_type) {
                self.format_type_section(output, node_type, nodes, file_map, graph);
            }
        }

        Ok(())
    }

    fn format_type_section(
        &self,
        output: &mut String,
        node_type: NodeType,
        nodes: &[(NodeIndex, &Node)],
        file_map: &HashMap<String, String>,
        graph: &DependencyGraph,
    ) {
        if nodes.is_empty() {
            return;
        }

        // Compact section header
        output.push_str(&format!("## {}\n", self.type_symbol(node_type)));

        if self.use_hierarchical {
            // Group by file for better structure
            let mut by_file: HashMap<String, Vec<(NodeIndex, &Node)>> = HashMap::new();
            for &(idx, node) in nodes {
                let file_key = if self.compress_ids {
                    file_map
                        .get(&node.file_path.to_string_lossy().to_string())
                        .cloned()
                        .unwrap_or_else(|| "?".to_string())
                } else {
                    node.file_path.to_string_lossy().to_string()
                };
                by_file.entry(file_key).or_default().push((idx, node));
            }

            // deterministic order by file key
            let mut __keys: Vec<String> = by_file.keys().cloned().collect();
            __keys.sort();
            for file_key in __keys {
                output.push_str(&format!("### {}\n", file_key));
                let mut file_nodes = by_file.get(&file_key).cloned().unwrap_or_default();
                // sort by line then name
                file_nodes.sort_by(|(ia, na), (ib, nb)| {
                    let la = graph.node_weight(*ia).map(|n| n.line_number).unwrap_or(0);
                    let lb = graph.node_weight(*ib).map(|n| n.line_number).unwrap_or(0);
                    la.cmp(&lb).then_with(|| na.name.cmp(&nb.name))
                });
                for (idx, node) in file_nodes {
                    self.format_node_compact(output, node, idx, graph);
                }
                output.push('\n');
            }
        } else {
            // Flat format
            for &(idx, node) in nodes {
                self.format_node_compact(output, node, idx, graph);
            }
            output.push('\n');
        }
    }

    fn format_node_compact(
        &self,
        output: &mut String,
        node: &Node,
        idx: NodeIndex,
        graph: &DependencyGraph,
    ) {
        // Ultra-compact format: signature [relationships]
        if self.include_metadata {
            if let Some(ref sig) = node.signature {
                if !sig.is_empty() {
                    // Use compact signature if available
                    output.push_str(&format!("- {}", self.compact_signature(sig)));
                } else {
                    output.push_str(&format!("- {}()", node.name));
                }
            } else {
                output.push_str(&format!("- {}()", node.name));
            }
            output.push_str(&format!(":{}", node.line_number));
        } else {
            output.push_str(&format!("- {}", node.name));
        }

        // Compact relationships
        let outgoing = self.get_outgoing_edges(idx, graph);
        if !outgoing.is_empty() {
            output.push_str(" →");
            let mut first = true;
            for (_edge, target) in outgoing.iter().take(5) {
                // Limit to reduce tokens
                if !first {
                    output.push(',');
                }
                output.push_str(&format!("{}", target.name));
                first = false;
            }
            if outgoing.len() > 5 {
                output.push_str(&format!("+{}", outgoing.len() - 5));
            }
        }

        output.push('\n');
    }

    fn format_flat(
        &self,
        output: &mut String,
        by_type: &HashMap<NodeType, Vec<(NodeIndex, &Node)>>,
        file_map: &HashMap<String, String>,
        graph: &DependencyGraph,
    ) -> Result<()> {
        // Simple flat list optimized for LLM scanning with deterministic type order
        let type_order = [
            NodeType::Module,
            NodeType::Class,
            NodeType::Interface,
            NodeType::Function,
            NodeType::Variable,
            NodeType::Enum,
        ];

        for node_type in type_order {
            let Some(nodes) = by_type.get(&node_type) else {
                continue;
            };
            if nodes.is_empty() {
                continue;
            }

            output.push_str(&format!("## {}\n", self.type_symbol(node_type)));

            for &(idx, node) in nodes.iter() {
                let file_ref = if self.compress_ids {
                    file_map
                        .get(&node.file_path.to_string_lossy().to_string())
                        .cloned()
                        .unwrap_or_else(|| "?".to_string())
                } else {
                    node.file_path.to_string_lossy().to_string()
                };

                output.push_str(&format!("{}:{} ", file_ref, node.line_number));
                output.push_str(&node.name);

                // Compact relationships
                let outgoing = self.get_outgoing_edges(idx, graph);
                if !outgoing.is_empty() {
                    output.push_str(" →");
                    for (i, (_, target)) in outgoing.iter().take(3).enumerate() {
                        if i > 0 {
                            output.push(',');
                        }
                        output.push_str(&target.name);
                    }
                }
                output.push('\n');
            }
            output.push('\n');
        }

        Ok(())
    }

    fn format_dependency_summary(&self, output: &mut String, graph: &DependencyGraph) {
        output.push_str("## DEPS\n");

        let mut edge_counts: HashMap<String, usize> = HashMap::new();
        for edge_ref in graph.edge_references() {
            let edge_type = format!("{:?}", edge_ref.weight().edge_type);
            *edge_counts.entry(edge_type).or_insert(0) += 1;
        }

        let mut keys: Vec<_> = edge_counts.keys().cloned().collect();
        keys.sort();
        for edge_type in keys {
            let count = edge_counts[&edge_type];
            output.push_str(&format!("{}: {}\n", edge_type, count));
        }
    }

    fn type_symbol(&self, node_type: NodeType) -> &'static str {
        match node_type {
            NodeType::Module => "MOD",
            NodeType::Class => "CLS",
            NodeType::Function => "FN",
            NodeType::Variable => "VAR",
            NodeType::Interface => "IF",
            NodeType::Enum => "ENUM",
        }
    }

    fn get_outgoing_edges<'a>(
        &self,
        node_idx: NodeIndex,
        graph: &'a DependencyGraph,
    ) -> Vec<(&'a Edge, &'a Node)> {
        let mut edges = Vec::new();
        for edge_ref in graph.edges(node_idx) {
            let target_idx = edge_ref.target();
            let edge_weight = edge_ref.weight();
            if let Some(target_node) = graph.node_weight(target_idx) {
                edges.push((edge_weight, target_node));
            }
        }
        edges
    }

    /// Extract common path prefixes and build directory tree structure
    fn build_directory_tree(
        &self,
        by_type: &HashMap<NodeType, Vec<(NodeIndex, &Node)>>,
    ) -> DirectoryTree {
        let mut all_paths = Vec::new();

        for nodes in by_type.values() {
            for (_, node) in nodes {
                all_paths.push(node.file_path.to_string_lossy().to_string());
            }
        }

        all_paths.sort();
        all_paths.dedup();

        DirectoryTree::build(all_paths)
    }

    /// Group nodes into semantic architectural clusters
    fn build_semantic_clusters<'a>(
        &self,
        by_type: &HashMap<NodeType, Vec<(NodeIndex, &'a Node)>>,
    ) -> HashMap<String, Vec<(NodeIndex, &'a Node)>> {
        let mut clusters = HashMap::new();

        for nodes in by_type.values() {
            for &(idx, node) in nodes {
                let cluster_name = self.language_adapter.classify_node_cluster(node);
                clusters
                    .entry(cluster_name)
                    .or_insert_with(Vec::new)
                    .push((idx, node));
            }
        }

        clusters
    }

    /// Classify a node into an architectural cluster
    #[allow(dead_code)]
    fn classify_node_cluster(&self, node: &Node) -> String {
        let path = node.file_path.to_string_lossy();

        if path.contains("/services/") {
            "CORE_SERVICES".to_string()
        } else if path.contains("/entities/") {
            "DATA_ENTITIES".to_string()
        } else if path.contains("/components/") {
            "UI_COMPONENTS".to_string()
        } else if path.contains("/widgets/dialogs/") {
            "DIALOG_SYSTEM".to_string()
        } else if path.contains("/widgets/ribbon/") {
            "RIBBON_SYSTEM".to_string()
        } else if path.contains("/widgets/buttons/") {
            "BUTTON_SYSTEM".to_string()
        } else if path.contains("/widgets/view_widgets/") {
            "VIEW_SYSTEM".to_string()
        } else if path.contains("/widgets/") {
            "UI_WIDGETS".to_string()
        } else if path.contains("/menus/") {
            "MENU_SYSTEM".to_string()
        } else {
            "UTILITY_LAYER".to_string()
        }
    }

    /// Build enhanced file map with semantic prefixes
    fn build_enhanced_file_map(&self, directory_tree: &DirectoryTree) -> HashMap<String, String> {
        if !self.compress_ids {
            return HashMap::new();
        }

        let mut file_map = HashMap::new();
        let mut counters: HashMap<String, usize> = HashMap::new();

        // Assign IDs in deterministic path order
        let mut paths: Vec<_> = directory_tree.semantic_prefixes.keys().cloned().collect();
        paths.sort();
        for path in paths {
            let prefix = directory_tree
                .semantic_prefixes
                .get(&path)
                .cloned()
                .unwrap_or_else(|| "U".to_string());
            let counter = counters.entry(prefix.clone()).or_insert(0);
            file_map.insert(path.clone(), format!("{}{}", prefix, counter));
            *counter += 1;
        }

        file_map
    }

    /// Format output using semantic clusters with nested call hierarchies
    fn format_with_clusters(
        &self,
        output: &mut String,
        clusters: &HashMap<String, Vec<(NodeIndex, &Node)>>,
        directory_tree: &DirectoryTree,
        graph: &DependencyGraph,
    ) -> Result<()> {
        // Directory tree header
        output.push_str("## DIRECTORY_TREE\n");
        output.push_str(&format!("ROOT: {}\n", directory_tree.common_prefix));
        output.push_str(&directory_tree.format_tree());

        // Semantic clusters with call hierarchies
        output.push_str("## ARCHITECTURAL_CLUSTERS\n\n");

        let mut cluster_names: Vec<_> = clusters.keys().cloned().collect();
        cluster_names.sort();
        for cluster_name in cluster_names {
            let nodes = match clusters.get(&cluster_name) {
                Some(v) => v,
                None => continue,
            };
            if nodes.is_empty() {
                continue;
            }

            // Calculate cluster metrics
            let max_depth = self.calculate_max_call_depth(nodes, graph);
            output.push_str(&format!("### {}\n", cluster_name));
            output.push_str(&format!(
                "NODES:{} CALL_DEPTH:{}\n\n",
                nodes.len(),
                max_depth
            ));

            // Group by file and build call hierarchies
            let mut by_file: BTreeMap<String, Vec<(NodeIndex, &Node)>> = BTreeMap::new();
            for &(idx, node) in nodes {
                let file_key = self
                    .language_adapter
                    .extract_filename(&node.file_path.to_string_lossy());
                by_file.entry(file_key).or_default().push((idx, node));
            }

            for (file, mut file_nodes) in by_file {
                output.push_str(&format!("{}→[", file));
                // Sort within file for deterministic order
                file_nodes.sort_by(|a, b| {
                    let (_, na) = a;
                    let (_, nb) = b;
                    na.line_number
                        .cmp(&nb.line_number)
                        .then_with(|| na.name.cmp(&nb.name))
                });
                let behavioral_entities = self.build_behavioral_entities(&file_nodes, graph);
                let entity_strings: Vec<String> = behavioral_entities
                    .iter()
                    .map(|entity| self.format_behavioral_entity(entity))
                    .collect();

                output.push_str(&entity_strings.join(","));
                output.push_str("] ");
            }
            output.push('\n');
        }

        Ok(())
    }

    /// Format advanced dependency patterns
    fn format_advanced_dependencies(
        &self,
        output: &mut String,
        graph: &DependencyGraph,
        _clusters: &HashMap<String, Vec<(NodeIndex, &Node)>>,
    ) {
        output.push_str("## DEPENDENCY_PATTERNS\n\n");

        // Edge type analysis
        let mut edge_patterns: HashMap<String, usize> = HashMap::new();
        let mut cluster_edges: HashMap<(String, String), usize> = HashMap::new();

        for edge_ref in graph.edge_references() {
            let edge_type = format!("{:?}", edge_ref.weight().edge_type);
            *edge_patterns.entry(edge_type).or_insert(0) += 1;

            // Cross-cluster dependencies
            if let (Some(source_node), Some(target_node)) = (
                graph.node_weight(edge_ref.source()),
                graph.node_weight(edge_ref.target()),
            ) {
                let source_cluster = self.language_adapter.classify_node_cluster(source_node);
                let target_cluster = self.language_adapter.classify_node_cluster(target_node);
                if source_cluster != target_cluster {
                    *cluster_edges
                        .entry((source_cluster, target_cluster))
                        .or_insert(0) += 1;
                }
            }
        }

        // Pattern summary
        output.push_str("### EDGE_PATTERNS\n");
        for (pattern, count) in edge_patterns {
            output.push_str(&format!("{}: {} edges\n", pattern, count));
        }

        output.push_str("\n### CROSS_CLUSTER_FLOW\n");
        let mut pairs: Vec<_> = cluster_edges.into_iter().collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        for ((source, target), count) in pairs {
            output.push_str(&format!("{}→{}: {}\n", source, target, count));
        }
        output.push('\n');
    }

    /// Extract just the filename from a path
    #[allow(dead_code)]
    fn extract_filename(&self, path: &str) -> String {
        self.language_adapter.extract_filename(path)
    }

    /// Add comprehensive interpretation key for LLM consumption
    fn add_interpretation_key(&self, output: &mut String) {
        output.push_str("# EMBARGO: LLM-Optimized Codebase Dependency Graph\n\n");
        output.push_str("**SYSTEM PROMPT FOR LLM INTERPRETATION:**\n");
        output.push_str(
            "You are analyzing a codebase dependency graph optimized for AI understanding. ",
        );
        output.push_str(
            "This format reveals code architecture, execution flows, and behavioral patterns.\n\n",
        );

        output.push_str("## INTERPRETATION KEY\n\n");

        output.push_str("### STRUCTURE\n");
        output.push_str("- **NODES:X EDGES:Y** = Total code entities and relationships\n");
        output.push_str(
            "- **DIRECTORY_TREE** = Hierarchical file organization with semantic prefixes\n",
        );
        output.push_str("- **ARCHITECTURAL_CLUSTERS** = Code grouped by functional purpose\n");
        output.push_str("- **DEPENDENCY_PATTERNS** = Cross-module relationship analysis\n\n");

        output.push_str("### BEHAVIORAL NOTATION\n");
        output.push_str("- **filename.rs→[...]** = File containing list of functions/entities\n");
        output.push_str("- **function()[ENTRY]** = Public API entry point, start analysis here\n");
        output.push_str("- **function()[HOT]** = Performance-critical, optimization target\n");
        output.push_str("- **function()→{calls}** = Immediate function calls (execution flow)\n");
        output.push_str("- **module::function** = Cross-module dependency\n\n");

        output.push_str("### ANALYSIS GUIDANCE\n");
        output.push_str(
            "1. **Entry Points**: Start with [ENTRY] functions to understand public APIs\n",
        );
        output.push_str("2. **Execution Flow**: Follow →{calls} to trace code execution paths\n");
        output.push_str("3. **Hot Paths**: Focus [HOT] functions for performance analysis\n");
        output.push_str("4. **Architecture**: Use clusters to understand system organization\n");
        output.push_str("5. **Dependencies**: Cross-cluster flows show coupling patterns\n\n");

        output.push_str("### SEMANTIC PREFIXES\n");
        output.push_str("- **S[N]** = Services (business logic)\n");
        output.push_str("- **E[N]** = Entities (data models)\n");
        output.push_str("- **C[N]** = Components (UI elements)\n");
        output.push_str("- **D[N]** = Dialogs (modal interfaces)\n");
        output.push_str("- **R[N]** = Ribbon/Toolbar (controls)\n");
        output.push_str("- **B[N]** = Buttons (actions)\n");
        output.push_str("- **V[N]** = Views (display components)\n");
        output.push_str("- **M[N]** = Menus (navigation)\n");
        output.push_str("- **T[N]** = Type widgets (specialized UI)\n");
        output.push_str("- **W[N]** = General widgets\n");
        output.push_str("- **U[N]** = Utilities (helpers)\n\n");

        output.push_str("### AI REASONING TASKS\n");
        output.push_str("- **Code Understanding**: Follow [ENTRY]→{calls} chains\n");
        output.push_str("- **Bug Hunting**: Trace execution flows through clusters\n");
        output.push_str("- **Refactoring**: Analyze cross-cluster dependencies\n");
        output.push_str("- **Performance**: Focus on [HOT] functions and call depths\n");
        output.push_str("- **Architecture**: Understand cluster responsibilities\n\n");

        output.push_str("---\n\n");
    }

    /// Calculate maximum call depth in a cluster
    fn calculate_max_call_depth(
        &self,
        nodes: &[(NodeIndex, &Node)],
        graph: &DependencyGraph,
    ) -> usize {
        let mut max_depth = 0;

        for &(node_idx, _) in nodes {
            let depth = self.calculate_node_call_depth(
                node_idx,
                graph,
                &mut std::collections::HashSet::new(),
            );
            max_depth = max_depth.max(depth);
        }

        max_depth
    }

    /// Build behavioral entities (compact format with nested calls)
    fn build_behavioral_entities(
        &self,
        file_nodes: &[(NodeIndex, &Node)],
        graph: &DependencyGraph,
    ) -> Vec<BehavioralEntity> {
        let mut entities = Vec::new();

        for &(node_idx, node) in file_nodes {
            if matches!(node.node_type, crate::core::NodeType::Function) {
                let nested_calls = self.extract_immediate_calls(node_idx, graph, file_nodes);
                let annotations = self.get_compact_annotations(node, graph, file_nodes);

                entities.push(BehavioralEntity {
                    name: node.name.clone(),
                    signature: node.signature.clone(),
                    annotations,
                    nested_calls,
                });
            }
        }

        // Sort by importance (entry points first, then by call complexity)
        entities.sort_by(|a, b| {
            let a_is_entry = a.annotations.contains(&"ENTRY".to_string());
            let b_is_entry = b.annotations.contains(&"ENTRY".to_string());

            match (a_is_entry, b_is_entry) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => b.nested_calls.len().cmp(&a.nested_calls.len()),
            }
        });

        entities
    }

    /// Extract immediate function calls (depth 1 only for compactness)
    fn extract_immediate_calls(
        &self,
        node_idx: NodeIndex,
        graph: &DependencyGraph,
        file_nodes: &[(NodeIndex, &Node)],
    ) -> Vec<String> {
        let mut calls = Vec::new();
        let file_node_indices: std::collections::HashSet<NodeIndex> =
            file_nodes.iter().map(|(idx, _)| *idx).collect();

        for edge_ref in graph.edges(node_idx) {
            if matches!(edge_ref.weight().edge_type, crate::core::EdgeType::Call) {
                let target_idx = edge_ref.target();
                if let Some(target_node) = graph.node_weight(target_idx) {
                    if file_node_indices.contains(&target_idx) {
                        // Internal call - allow language adapter to override display
                        if let Some(display) = self.language_adapter.format_call_display(
                            target_idx,
                            target_node,
                            graph,
                        ) {
                            calls.push(display);
                        } else {
                            calls.push(target_node.name.clone());
                        }
                    } else {
                        // External call - show with simplified module context
                        let module_name = self
                            .language_adapter
                            .extract_module_from_path(&target_node.file_path.to_string_lossy());
                        if module_name == "unknown" || module_name.is_empty() {
                            if let Some(display) = self.language_adapter.format_call_display(
                                target_idx,
                                target_node,
                                graph,
                            ) {
                                calls.push(display);
                            } else {
                                calls.push(target_node.name.clone());
                            }
                        } else {
                            // Let adapter override the callee name if applicable
                            let name = if let Some(display) = self
                                .language_adapter
                                .format_call_display(target_idx, target_node, graph)
                            {
                                display
                            } else {
                                target_node.name.clone()
                            };
                            calls.push(format!("{}::{}", module_name, name));
                        }
                    }
                }
            }
        }

        // Sort calls: internal first, then external, prioritize common patterns
        calls.sort_by(|a, b| {
            let a_internal = !a.contains("::");
            let b_internal = !b.contains("::");

            match (a_internal, b_internal) {
                (true, false) => std::cmp::Ordering::Less, // Internal calls first
                (false, true) => std::cmp::Ordering::Greater,
                _ => {
                    // Prioritize common patterns within same category
                    let a_priority = self.get_call_priority(a);
                    let b_priority = self.get_call_priority(b);
                    a_priority.cmp(&b_priority)
                }
            }
        });

        // Limit to first 6 calls for better insight while maintaining compactness
        calls.truncate(6);
        calls
    }

    /// Get priority for call ordering (lower number = higher priority)
    fn get_call_priority(&self, call_name: &str) -> u8 {
        self.language_adapter.get_call_priority(call_name)
    }

    /// Get compact annotations for a function
    fn get_compact_annotations(
        &self,
        node: &Node,
        graph: &DependencyGraph,
        file_nodes: &[(NodeIndex, &Node)],
    ) -> Vec<String> {
        let mut annotations = Vec::new();

        // Find the NodeIndex for this node
        let mut current_node_idx = None;
        for &(idx, file_node) in file_nodes {
            if file_node.name == node.name && file_node.line_number == node.line_number {
                current_node_idx = Some(idx);
                break;
            }
        }

        if let Some(node_idx) = current_node_idx {
            // Check if this function is called by other functions in the same file
            let mut is_called_internally = false;

            for &(caller_idx, _) in file_nodes {
                if caller_idx == node_idx {
                    continue;
                }

                for edge_ref in graph.edges(caller_idx) {
                    if matches!(edge_ref.weight().edge_type, crate::core::EdgeType::Call) {
                        if edge_ref.target() == node_idx {
                            is_called_internally = true;
                            break;
                        }
                    }
                }
                if is_called_internally {
                    break;
                }
            }

            // More precise entry point detection
            if !is_called_internally {
                // True entry points: main, new, parse_file, or public methods
                if node.name == "main"
                    || node.name == "new"
                    || node.name == "parse_file"
                    || node.name.starts_with("format_")
                    || node.visibility.as_ref().map_or(false, |v| v == "public")
                {
                    annotations.push("ENTRY".to_string());
                }
            }

            // Add performance hints
            if node.name.contains("resolve")
                || node.name.contains("compute")
                || node.name.contains("build")
            {
                annotations.push("HOT".to_string());
            }
        }

        // Merge language-specific annotations
        let mut lang = self.language_adapter.language_specific_annotations(node);
        annotations.append(&mut lang);
        annotations
    }

    /// Format a behavioral entity in ultra-compact form for LLM consumption
    fn format_behavioral_entity(&self, entity: &BehavioralEntity) -> String {
        // Use compact signature if available, otherwise fall back to name()
        let mut result = if let Some(ref sig) = entity.signature {
            if sig.is_empty() {
                format!("{}()", entity.name)
            } else {
                self.compact_signature(sig)
            }
        } else {
            format!("{}()", entity.name)
        };

        // Add annotations
        if !entity.annotations.is_empty() {
            result.push_str(&format!("[{}]", entity.annotations.join(",")));
        }

        // Add nested calls if any
        if !entity.nested_calls.is_empty() {
            result.push_str(&format!("→{{{}}}", entity.nested_calls.join(",")));
        }

        result
    }

    /// Convert verbose signature to ultra-compact format for LLM consumption
    fn compact_signature(&self, signature: &str) -> String {
        let mut compact = signature.to_string();

        // Remove excessive whitespace and newlines
        compact = compact.replace('\n', " ").replace('\t', " ");

        // Collapse multiple spaces into single space
        while compact.contains("  ") {
            compact = compact.replace("  ", " ");
        }

        // Remove unnecessary tokens for LLM efficiency
        compact = compact
            .replace("&mut self, ", "") // Remove common self parameter
            .replace("&self, ", "") // Remove immutable self parameter
            .replace("&self", "") // Remove standalone self parameter
            .replace("&Path", "Path") // Simplify common types
            .replace("&str", "str") // Simplify string references
            .replace("&[u8]", "bytes") // Simplify byte slices
            .replace("&TSNode", "Node") // Simplify tree-sitter nodes
            .replace("&mut Vec<Node>", "nodes") // Simplify common parameters
            .replace("&mut Vec<Edge>", "edges") // Simplify common parameters
            .replace("Vec<Node>", "nodes") // Simplify return types
            .replace("Vec<Edge>", "edges") // Simplify return types
            .replace("Option<", "?") // Simplify Option types
            .replace("Result<", "!") // Simplify Result types
            .replace("PathBuf", "Path") // Simplify path types
            .replace("String", "str") // Simplify string types
            .replace("usize", "int") // Simplify integer types
            .replace("bool", "bool") // Keep bool as is
            .replace("()", "void") // Simplify unit type
            .replace(" -> ", "→") // Use arrow symbol
            .replace(" ->", "→") // Handle space variations
            .replace("-> ", "→") // Handle space variations
            .replace(" ->", "→") // Handle space variations
            .replace("->", "→") // Handle no spaces
            .replace("::", ".") // Use dot notation
            .replace("()", "()") // Keep parentheses
            .replace("( ", "(") // Remove space after opening paren
            .replace(" )", ")") // Remove space before closing paren
            .replace(" ,", ",") // Remove space before comma
            .replace(", ", ",") // Remove space after comma
            .replace(" ,", ",") // Handle both cases
            .replace("  ", " ") // Collapse remaining double spaces
            .trim()
            .to_string();

        // Final cleanup - remove any remaining excessive whitespace
        compact.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    /// Extract module name from file path
    #[allow(dead_code)]
    fn extract_module_from_path(&self, path: &str) -> String {
        self.language_adapter.extract_module_from_path(path)
    }

    /// Calculate call depth for a single node (with cycle detection)
    fn calculate_node_call_depth(
        &self,
        node_idx: NodeIndex,
        graph: &DependencyGraph,
        visited: &mut std::collections::HashSet<NodeIndex>,
    ) -> usize {
        if visited.contains(&node_idx) {
            return 0; // Prevent infinite recursion
        }

        visited.insert(node_idx);

        let mut max_child_depth = 0;
        for edge_ref in graph.edges(node_idx) {
            if matches!(edge_ref.weight().edge_type, crate::core::EdgeType::Call) {
                let child_depth = self.calculate_node_call_depth(edge_ref.target(), graph, visited);
                max_child_depth = max_child_depth.max(child_depth);
            }
        }

        visited.remove(&node_idx);
        1 + max_child_depth
    }

    /// Build call trees for functions in a file
    #[allow(dead_code)]
    fn build_call_trees(
        &self,
        file_nodes: &[(NodeIndex, &Node)],
        graph: &DependencyGraph,
    ) -> Vec<CallTreeNode> {
        let mut trees = Vec::new();
        let mut processed = std::collections::HashSet::new();

        // Find entry points (functions not called by others in same file)
        let file_function_indices: std::collections::HashSet<NodeIndex> = file_nodes
            .iter()
            .filter(|(_, node)| matches!(node.node_type, crate::core::NodeType::Function))
            .map(|(idx, _)| *idx)
            .collect();

        let mut entry_points = file_function_indices.clone();

        // Remove functions that are called by other functions in same file
        for &(caller_idx, _) in file_nodes {
            for edge_ref in graph.edges(caller_idx) {
                if matches!(edge_ref.weight().edge_type, crate::core::EdgeType::Call) {
                    let target_idx = edge_ref.target();
                    if file_function_indices.contains(&target_idx) {
                        entry_points.remove(&target_idx);
                    }
                }
            }
        }

        // Build trees starting from entry points
        for &entry_idx in &entry_points {
            if !processed.contains(&entry_idx) {
                if let Some(node) = graph.node_weight(entry_idx) {
                    let tree =
                        self.build_call_tree_recursive(entry_idx, node, graph, &mut processed, 0);
                    trees.push(tree);
                }
            }
        }

        // Handle any remaining functions (potential cycles or orphans)
        for &(func_idx, node) in file_nodes {
            if matches!(node.node_type, crate::core::NodeType::Function)
                && !processed.contains(&func_idx)
            {
                let tree = self.build_call_tree_recursive(func_idx, node, graph, &mut processed, 0);
                trees.push(tree);
            }
        }

        trees
    }

    /// Recursively build call tree
    #[allow(dead_code)]
    fn build_call_tree_recursive(
        &self,
        node_idx: NodeIndex,
        node: &Node,
        graph: &DependencyGraph,
        processed: &mut std::collections::HashSet<NodeIndex>,
        depth: usize,
    ) -> CallTreeNode {
        processed.insert(node_idx);

        let mut children = Vec::new();

        if depth < 4 {
            // Limit depth to prevent explosion
            for edge_ref in graph.edges(node_idx) {
                if matches!(edge_ref.weight().edge_type, crate::core::EdgeType::Call) {
                    let target_idx = edge_ref.target();
                    if let Some(target_node) = graph.node_weight(target_idx) {
                        if !processed.contains(&target_idx) {
                            let child_tree = self.build_call_tree_recursive(
                                target_idx,
                                target_node,
                                graph,
                                processed,
                                depth + 1,
                            );
                            children.push(child_tree);
                        } else {
                            // Reference to already processed node (potential cycle)
                            children.push(CallTreeNode {
                                name: format!("{}[REF]", target_node.name),
                                annotations: vec!["CYCLE".to_string()],
                                children: Vec::new(),
                            });
                        }
                    }
                }
            }
        }

        let mut annotations = self.get_function_annotations(node, &children);
        // Merge language-specific annotations (e.g., Python __init__)
        let mut lang = self.language_adapter.language_specific_annotations(node);
        annotations.append(&mut lang);

        CallTreeNode {
            name: node.name.clone(),
            annotations,
            children,
        }
    }

    /// Get annotations for a function based on its characteristics
    #[allow(unused_variables)]
    fn get_function_annotations(&self, node: &Node, children: &[CallTreeNode]) -> Vec<String> {
        let mut annotations = Vec::new();

        // Entry point detection
        if node.visibility.as_ref().map_or(false, |v| v == "public") {
            annotations.push("ENTRY".to_string());
        }

        // Complexity indicators
        if children.len() > 5 {
            annotations.push("COMPLEX".to_string());
        }

        // Performance hints
        if node.name.contains("resolve") || node.name.contains("compute") {
            annotations.push("HOT_PATH".to_string());
        }

        if node.name.contains("recursive")
            || children
                .iter()
                .any(|c| c.annotations.contains(&"CYCLE".to_string()))
        {
            annotations.push("RECURSIVE".to_string());
        }

        // Merge language-specific annotations at this stage too
        let mut lang = self.language_adapter.language_specific_annotations(node);
        annotations.append(&mut lang);
        annotations
    }

    /// Format call tree with proper indentation
    #[allow(dead_code)]
    fn format_call_tree(&self, output: &mut String, tree: &CallTreeNode, depth: usize) {
        let indent = if depth == 0 {
            "├─ ".to_string()
        } else {
            "│  ".repeat(depth) + "├─ "
        };

        output.push_str(&format!("{}{}()", indent, tree.name));

        if !tree.annotations.is_empty() {
            output.push_str(&format!(" [{}]", tree.annotations.join(",")));
        }

        output.push('\n');

        for (i, child) in tree.children.iter().enumerate() {
            if i == tree.children.len() - 1 {
                // Last child gets different formatting
                let child_indent = "│  ".repeat(depth + 1) + "└─ ";
                output.push_str(&format!("{}{}()", child_indent, child.name));
                if !child.annotations.is_empty() {
                    output.push_str(&format!(" [{}]", child.annotations.join(",")));
                }
                output.push('\n');

                // Continue with children of last child
                for grandchild in &child.children {
                    self.format_call_tree(output, grandchild, depth + 2);
                }
            } else {
                self.format_call_tree(output, child, depth + 1);
            }
        }
    }
}

/// Represents a node in the call tree
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CallTreeNode {
    name: String,
    annotations: Vec<String>,
    children: Vec<CallTreeNode>,
}

/// Represents a behavioral entity with compact nested calls
#[derive(Debug, Clone)]
struct BehavioralEntity {
    name: String,
    #[allow(dead_code)]
    signature: Option<String>,
    annotations: Vec<String>,
    nested_calls: Vec<String>,
}

/// Directory tree structure for path compression (dynamic)
#[derive(Debug)]
struct DirectoryTree {
    common_prefix: String,
    semantic_prefixes: HashMap<String, String>,
    root: DirNode,
}

#[derive(Debug, Default)]
struct DirNode {
    name: String,
    children: std::collections::BTreeMap<String, DirNode>,
    file_count: usize,
    prefix_counts: HashMap<String, usize>,
}

impl DirNode {
    fn new(name: String) -> Self {
        Self {
            name,
            children: std::collections::BTreeMap::new(),
            file_count: 0,
            prefix_counts: HashMap::new(),
        }
    }

    fn add_file(&mut self, parts: &[&str], prefix: &str) {
        if parts.is_empty() {
            return;
        }
        if parts.len() == 1 {
            // File at this directory
            self.file_count += 1;
            *self.prefix_counts.entry(prefix.to_string()).or_insert(0) += 1;
        } else {
            let seg = parts[0];
            let child = self
                .children
                .entry(seg.to_string())
                .or_insert_with(|| DirNode::new(seg.to_string()));
            child.add_file(&parts[1..], prefix);
        }
    }

    fn finalize_counts(&mut self) {
        let keys: Vec<String> = self.children.keys().cloned().collect();
        for k in keys {
            if let Some(child) = self.children.get_mut(&k) {
                child.finalize_counts();
                self.file_count += child.file_count;
                for (p, c) in &child.prefix_counts {
                    *self.prefix_counts.entry(p.clone()).or_insert(0) += c;
                }
            }
        }
    }
}

impl DirectoryTree {
    fn build(paths: Vec<String>) -> Self {
        let common_prefix = Self::find_common_prefix(&paths);
        let semantic_prefixes = Self::build_semantic_prefixes(&paths, &common_prefix);

        let mut root = DirNode::new("".to_string());
        for path in &paths {
            let rel = path
                .strip_prefix(&common_prefix)
                .unwrap_or(path)
                .trim_start_matches('/')
                .to_string();
            if rel.is_empty() {
                continue;
            }
            let parts: Vec<&str> = rel.split('/').collect();
            let prefix = semantic_prefixes
                .get(path)
                .cloned()
                .unwrap_or_else(|| "U".to_string());
            root.add_file(&parts, &prefix);
        }
        let mut tree = Self {
            common_prefix,
            semantic_prefixes,
            root,
        };
        tree.root.finalize_counts();
        tree
    }

    fn find_common_prefix(paths: &[String]) -> String {
        if paths.is_empty() {
            return String::new();
        }

        let first = &paths[0];
        let mut prefix_len = first.len();

        for path in paths.iter().skip(1) {
            prefix_len = first
                .chars()
                .zip(path.chars())
                .take_while(|(a, b)| a == b)
                .count();
        }

        // Trim to last directory separator to avoid partial segments
        let prefix: String = first.chars().take(prefix_len).collect();
        if let Some(pos) = prefix.rfind('/') {
            prefix[..=pos].to_string()
        } else {
            String::new()
        }
    }

    fn build_semantic_prefixes(paths: &[String], common_prefix: &str) -> HashMap<String, String> {
        let mut prefixes = HashMap::new();

        for path in paths {
            let relative_path = path.strip_prefix(common_prefix).unwrap_or(path);
            let semantic_prefix = if relative_path.contains("services/") {
                "S"
            } else if relative_path.contains("entities/") || relative_path.contains("models/") {
                "E"
            } else if relative_path.contains("components/") {
                "C"
            } else if relative_path.contains("widgets/dialogs/") {
                "D"
            } else if relative_path.contains("widgets/ribbon/") {
                "R"
            } else if relative_path.contains("widgets/buttons/") {
                "B"
            } else if relative_path.contains("widgets/view_widgets/")
                || relative_path.contains("views/")
            {
                "V"
            } else if relative_path.contains("widgets/mobile_widgets/") {
                "MB"
            } else if relative_path.contains("widgets/type_widgets/") {
                "T"
            } else if relative_path.contains("widgets/") {
                "W"
            } else if relative_path.contains("menus/") {
                "M"
            } else if relative_path.contains("api/") {
                "A"
            } else if relative_path.contains("controllers/") {
                "CTL"
            } else if relative_path.contains("utils/") || relative_path.contains("helpers/") {
                "U"
            } else if relative_path.contains("tests/") {
                "TST"
            } else {
                "U"
            };

            prefixes.insert(path.clone(), semantic_prefix.to_string());
        }

        prefixes
    }

    fn format_tree(&self) -> String {
        let mut out = String::new();
        // Root children
        let len = self.root.children.len();
        for (i, (_name, node)) in self.root.children.iter().enumerate() {
            let last = i + 1 == len;
            Self::format_dir_node(node, "", last, &mut out);
        }
        out.push('\n');
        out
    }

    fn format_dir_node(node: &DirNode, indent: &str, is_last: bool, out: &mut String) {
        let connector = if is_last { "└─ " } else { "├─ " };
        out.push_str(indent);
        out.push_str(connector);
        out.push_str(&node.name);
        out.push_str("/");

        if !node.prefix_counts.is_empty() {
            let mut parts: Vec<(String, usize)> = node
                .prefix_counts
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect();
            parts.sort_by(|a, b| a.0.cmp(&b.0));
            let details: Vec<String> = parts
                .into_iter()
                .map(|(k, v)| format!("{}[{}]", k, v))
                .collect();
            out.push_str(" → ");
            out.push_str(&details.join(" "));
        }
        out.push('\n');

        let new_indent = format!("{}{}", indent, if is_last { "   " } else { "│  " });
        let len = node.children.len();
        for (i, (_name, child)) in node.children.iter().enumerate() {
            let last = i + 1 == len;
            Self::format_dir_node(child, &new_indent, last, out);
        }
    }
}

impl Default for LLMOptimizedFormatter {
    fn default() -> Self {
        Self::new()
    }
}

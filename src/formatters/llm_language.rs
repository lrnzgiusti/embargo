use crate::core::{DependencyGraph, Node, NodeType};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;

/// Language-specific hooks to tune the LLM-optimized formatter
pub trait LlmLanguageAdapter {
    /// Adapter name (e.g., "default", "python")
    #[allow(dead_code)]
    fn name(&self) -> &'static str {
        "default"
    }

    /// Classify a node into an architectural cluster for grouping
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

    /// Priority for call sorting (lower number = higher priority)
    fn get_call_priority(&self, call_name: &str) -> u8 {
        if call_name.starts_with("extract_") {
            1
        } else if call_name.starts_with("process_") {
            2
        } else if call_name.starts_with("build_") {
            3
        } else if call_name.starts_with("format_") {
            4
        } else if call_name.contains("::new") || call_name == "new" {
            1
        } else if call_name.contains("parse") {
            1
        } else {
            5
        }
    }

    /// Additional language-specific function annotations (merged with generic ones)
    fn language_specific_annotations(&self, _node: &Node) -> Vec<String> {
        Vec::new()
    }

    /// Optional display override for a called target (e.g., Python __init__ -> ClassName())
    fn format_call_display(
        &self,
        _target_idx: NodeIndex,
        _target_node: &Node,
        _graph: &DependencyGraph,
    ) -> Option<String> {
        None
    }

    /// Extract filename from a path
    fn extract_filename(&self, path: &str) -> String {
        path.split('/').last().unwrap_or("unknown").to_string()
    }

    /// Extract module name from a path
    fn extract_module_from_path(&self, path: &str) -> String {
        if let Some(stem) = std::path::Path::new(path).file_stem() {
            stem.to_string_lossy().to_string()
        } else {
            "unknown".to_string()
        }
    }
}

/// Default adapter that mirrors the existing generic behavior
pub struct DefaultLanguageAdapter;

impl DefaultLanguageAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl LlmLanguageAdapter for DefaultLanguageAdapter {}

/// Python-specific adapter for richer intra-file and instantiation hints
pub struct PythonLanguageAdapter;

impl PythonLanguageAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl LlmLanguageAdapter for PythonLanguageAdapter {
    fn name(&self) -> &'static str {
        "python"
    }

    fn classify_node_cluster(&self, node: &Node) -> String {
        let path = node.file_path.to_string_lossy();

        // Common Python project layouts
        if path.contains("/services/") {
            "SERVICES".to_string()
        } else if path.contains("/models/") || path.contains("/entities/") {
            "DATA_MODELS".to_string()
        } else if path.contains("/views/") {
            "VIEWS".to_string()
        } else if path.contains("/controllers/") {
            "CONTROLLERS".to_string()
        } else if path.contains("/api/") {
            "API_LAYER".to_string()
        } else if path.contains("/utils/") || path.contains("/helpers/") {
            "UTILS".to_string()
        } else if path.contains("/tests/") || node.name.starts_with("test_") {
            "TESTS".to_string()
        } else {
            // fall back to generic classification
            LlmLanguageAdapter::classify_node_cluster(&DefaultLanguageAdapter, node)
        }
    }

    fn get_call_priority(&self, call_name: &str) -> u8 {
        // Prioritize common Python workflow verbs
        if call_name.starts_with("load_")
            || call_name.starts_with("save_")
            || call_name.starts_with("from_")
            || call_name.starts_with("to_")
            || call_name.starts_with("parse")
        {
            1
        } else if call_name == "__init__" {
            1
        } else if call_name.starts_with("fit") || call_name.starts_with("predict") {
            2
        } else {
            LlmLanguageAdapter::get_call_priority(&DefaultLanguageAdapter, call_name)
        }
    }

    fn language_specific_annotations(&self, node: &Node) -> Vec<String> {
        let mut ann = Vec::new();
        if node.name == "__init__" {
            ann.push("CTOR".to_string());
        }
        if node.name.starts_with("test_") {
            ann.push("TEST".to_string());
        }
        if node.name.starts_with("__") && node.name.ends_with("__") {
            ann.push("DUNDER".to_string());
        }
        ann
    }

    fn format_call_display(
        &self,
        target_idx: NodeIndex,
        target_node: &Node,
        graph: &DependencyGraph,
    ) -> Option<String> {
        // If the call targets a method named __init__, prefer the class name as a constructor
        if target_node.node_type == NodeType::Function && target_node.name == "__init__" {
            // Find owning class via incoming Contains edge
            for edge_ref in graph.edges_directed(target_idx, petgraph::Direction::Incoming) {
                if let Some(source_node) = graph.node_weight(edge_ref.source()) {
                    if source_node.node_type == NodeType::Class {
                        return Some(format!("{}()", source_node.name));
                    }
                }
            }
        }
        None
    }
}

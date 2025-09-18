use anyhow::Result;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::core::{DependencyGraph, EdgeType, NodeType};

/// JSON formatter optimized for LLM consumption with minimal tokens
pub struct JsonCompactFormatter {
    /// Include full metadata or just essential information
    minimal: bool,
}

impl JsonCompactFormatter {
    pub fn new() -> Self {
        Self { minimal: true }
    }

    pub fn format_to_file(&self, graph: &DependencyGraph, output_path: &Path) -> Result<()> {
        let json_content = self.format_graph(graph)?;
        fs::write(output_path, json_content)?;
        Ok(())
    }

    fn format_graph(&self, graph: &DependencyGraph) -> Result<String> {
        let node_indices: Vec<NodeIndex> = graph.node_indices().collect();

        // Build compressed node mappings
        let mut nodes = Vec::new();
        let mut node_id_map = HashMap::new();
        let mut file_map = HashMap::new();
        let mut file_id = 0u16;

        // First pass: build file mapping
        for &idx in &node_indices {
            if let Some(node) = graph.node_weight(idx) {
                let path_str = node.file_path.to_string_lossy();
                if !file_map.contains_key(path_str.as_ref()) {
                    file_map.insert(path_str.to_string(), file_id);
                    file_id += 1;
                }
            }
        }

        // Second pass: build nodes
        for (node_idx, &idx) in node_indices.iter().enumerate() {
            if let Some(node) = graph.node_weight(idx) {
                node_id_map.insert(idx, node_idx);

                let file_id = file_map[&node.file_path.to_string_lossy().to_string()];

                let node_json = if self.minimal {
                    json!({
                        "n": node.name,
                        "t": self.type_code(node.node_type),
                        "f": file_id,
                        "l": node.line_number
                    })
                } else {
                    let mut node_obj = json!({
                        "id": node.id,
                        "name": node.name,
                        "type": self.type_code(node.node_type),
                        "file": file_id,
                        "line": node.line_number,
                        "lang": node.language
                    });

                    if let Some(ref sig) = node.signature {
                        node_obj["sig"] = json!(sig);
                    }
                    if let Some(ref vis) = node.visibility {
                        node_obj["vis"] = json!(vis);
                    }

                    node_obj
                };

                nodes.push(node_json);
            }
        }

        // Build edges efficiently
        let mut edges = Vec::new();
        for edge_ref in graph.edge_references() {
            let source_idx = edge_ref.source();
            let target_idx = edge_ref.target();

            if let (Some(&src_id), Some(&tgt_id)) =
                (node_id_map.get(&source_idx), node_id_map.get(&target_idx))
            {
                let edge_json = if self.minimal {
                    json!([src_id, tgt_id, self.edge_code(edge_ref.weight().edge_type)])
                } else {
                    json!({
                        "src": src_id,
                        "tgt": tgt_id,
                        "type": self.edge_code(edge_ref.weight().edge_type),
                        "ctx": edge_ref.weight().context
                    })
                };
                edges.push(edge_json);
            }
        }

        // Build file mapping for output
        let files: Vec<String> = {
            let mut file_vec = vec![String::new(); file_map.len()];
            for (path, id) in file_map {
                file_vec[id as usize] = path;
            }
            file_vec
        };

        let output = json!({
            "meta": {
                "nodes": graph.node_count(),
                "edges": graph.edge_count(),
                "format": if self.minimal { "compact" } else { "full" }
            },
            "files": files,
            "nodes": nodes,
            "edges": edges
        });

        Ok(serde_json::to_string(&output)?)
    }

    fn type_code(&self, node_type: NodeType) -> u8 {
        match node_type {
            NodeType::Module => 0,
            NodeType::Class => 1,
            NodeType::Function => 2,
            NodeType::Variable => 3,
            NodeType::Interface => 4,
            NodeType::Enum => 5,
        }
    }

    fn edge_code(&self, edge_type: EdgeType) -> u8 {
        match edge_type {
            EdgeType::Import => 0,
            EdgeType::Call => 1,
            EdgeType::Inheritance => 2,
            EdgeType::Implements => 3,
            EdgeType::Uses => 4,
            EdgeType::Contains => 5,
        }
    }
}

impl Default for JsonCompactFormatter {
    fn default() -> Self {
        Self::new()
    }
}

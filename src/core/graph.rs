//! Dependency graph data structures.
//!
//! This module defines the core types for representing code entities and their relationships.

use petgraph::{graph::NodeIndex, Directed, Graph};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Type of code entity in the dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum NodeType {
    /// Module or file-level import
    Module,
    /// Class or struct definition
    Class,
    /// Function or method definition
    Function,
    /// Variable or constant declaration
    Variable,
    /// Interface or trait definition
    Interface,
    /// Enum type definition
    Enum,
}

/// Type of relationship between code entities.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum EdgeType {
    /// Module import relationship
    Import,
    /// Function call relationship
    Call,
    /// Class inheritance relationship
    Inheritance,
    /// Interface implementation
    Implements,
    /// General usage relationship
    Uses,
    /// Containment (e.g., class contains method)
    Contains,
}

/// A node representing a code entity in the dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier: `filepath:type:name:line`
    pub id: String,
    /// Entity name
    pub name: String,
    /// Entity type
    pub node_type: NodeType,
    /// Source file path
    pub file_path: PathBuf,
    /// Line number where entity is defined
    pub line_number: usize,
    /// Programming language
    pub language: String,
    /// Function/method signature with parameters and types
    pub signature: Option<String>,
    /// Documentation string
    pub docstring: Option<String>,
    /// Visibility modifier (public, private, etc.)
    pub visibility: Option<String>,
}

/// An edge representing a relationship between two code entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Type of relationship
    pub edge_type: EdgeType,
    /// Source node identifier
    pub source_id: String,
    /// Target node identifier
    pub target_id: String,
    /// Additional context about the relationship
    pub context: Option<String>,
}

/// Directed graph of code dependencies using petgraph.
pub type DependencyGraph = Graph<Node, Edge, Directed>;

impl Node {
    pub fn new(
        id: String,
        name: String,
        node_type: NodeType,
        file_path: PathBuf,
        line_number: usize,
        language: String,
    ) -> Self {
        Self {
            id,
            name,
            node_type,
            file_path,
            line_number,
            language,
            signature: None,
            docstring: None,
            visibility: None,
        }
    }

    pub fn with_signature(mut self, signature: String) -> Self {
        self.signature = Some(signature);
        self
    }

    pub fn with_docstring(mut self, docstring: String) -> Self {
        self.docstring = Some(docstring);
        self
    }

    pub fn with_visibility(mut self, visibility: String) -> Self {
        self.visibility = Some(visibility);
        self
    }
}

impl Edge {
    pub fn new(edge_type: EdgeType, source_id: String, target_id: String) -> Self {
        Self {
            edge_type,
            source_id,
            target_id,
            context: None,
        }
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }
}

/// Builder for constructing dependency graphs incrementally.
pub struct GraphBuilder {
    graph: DependencyGraph,
    node_map: HashMap<String, NodeIndex>,
}

impl GraphBuilder {
    /// Creates a new empty graph builder.
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) -> NodeIndex {
        let id = node.id.clone();
        let index = self.graph.add_node(node);
        self.node_map.insert(id, index);
        index
    }

    pub fn add_edge(&mut self, edge: Edge) -> Option<petgraph::graph::EdgeIndex> {
        let source_idx = self.node_map.get(&edge.source_id)?;
        let target_idx = self.node_map.get(&edge.target_id)?;
        Some(self.graph.add_edge(*source_idx, *target_idx, edge))
    }

    pub fn build(self) -> DependencyGraph {
        self.graph
    }

    #[allow(dead_code)]
    pub fn get_node_index(&self, id: &str) -> Option<NodeIndex> {
        self.node_map.get(id).copied()
    }
}

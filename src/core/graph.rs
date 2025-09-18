use petgraph::{graph::NodeIndex, Directed, Graph};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum NodeType {
    Module,
    Class,
    Function,
    Variable,
    Interface,
    Enum,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum EdgeType {
    Import,
    Call,
    Inheritance,
    Implements,
    Uses,
    Contains,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub file_path: PathBuf,
    pub line_number: usize,
    pub language: String,
    pub signature: Option<String>,
    pub docstring: Option<String>,
    pub visibility: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub edge_type: EdgeType,
    pub source_id: String,
    pub target_id: String,
    pub context: Option<String>,
}

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

pub struct GraphBuilder {
    graph: DependencyGraph,
    node_map: HashMap<String, NodeIndex>,
}

impl GraphBuilder {
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

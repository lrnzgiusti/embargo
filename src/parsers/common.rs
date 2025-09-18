use anyhow::Result;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tree_sitter::{Language, Node as TSNode, Parser, Tree};

pub struct TreeSitterParser {
    parser: Parser,
    #[allow(dead_code)]
    language: Language,
}

impl TreeSitterParser {
    pub fn new(language: Language) -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(language)?;
        Ok(Self { parser, language })
    }

    pub fn parse_file(&mut self, file_path: &Path) -> Result<Tree> {
        let source = self.read_file_optimized(file_path)?;
        let tree = self
            .parser
            .parse(&source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse file: {}", file_path.display()))?;
        Ok(tree)
    }

    pub fn get_source(&self, file_path: &Path) -> Result<String> {
        self.read_file_optimized(file_path)
    }

    /// Optimized file reading with buffering for better I/O performance
    fn read_file_optimized(&self, file_path: &Path) -> Result<String> {
        let file = File::open(file_path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len() as usize;

        // Use buffered reader with optimal buffer size
        let mut reader =
            BufReader::with_capacity(if file_size < 8192 { file_size } else { 8192 }, file);

        // Pre-allocate string with known capacity
        let mut content = String::with_capacity(file_size);
        reader.read_to_string(&mut content)?;
        Ok(content)
    }
}

pub fn extract_text<'a>(node: &TSNode, source: &'a [u8]) -> &'a str {
    std::str::from_utf8(&source[node.byte_range()]).unwrap_or("")
}

pub fn generate_node_id(file_path: &Path, node_type: &str, name: &str, line: usize) -> String {
    format!(
        "{}:{}:{}:{}",
        file_path
            .to_string_lossy()
            .replace('/', "_")
            .replace('\\', "_"),
        node_type,
        name,
        line
    )
}

pub fn extract_docstring(node: &TSNode, source: &[u8]) -> Option<String> {
    for child in node.children(&mut node.walk()) {
        if child.kind() == "expression_statement" {
            if let Some(string_node) = child.child(0) {
                if string_node.kind() == "string" {
                    let docstring = extract_text(&string_node, source);
                    if docstring.starts_with("\"\"\"") || docstring.starts_with("'''") {
                        return Some(
                            docstring
                                .trim_matches(|c| c == '"' || c == '\'')
                                .to_string(),
                        );
                    }
                }
            }
        }
    }
    None
}

pub fn find_child_by_kind<'a>(node: &'a TSNode, kind: &str) -> Option<TSNode<'a>> {
    for child in node.children(&mut node.walk()) {
        if child.kind() == kind {
            return Some(child);
        }
    }
    None
}

pub fn find_children_by_kind<'a>(node: &'a TSNode<'a>, kind: &str) -> Vec<TSNode<'a>> {
    let mut results = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == kind {
            results.push(child);
        }
    }
    results
}

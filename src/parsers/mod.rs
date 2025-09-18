pub mod cache;
pub mod common;
pub mod cpp;
pub mod csharp;
pub mod go;
pub mod java;
pub mod javascript;
pub mod python;
pub mod rust;
pub mod typescript;

use anyhow::Result;
use std::path::Path;

use crate::core::{CallSite, Edge, Node};

#[derive(Debug, Clone)]
pub struct ParseResult {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub call_sites: Option<Vec<CallSite>>,
}

pub trait LanguageParser {
    fn parse_file(&self, file_path: &Path) -> Result<ParseResult>;
    #[allow(dead_code)]
    fn language_name(&self) -> &str;
}

pub struct ParserFactory;

impl ParserFactory {
    pub fn new() -> Self {
        Self
    }

    pub fn get_parser(&self, language: &str) -> Result<Box<dyn LanguageParser + Send + Sync>> {
        match language {
            "python" => Ok(Box::new(python::PythonParser::new()?)),
            "typescript" => Ok(Box::new(typescript::TypeScriptParser::new()?)),
            "javascript" => Ok(Box::new(javascript::JavaScriptParser::new()?)),
            "cpp" | "c++" => Ok(Box::new(cpp::CppParser::new()?)),
            "rust" => Ok(Box::new(rust::RustParser::new()?)),
            "java" => Ok(Box::new(java::JavaParser::new()?)),
            "go" => Ok(Box::new(go::GoParser::new()?)),
            "csharp" | "c#" => Ok(Box::new(csharp::CSharpParser::new()?)),
            _ => anyhow::bail!("Unsupported language: {}", language),
        }
    }
}

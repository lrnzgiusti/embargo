//! # EMBARGO
//!
//! Fast codebase dependency extraction for AI code analysis.
//!
//! EMBARGO analyzes source code across multiple programming languages and generates
//! structured dependency graphs optimized for LLM consumption.
//!
//! ## Output Formats
//!
//! - **LLM-Optimized**: Compact format with semantic clustering and behavioral notation
//! - **Markdown**: Traditional readable format with full details
//! - **JSON-Compact**: Minimal token format for programmatic consumption
//!
//! ## Supported Languages
//!
//! Python, TypeScript, Rust, C++, JavaScript, Java, C#, Go

pub mod core;
pub mod formatters;
pub mod parsers;

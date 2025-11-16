use anyhow::Result;
use std::path::Path;

use super::{DependencyGraph, FileScanner, FunctionResolver};
use crate::parsers::{cache::ParseCache, ParserFactory};

pub struct CodebaseAnalyzer {
    file_scanner: FileScanner,
    parser_factory: ParserFactory,
    function_resolver: FunctionResolver,
    parse_cache: ParseCache,
}

impl CodebaseAnalyzer {
    pub fn new() -> Self {
        Self {
            file_scanner: FileScanner::new(),
            parser_factory: ParserFactory::new(),
            function_resolver: FunctionResolver::new(),
            parse_cache: ParseCache::new(None).unwrap_or_else(|err| {
                eprintln!("Warning: Failed to initialize disk parse cache: {err}");
                ParseCache::in_memory_only()
            }),
        }
    }

    pub fn analyze(&mut self, root_path: &Path, languages: &[&str]) -> Result<DependencyGraph> {
        println!("Scanning files...");
        let files = self.file_scanner.scan_directory(root_path, languages)?;
        println!("Found {} files to analyze", files.len());

        let mut graph_builder = super::graph::GraphBuilder::new();

        println!("Parsing files with cache optimization...");

        // Check which files need reparsing
        let mut cached_count = 0;
        let mut parse_results = Vec::with_capacity(files.len());

        // Process files with cache checking (sequential for cache access)
        for file_info in &files {
            match self.parse_cache.needs_update(&file_info.path) {
                Ok(needs_update) => {
                    if !needs_update {
                        if let Some(cached_result) = self.parse_cache.get(&file_info.path) {
                            parse_results.push(cached_result);
                            cached_count += 1;
                            continue;
                        }
                    }
                }
                Err(err) => {
                    eprintln!(
                        "Warning: Failed to validate cache entry for {}: {}",
                        file_info.path.display(),
                        err
                    );
                }
            }

            // Parse file if not cached or cache miss
            if let Ok(parser) = self.parser_factory.get_parser(&file_info.language) {
                match parser.parse_file(&file_info.path) {
                    Ok(result) => {
                        // Store in cache for next time
                        if let Err(e) = self.parse_cache.store(&file_info.path, &result) {
                            eprintln!(
                                "Warning: Failed to cache {}: {}",
                                file_info.path.display(),
                                e
                            );
                        }
                        parse_results.push(result);
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to parse {}: {}",
                            file_info.path.display(),
                            e
                        );
                    }
                }
            } else {
                eprintln!(
                    "Warning: Unsupported language '{}' for file {}",
                    file_info.language,
                    file_info.path.display()
                );
            }
        }

        println!(
            "Cache hits: {}, Parsed: {}",
            cached_count,
            parse_results.len() - cached_count
        );

        println!("Building dependency graph...");

        // Pre-calculate total capacity to avoid reallocations
        let total_nodes: usize = parse_results.iter().map(|r| r.nodes.len()).sum();
        let _total_edges: usize = parse_results.iter().map(|r| r.edges.len()).sum();

        // Pre-allocate collections with known capacity
        let mut all_nodes = Vec::with_capacity(total_nodes);
        let mut all_call_sites: Vec<crate::core::CallSite> = Vec::new();

        for mut parse_result in parse_results {
            for node in &parse_result.nodes {
                // Retain a separate copy for the resolver indexes
                all_nodes.push(node.clone());
            }

            for node in parse_result.nodes.drain(..) {
                graph_builder.add_node(node);
            }

            for edge in parse_result.edges {
                graph_builder.add_edge(edge);
            }

            if let Some(call_sites) = parse_result.call_sites {
                all_call_sites.extend(call_sites);
            }
        }

        println!("Resolving function calls...");

        // Build function resolution index using optimized parallel processing
        let mut resolver = self.function_resolver.clone();
        resolver.build_indexes(&all_nodes)?;

        // Resolve function calls into edges when call sites are available
        if !all_call_sites.is_empty() {
            let call_edges = resolver.resolve_calls(&all_call_sites);
            let mut added = 0usize;
            for edge in call_edges {
                if graph_builder.add_edge(edge).is_some() {
                    added += 1;
                }
            }
            println!("Resolved {} call edges", added);
        } else {
            println!("No call sites detected; skipping call resolution");
        }

        Ok(graph_builder.build())
    }
}

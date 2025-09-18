use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use super::ParseResult;
use crate::core::{CallSite, Edge, Node};

const DEFAULT_MAX_MEMORY_ENTRIES: usize = 1000;

/// Fast cache for parsed results using file modification timestamps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFileEntry {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub call_sites: Option<Vec<CallSite>>,
    pub timestamp: u64,
    pub file_size: u64,
}

/// High-performance thread-safe cache with memory and (best-effort) disk storage
pub struct ParseCache {
    memory_cache: DashMap<PathBuf, ParsedFileEntry>,
    cache_dir: Option<PathBuf>,
    max_memory_entries: usize,
}

impl ParseCache {
    pub fn new(cache_dir: Option<PathBuf>) -> Result<Self> {
        let resolved_dir = cache_dir.unwrap_or_else(|| std::env::temp_dir().join("embargo_cache"));
        let cache_dir = match fs::create_dir_all(&resolved_dir) {
            Ok(()) => Some(resolved_dir),
            Err(err) => {
                eprintln!(
                    "Warning: Failed to initialize disk cache at {}: {err}",
                    resolved_dir.display()
                );
                None
            }
        };

        Ok(Self {
            memory_cache: DashMap::with_capacity(DEFAULT_MAX_MEMORY_ENTRIES),
            cache_dir,
            max_memory_entries: DEFAULT_MAX_MEMORY_ENTRIES,
        })
    }

    /// Build an in-memory-only cache without touching the filesystem
    pub fn in_memory_only() -> Self {
        Self {
            memory_cache: DashMap::with_capacity(DEFAULT_MAX_MEMORY_ENTRIES),
            cache_dir: None,
            max_memory_entries: DEFAULT_MAX_MEMORY_ENTRIES,
        }
    }

    /// Check if file needs reparsing based on modification time and size
    pub fn needs_update(&self, file_path: &Path) -> Result<bool> {
        let metadata = fs::metadata(file_path)?;
        let current_timestamp = metadata
            .modified()?
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let current_size = metadata.len();

        if let Some(entry) = self.memory_cache.get(file_path) {
            return Ok(entry.timestamp != current_timestamp || entry.file_size != current_size);
        }

        if let Some(cache_path) = self.cache_path(file_path) {
            if cache_path.exists() {
                if let Ok(entry) = self.load_from_disk(&cache_path) {
                    return Ok(
                        entry.timestamp != current_timestamp || entry.file_size != current_size
                    );
                }
            }
        }

        Ok(true)
    }

    /// Get cached parse result if valid
    pub fn get(&self, file_path: &Path) -> Option<ParseResult> {
        if let Some(entry) = self.memory_cache.get(file_path) {
            return Some(ParseResult {
                nodes: entry.nodes.clone(),
                edges: entry.edges.clone(),
                call_sites: entry.call_sites.clone(),
            });
        }

        if let Some(cache_path) = self.cache_path(file_path) {
            if let Ok(entry) = self.load_from_disk(&cache_path) {
                let result = ParseResult {
                    nodes: entry.nodes.clone(),
                    edges: entry.edges.clone(),
                    call_sites: entry.call_sites.clone(),
                };

                if self.memory_cache.len() < self.max_memory_entries {
                    self.memory_cache.insert(file_path.to_path_buf(), entry);
                }

                return Some(result);
            }
        }

        None
    }

    /// Store parse result in cache
    pub fn store(&self, file_path: &Path, result: &ParseResult) -> Result<()> {
        let metadata = fs::metadata(file_path)?;
        let timestamp = metadata
            .modified()?
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let file_size = metadata.len();

        let entry = ParsedFileEntry {
            nodes: result.nodes.clone(),
            edges: result.edges.clone(),
            call_sites: result.call_sites.clone(),
            timestamp,
            file_size,
        };

        if self.memory_cache.len() >= self.max_memory_entries {
            if let Some(entry) = self.memory_cache.iter().next() {
                let key = entry.key().clone();
                drop(entry);
                self.memory_cache.remove(&key);
            }
        }
        self.memory_cache
            .insert(file_path.to_path_buf(), entry.clone());

        if let Some(cache_path) = self.cache_path(file_path) {
            self.store_to_disk(&cache_path, &entry)?;
        }

        Ok(())
    }

    /// Clear all caches
    #[allow(dead_code)]
    pub fn clear(&self) -> Result<()> {
        self.memory_cache.clear();
        if let Some(cache_dir) = &self.cache_dir {
            if cache_dir.exists() {
                fs::remove_dir_all(cache_dir)?;
                fs::create_dir_all(cache_dir)?;
            }
        }
        Ok(())
    }

    /// Get cache hit statistics
    #[allow(dead_code)]
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            memory_entries: self.memory_cache.len(),
            disk_cache_size: self.get_disk_cache_size(),
        }
    }

    fn cache_path(&self, file_path: &Path) -> Option<PathBuf> {
        let cache_dir = self.cache_dir.as_ref()?;

        let mut hasher = DefaultHasher::new();
        file_path.hash(&mut hasher);
        let hash = hasher.finish();

        Some(cache_dir.join(format!("cache_{:x}.bincode", hash)))
    }

    fn load_from_disk(&self, cache_path: &Path) -> Result<ParsedFileEntry> {
        let data = fs::read(cache_path)?;
        let entry: ParsedFileEntry = bincode::deserialize(&data)?;
        Ok(entry)
    }

    fn store_to_disk(&self, cache_path: &Path, entry: &ParsedFileEntry) -> Result<()> {
        let data = bincode::serialize(entry)?;
        fs::write(cache_path, data)?;
        Ok(())
    }

    #[allow(dead_code)]
    fn get_disk_cache_size(&self) -> usize {
        if let Some(cache_dir) = &self.cache_dir {
            if let Ok(entries) = fs::read_dir(cache_dir) {
                entries.filter_map(|e| e.ok()).count()
            } else {
                0
            }
        } else {
            0
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct CacheStats {
    pub memory_entries: usize,
    pub disk_cache_size: usize,
}

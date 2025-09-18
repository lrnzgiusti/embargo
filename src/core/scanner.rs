use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub language: String,
    #[allow(dead_code)]
    pub extension: String,
}

pub struct FileScanner;

impl FileScanner {
    pub fn new() -> Self {
        Self
    }

    pub fn scan_directory(&self, root_path: &Path, languages: &[&str]) -> Result<Vec<FileInfo>> {
        let supported_extensions = self.get_extensions_for_languages(languages);

        // Collect all entries first for parallel processing
        let entries: Vec<_> = WalkDir::new(root_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| entry.path().is_file())
            .collect();

        // Process entries in parallel
        let files: Vec<FileInfo> = entries
            .par_iter()
            .filter_map(|entry| {
                let path = entry.path();
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .and_then(|extension| {
                        supported_extensions
                            .get(extension)
                            .map(|language| FileInfo {
                                path: path.to_path_buf(),
                                language: language.clone(),
                                extension: extension.to_string(),
                            })
                    })
            })
            .collect();

        Ok(files)
    }

    fn get_extensions_for_languages(
        &self,
        languages: &[&str],
    ) -> std::collections::HashMap<&str, String> {
        let mut extensions = std::collections::HashMap::with_capacity(languages.len() * 3);

        for &language in languages {
            match language {
                "python" => {
                    extensions.insert("py", "python".to_string());
                    extensions.insert("pyi", "python".to_string());
                    extensions.insert("pyw", "python".to_string());
                }
                "typescript" => {
                    extensions.insert("ts", "typescript".to_string());
                    extensions.insert("tsx", "typescript".to_string());
                }
                "javascript" => {
                    extensions.insert("js", "javascript".to_string());
                    extensions.insert("jsx", "javascript".to_string());
                    extensions.insert("mjs", "javascript".to_string());
                }
                "rust" => {
                    extensions.insert("rs", "rust".to_string());
                }
                "go" => {
                    extensions.insert("go", "go".to_string());
                }
                "java" => {
                    extensions.insert("java", "java".to_string());
                }
                "cpp" | "c++" => {
                    extensions.insert("cpp", "cpp".to_string());
                    extensions.insert("cxx", "cpp".to_string());
                    extensions.insert("cc", "cpp".to_string());
                    extensions.insert("hpp", "cpp".to_string());
                }
                "c" => {
                    extensions.insert("c", "c".to_string());
                    extensions.insert("h", "c".to_string());
                }
                "csharp" | "c#" => {
                    extensions.insert("cs", "csharp".to_string());
                }
                _ => {}
            }
        }

        extensions
    }
}

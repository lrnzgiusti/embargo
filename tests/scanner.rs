use embargo::core::scanner::FileScanner;
use std::fs;
use std::path::Path;

fn touch<P: AsRef<Path>>(p: P) {
    fs::write(p, "// test").unwrap();
}

#[test]
fn scanner_filters_by_language_extensions() {
    let dir = tempfile::TempDir::new().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("a")).unwrap();
    fs::create_dir_all(root.join("b")).unwrap();

    // rust, python, js files
    touch(root.join("a/lib.rs"));
    touch(root.join("a/main.py"));
    touch(root.join("b/app.js"));
    touch(root.join("b/readme.txt")); // ignored

    let scanner = FileScanner::new();
    let files = scanner
        .scan_directory(root, &["rust", "python", "javascript"])
        .unwrap();

    let mut langs: Vec<_> = files.iter().map(|f| f.language.as_str()).collect();
    langs.sort();
    assert_eq!(langs, vec!["javascript", "python", "rust"]);
}

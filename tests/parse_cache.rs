use embargo::parsers::rust::RustParser;
use embargo::parsers::{cache::ParseCache, LanguageParser};
use std::fs;
use std::time::Duration;

#[test]
fn parse_cache_stores_and_detects_updates() {
    let dir = tempfile::TempDir::new().unwrap();
    let file = dir.path().join("prog.rs");
    fs::write(&file, "fn a() {}\n").unwrap();

    let parser = RustParser::new().unwrap();
    let result = parser.parse_file(&file).unwrap();

    let cache = ParseCache::new(None).unwrap();

    // Initially no cache, needs update should be true
    assert!(cache.needs_update(&file).unwrap());

    cache.store(&file, &result).unwrap();

    // Immediately after store, should not need update
    assert!(!cache.needs_update(&file).unwrap());
    assert!(cache.get(&file).is_some());

    // Modify file to force update
    std::thread::sleep(Duration::from_millis(5));
    fs::write(&file, "fn a() {}\nfn b() {}\n").unwrap();

    assert!(cache.needs_update(&file).unwrap());
    let new_result = parser.parse_file(&file).unwrap();
    cache.store(&file, &new_result).unwrap();
    assert!(cache.get(&file).is_some());
}

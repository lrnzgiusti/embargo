use criterion::{black_box, criterion_group, criterion_main, Criterion};
use embargo::core::CodebaseAnalyzer;
use std::path::Path;

fn benchmark_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("codebase_analysis");

    // Create test directories with sample code
    let test_dir = std::env::temp_dir().join("embargo_bench");
    std::fs::create_dir_all(&test_dir).unwrap();

    // Create sample Python files
    for i in 0..10 {
        let content = format!(
            r#"
class TestClass{}:
    def __init__(self):
        self.value = {}
    
    def process(self):
        return self.calculate() * 2
    
    def calculate(self):
        return self.value + 10

def main():
    instance = TestClass{}()
    return instance.process()

if __name__ == "__main__":
    main()
"#,
            i, i, i
        );
        std::fs::write(test_dir.join(format!("test_{}.py", i)), content).unwrap();
    }

    // Create sample TypeScript files
    for i in 0..10 {
        let content = format!(
            r#"
interface Data{} {{
    value: number;
    name: string;
}}

class Service{} {{
    private data: Data{};
    
    constructor(data: Data{}) {{
        this.data = data;
    }}
    
    process(): number {{
        return this.calculate() * 2;
    }}
    
    private calculate(): number {{
        return this.data.value + 10;
    }}
}}

export function createService{}(value: number): Service{} {{
    return new Service{}({{ value, name: "test{}" }});
}}
"#,
            i, i, i, i, i, i, i, i
        );
        std::fs::write(test_dir.join(format!("service_{}.ts", i)), content).unwrap();
    }

    // Benchmark small codebase
    group.bench_function("small_codebase", |b| {
        b.iter(|| {
            let mut analyzer = CodebaseAnalyzer::new();
            let result =
                analyzer.analyze(black_box(&test_dir), black_box(&["python", "typescript"]));
            black_box(result)
        });
    });

    // Create larger test set for scalability testing
    let large_test_dir = std::env::temp_dir().join("embargo_bench_large");
    std::fs::create_dir_all(&large_test_dir).unwrap();

    for i in 0..100 {
        let content = format!(
            r#"
class Component{} {{
    constructor(private id: number = {}) {{}}
    
    render(): string {{
        return `<div id="${{this.id}}">${{this.process()}}</div>`;
    }}
    
    process(): string {{
        return this.helper().toUpperCase();
    }}
    
    private helper(): string {{
        return `component_${{this.id}}`;
    }}
}}

export default Component{};
"#,
            i, i, i
        );
        std::fs::write(large_test_dir.join(format!("component_{}.ts", i)), content).unwrap();
    }

    // Benchmark larger codebase
    group.bench_function("large_codebase", |b| {
        b.iter(|| {
            let mut analyzer = CodebaseAnalyzer::new();
            let result = analyzer.analyze(black_box(&large_test_dir), black_box(&["typescript"]));
            black_box(result)
        });
    });

    group.finish();
}

fn benchmark_cache_performance(c: &mut Criterion) {
    use embargo::parsers::cache::ParseCache;
    use tempfile::TempDir;

    let mut group = c.benchmark_group("cache_performance");

    // Setup test files
    let test_dir = TempDir::new().unwrap();
    let test_file = test_dir.path().join("test.py");
    std::fs::write(&test_file, "def test(): return 42").unwrap();

    group.bench_function("cache_store_and_retrieve", |b| {
        b.iter(|| {
            let cache = ParseCache::new(None).unwrap();
            // First access - cache miss
            let needs_update = cache.needs_update(black_box(&test_file)).unwrap();
            black_box(needs_update);

            // Second access - should be cache hit
            let needs_update_2 = cache.needs_update(black_box(&test_file)).unwrap();
            black_box(needs_update_2);
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_analysis, benchmark_cache_performance);
criterion_main!(benches);

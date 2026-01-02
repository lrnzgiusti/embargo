use embargo::core::graph::{Edge, EdgeType, GraphBuilder, Node, NodeType};
use embargo::formatters::{LLMOptimizedFormatter, OutputVerbosity};
use std::path::PathBuf;

fn func(path: &str, id: &str, name: &str, line: usize, public: bool) -> Node {
    let n = Node::new(
        id.to_string(),
        name.to_string(),
        NodeType::Function,
        PathBuf::from(path),
        line,
        "rust".to_string(),
    );
    if public {
        n.with_visibility("public".to_string())
    } else {
        n
    }
}

#[test]
fn llm_clusters_large_graph_matches_golden() {
    let mut gb = GraphBuilder::new();

    // Functions across files and clusters
    let a_main = func("proj/src/a.rs", "A_MAIN", "a_main", 1, true);
    let a_helper = func("proj/src/a.rs", "A_HELPER", "a_helper", 10, false);
    let b1 = func("proj/src/b.rs", "B1", "b1", 2, false);
    let svc = func("proj/services/svc.rs", "SVC", "svc_compute", 3, false);

    let aid_main = a_main.id.clone();
    let aid_helper = a_helper.id.clone();
    let bid = b1.id.clone();
    let sid = svc.id.clone();

    gb.add_node(a_main);
    gb.add_node(a_helper);
    gb.add_node(b1);
    gb.add_node(svc);

    // Calls: a_main -> a_helper, a_main -> b1, b1 -> svc
    gb.add_edge(Edge::new(
        EdgeType::Call,
        aid_main.clone(),
        aid_helper.clone(),
    ));
    gb.add_edge(Edge::new(EdgeType::Call, aid_main, bid.clone()));
    gb.add_edge(Edge::new(EdgeType::Call, bid, sid));

    let graph = gb.build();

    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path().to_path_buf();
    LLMOptimizedFormatter::new()
        .with_verbosity(OutputVerbosity::Verbose)
        .format_to_file(&graph, &path)
        .unwrap();
    let out = std::fs::read_to_string(&path).unwrap();

    // Extract from DIRECTORY_TREE through DEPENDENCY_PATTERNS
    let start = out
        .find("## DIRECTORY_TREE\n")
        .expect("directory tree present");
    let dep = out
        .find("## DEPENDENCY_PATTERNS\n")
        .expect("dependency patterns present");
    // include dependency patterns section entirely
    let tail = &out[dep..];
    // find end of dependency patterns (two newlines after CROSS_CLUSTER_FLOW block)
    let end = tail.len();
    let slice = format!("{}{}", &out[start..dep], &tail[..end]);

    let golden_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/llm_clusters_large.md");
    let expected = std::fs::read_to_string(golden_path).unwrap();

    assert_eq!(slice, expected);
}

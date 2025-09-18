# Cross-Language Call Resolution Specification

## Overview
Enhance EMBARGO so the dependency graph captures interactions that cross language boundaries (for example, a TypeScript client invoking a Python API or Python orchestrators executing Rust binaries). The goal is to infer and model these cross-language calls automatically from the existing multi-language parsing pipeline, producing explicit edges that reflect end-to-end behaviour across services, CLIs, RPCs, or foreign-function interfaces.

## Goals
- Detect and represent cross-language call relationships as `EdgeType::Call` entries linking the true caller and callee nodes, even when they live in different languages.
- Reuse the existing parsing, caching, and resolving infrastructure with minimal disruption; additions should slot into current abstractions.
- Provide confidence scores or annotations explaining why a cross-language link was inferred, without impacting base semantics of same-language analysis.
- Maintain performance characteristics (sub-second analysis on medium codebases) by piggybacking on existing traversal passes instead of introducing full second-phase global scans.

## Current Capabilities and Gaps
- Parsers emit language-local `Node` and `Edge` structures plus optional `CallSite` entries.
- `FunctionResolver` resolves intra-language calls using hash-based lookups, module context, and heuristics.
- No shared registry exists for external invocation targets (HTTP endpoints, CLI commands, FFI symbols, etc.). Call sites referencing external systems appear as unresolved identifiers.
- `EdgeType` already covers `Call`, so no new variant is required; we only need heavier metadata around cross-language cases.

## High-Level Approach
1. **Canonical Call Contracts:** Introduce metadata describing exported entrypoints that can be consumed cross-language (HTTP routes, RPC method names, CLI commands, shared libraries, etc.).
2. **Cross-Language Resolver:** Extend the resolver pipeline to look up call sites against the canonical contracts, in addition to same-language indexes.
3. **Evidence Tracking:** Attach structured context (e.g., matched HTTP path, command string) to the resulting edges via `Edge::with_context` to support debugging and downstream consumers.
4. **Confidence Model:** Emit companion annotations (score or tag) so downstream tooling can treat low-confidence matches differently.

## Detailed Design

### Parser Enhancements
- **Export Signatures:**
  - Update language parsers to identify and tag constructs that expose functionality cross-boundary:
    - Python: FastAPI/Django route decorators, `Flask.route`, `click.command`, `argparse` CLI commands, subprocess invocation targets.
    - TypeScript/JavaScript: Express/Koa route definitions, gRPC service definitions, exported CLI entrypoints (e.g., `yargs`), HTTP client wrappers.
    - Rust/C++: Functions marked `extern "C"`, public binaries with `main`, and HTTP frameworks (Actix, Rocket) route macros.
  - Emit new metadata via `Node` attributes or a dedicated `NodeMeta` map stored inside `ParseResult` (see “Data Model Changes”).

- **Invocation Signatures:**
  - Capture structured details for outbound calls when parsers encounter frameworks/APIs (e.g., `requests.get("/api/users")`, `fetch('/api/users')`, `subprocess.run(['my_binary', 'arg'])`).
  - Store details inside `CallSite.context` using structured JSON fragments or key-value pairs (e.g., `protocol=http`, `path=/api/users`, `method=GET`).

### Data Model Changes
- Extend `CallSite` with an optional `protocol: Option<String>` and `metadata: Option<HashMap<String, String>>`. Maintain backward compatibility by defaulting to `None`/empty.
- Add a `NodeExport` struct stored in `ParseResult` describing exported entrypoints:
  ```rust
  pub struct NodeExport {
      pub node_id: String,
      pub protocol: String,        // e.g., "http", "cli", "ffi"
      pub signature: String,       // e.g., "GET /api/users", "my_binary --flag", "libfoo::bar"
      pub attributes: HashMap<String, String>,
  }
  ```
- Update `ParseResult` to include `exports: Vec<NodeExport>`. Ensure serialization compatibility for cache reads by versioning cache entries if necessary.

### Parse Cache Adjustments
- Increment cache schema versioning (e.g., embed a `cache_version` field in `ParsedFileEntry`). Reject or upgrade old cache entries gracefully to avoid mismatched structures.
- Ensure `ParseCache::store` serializes the new `exports` data and `ParseCache::get` restores it.

### Resolver Extensions
- Introduce a `CrossLanguageRegistry` within `FunctionResolver` responsible for indexing `NodeExport` entries across all languages. Suggested structure:
  ```rust
  struct CrossLanguageRegistry {
      http_routes: MultiMap<String /* method+path */, ExportRef>,
      cli_commands: MultiMap<String /* command name */, ExportRef>,
      ffi_symbols: MultiMap<String /* symbol name */, ExportRef>,
      // Additional protocol-specific maps as needed
  }
  ```
  where `ExportRef` holds `node_id`, `language`, and optional attributes.

- During `build_indexes`, hydrate the registry from `ParseResult::exports` data.
- Extend `resolve_calls` to:
  1. Attempt standard intra-language resolution.
  2. If unresolved, inspect `call_site.protocol` / `metadata` and query the registry.
  3. Construct `EdgeType::Call` edges for matches, with `context` enriched by protocol details and a `confidence` attribute (e.g., `context = "protocol:http;confidence:high;path:/api/users"`).
  4. If multiple matches exist, pick the highest-confidence candidate (same repository/language, matching host/module, etc.) or produce multiple edges flagged `confidence=low` (configurable).

### Confidence Heuristics
- Define a scoring function considering:
  - Exactness of signature match (full method + path vs. partial).
  - Shared path prefixes or namespaces between caller and callee files.
  - Presence of import statements referencing the target module/service.
- Encode the score into the `context` string or extend `Edge` with a new optional `confidence` field if upgrades are acceptable.

### Formatter Awareness
- Update formatters to highlight cross-language edges distinctly:
  - Add a legend entry explaining cross-language `Call` edges.
  - Include protocol and confidence annotations in textual output.
- Ensure JSON output exposes structured metadata for downstream consumers.

### Configuration and CLI
- Add CLI flags to control cross-language analysis, e.g.:
  - `--enable-cross-language` (default `true`).
  - `--min-cross-language-confidence <float>`.
- Extend `Cli` struct (in `src/main.rs`) and propagate settings into `CodebaseAnalyzer`.

### Testing Strategy
- **Unit Tests:**
  - Parser-level tests verifying extraction of exports/invocations for each supported framework.
  - Resolver tests ensuring registry indexing and matching logic produce expected edges.
- **Integration Tests:**
  - Add sample apps under `test_apps/` that demonstrate HTTP, CLI, and FFI flows across languages.
  - Validate generated graphs include cross-language edges with appropriate metadata and confidence.
- **Benchmarking:**
  - Update `benches/performance.rs` to measure the impact of registry lookups; target <10% degradation.

### Migration Considerations
- Update cache versioning to avoid deserializing older entries lacking `exports`.
- Provide a fallback path (warnings + in-memory-only) if caches cannot be upgraded.
- Maintain backwards compatibility for JSON consumers by guarding new fields behind optional properties.

### Rollout Plan
1. Land parser metadata extraction incrementally per language, gated behind feature flags.
2. Add registry and resolver extensions once a minimal set of exports and invocations are supported.
3. Update formatters + CLI.
4. Expand coverage with more protocols/frameworks based on user feedback.

## Open Questions
- Should “confidence” be part of `Edge` proper or remain a context string? (Impacts serialization compatibility.)
- How to handle ambiguous matches spanning multiple services with identical routes? Potential solution: keep multiple edges flagged `confidence=low`.
- Need for user-provided configuration (YAML manifest) to seed service definitions that static analysis cannot infer?

## Risks and Mitigations
- **High False Positives:** Mitigate via conservative heuristics and optional confidence thresholds.
- **Performance Regressions:** Use lightweight metadata extraction and avoid second-pass file reads; rely on existing parse results.
- **Framework Drift:** Keep metadata extraction modular (per-language adapters) so new frameworks can be plugged in without touching core logic.

## Success Criteria
- Analyzer produces at least one accurate cross-language call edge in the sample test apps without manual annotation.
- No regression in existing language support or formatting outputs.
- Feature can be toggled via CLI/config to compare graph outputs before and after cross-language augmentation.

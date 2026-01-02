# EMBARGO Output Interpretation Guide

Reference for understanding LLM-optimized dependency graph output.

## Structure

| Section | Description |
|---------|-------------|
| `NODES:X EDGES:Y` | Total code entities and relationships |
| `DIRECTORY_TREE` | Hierarchical file organization with semantic prefixes |
| `ARCHITECTURAL_CLUSTERS` | Code grouped by functional purpose |
| `DEPENDENCY_PATTERNS` | Cross-module relationship analysis (verbose mode) |

## Behavioral Notation

| Notation | Meaning |
|----------|---------|
| `filename.rs->[...]` | File containing list of functions |
| `function()[ENTRY]` | Public API entry point |
| `function()[HOT]` | Performance-critical function |
| `function()->{calls}` | Immediate function calls |
| `module::function` | Cross-module dependency |

## Semantic Prefixes

| Prefix | Component Type |
|--------|---------------|
| `S[N]` | Services (business logic) |
| `E[N]` | Entities (data models) |
| `C[N]` | Components (UI elements) |
| `D[N]` | Dialogs (modal interfaces) |
| `R[N]` | Ribbon/Toolbar (controls) |
| `B[N]` | Buttons (actions) |
| `V[N]` | Views (display components) |
| `M[N]` | Menus (navigation) |
| `W[N]` | General widgets |
| `U[N]` | Utilities (helpers) |

## Analysis Strategy

1. Start with `[ENTRY]` functions to understand public APIs
2. Follow `->{calls}` to trace execution paths
3. Focus `[HOT]` functions for performance analysis
4. Use clusters to understand system organization
5. Cross-cluster flows reveal coupling patterns

## Verbosity Levels

```
--verbosity compact   # Minimal tokens, no interpretation key
--verbosity standard  # Core + interpretation key (default)
--verbosity verbose   # Full output with dependency patterns
```

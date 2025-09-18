## DIRECTORY_TREE
ROOT: proj/
├─ services/ → S[1]
└─ src/ → U[2]

## ARCHITECTURAL_CLUSTERS

### CORE_SERVICES
NODES:1 CALL_DEPTH:1

svc.rs→[svc_compute()[HOT]] 
### UTILITY_LAYER
NODES:3 CALL_DEPTH:3

a.rs→[a_main()[ENTRY]→{a_helper,b::b1},a_helper()] b.rs→[b1()→{svc::svc_compute}] 

## DEPENDENCY_PATTERNS

### EDGE_PATTERNS
Call: 3 edges

### CROSS_CLUSTER_FLOW
UTILITY_LAYER→CORE_SERVICES: 1


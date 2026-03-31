# L2-01 Domain Types and IR Contracts
- Task: `cortex-autonomy-refactor`
- Stage: `L2`

## 1) Canonical Cognition Types (Owned by Cortex)
Location target:
- `core/src/cortex/cognition.rs`

### 1.1 Goal Tree
```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GoalNode {
    pub node_id: String,
    pub summary: String,
    pub weight: i32,
    #[serde(default)]
    pub children: Vec<GoalNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GoalTree {
    pub root_partition: Vec<String>,  // compile-time immutable string[]
    pub user_partition: GoalNode,     // fixed root node id: "user-root"
}
```

Partitioning rule:
1. Root/User partition split exists only at `GoalTree` level.
2. Node type is unified as `GoalNode`.

Root partition source:
1. compile-time Rust constants.
2. reconstructed each cycle/load from constants.

### 1.2 L1-memory
```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct L1Memory {
    #[serde(default)]
    pub entries: Vec<String>, // exactly string[]
}
```

### 1.3 Cognition State
```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CognitionState {
    pub revision: u64,
    pub goal_tree: GoalTree,
    pub l1_memory: L1Memory,
}
```

## 2) Patch Models (Primary Output IR Semantics)

### 2.1 Goal-tree Patch Ops
```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum GoalTreePatchOp {
    Sprout {
        parent_node_id: String,
        node_id: String,
        summary: String,
        weight: i32,
    },
    Prune {
        node_id: String,
    },
    Tilt {
        node_id: String,
        weight: i32,
    },
}
```

### 2.2 L1-memory Patch Ops
```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum L1MemoryPatchOp {
    Append { value: String },
    Insert { index: usize, value: String },
    Remove { index: usize },
}
```

## 3) Output Contracts: Strict Separation

### 3.1 Primary Output IR (intermediate)
Primary emits patch-oriented content through `<output-ir>`:
1. `<acts>`
2. `<goal-tree-patch>`
3. `<l1-memory-patch>`

This is intermediate cognition IR, not final Cortex boundary output.

### 3.2 Cortex Boundary Output (final)
Cortex returns final state, not patches:

```rust
pub struct CortexOutput {
    pub acts: Vec<Act>,
    pub new_cognition_state: CognitionState,
}
```

Cortex internal pipeline:
1. parse patches from output IR.
2. apply patches inside Cortex to current cognition state.
3. emit full `new_cognition_state`.

## 4) Input/Output IR First-level XML

### 4.1 InputIR
Required first-level sections:
1. `<senses>`
2. `<act-descriptor-catalog>`
3. `<goal-tree>`
4. `<l1-memory>`

Input helper policy:
1. `goal-tree` section helper exists, but only processes user partition.
2. root partition is injected directly as compile-time constants (no helper processing).
3. `l1-memory` is passthrough (no helper).

### 4.2 OutputIR
Required first-level sections:
1. `<acts>`
2. `<goal-tree-patch>`
3. `<l1-memory-patch>`

Section-body policy:
1. first-level XML enforced deterministically.
2. inner content can be Markdown/JSON snippets as needed.
3. no extra context section.

## 5) JSON Schema Constraints for Output Helpers
No wrapper duplication (`acts`, `patch`) at top-level.

Helper output schemas:
1. `acts_helper` -> `ActDraft[]`
2. `goal_tree_patch_helper` -> `GoalTreePatchOp[]`
3. `l1_memory_patch_helper` -> `L1MemoryPatchOp[]`

Failure policy:
1. helper failure => empty array for that helper.
2. primary failure/timeout/contract fail => noop (`acts=[]`, unchanged cognition_state).

## 6) Backward-Incompatible Contract Changes
1. remove `goal_stack` / `goal_stack_patch` contracts.
2. replace with `goal_tree` / `goal_tree_patch` + `l1_memory` / `l1_memory_patch`.
3. `Cortex::cortex` accepts empty senses for autonomous tick.


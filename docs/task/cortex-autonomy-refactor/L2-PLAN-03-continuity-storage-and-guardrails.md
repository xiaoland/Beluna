# L2-03 Continuity Storage and Guardrails
- Task: `cortex-autonomy-refactor`
- Stage: `L2`

## 1) Continuity Scope
Continuity responsibilities:
1. persist/load cognition state JSON,
2. validate guardrails when persisting new cognition state,
3. expose read snapshot,
4. provide `on_act` middleware hook (currently no-op, returns `Continue`).

Continuity explicitly does not:
1. apply cognition patches,
2. consume spine execution events,
3. emit dispatch result objects.

## 2) Persisted JSON Shape
Mutable cognition is persisted as-is (including root partition snapshot for recovery checks):

```json
{
  "version": 1,
  "cognition_state": {
    "revision": 42,
    "goal_tree": {
      "root_partition": ["..."] ,
      "user_partition": {
        "node_id": "user-root",
        "summary": "user goals",
        "weight": 0,
        "children": []
      }
    },
    "l1_memory": {
      "entries": ["note 0", "note 1"]
    }
  }
}
```

On load, root partition must still match compile-time constants.

## 3) Runtime Config
Add continuity config:
```rust
pub struct ContinuityRuntimeConfig {
    pub state_path: PathBuf,
}
```

Default path:
- `./state/continuity.json`

## 4) Boot Load Algorithm
1. if file missing: initialize default cognition state from compile-time root partition + empty user tree + empty l1-memory.
2. if file exists:
- parse JSON,
- validate schema/version,
- validate root partition equality with compile-time constants,
- validate tree structure and l1-memory type.
3. store validated state in memory.

## 5) Persist Algorithm (Direct JSON, Atomic)
1. validate candidate `new_cognition_state` against guardrails.
2. serialize JSON to `state_path.tmp`.
3. flush + fsync temp file.
4. rename temp to target.
5. fsync parent directory (best effort).
6. replace in-memory snapshot only after successful write.

No abstraction layer is introduced.

## 6) Guardrail Rules

### 6.1 Root Partition Guardrail
1. `new_cognition_state.goal_tree.root_partition` must exactly equal compile-time root partition (`string[]`, same order and values).
2. mismatch => reject persist with continuity error.

### 6.2 User Tree Structural Guardrail
1. `user_partition.node_id` must be `user-root`.
2. node ids must be unique globally.
3. tree must be acyclic.
4. optional weight clamp on ingest (`-1000..=1000`).

### 6.3 L1-memory Guardrail
1. type must be `string[]`.
2. no extra semantics (ordering is intrinsic to array model).

## 7) Continuity Public API Surface
```rust
pub fn cognition_state_snapshot(&self) -> CognitionState;

pub fn persist_cognition_state(&mut self, state: CognitionState)
    -> Result<(), ContinuityError>;

pub fn on_act(&mut self, act: &Act, ctx: &DispatchContext)
    -> Result<DispatchDecision, ContinuityError>;
```

Current `on_act` behavior:
1. always returns `DispatchDecision::Continue`.
2. reserved extension point.

## 8) Afferent Sender Ownership
Continuity holds afferent sender handle for future sense emission use-cases, but current phase does not emit senses from continuity.


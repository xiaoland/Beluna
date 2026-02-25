# Cortex Topography & Sequence

## Topography

Cortex 是无状态认知边界，纯函数签名：

```
cortex(senses, physical_state, cognition_state) -> (acts, new_cognition_state, wait_for_sense)
```

### 组件拓扑

```
                              ┌──────────────────────────────────────────┐
                              │              Cortex Runtime              │
                              │            (runtime.rs: Cortex)          │
                              │                                          │
  senses ──────────────────►  │  ┌─ Input Helpers (parallel) ──────────┐ │
  physical_state ──────────►  │  │  sense_input_helper                 │ │
  cognition_state ─────────►  │  │  proprioception_input_helper        │ │
                              │  │  act_descriptor_input_helper        │ │
                              │  │  goal_tree_input_helper             │ │
                              │  │  l1_memory_input_helper             │ │
                              │  └──────────────┬──────────────────────┘ │
                              │                 │ IR sections            │
                              │                 ▼                        │
                              │  ┌─ IR Assembly (ir.rs) ───────────────┐ │
                              │  │  build_input_ir()                   │ │
                              │  │  build_primary_input_payload()      │ │
                              │  └──────────────┬──────────────────────┘ │
                              │                 │ primary_input_payload  │
                              │                 ▼                        │
                              │  ┌─ Primary Engine ────────────────────┐ │
                              │  │  Cognitive Micro-loop               │ │
                              │  │  (max_internal_steps iterations)    │ │
                              │  │  ┌────────────────────────────┐     │ │
                              │  │  │ LLM turn ──► tool calls?   │     │ │
                              │  │  │   yes: expand-sense-raw /  │     │ │
                              │  │  │         expand-sense-with- │     │ │
                              │  │  │         sub-agent          │     │ │
                              │  │  │   no:  emit final output   │     │ │
                              │  │  └─────────────┬──────────────┘     │ │
                              │  └────────────────┼────────────────────┘ │
                              │                   │ primary_output text  │
                              │                   ▼                      │
                              │  ┌─ Output IR Parse (ir.rs) ───────────┐ │
                              │  │  parse_output_ir()                  │ │
                              │  │  -> OutputIrSections                │ │
                              │  └──────────────┬──────────────────────┘ │
                              │                 │ sections               │
                              │                 ▼                        │
                              │  ┌─ Output Helpers (parallel) ─────────┐ │
                              │  │  acts_output_helper                 │ │
                              │  │  goal_tree_patch_output_helper      │ │
                              │  │  l1_memory_flush_output_helper      │ │
                              │  └──────────────┬──────────────────────┘ │
                              │                 │ structured outputs     │
                              │                 ▼                        │
                              │  ┌─ Cognition Patch (cognition_patch.rs)│ │
                              │  │  apply_cognition_patches()          │ │
                              │  └──────────────┬──────────────────────┘ │
                              │                 │                        │
                              └─────────────────┼────────────────────────┘
                                                │
                                                ▼
                              CortexOutput { acts, new_cognition_state, wait_for_sense }
```

### 文件拓扑

```
cortex/
├── mod.rs                  公共导出
├── runtime.rs              运行时编排（Cortex struct + HelperRuntime impl）
├── cognition.rs            CognitionState, GoalTree, GoalNode, GoalTreePatchOp
├── cognition_patch.rs      apply_cognition_patches()
├── ir.rs                   Input/Output IR 构建与解析
├── prompts.rs              System/User prompt 模板
├── clamp.rs                act_instance_id 生成（UUIDv7）
├── error.rs                CortexError / CortexErrorKind
├── types.rs                CortexOutput, ReactionLimits, InputIr, OutputIr
├── testing.rs              TestHooks
├── AGENTS.md
└── helpers/
    ├── mod.rs                      CognitionOrgan enum, HelperRuntime trait, CortexHelper
    ├── sense_input_helper.rs       Sense → <somatic-senses> + SenseToolContext
    ├── proprioception_input_helper.rs  Proprioception → <proprioception>
    ├── act_descriptor_input_helper.rs  Descriptors → <somatic-act-descriptor-catalog>
    ├── goal_tree_input_helper.rs   GoalTree → <instincts> + <willpower-matrix>
    ├── l1_memory_input_helper.rs   L1Memory → <focal-awareness>
    ├── acts_output_helper.rs       <somatic-acts> → Act[]
    ├── goal_tree_patch_output_helper.rs  <willpower-matrix-patch> → GoalTreePatchOp[]
    └── l1_memory_flush_output_helper.rs  <new-focal-awareness> → L1Memory
```

### 依赖关系

```
Cortex ─────► AI Gateway (chat sessions/threads/turns)
Cortex ─────► Observability (metrics recording)
```

Cortex 不持有 Spine、Stem、Continuity 引用。它是纯函数调用，由 Stem 驱动。

---

## Sequence Diagram

### 正常认知周期

```mermaid
sequenceDiagram
    participant Stem
    participant Cortex as Cortex Runtime
    participant IH as Input Helpers (×5)
    participant IR as IR Assembly
    participant Primary as Primary Engine (LLM)
    participant OH as Output Helpers (×3)
    participant Patch as Cognition Patch

    Stem->>Cortex: cortex(senses, physical_state, cognition_state)
    activate Cortex

    Note over Cortex: emit ReactionStarted telemetry

    par 并行输入处理
        Cortex->>IH: sense_helper.to_input_ir_section()
        Cortex->>IH: proprioception_helper.to_input_ir_section()
        Cortex->>IH: act_descriptor_helper.to_input_ir_section()
        Cortex->>IH: goal_tree_helper.to_input_ir_sections()
        Cortex->>IH: l1_memory_helper.to_input_ir_section()
    end
    IH-->>Cortex: 6 IR sections (instincts + willpower split from goal_tree)

    Cortex->>IR: build_input_ir() + build_primary_input_payload()
    IR-->>Cortex: InputIr + primary_input_payload

    Cortex->>Primary: run_primary_engine(primary_input_payload)
    activate Primary
    loop max_internal_steps (default 4)
        Primary->>Primary: LLM turn
        alt tool calls present
            Primary->>Primary: expand-sense-raw / expand-sense-with-sub-agent
            Primary->>Primary: feed tool results → next turn
        else no tool calls
            Primary-->>Cortex: output text (contains Output IR tags)
        end
    end
    deactivate Primary

    Cortex->>IR: parse_output_ir(output_text)
    IR-->>Cortex: OutputIrSections

    par 并行输出处理
        Cortex->>OH: acts_helper.to_structured_output()
        Cortex->>OH: goal_tree_patch_helper.to_structured_output()
        Cortex->>OH: l1_memory_flush_helper.to_structured_output()
    end
    OH-->>Cortex: Act[], GoalTreePatchOp[], L1Memory

    Cortex->>Patch: apply_cognition_patches(ops, l1_flush)
    Patch-->>Cortex: new CognitionState (revision+1)

    Note over Cortex: emit ReactionCompleted telemetry

    Cortex-->>Stem: CortexOutput { acts, new_cognition_state, wait_for_sense }
    deactivate Cortex
```

### 失败降级序列

```mermaid
sequenceDiagram
    participant Stem
    participant Cortex
    participant IH as Input Helpers
    participant Primary
    participant OH as Output Helpers

    Stem->>Cortex: cortex(senses, physical_state, cognition_state)

    par Input helper failures → fallback
        IH--xCortex: sense_helper failed → deterministic empty section
        IH-->>Cortex: other helpers succeed
    end

    alt Primary timeout / failure
        Cortex-->>Stem: noop CortexOutput (original cognition_state, no acts)
    else Primary success, output helper partial failure
        OH--xCortex: acts_helper failed → empty acts
        OH-->>Cortex: goal_tree_patch + l1_memory succeed
        Cortex-->>Stem: CortexOutput (partial: no acts, updated cognition)
    end
```

### Primary 认知微循环详细序列

```mermaid
sequenceDiagram
    participant Runtime as Cortex Runtime
    participant GW as AI Gateway
    participant LLM as LLM Provider

    Runtime->>GW: open_session(cortex-primary-session-{cycle_id})
    GW-->>Runtime: session handle
    Runtime->>GW: open_thread(system_prompt)
    GW-->>Runtime: thread handle

    loop step = 0..max_internal_steps
        Runtime->>GW: thread.turn_once(user_prompt + tools)
        GW->>LLM: chat request
        LLM-->>GW: response (text + tool_calls?)
        GW-->>Runtime: ChatResponse

        alt tool_calls is empty
            Note over Runtime: final output obtained
            Runtime->>GW: session.close()
            Runtime-->>Runtime: return output text
        else tool_calls present
            loop for each tool_call
                alt expand-sense-raw
                    Runtime->>Runtime: lookup sense by sense_instance_id
                else expand-sense-with-sub-agent
                    Runtime->>GW: sub-agent LLM call per task
                    GW-->>Runtime: summarized sense data
                end
            end
            Note over Runtime: feed tool results as next turn input
        end
    end

    Note over Runtime: exceeded max_internal_steps → error
```

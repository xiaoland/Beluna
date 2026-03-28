# L0 PLAN - Context and Baseline

## Purpose
Frame the doc-system remediation as a controlled migration where:
- `SVCv9` is the normative SSoT.
- `SVCv8` is reference context for migration continuity and regression checks.

## SSoT Precedence
1. `/Users/lanzhijiang/Downloads/svc_v_9_with_alignment.md` (normative).
2. `/Users/lanzhijiang/Library/Mobile Documents/com~apple~CloudDocs/sustainable_vibe_coding_framework_v_8.md` (historical context only).
3. Current Beluna authoritative docs (`docs/00/10/20/30/40`) as migration targets.

## Why V8 Still Matters
V8 is still useful to:
- detect whether existing Beluna conventions came from pre-v9 assumptions,
- separate "already-v8-aligned" areas from "new-in-v9" requirements,
- avoid over-correcting stable patterns that are valid in both versions.

## V8 -> V9 Delta Summary (Relevant to this task)
1. Alignment substrate:
- V8: absent.
- V9: optional `15-alignment` coordination substrate (not a truth layer).

2. Canonical semantics ownership:
- V8: PRD owns what/why; canonical semantics less explicit.
- V9: explicit PRD ownership of canonical product/domain semantics.

3. AGENTS execution protocol:
- V8: AGENTS practical/short guidance.
- V9: same, plus pre-execution restatement protocol for reference-sensitive/risky changes.

4. Anti-pattern refinements:
- V9 adds alignment-specific anti-patterns and admission constraints.

## Current Beluna Drift Focus (unchanged from initial assessment)
1. `00-meta` mandatory posture vs V9 minimal/optional baseline stance.
2. Ritualized read path vs V9 contextual read strategy.
3. Canonical semantics owned in `00-meta/concepts.md` vs V9 PRD ownership.
4. Unit TDD universal template vs V8/V9 hard-unit admission intent.
5. Large mutable snapshots in component `AGENTS.md`.
6. Missing explicit demotion lifecycle language.

## Non-Goals
- No runtime code refactor.
- No file-system deletion until policy decisions are locked.
- No historical task rewrite.

# L1 Plan 04 - Cortex Primary Tooling and Act Emission
- Task: `cortex-loop-architecture`
- Micro-task: `04-cortex-primary-tooling-and-act-emission`
- Stage: `L1`
- Date: `2026-03-01`
- Status: `DRAFT_FOR_APPROVAL`

## High-Level Strategy
1. Replace prompt-parsed act extraction with structured AI Gateway tool-calling in Primary.
2. Replace boolean wait with bounded per-act waiting seconds (`wait_for_sense`, `0` means no wait).
3. Merge sense expansion tools into one polymorphic tool.
4. Remove prompt sections that exist only for legacy parsing (`<somatic-acts>`, act catalog section).

## Architectural Design
1. Primary tool contract family:
- dynamic act tools, one per act descriptor; tool name is transport-safe alias mapped to fq act id (`endpoint-id/neural-signal-descriptor-id`)
- `expand-senses` with `mode: raw | sub-agent`
- `overwrite-sense-deferral-rule` and `reset-sense-deferral-rules` as wrappers over afferent pathway control port
- `sleep` to set `ignore_all_trigger_until` (replaces removed `core.control/sleep` act path)
- existing goal-forest patch tool upgraded separately in micro-task `05`.
2. Each dynamic act tool argument schema includes:
- `payload`
- `wait_for_sense` (integer seconds)
3. Wait semantics:
- `wait_for_sense` is integer in `[0, max_waiting_seconds]`.
- `0` means no wait.
- invalid values are rejected by deterministic validation (no silent clamp).
4. Optional emitted-sense declaration:
- act descriptors may declare `emitted_sense_ids[]`.
- runtime uses this for correlation when waiting and matching returned senses.
5. Naming/readability policy:
- external tool field remains `wait_for_sense` to match product contract.
- internal runtime fields use `wait_for_sense_seconds` to avoid legacy bool confusion.
6. Wait implementation policy:
- wait behavior is implemented through afferent-pathway deferral rule control, not a Cortex wait-specific pending-sense buffer.

## Key Technical Decisions
1. Act emission path is tool-call only after cutover; no dual execution path.
2. Wait is evaluated per act and remains bounded; infinite block is disallowed.
3. Expand-sense behavior is unified behind one tool:
- `senses_to_expand[]` with `sense_id` required and `instruction` optional.
- `sense_id` is composite: `"{monotonic_internal_sense_id}. {fq-sense-id}"`.
- `instruction` required only when `mode=sub-agent`.
4. Runtime output type from Cortex is structured act envelope, not text section parsing.

## Sense Render Contract
Senses delivered to Primary use deterministic text lines:
```text
- [monotonic internal sense id]. [fq-sense-id]: [key=value,key=value,...]; [payload-truncated-if-needed]
```

## Dependency Requirements
1. AI Gateway tool-calling must remain stable for streaming turns.
2. `max_waiting_seconds` config key and policy must be available in runtime config.
3. Micro-task `02` and `07` contracts should align on `act_instance_id` correlation fields.
4. Micro-task `03` rule-control port is required for Primary sense-gating tool wrappers and wait-gate orchestration.
5. Observability updates are required for tool-call validation failures and wait timeouts.

## L1 Exit Criteria
1. Structured act tool schema and bounded wait policy are fixed.
2. Legacy prompt-based act dispatch path is fully deprecated in design.
3. Single `expand-senses(mode)` surface is defined.

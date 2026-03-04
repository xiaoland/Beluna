# Cortex Topology Analysis

## Architectural Role

Cortex is responsible for cognition-only decisions. It does not own physical-state writes, dispatch execution, or persistence storage.

## Stable Invariants

1. Stateless runtime boundary:
- same input contract -> same deterministic post-processing path.
2. Emitted acts are non-binding proposals.
3. Per-act wait semantics are bounded integer ticks, not cycle-level bool.
4. Primary and helper failures degrade safely (noop/empty fallback paths).
5. Goal-forest patching uses tool flow (`patch-goal-forest`), not output-tag parsing.

## Control and Ownership Split

1. Stem:
- tick authority
- physical-state writer (`StemControlPort`)
- pathway ownership.
2. CortexRuntime:
- afferent consumer owner
- cycle executor
- efferent producer.
3. Continuity:
- cognition persistence + act gate.

## Sense Handling Model

1. Senses are text payload + weight + optional act correlation id.
2. Primary receives deterministic rendered sense lines with monotonic internal ids.
3. Expansion is unified under `expand-senses`.

## Wait-for-Sense Model

1. For each emitted act, Primary may set `wait_for_sense` (ticks).
2. `wait_for_sense > 0` is accepted only when the act descriptor declares non-empty `emitted_sense_ids`.
3. Runtime does not mutate afferent deferral rules for wait behavior.
4. Runtime skips admitted ticks until one buffered sense matches:
- `sense.act_instance_id == dispatched act_instance_id`
- `fq-sense-id` is in descriptor-declared `emitted_sense_ids`
5. If no match appears before wait ticks are exhausted, wait gate expires and normal tick execution resumes.

## Maintainability Hotspots

1. Tool alias mapping:
- must remain deterministic and collision-safe.
2. Goal-forest reset:
- relies on AI Gateway atomic thread message mutation.
3. Wait gate rule lifecycle:
- must always clear after wait completion/timeout.
4. Runtime sequencing:
- `act_seq_no` must remain monotonic per cycle.

## Recommended Guardrails

1. Keep Cortex/Stem interface narrow (`CortexDeps`, ports only).
2. Keep Primary tool schemas versioned and deterministic.
3. Keep drift sweeps for legacy contract forms (boolean cycle wait flag, split expand tools, control-sense orchestration).

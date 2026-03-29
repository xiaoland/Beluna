# Quick Reference

## What Is Emitted

Current contract event families (`family`):

- `ai-gateway.request`
- `ai-gateway.turn`
- `ai-gateway.thread`
- `cortex.tick`
- `cortex.organ`
- `cortex.goal-forest`
- `stem.tick`
- `stem.signal`
- `stem.dispatch`
- `stem.proprioception`
- `stem.descriptor.catalog`
- `stem.afferent.rule`
- `spine.adapter`
- `spine.endpoint`
- `spine.dispatch`

## Fast Source Navigation

Find all business call sites:

```bash
rg -n "observability_runtime::emit_[a-z0-9_]+\(" core/src/{ai_gateway,cortex,stem,spine}
```

Find all runtime wrappers:

```bash
rg -n "pub fn emit_" core/src/observability/runtime
```

Find canonical payload schema:

```bash
rg -n "pub struct .*Event" core/src/observability/contract/mod.rs
```

Find final sink (`target=observability.contract`):

```bash
rg -n "target: \"observability\.contract\"|contract_event|payload = %payload" core/src/observability/runtime/emit.rs
```

## Triage by Symptom

AI request/turn failures:

- check `ai-gateway.request` (`kind=attempt_failed|failed`)
- check `ai-gateway.turn` (`status=error`)
- check `cortex.organ` (`status=error`) if failure originated from cortex organ execution

Dispatch degraded/lost:

- check `stem.dispatch` (`kind=result`, `terminal_outcome_when_present`)
- check `spine.dispatch` (`kind=outcome`, `outcome_when_present`, `reason_code_when_present`)
- check `stem.signal` efferent `transition_kind=result` and `reason_when_present`

Descriptor/endpoint topology changes:

- endpoint lifecycle: `spine.endpoint` (`connected|dropped`)
- descriptor catalog: `stem.descriptor.catalog` (`change_mode=snapshot|update|drop`)

Afferent deferral behavior:

- rule changes: `stem.afferent.rule` (`kind=add|remove`)
- queue transitions: `stem.signal` (`direction=afferent`, `transition_kind=defer|release`)

Goal forest evolution:

- snapshots: `cortex.goal-forest` (`kind=snapshot`)
- patches: `cortex.goal-forest` (`kind=patch`) + `patch_*` fields

## Level Escalation Rules (flatten)

`warn` is produced when any of the following is true:

- `ai-gateway.request`: `error_when_present` exists
- `ai-gateway.turn`: `status == error`
- `cortex.tick`: `error_when_present` exists
- `cortex.organ`: `status == error`
- `stem.signal`: `reason_when_present` exists
- `stem.dispatch`: `terminal_outcome_when_present in {rejected, lost}`
- `spine.adapter`: `kind_or_state == faulted`
- `spine.dispatch`: `outcome_when_present in {rejected, lost}`

All other contract events are emitted as `info` by current flatten logic.

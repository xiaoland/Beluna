# L3-04 - Socket Protocol And Process Control
- Task Name: `body-endpoints-mvp`
- Stage: `L3`
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

## 1) WS0 Compatibility Validation Steps
1. inspect core wire parser for accepted message `type` values.
2. inspect server dispatcher for endpoint lifecycle handling.
3. run live socket probe:
- send candidate `endpoint_register`.
- observe accepted/rejected behavior.

Pass criteria:
1. core accepts registration and can issue invoke events.

Fail criteria:
1. unknown-type rejection or no invoke path.
2. stop and escalate to user before integration implementation.

## 2) Process Supervision Rules (`runtime/src`)
1. process state stored in local state file:
- core PID
- std-body PID
- socket path
- timestamp
2. `start` is idempotent-safe:
- if alive state exists, return already-running error.
3. `stop` is resilient:
- always attempt socket exit first,
- then bounded wait,
- then force-kill fallback if required.

## 3) Socket Reconnect Policy (`std-body`)
1. reconnect with bounded backoff.
2. re-register routes on each reconnect.
3. preserve request handling only while connection is healthy.

## 4) Envelope Handling Rules
1. parse incoming JSON as minimally-typed envelope.
2. route by `type`.
3. preserve unknown message tolerance:
- log and ignore unsupported types.
4. always include `request_id` in result envelopes for correlation.

## 5) Endpoint Result Mapping Rules
1. applied:
- must include `actual_cost_micro` and `reference_id`.
2. rejected:
- must include stable `reason_code` and `reference_id`.
3. deferred:
- must include `reason_code`.

## 6) Safety Controls
1. shell:
- argv-only execution
- timeout cap
- stdout/stderr byte caps
2. web:
- http/https only
- timeout cap
- response byte cap
3. no unbounded memory reads from endpoint outputs.

Status: `READY_FOR_REVIEW`

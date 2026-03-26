# Apple Universal Data And State

## Owned State

1. Local UI connection state (socket path, connect/disconnect intent, retry state).
2. Local chat/session state (bounded in-memory message buffer, pagination state).
3. Local persisted endpoint-side sense/act history and restore metadata.

## Consumed State

1. Core protocol messages and dispatch outcomes over Unix socket NDJSON.
2. Endpoint identity/correlation semantics required by the shared identity contract.
3. Runtime connectivity signals used to drive UI lifecycle and recovery states.

## Local Invariants

1. Socket I/O and decoding stay off the main thread to preserve UI responsiveness.
2. Local history remains bounded in memory and restorable from local persistence.
3. Apple Universal does not own core runtime authority state or observability policy.

## Authority Boundaries

1. Apple Universal owns endpoint UX state and local history persistence.
2. Core owns runtime authority state, dispatch routing/outcomes, and runtime observability export.
3. Any cross-unit contract or authority change must escalate to Product TDD artifacts.

## Failure-Sensitive Assumptions

1. Reconnect/disconnect cycles are expected and must not corrupt local history or UI state.
2. Persisted local history may be missing/corrupt and must fail safe during restore.
3. Protocol incompatibility can occur; decode/contract failures must surface explicitly.

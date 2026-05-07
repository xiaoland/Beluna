# Apple Universal Data And State

## Owned State

1. Local UI connection state (socket path, connect/disconnect intent, retry state).
2. Local chat/session state (bounded in-memory message buffer, pagination state).
3. Local persisted endpoint-side sense/act history and restore metadata.
4. Socket discovery candidates and recent successful socket path.
5. Settings-integrated Moira Loom UI state for selected launch/profile context, selected wake, selected tick, loading state, and error/status presentation.

## Consumed State

1. Core protocol messages and dispatch outcomes over Unix socket NDJSON.
2. Endpoint identity/correlation semantics required by the shared identity contract.
3. Runtime connectivity signals used to drive UI lifecycle and recovery states.
4. Embedded Moira runtime status, resource conflict status, Clotho context, Atropos status, and Lachesis read models.

## Local Invariants

1. Socket I/O and decoding stay off the main thread to preserve UI responsiveness.
2. Local history remains bounded in memory and restorable from local persistence.
3. Core retains runtime authority state and observability policy.
4. Moira runtime owns local preparation, supervision, and observability semantics behind Apple-native Loom UI.
5. Chat history persistence remains separate from Moira telemetry storage.

## Authority Boundaries

1. Apple Universal owns endpoint UX state and local history persistence.
2. Apple Universal owns Apple-native Moira Loom presentation state.
3. Core owns runtime authority state, dispatch routing/outcomes, and runtime observability export.
4. Moira owns local control-plane and observability runtime state.
5. Any cross-unit contract or authority change must escalate to Product TDD artifacts.

## Failure-Sensitive Assumptions

1. Reconnect/disconnect cycles are expected; local history and UI state stay intact across them.
2. Persisted local history may be missing/corrupt and must fail safe during restore.
3. Protocol incompatibility can occur; decode/contract failures must surface explicitly.
4. Embedded Moira runtime resources may be claimed by another process; Apple UI must surface the conflict clearly.

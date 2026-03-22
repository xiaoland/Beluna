# Apple Universal Design

## Responsibility

1. Provide chat-oriented endpoint UX on Apple platforms.
2. Manage connection lifecycle to Core Unix socket endpoint.
3. Persist and restore local endpoint-side chat history.

## Boundary Rules

1. Do not re-implement Core domain logic.
2. Keep protocol compatibility explicit and typed.
3. Keep I/O and decoding off main thread to preserve UI responsiveness.
4. Keep observability export in Core, not in app UI surfaces.

Decision Source: ADR-001.

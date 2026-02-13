# L0 Plan - Body Endpoints MVP (std-body)
- Task Name: `body-endpoints-mvp`
- Stage: `L0` (request + context analysis only)
- Date: `2026-02-12`
- Status: `DRAFT_FOR_APPROVAL`

## 1) Problem Deconstruction
Requested deliverable is to complete MVP with minimum concrete Body endpoints, explicitly outside `core` and inside a new `std-body` scope:

1. Apple Universal App endpoint:
- keep app logic simple and chatbot-oriented for MVP.
2. POSIX Shell endpoint:
- execute shell actions as a Body capability.
3. Web endpoint:
- support network fetch as a Body capability.

Non-negotiable boundary from request:
1. Body endpoint implementations should live in `std-body`, not inside `core/src/*`.

## 2) Context Collection (Sub-Agent Style)
To reduce cognitive load, context was split into focused tracks:

1. Track A (runtime and execution path):
- `core/src/server.rs`
- `core/src/continuity/engine.rs`
- `core/src/spine/*`
- `core/src/admission/*`

2. Track B (product/contracts/docs):
- `docs/overview.md`
- `docs/modules/spine/README.md`
- `docs/features/spine/*`
- `docs/contracts/spine/README.md`
- previous execution task result `docs/task/spine-implementation/RESULT.md`

3. Track C (external references via Firecrawl):
- POSIX Shell Command Language (Open Group)
- Fetch API references (MDN + WHATWG linkouts)
- Apple URLSession references (Apple docs discoverability via Firecrawl search)

## 3) Current Codebase Reality

### 3.1 No existing `std-body` component
Current repository top-level components are:
1. `core/`
2. `apple-universal/` (currently only `AGENTS.md`, no app code checked in yet)
3. `docs/`

`std-body/` does not exist yet.

### 3.2 Spine core is ready for pluggable endpoints
Spine already has:
1. async `EndpointPort`
2. route table (`RouteKey { affordance_key, capability_handle }`)
3. registry (`InMemoryEndpointRegistry`)
4. mechanical router (`RoutingSpineExecutor`)

This is a good fit for introducing external endpoint implementations.

### 3.3 Runtime wiring is still core-local and defaulted
`ContinuityEngine::with_defaults` currently:
1. creates in-memory registry,
2. registers built-in native endpoint(s),
3. wires a default `RoutingSpineExecutor`.

There is currently no runtime extension point that loads endpoint sets from a separate `std-body` module/crate.

### 3.4 Admission constraints matter for endpoint design
`AffordanceRegistry` is keyed by `affordance_key` and each profile has one primary `capability_handle`.

Implication:
1. easiest MVP path is one affordance per endpoint capability (no admission model refactor),
2. multiplexing many capability handles under one affordance would require admission design changes.

### 3.5 Ingress exists; actionable egress is still thin
Current Unix socket shell (`core/src/spine/adapters/unix_socket.rs` + wire parser):
1. ingests `sense` and other control inputs into runtime,
2. does not provide a formal outward response channel for a chat UI yet.

This affects Apple chatbot endpoint scope and needs explicit treatment in L1.

## 4) Fit-Gap Matrix Against Requested Endpoints
1. Apple Universal App endpoint:
- Status: `GAP`
- Reason: no app code present; no core<->app response channel contract yet.

2. POSIX Shell endpoint:
- Status: `GAP`
- Reason: no endpoint implementation, schema, timeout/safety guardrails, or registration path from non-core component.

3. Web fetch endpoint:
- Status: `GAP`
- Reason: no endpoint implementation, schema, URL policy, timeout/size caps, or registration path from non-core component.

4. Spine readiness for endpoint invocation:
- Status: `READY`
- Reason: route-based endpoint invocation and deterministic event mapping already implemented.

## 5) Architectural Trade-Offs Identified
1. `std-body` integration model
- Option A: `std-body` as Rust library crate linked by `core` (fastest MVP cutover).
- Option B: `std-body` as separate process over IPC (cleaner separation, larger protocol/runtime scope).

2. Endpoint naming model
- Option A: distinct affordance keys (`tool.shell.exec`, `tool.web.fetch`, `chat.reply.emit`) to match current admission structure.
- Option B: one affordance with many capability handles (requires admission registry changes).

3. Chatbot loop completion strategy
- Option A: minimal MVP: endpoint effects + logs/sense reinjection only.
- Option B: explicit outbound stream contract for app-visible assistant replies.

4. Safety envelope for shell/fetch
- permissive by default (faster, risky) vs policy-gated execution (slower, safer).

## 6) External Source Findings (Firecrawl)
1. POSIX shell execution model and quoting/expansion phases are strict and injection-sensitive.
- Source: Open Group POSIX Shell Command Language
- Link: https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html
- Relevance: shell endpoint must avoid naive string interpolation and define explicit invocation policy.

2. `fetch()` resolves on headers even for HTTP error statuses; status handling is caller responsibility.
- Source: MDN Fetch API (with WHATWG fetch spec references)
- Link: https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API
- Relevance: web endpoint should separate transport failure from HTTP non-2xx and normalize deterministic rejection codes.

3. Apple URLSession async APIs are the platform baseline for simple network-driven app flows.
- Source: Apple Developer Documentation (`URLSession`, `data(from:)`) discovered via Firecrawl search
- Link: https://developer.apple.com/documentation/foundation/urlsession
- Link: https://developer.apple.com/documentation/foundation/urlsession/data(from:)
- Relevance: Apple-side MVP can stay simple with async URLSession-driven chat I/O and avoid custom networking stack.

## 7) L0 Recommendation (Boundary for L1)
Recommended L1 direction:

1. Introduce new top-level `std-body/` Rust crate (library-first).
2. Implement three minimum endpoints in `std-body`:
- Apple chat endpoint (MVP output effect),
- POSIX shell endpoint,
- Web fetch endpoint.
3. Register these endpoints into Spine via runtime wiring, while keeping Spine core unchanged.
4. Keep admission model unchanged by using distinct affordance keys per endpoint.
5. Define strict endpoint payload schemas and deterministic outcome mapping (`Applied/Rejected/Deferred`).
6. Add bounded timeout/output-size controls for shell and fetch endpoints in MVP.

## 8) Open Questions Requiring User Decision
1. Integration boundary for `std-body`:
- `A)` library crate linked into `core` runtime (recommended for MVP speed),
- `B)` separate process/service (cleaner isolation, larger scope).

2. Apple endpoint scope in this MVP:
- `A)` only emit assistant response payload for app display (recommended),
- `B)` full bidirectional app session protocol redesign.

3. Shell safety policy:
- `A)` allow arbitrary commands with timeout/output caps,
- `B)` allowlist command families only (safer, more configuration).

4. Web fetch policy:
- `A)` open outbound HTTP(S) with timeout/body caps,
- `B)` allowlisted domains only.

## 9) Working Assumptions (If Not Overridden)
1. `std-body` will be a Rust crate integrated as a dependency (not a new standalone runtime process) for this MVP.
2. Endpoint keys will be separated by affordance to avoid admission architecture changes.
3. Apple app remains chatbot-simple and does not require a full duplex protocol redesign in this task.
4. Shell/fetch endpoints will ship with deterministic timeout and output truncation limits.

## 10) L0 Exit Criteria
L0 is complete when:
1. repository and runtime constraints are mapped,
2. endpoint gaps are explicit,
3. core architectural trade-offs are surfaced,
4. user decisions needed for L1 are enumerated.

Status: `READY_FOR_L1_APPROVAL`

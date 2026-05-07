# Boundary

## Target Shape

Moira becomes a library-first runtime unit with these internal owners:

1. `clotho`
- Core artifact selection and installation.
- Profile document ownership and wake-input preparation.
- Later Core-schema validation coordination while Core remains schema authority.

2. `lachesis`
- OTLP log receiver lifecycle.
- Raw event storage.
- Wake/tick read models.
- Source-grounded query and projection surfaces.

3. `atropos`
- Core wake, graceful stop, force kill, and supervised process status.
- Readiness gating against Lachesis receiver status.
- Terminal reason and supervision state.

4. `moira-platform`
- OS-specific process, sandbox, ledger, filesystem, and permission adapters.
- Build-feature and target-OS selection for supported adapters.
- Current task defers sandbox and ledger implementation while reserving their boundary.

5. `moira-host`
- Host-facing runtime API.
- Event stream, lifecycle hooks, typed command/query surface, and path configuration.
- Binding surface needed by the Apple Universal host in this task.
- Rust-native and Windows-native host bindings remain future expansion paths.

6. `moira-authority`
- Future single local runtime ownership.
- Future owner election and attach-mode client coordination.
- Future IPC endpoint discovery for clients that include Moira backend code and reach a shared authority.

## Human Interface Clients

`apple-universal`, `cli`, and future `win-native` should be treated as Beluna Human Interface clients.

Expected responsibilities:

- Present human-facing endpoint UX.
- Present platform-native Loom-equivalent operation surfaces where relevant.
- Host Moira runtime through typed API or bindings.
- Preserve Core endpoint protocol compatibility.
- Keep Core domain decisions inside Core.

## Legacy Tauri/Vue Loom

The current Tauri/Vue Loom is a legacy implementation of Moira's human-facing surface.

Target disposition:

- Preserve as a reference only while replacement hosts are being built.
- Extract backend runtime behavior away from Tauri types.
- Retire Tauri command/event surfaces after a replacement Human Interface path covers the required operator workflows.
- Delete Vue Loom frontend after the replacement path is verified and durable docs reflect the new architecture.

## Authority Rules

Core owns:

- runtime behavior
- cognition and routing
- config schema shape
- endpoint protocol authority
- observability emission semantics

Moira owns:

- local artifact/profile preparation
- local Core process supervision
- local observability ingestion, storage, query, and projection
- future sandbox and ledger supervision policies and adapters

Human Interface clients own:

- platform-native UX
- endpoint interaction presentation
- Moira-hosted operator workflows
- client-local history or UI state

## Current Task Runtime Rule

Apple Universal embeds Moira backend and runs a process-local Moira runtime for the minimum Loom.

Runtime behavior:

- Apple Universal can use Clotho and Atropos to prepare and start Core.
- Apple Universal can show Lachesis receiver status for its embedded runtime.
- If a local resource is already claimed by another process, Apple Universal surfaces that resource state.
- Body endpoint socket discovery remains independent from Atropos ownership. The app can connect to a known or discovered Core socket, such as a configured path or a platform candidate path.
- The actual default socket path remains a Core/deployment contract decision.

## Future Single Local Authority Rule

Moira backend should eventually have one live local authority per user/session or configured local scope.

Future Human Interface clients may all include the Moira package. At runtime, each process can follow one of two paths:

1. Owner mode
- Acquires the authority lock.
- Opens local Moira state.
- Starts receiver and supervision resources.
- Serves local IPC.

2. Attach mode
- Finds the owner endpoint.
- Uses IPC for queries and operations.
- Presents UI using shared Moira authority state.

This preserves library embedding while keeping Moira-owned state, receivers, and supervised Core lifecycle single-owned.

## This Task's Product Surface

This task implements the minimum Moira Loom surface inside `apple-universal`.

Minimum target:

- Moira runtime status.
- Core launch-target/profile visibility sufficient to understand what can be woken.
- Wake/stop status controls only if the runtime API slice reaches that capability safely.
- Lachesis receiver status.
- Wake list and tick list.
- Selected tick raw-first inspection.

Follow-on product surfaces:

- CLI Moira commands.
- Windows Human Interface host.
- Full native Apple Loom workspace.
- Rich chronology, Cortex/Stem/Spine narrative panels, and goal-forest comparisons.
- Sandbox and ledger UX.

## First-Scope Exclusions

- Sandbox implementation.
- Ledger implementation.
- Windows adapter implementation.
- CLI embedding implementation.
- Full native Loom redesign.
- Core runtime behavior changes.

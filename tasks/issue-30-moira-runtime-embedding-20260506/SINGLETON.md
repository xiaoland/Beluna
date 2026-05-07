# Future Single Local Moira Authority

This file captures the runtime ownership problem created by embedding Moira backend code in multiple Human Interface clients.

## Problem

`apple-universal`, future `win-native`, and `cli` may run at the same time. A single client may also have multiple processes or instances, depending on platform behavior.

Moira backend owns local resources that prefer one live owner:

- OTLP receiver bind address.
- DuckDB telemetry store writes.
- Clotho artifact/profile writes.
- Atropos supervised Core process state.
- Future sandbox and ledger adapters.

Library embedding solves packaging and user-install friction. Runtime coordination still needs a future design.

## Current Task Model

This task uses a smaller model:

- Apple Universal embeds Moira backend.
- Apple Universal runs a process-local Moira runtime.
- Attach mode belongs to a later task packet.
- Multiple clients may each include Moira backend code.
- Resource conflicts are reported through runtime status.
- Body endpoint socket discovery stays available as a direct client capability.

For example, Apple Universal can still connect to a configured or discovered Core socket when Core was started by another process.

## Future Target Rule

There is one live Moira authority per user/session or configured local scope.

Every process that embeds Moira can eventually choose one runtime role:

1. Owner mode
- Acquires the authority lock.
- Opens local Moira state.
- Starts receiver and supervision resources.
- Serves local IPC.

2. Attach mode
- Finds the owner endpoint.
- Uses IPC to query and command the owner.
- Presents local UI using shared Moira authority state.

## Candidate Coordination Mechanisms

1. Lock file plus local IPC endpoint
- Owner takes an advisory lock in app support or runtime directory.
- Owner writes endpoint metadata beside the lock.
- Attach clients connect to Unix domain socket on Apple platforms.

2. OS service or launch agent
- Moira authority runs as a platform service installed by a Human Interface client.
- Clients connect to the service.
- Better long-running semantics, higher packaging and trust surface.

3. First-client owner election
- First Human Interface process becomes owner.
- Later processes attach.
- A later client can acquire ownership after stale-lock recovery.

## Current Task Bias

Use process-local Moira runtime inside Apple Universal.

Reasons:

- Fits the minimum Apple proof.
- Keeps package embedding work bounded.
- Lets the UI design surface resource conflicts early.
- Preserves direct body endpoint socket use.

## Future Questions

1. Should the owner be tied to the app process lifetime or a helper process lifetime?
2. Should Apple Universal keep its current single-instance guard once Moira authority coordination exists?
3. Should the authority endpoint live under Application Support, Caches, or a runtime directory?
4. How should stale lock recovery prove the old owner has exited?
5. Should attach-mode calls use a private Unix socket protocol, XPC on macOS, or a Rust-owned IPC protocol?

## Current Task Verification Ideas

- Unit test resource-claim success.
- Unit test resource conflict reporting.
- Integration smoke with two processes:
  1. first process starts an embedded Moira runtime
  2. second process starts an embedded Moira runtime
  3. conflicting resources appear as status
  4. body endpoint socket discovery still presents available Core sockets

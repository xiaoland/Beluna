# Issue 30 Follow-On: Apple Core Control

Follow-on packet after issue #30 minimum Apple Loom and Tauri/Vue retirement.

This packet owns Apple-native Core lifecycle controls through the embedded Moira runtime.

UI integration target: a standalone Core Control panel parallel to Settings. Settings remains the home for Moira configuration such as runtime paths, receiver bind address, socket candidates, refresh policy, and diagnostics.

## Files

- [PLAN.md](./PLAN.md): MVT anchors, first-scope boundaries, and verification.

## Current Packet Status

Mode: opened for scope confirmation.

Current target:

- Wake Core from Apple Universal through Moira-owned Clotho and Atropos semantics.
- Stop supervised Core from Apple Universal.
- Expose force-kill through an explicit second confirmation path.
- Keep launch context, resource conflicts, and terminal supervision state visible in the Core Control panel.

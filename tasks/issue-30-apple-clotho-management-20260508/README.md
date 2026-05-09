# Issue 30 Follow-On: Apple Clotho Management

Follow-on packet after Apple Core Control.

This packet owns Apple-native target and profile management for the Core Control surface. Clotho preparation and Atropos lifecycle remain in one operator view centered on the selected target and profile.

UI integration target: extend the existing Core Control panel rather than creating a separate Clotho panel. Settings remains configuration-focused. O11y / Lachesis remains a separate investigation panel.

## Files

- [PLAN.md](./PLAN.md): MVT anchors, first-scope boundaries, and verification.

## Current Packet Status

Mode: Execute.

Current target:

- Manage launch targets from Apple Universal through Moira-owned Clotho semantics.
- Manage app-local JSONC profiles from Apple Universal.
- Keep target selection, profile selection, preparation actions, wake, stop, and force-kill in the same Core Control panel.
- Preserve Core authority over config schema and runtime behavior.

Current slice:

- Expose known-local-build registration and structured profile draft load/save through `moira/ffi`.
- Bind those operations into Apple Universal through `MoiraRuntimeClient`.
- Add Core Control Launch Context Create/Edit entry points plus focused target/profile editor sheets for the first operator proof, including target update by stable `buildId`, `core_config`, env files, and inline environment variables.
- Cover the new Rust FFI and Apple view-model paths with targeted tests.

# Apple Universal Operations

1. Keep connection lifecycle explicit (manual connect/disconnect + retry strategy).
2. Apply socket path changes on reconnect.
3. Preserve responsiveness under reconnect and pagination scenarios.
4. Validate protocol/lifecycle behavior with focused tests where practical.
5. Present socket discovery candidates in the Settings-integrated operations panel.
6. Initialize embedded process-local Moira runtime for the minimum Loom surface.
7. Surface Moira runtime resource conflicts as operator-visible status.
8. Keep body endpoint connection usable when Core is already listening from another launch path.
9. Keep Moira operations UI grouped with connection and runtime status in the first slice.
10. Permit multiple Apple Universal instances; rely on Core-assigned endpoint ids and future user-configured endpoint names for runtime disambiguation.
11. Load Moira status through `MoiraOperationsViewModel` and an async `MoiraRuntimeClient`.
12. Keep Rust binding setup behind a replaceable client adapter.
13. Keep dynamic Rust calls off the main actor; the current adapter runs status refresh work through a detached utility task.
14. Bundle macOS Moira FFI runtime dylibs through the BelunaApp target's `Build Moira FFI` script phase.
15. Keep bundled Rust dylibs in `BelunaApp.app/Contents/Frameworks` so `Bundle.main.privateFrameworksURL` can resolve the runtime.

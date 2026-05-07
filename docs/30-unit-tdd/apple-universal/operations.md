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

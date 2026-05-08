# Apple Universal Interfaces

## Endpoint Runtime Protocol

1. Unix socket NDJSON client to Core endpoint.
2. Endpoint identity and descriptor registration for chat presentation.
3. Correlated result senses include `act_instance_id` metadata/body field as defined by protocol contract.
4. Apple Universal submits a configured endpoint name; Core/Spine returns an assigned runtime endpoint id through routing behavior.
5. Multiple app instances may authenticate concurrently, each receiving a distinct Core-assigned runtime endpoint id.

## Moira Host Interface

1. Embedded process-local Moira backend runtime for the first minimum Loom.
2. Host-facing runtime status and resource conflict status.
3. Clotho launch-target/profile context needed by the first Core Control surface.
4. Atropos runtime phase and wake/stop operation surface where available.
5. Lachesis receiver status, wake list, tick list, and selected tick raw records.
6. Swift host boundary starts with `MoiraRuntimeClient`, `MoiraRuntimeSnapshot`, and `MoiraOperationsViewModel`.
7. Rust binding adapters must implement `MoiraRuntimeClient` so SwiftUI views stay independent of ABI mechanics.
8. The first macOS adapter uses dynamic loading for `libmoira_ffi.dylib`, resolves `moira_runtime_status_json`, and decodes returned JSON into Swift DTOs.
9. The macOS build bundles `libmoira_ffi.dylib` and `libduckdb.dylib` under `BelunaApp.app/Contents/Frameworks`.
10. The first dynamic adapter also resolves `moira_runtime_shutdown_json` for explicit runtime cleanup in integration tests.

## Socket Discovery Interface

1. User-configured socket path.
2. Last successful socket path.
3. App-local runtime candidate path selected by Apple Universal.
4. Platform candidate paths supported by deployment docs.
5. Paths reported by embedded Moira runtime after Atropos starts Core.

## User Interface Surface

1. Connection controls (socket path, connect/disconnect, retry).
2. Chat history controls and bounded in-memory buffering.
3. Local persistence for sense/act history.
4. Settings-integrated Moira operations panel.
5. Raw-first local observability browsing for the selected wake/tick.

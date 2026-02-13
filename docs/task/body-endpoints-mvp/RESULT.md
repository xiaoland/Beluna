# RESULT - body-endpoints-mvp

- Date: 2026-02-12
- Status: Completed

## Outcome

Beluna MVP body endpoint path is now available through external endpoint clients over Spine UnixSocket:
1. Shell endpoint (`tool.shell.exec` / `cap.std.shell`) via `std-body`.
2. Web fetch endpoint (`tool.web.fetch` / `cap.std.web.fetch`) via `std-body`.
3. Apple chat endpoint (`chat.reply.emit` / `cap.apple.universal.chat`) via Apple Universal SwiftUI endpoint client.

A dedicated `beluna-runtime` lifecycle command surface (`start`/`stop`/`status`) was added as process glue without coupling core to std-body internals.

## Implemented Work

1. Spine UnixSocket endpoint lifecycle support (approved scope adjustment)
- Added endpoint lifecycle wire/message handling in:
  - `/Users/lanzhijiang/Development/Beluna/core/src/spine/adapters/wire.rs`
  - `/Users/lanzhijiang/Development/Beluna/core/src/spine/adapters/unix_socket.rs`
- Integrated remote endpoint registration/invoke routing in:
  - `/Users/lanzhijiang/Development/Beluna/core/src/spine/adapters/unix_socket_runtime.rs`

2. `std-body` endpoint host and handlers
- Added crate and host runtime:
  - `/Users/lanzhijiang/Development/Beluna/std-body/Cargo.toml`
  - `/Users/lanzhijiang/Development/Beluna/std-body/src/main.rs`
  - `/Users/lanzhijiang/Development/Beluna/std-body/src/host.rs`
- Added endpoint payloads and handlers:
  - `/Users/lanzhijiang/Development/Beluna/std-body/src/payloads.rs`
  - `/Users/lanzhijiang/Development/Beluna/std-body/src/shell.rs`
  - `/Users/lanzhijiang/Development/Beluna/std-body/src/web.rs`
- Used core endpoint models instead of duplicated generic types:
  - `/Users/lanzhijiang/Development/Beluna/std-body/src/wire.rs`

3. `beluna-runtime` lifecycle glue
- Added runtime crate and command surface:
  - `/Users/lanzhijiang/Development/Beluna/runtime/Cargo.toml`
  - `/Users/lanzhijiang/Development/Beluna/runtime/src/main.rs`
  - `/Users/lanzhijiang/Development/Beluna/runtime/src/config.rs`
  - `/Users/lanzhijiang/Development/Beluna/runtime/src/process_supervisor.rs`
  - `/Users/lanzhijiang/Development/Beluna/runtime/src/commands/start.rs`
  - `/Users/lanzhijiang/Development/Beluna/runtime/src/commands/stop.rs`
  - `/Users/lanzhijiang/Development/Beluna/runtime/src/commands/status.rs`
- Added default runtime config template:
  - `/Users/lanzhijiang/Development/Beluna/runtime/beluna-runtime.jsonc`

4. Contract and behavior tests
- `std-body` tests:
  - `/Users/lanzhijiang/Development/Beluna/std-body/tests/shell_endpoint.rs`
  - `/Users/lanzhijiang/Development/Beluna/std-body/tests/web_endpoint.rs`
  - `/Users/lanzhijiang/Development/Beluna/std-body/tests/host_protocol.rs`
  - `/Users/lanzhijiang/Development/Beluna/std-body/tests/apple_protocol_fixture.rs`
- `runtime` lifecycle tests:
  - `/Users/lanzhijiang/Development/Beluna/runtime/tests/lifecycle.rs`

5. Apple Universal app endpoint client
- Added minimal chat-style SwiftUI app package:
  - `/Users/lanzhijiang/Development/Beluna/apple-universal/Package.swift`
  - `/Users/lanzhijiang/Development/Beluna/apple-universal/Sources/BelunaAppleUniversalApp/BelunaAppleUniversalApp.swift`
  - `/Users/lanzhijiang/Development/Beluna/apple-universal/Sources/BelunaAppleUniversalApp/App/ChatView.swift`
  - `/Users/lanzhijiang/Development/Beluna/apple-universal/Sources/BelunaAppleUniversalApp/App/ChatViewModel.swift`
  - `/Users/lanzhijiang/Development/Beluna/apple-universal/Sources/BelunaAppleUniversalApp/Spine/SpineUnixSocketClient.swift`
  - `/Users/lanzhijiang/Development/Beluna/apple-universal/Sources/BelunaAppleUniversalApp/Spine/SpineWire.swift`
- Added protocol tests:
  - `/Users/lanzhijiang/Development/Beluna/apple-universal/Tests/BelunaAppleUniversalTests/SpineWireTests.swift`

## Documentation Updates

- `/Users/lanzhijiang/Development/Beluna/docs/modules/body/README.md`
- `/Users/lanzhijiang/Development/Beluna/docs/modules/README.md`
- `/Users/lanzhijiang/Development/Beluna/docs/modules/spine/README.md`
- `/Users/lanzhijiang/Development/Beluna/docs/overview.md`

## Verification

Executed:

```bash
cd /Users/lanzhijiang/Development/Beluna/core && cargo test
cd /Users/lanzhijiang/Development/Beluna/std-body && cargo test
cd /Users/lanzhijiang/Development/Beluna/runtime && cargo test
cd /Users/lanzhijiang/Development/Beluna/apple-universal && swift test
```

Result:
1. all core unit/BDT/integration tests passed.
2. all std-body tests passed (shell/web/host/apple fixtures).
3. runtime lifecycle tests passed.
4. apple-universal Swift protocol tests passed.

## Notes

1. This task required a scoped deviation from the original "no core changes" constraint to add Spine UnixSocket endpoint lifecycle messages. That scope change was explicitly approved during execution.
2. Apple Universal App remains an external endpoint client and is implemented outside `std-body`.
3. Follow-up refactor: core is now library-only (no `main`), and core config loading/validation is consolidated in `beluna-runtime` using core schema.

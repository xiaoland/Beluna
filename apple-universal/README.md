# Apple Universal App

Minimal SwiftUI chat-style Body Endpoint for Beluna Spine UnixSocket.

## What it does

1. Connects to Spine Unix socket (`/tmp/beluna.sock` by default).
2. Lets user edit socket path at runtime and persists it (UserDefaults).
3. Lets user connect/disconnect without quitting the app (connection intent is persisted).
4. Uses POSIX Unix socket I/O to avoid `Network.framework` AF_UNIX diagnostics.
5. Registers body endpoint route:
- `macos-app.01` / `present.message`
6. Sends user messages as `sense` payloads aligned with the OpenAI Responses subset.
7. Receives `act` and renders assistant chat bubbles.
8. Sends invoke outcome back as correlated `sense` (echoes `neural_signal_id` and route markers).

## Run

```bash
cd /Users/lanzhijiang/Development/Beluna/apple-universal
swift run BelunaAppleUniversalApp
```

## Test

```bash
cd /Users/lanzhijiang/Development/Beluna/apple-universal
swift test
```

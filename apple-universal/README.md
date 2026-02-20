# Apple Universal App

Minimal SwiftUI chat-style Body Endpoint for Beluna Spine UnixSocket.

## What it does

1. Connects to Spine Unix socket (`/tmp/beluna.sock` by default).
2. Moves connection configuration to `SettingView` (socket path + connect controls).
3. Persists connection intent and socket path in UserDefaults.
4. Uses POSIX Unix socket I/O to avoid `Network.framework` AF_UNIX diagnostics.
5. Auto-reconnects with exponential backoff (up to 5 retries), with manual retry support.
6. Enforces single-instance runtime lock to avoid duplicate app instances.
7. In Xcode debug sessions, defaults to manual connect to avoid accidental side effects.
8. Registers body endpoint route:
   - `macos-app.01` / `present.message`
9. Sends user messages as `sense` payloads aligned with the OpenAI Responses subset.
10. Receives `act` and renders assistant chat bubbles.
11. Sends invoke outcome back as correlated `sense` (echoes `neural_signal_id` and route markers).

## Run

## Test

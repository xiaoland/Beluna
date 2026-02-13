# Apple Universal App

Minimal SwiftUI chat-style Body Endpoint for Beluna Spine UnixSocket.

## What it does

1. Connects to Spine Unix socket (`/tmp/beluna.sock` by default).
2. Registers body endpoint route:
- `macos-app.01` / `present.message`
3. Sends user messages as `sense` payloads aligned with the OpenAI Responses subset.
4. Receives `act` and renders assistant chat bubbles.
5. Sends invoke outcome back as correlated `sense` (echoes `neural_signal_id` and route markers).

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

# Apple Universal App

Minimal SwiftUI chat-style Body Endpoint for Beluna Spine UnixSocket.

## What it does

1. Connects to Spine Unix socket (`/tmp/beluna.sock` by default).
2. Registers body endpoint route:
- `chat.reply.emit` / `cap.apple.universal.chat`
3. Sends user messages as `sense` payloads aligned with the OpenAI Responses subset.
4. Receives `body_endpoint_invoke` and renders assistant chat bubbles.
5. Sends `body_endpoint_result` for every invoke.

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

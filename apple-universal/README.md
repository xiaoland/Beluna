# Apple Universal App

SwiftUI-based Beluna Body Endpoint for Apple platforms (currently focused on macOS).

## What it does

1. Connects to Beluna Core via Unix socket (`/tmp/beluna.sock` by default).
2. Uses POSIX socket I/O with reconnect/backoff and manual retry support.
3. Persists connection settings (socket path, auto-connect), observability settings, and message capacity in `UserDefaults`.
4. Uses endpoint IDs aligned with Apple body endpoint identity:
- `apple.universal` (family)
- `macos.app` (macOS runtime)
- `ios.app` (iOS runtime)
5. Registers NDJSON auth `ns_descriptors` (`method=auth`) with semantic IDs:
- act: `present.message.text`
- senses: `user.message.text`, `present.message.text.success`, `present.message.text.failure`
6. Uses text payload senses:
- `payload: String`
- `weight: Double`
- optional `act_instance_id`.
7. Sends user text as plain string payload for `user.message.text`.
8. Reports correlated success/failure senses with `act_instance_id` body field (not metadata object).
9. Receives acts, sends `act_ack`, renders assistant messages, then reports success/rejection senses.
10. Persists local Sense/Act history to disk and restores it after app restart.
11. Exposes "Clear Local History" in `SettingView`.
12. Keeps a bounded in-memory ring buffer with incremental pagination for visible messages.
13. Polls Core Prometheus metrics (5s when connected + manual refresh).
14. Polls Core logs (3s), pairs `cortex_organ_input` and `cortex_organ_output`, and renders cycle cards.
15. Supports configurable metrics endpoint and log directory.

## Run

1. Open the Xcode project:
- `open /Users/lanzhijiang/Development/Beluna/apple-universal/BelunaApp.xcodeproj`
2. Select scheme `BelunaApp`, target macOS, then Run.
3. In app settings, confirm the Unix socket path (default `/tmp/beluna.sock`) matches Beluna Core.

## Build

```bash
xcodebuild \
  -project /Users/lanzhijiang/Development/Beluna/apple-universal/BelunaApp.xcodeproj \
  -scheme BelunaApp \
  -configuration Debug \
  -destination 'platform=macOS' \
  build
```

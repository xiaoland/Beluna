# Issue 30 Moira Runtime Embedding

Task packet for <https://github.com/xiaoland/Beluna/issues/30>.

This packet captures the proposed reshaping of Moira from a Tauri desktop app into an embeddable backend/runtime unit.

The execution scope for this issue is intentionally narrow: implement the minimum Moira Loom surface inside `apple-universal` only. Broader Human Interface host coverage, including `cli` and future `win-native`, remains design context and follow-on work.

## Files

- [PLAN.md](./PLAN.md): MVT anchors, scope, governing anchors, and open decisions.
- [BOUNDARY.md](./BOUNDARY.md): target unit/container boundary and role changes.
- [SEQUENCE.md](./SEQUENCE.md): proposed implementation slices and verification gates.
- [PACKAGING.md](./PACKAGING.md): internal package and embedding design questions.
- [SINGLETON.md](./SINGLETON.md): future single local Moira authority problem and current task's smaller runtime model.
- [APPLE-UNIVERSAL-LOOM.md](./APPLE-UNIVERSAL-LOOM.md): minimum Apple Universal Loom scope and UI integration questions.
- [APPLE-UNIVERSAL-UI-INTEGRATION.md](./APPLE-UNIVERSAL-UI-INTEGRATION.md): Apple Universal navigation, body endpoint, and Moira Loom integration design notes.
- [APPLE-UNIVERSAL-CLEANUP.md](./APPLE-UNIVERSAL-CLEANUP.md): Apple Universal source cleanup scope before Moira UI integration.
- [OPEN-QUESTIONS.md](./OPEN-QUESTIONS.md): unresolved technical and product decisions.

## Current Packet Status

Mode: Slice 1 Apple Universal source cleanup implemented locally.

Latest local verification:

- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS' -only-testing:BelunaAppTests`
- Result: passed on 2026-05-07.
- Real app smoke test with Computer Use: app launched from the Xcode build product, main window rendered, Settings opened through the toolbar gear button, and Connection / Chat / Status / Moira sections rendered.

Full scheme note:

- `xcodebuild test -project apple-universal/BelunaApp.xcodeproj -scheme BelunaApp -destination 'platform=macOS'`
- Result: passed on 2026-05-07 after removing Apple Universal process singleton guarding.
- Follow-up decision: Apple Universal process singleton guarding was removed. Core/Spine-assigned runtime endpoint ids now carry the multi-instance disambiguation responsibility.

This packet is tactical. Durable truths should be promoted into Product TDD and affected Unit TDD docs after human confirmation.

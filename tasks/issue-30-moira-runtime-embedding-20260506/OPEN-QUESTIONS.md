# Open Questions

## Task Scope

1. Should this issue include wake/stop controls in Apple Universal, or stop at status plus observability browsing?
2. Should this issue include Clotho write operations, or read-only launch/profile context?
3. Should Tauri/Vue deletion happen in this issue after Apple minimum coverage, or become a follow-on issue?

## Packaging

1. Which binding strategy should Apple Universal use first: UniFFI, C ABI, Swift Package wrapper, or another local mechanism?
2. Should Moira runtime be extracted into a new crate path before any Apple UI work?
3. Should Apple Universal build Moira from source during Xcode builds, or consume a locally built artifact?
4. How should debug and release builds locate DuckDB and other native dependencies?

## Runtime API

1. What is the minimum stable `MoiraRuntime` API for Apple Universal?
2. Should host APIs be organized by mythic owner or by operation category?
3. How should live Lachesis pulses cross the Rust/Swift boundary?
4. How should cancellation and shutdown be modeled when the Apple app exits?

## Runtime Multiplicity

1. Which resource conflicts should the Apple minimum UI surface: OTLP receiver bind, DuckDB write access, Atropos process ownership, or all of them?
2. Which Core socket candidates should the body endpoint UI discover by default?
3. Should socket discovery include configured path, recent successful path, `/var/run/beluna.sock`-style platform candidates, and app-local runtime candidates?
4. Which issue should own future Owner/Attach authority coordination?

## Product/UI

1. Where should the minimum Loom surface live in Apple Universal navigation?
2. Which Moira data should be visible in the first viewport?
3. What is the first acceptable raw event inspection interaction on macOS?
4. What is the iOS/iPadOS story for the same minimum surface?

## Follow-On Split

1. Which issue should own CLI hosting?
2. Which issue should own Windows hosting?
3. Which issue should own full Apple-native Loom UX?
4. Which issue should own sandbox platform adapters?
5. Which issue should own ledger platform adapters?

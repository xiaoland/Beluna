# Issue 23 Clotho Follow-On Working Set

## Context

Moira Clotho already supports known local build registration plus app-local JSONC profile documents.
This working set covers the next `#23` slice:

- explicit forge from a local Beluna source folder
- published artifact discovery, checksum verification, and install isolation
- one Clotho-owned launch-target boundary consumed by Atropos

It intentionally does **not** absorb all of `#8`.
`#8` remains the producer-side release workflow issue; this task only locks the minimum contract that Moira consumes.

## Intended Change

1. Clotho can forge a reusable local launch target from a Beluna repo root or `core/` crate root.
2. Clotho can discover GitHub Releases, verify `SHA256SUMS`, install a supported artifact into an isolated local directory, and expose it as a launch target.
3. Atropos still wakes Core only from Clotho-prepared wake input.
4. Schema validation remains deferred; this task only preserves the extension seam and documentation anchors.

## Working Notes

- Docs-first execution order:
  - task note
  - `docs/20-product-tdd`
  - `docs/40-deployment`
  - `docs/30-unit-tdd/moira`
  - code/tests
  - issue sync
- Current minimum producer contract for `#8`:
  - archive asset: `beluna-core-<rust-target-triple>.tar.gz`
  - checksum file: `SHA256SUMS`
  - archive may contain executable `beluna`
  - current consumer lock: `aarch64-apple-darwin`
- Local forge is an explicit operator action, not an implicit wake-time compile.
- Clotho durable preparation truth stays app-local; selected launch target and selected profile remain session-local Loom query state.

## Verification

- Authoritative docs across `20/40/30` agree on the same release contract and operational flow.
- Rust tests cover source-root normalization, forge manifest update, installed artifact resolution, checksum mismatch, broken archive, and install-isolation paths.
- Frontend verification covers launch-target selection plus forge/install dialogs.
- Atropos still wakes through prepared wake input only; Lachesis remains isolated from Clotho/Atropos state.
- Real operator evidence now exists against a published GitHub release (`v0.0.9`):
  - Clotho discovered the release
  - verified `SHA256SUMS`
  - installed the artifact into app-local isolation
  - selected the installed launch target
  - woke Core successfully

## Promotion

- Cross-unit release contract truth belongs in `docs/20-product-tdd`.
- Runtime rollout and recovery truth belongs in `docs/40-deployment`.
- Moira-local preparation, UI, and verification truth belongs in `docs/30-unit-tdd/moira`.

## Execution Evidence

- The release-intake consumer path has now been exercised against a real published release:
  - release tag: `v0.0.9`
  - archive asset: `beluna-core-aarch64-apple-darwin.tar.gz`
  - checksum asset: `SHA256SUMS`
- This closes the previous gap between fixture-backed install tests and a true live operator walkthrough.

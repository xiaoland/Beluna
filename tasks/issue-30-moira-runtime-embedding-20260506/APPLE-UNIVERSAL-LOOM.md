# Apple Universal Minimum Loom

This file captures product and UI design questions for the minimum Moira Loom surface inside Apple Universal.

## Product Intent

Apple Universal becomes the first Beluna Human Interface host that embeds Moira backend and exposes local operator workflows inside the existing app experience.

The goal is a minimum native Loom shaped around Apple Universal.

## Minimum Operator Jobs

1. Know whether Moira backend is ready.
2. Know whether Lachesis receiver is listening or faulted.
3. See available wake/run history.
4. See ticks for one wake.
5. Inspect one tick through raw-first records.
6. See enough launch target/profile context to understand what Core would wake.

## Candidate App Integration

Decision for this task: use a Settings-integrated operations panel.

Follow-on candidates:

1. Top-level tab
- Add a dedicated Loom tab beside chat/settings.
- Best when operator workflows become frequent and deep.

2. Inspector panel
- Keep chat as the main experience and expose Loom as an inspector/sidebar.
- Best when Loom is mostly diagnostic.

3. Settings-integrated operations view
- Put Moira runtime status and launch/profile controls near connection settings.
- Best for the first minimal slice.

4. Separate window on macOS
- Allow a richer operator workspace in a dedicated macOS surface.
- Later fit for desktop, with iPad/iOS alternatives still needed.

## Minimum UI Surface

Sections:

- Runtime
  - Moira ready/faulted status.
  - Atropos phase if available.
  - Core pid and terminal reason if available.

- Receiver
  - endpoint
  - state
  - raw event count
  - wake count
  - tick count
  - last batch time or error

- Wakes
  - list of wake/run summaries
  - selected wake

- Ticks
  - list of ticks for selected wake
  - selected tick

- Raw Tick Detail
  - event rows/cards
  - scope name
  - event name
  - severity
  - trace/span ids
  - body/attributes/resource drilldown

## UI Questions

1. Should Moira controls share visual hierarchy with endpoint connection controls?
2. How much Clotho functionality is needed for the minimum Apple Loom: read-only launch/profile context, explicit wake preparation, or full artifact/profile management?
3. Should wake/stop controls land in this issue, or should this issue stop at read/query surfaces?
4. Should raw tick inspection use disclosure groups, a split view, or a table plus detail drawer?
5. What is the smallest mobile/iPad adaptation that keeps the surface coherent?

## Working Product Bias

- Start with read/query surfaces before broad preparation UX.
- Keep the first Apple Loom diagnostic and operationally clear.
- Use SwiftUI-native navigation and controls.
- Avoid carrying over current Vue layout assumptions.
- Preserve raw-first inspection as the reliable minimum.

## Verification Ideas

- View-model tests for status loading, run selection, tick selection, and error state.
- Binding DTO decoding tests.
- Manual macOS smoke path:
  1. open Apple Universal
  2. open Loom surface
  3. confirm receiver status
  4. select a wake
  5. select a tick
  6. inspect raw event detail

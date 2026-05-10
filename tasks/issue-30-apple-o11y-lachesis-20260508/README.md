# Issue 30 Follow-On: Apple O11y / Lachesis

Follow-on packet after issue #30 minimum Apple Loom and Tauri/Vue retirement.

This packet owns Apple-native observability and investigation surfaces for Moira-owned Lachesis data.

UI integration target: an Apple O11y / Lachesis panel parallel to Core Control and Settings. Settings remains the home for Moira configuration.

## Files

- [PLAN.md](./PLAN.md): MVT anchors, first-scope boundaries, and verification.

## Current Packet Status

Mode: Execute.

First slice landed:

- Add standalone Apple `O11y / Lachesis` panel parallel to Core Control and Settings.
- Use existing `MoiraLoomSnapshot` as the first binding for wake/tick/raw detail browsing.
- Provide selected tick raw-first inspection with a native raw event inspector.
- Add a raw-derived Tick Gantt view parallel to Raw view, including lifecycle interval blocks and selected item detail.
- Keep Cortex timeline, narrative investigation, owner drilldown, and event/pulse refresh as separate follow-on slices.

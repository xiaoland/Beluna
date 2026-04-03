# Moira Operations

## Startup

1. Load local Clotho preparation state and Atropos supervision state.
2. Initialize the local OTLP logs receiver and storage backend.
3. Expose Loom UI only after local control-plane state is ready.

## Wake Flow

1. Loom query state provides the current selected known local build ref plus the optional selected profile id for the next wake.
2. Clotho resolves the selected build manifest and derives the optional profile path from `profile_id`.
3. Atropos ensures the OTLP logs receiver is ready before starting supervised Core.
4. Atropos launches Core and records supervised wake tracking.
5. Loom refreshes runtime status through Atropos query orchestration while Lachesis reacts to ingest updates for wake and tick browsing.
6. Profile schema validation, local source-folder compile, and published artifact activation remain later wake-flow extensions rather than current required steps.

## Observability Flow

1. Receive Core OTLP logs locally.
2. Persist raw events before updating any derived read model.
3. Project the baseline read models needed for Loom wake list and tick timeline.
4. Project any additional chronology, interval-pairing, or targeted lookup indexes that materially improve operator-facing browsing without becoming alternate sources of truth.
5. Resolve the selected tick through tick chronology first, then expanded interval inspection, then sectional Cortex / Stem / Spine investigation, and finally source-grounded raw-event inspection.
6. Surface metrics/traces exporter status and handoff links without taking ownership of those signals.

## Shutdown

1. When operator quits Moira, the app exit hook initiates supervised Core graceful stop.
2. Offer explicit force-kill only through a second confirmation path.
3. Flush local observability state and close control-plane resources.

## Failure Handling

1. Checksum mismatch blocks published artifact activation.
2. Local source build failure blocks wake and surfaces explicit failure state.
3. OTLP receiver/storage readiness failure blocks supervised wake.
4. Unexpected Core exit becomes explicit terminal supervision state visible in Loom.
5. Missing selected build input, or an explicitly selected but unresolved profile input, blocks wake rather than letting Atropos invent a fallback path.

## Current Extension Boundary

1. Extend explicit owners rather than reviving catch-all helpers such as one shared control query module or one stacked control-plane page.
2. New preparation flows land under `clotho` backend ownership plus `query/clotho` and `presentation/clotho` frontend owners.
3. New supervision flows land under `atropos` backend ownership plus `query/atropos` and `presentation/atropos` frontend owners.
4. Shared shell affordances such as feature tabs and dialog scaffolding belong in `query/loom` and `presentation/loom/chrome`, but feature-specific semantics remain inside the corresponding mythic namespace.
5. Future persistence must choose an explicit owner before choosing filesystem, database, or app-state storage shape.

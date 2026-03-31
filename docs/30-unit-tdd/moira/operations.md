# Moira Operations

## Startup

1. Load local Clotho preparation state and Atropos supervision state.
2. Initialize the local OTLP logs receiver and storage backend.
3. Expose Loom UI only after local control-plane state is ready.

## Wake Flow

1. Resolve the selected Core artifact or local source build target.
2. Validate the selected JSONC profile against the Core schema authority.
3. Ensure the OTLP logs receiver is ready before starting supervised Core.
4. Launch Core and begin supervised wake tracking.

## Observability Flow

1. Receive Core OTLP logs locally.
2. Persist raw events before updating any derived read model.
3. Project the baseline read models needed for Loom wake list and tick timeline.
4. Project any additional chronology, interval-pairing, or targeted lookup indexes that materially improve operator-facing browsing without becoming alternate sources of truth.
5. Resolve the selected tick through tick chronology first, then expanded interval inspection, then sectional Cortex / Stem / Spine investigation, and finally source-grounded raw-event inspection.
6. Surface metrics/traces exporter status and handoff links without taking ownership of those signals.

## Shutdown

1. When operator quits Moira, initiate supervised Core stop.
2. Offer explicit force-kill only through a second confirmation path.
3. Flush local observability state and close control-plane resources.

## Failure Handling

1. Checksum mismatch blocks published artifact activation.
2. Local source build failure blocks wake and surfaces explicit failure state.
3. OTLP receiver/storage readiness failure blocks supervised wake.
4. Unexpected Core exit becomes explicit terminal supervision state visible in Loom.

## Cleanup Stage Before Next Feature Wave

1. Preserve the current Lachesis and Loom operator behavior while restructuring internal boundaries.
2. The backend extraction lands first: explicit `app`, `clotho`, `lachesis`, and `atropos` modules with thin Tauri command handlers.
3. The first frontend cleanup slice extracts `bridge` and `query state` ownership next while preserving the existing Loom surface.
4. The second frontend cleanup slice then makes `projection` and `presentation` concrete through explicit `projection/lachesis/*` and `presentation/*` owners while preserving the existing Loom surface.
5. The cleanup integration pass then removes transitional glue by separating backend-shaped bridge contracts from normalized Loom-facing projection models, and by keeping query-owned UI state out of any revived catch-all type bucket.
6. Introduce an explicit migration/version path for future Clotho and Atropos persistence rather than extending the current Lachesis-only store ad hoc.
7. Use the first post-cleanup cross-slice feature to validate the new split: local source-folder build plus wake and graceful stop.
8. Cleanup-stage non-goals:
- no full GitHub Releases artifact-management UX
- no broad new supervision UI
- no first-party endpoint-app launch flow
- no operator-facing feature expansion that obscures whether the refactor preserved current Lachesis behavior

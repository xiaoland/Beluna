# Moira Operations

## Startup

1. Load local artifact/profile/supervision state.
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
4. Project any additional tick-trace or thread indexes that materially improve operator-facing browsing without becoming alternate sources of truth.
5. Resolve the selected tick through lane-based chronology, conversation browser, Cortex / Stem / Spine drilldown, and raw-event fallback.
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

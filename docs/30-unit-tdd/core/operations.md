# Core Operations

## Startup

1. Load and validate config at boundary.
2. Initialize tracing and observability exporters.
3. Build runtime subsystems and spawn runtime tasks.

## Shutdown

1. Receive termination signal.
2. Close afferent ingress gate.
3. Cancel runtime tasks.
4. Drain efferent queue with bounded timeout.
5. Flush persistence and shutdown adapters.

## Observability Contract

1. JSON runtime logs are emitted locally.
2. OTLP export handles logs/metrics/traces per signal configuration.
3. Core owns runtime observability export responsibility.

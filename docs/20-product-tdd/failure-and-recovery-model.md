# Failure And Recovery Model

## Failure Classes

1. Protocol/integration failures
- Endpoint auth/register/message incompatibility.
- Transport disconnect/reconnect instability.

2. Runtime execution failures
- Dispatch non-ack outcomes (`Rejected`, `Lost`).
- Tick/cognition runtime task cancellation or timeout scenarios.

3. State continuity failures
- Cognition persistence/restore failure.
- Inconsistent continuity state after restart.

4. Operational observability failures
- Missing or degraded logs/metrics/traces.

## Recovery Expectations

1. Startup validates configuration boundary before full runtime admission.
2. Shutdown closes ingress, cancels tasks, and drains efferent path with bounded timeout.
3. Recovery preserves continuity expectations where product claims require it.
4. Failure signals remain diagnosable through explicit outcome classes and observability surfaces.

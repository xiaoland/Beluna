# User-Observable Workflows

These workflows describe observable product behavior for current primary users.

## Operator Runtime Session

1. Operator starts Beluna with explicit configuration.
2. Runtime announces readiness through observable startup signals.
3. Operator can submit interaction input and receive explicit action outcomes.
4. On shutdown or fault, operator receives clear lifecycle status and recovery path.

## Endpoint Integrator Workflow

1. Integrator connects an endpoint surface using the documented protocol contract.
2. Endpoint exchanges interaction data and receives explicit action/result outcomes.
3. Integrator can diagnose integration failures using runtime telemetry and terminal outcome semantics.

## Incident Handling Workflow

1. Runtime anomaly is detected through logs/metrics/traces.
2. Operator determines failure class and applies bounded recovery procedure.
3. Service resumes with preserved continuity where expected by product claim.

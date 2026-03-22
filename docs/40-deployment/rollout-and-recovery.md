# Rollout And Recovery

## Rollout

1. Generate/update schema from typed config when config model changes.
2. Deploy Core with validated config.
3. Verify endpoint registration and runtime startup telemetry.

## Recovery

1. Use process signals for graceful shutdown.
2. Ensure ingress closure and bounded efferent drain.
3. Restart with corrected config/runtime dependencies.
4. Use persisted cognition state and logs for incident analysis.

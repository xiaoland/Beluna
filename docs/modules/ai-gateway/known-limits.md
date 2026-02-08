# Known Limits

- Gateway is currently a module boundary; it is not yet exposed through the Unix socket runtime protocol.
- Copilot adapter behavior is conservative and may need adjustments across SDK/server versions.
- No live provider-network CI tests; current coverage is mock/in-process based.
- No multi-backend fallback routing in MVP.
- No pricing/cost estimation in MVP (usage-only telemetry scope).

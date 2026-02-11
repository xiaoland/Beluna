# Admission LLD

Determinism rules:
- no wall clock
- no randomness
- stable sort by `attempt_id`
- deterministic degradation ranking with tie-breaks and search caps

Outcome set:
- `Admitted { degraded }`
- `DeniedHard { code }`
- `DeniedEconomic { code }`

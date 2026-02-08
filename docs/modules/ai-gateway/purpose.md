# Purpose

AI Gateway is Beluna's provider/model-agnostic inference boundary.

It exists to:

- provide one internal inference API independent of backend dialect,
- normalize request and streaming response shapes into canonical types,
- enforce cross-cutting backend concerns (capabilities, reliability, budget, credentials, telemetry),
- isolate transport/dialect complexity inside adapters.

MVP supported dialects:

- `openai_compatible`
- `ollama`
- `github_copilot_sdk`

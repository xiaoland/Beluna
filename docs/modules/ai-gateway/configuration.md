# Configuration

Gateway config is nested under top-level `ai_gateway`.

## Required Fields

- `backends[]`

Each backend profile includes:

- `id`
- `dialect`
- `credential`
- `models[]`
- optional `endpoint`
- optional `capabilities`
- optional `copilot` (for `github_copilot_sdk`)

Capability fields are backend-specific toggles (for example: `tool_calls`, `parallel_tool_calls`, `json_mode`, `json_schema_mode`, `vision`, `resumable_streaming`).

Each model exposes route aliases through `models[].aliases`:

- `alias -> backend_id/model_id`

Convention:

- alias `default` must exist on at least one model.
- callers may also route directly with `<backend-id>/<model-id>`.

## Credential Shapes

- `{ "type": "env", "var": "..." }`
- `{ "type": "inline_token", "token": "..." }`
- `{ "type": "none" }`

## Policy Sections

- `chat`
- `resilience`

Schema source:

- `beluna.schema.json`

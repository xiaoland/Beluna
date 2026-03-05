# Configuration

Gateway config is nested under top-level `ai_gateway`.

## Required Fields

- `backends[]`
- `route_aliases`

Each backend profile includes:

- `id`
- `dialect`
- `credential`
- `models[]`
- optional `endpoint`
- optional `capabilities`
- optional `copilot` (for `github_copilot_sdk`)

Capability fields are backend-specific toggles (for example: `tool_calls`, `parallel_tool_calls`, `json_mode`, `json_schema_mode`, `vision`, `resumable_streaming`).

`route_aliases` maps an alias name to one concrete target:

- `alias -> { backend_id, model_id }`

Convention:

- alias `default` must exist.

## Credential Shapes

- `{ "type": "env", "var": "..." }`
- `{ "type": "inline_token", "token": "..." }`
- `{ "type": "none" }`

## Policy Sections

- `reliability`
- `budget`

Schema source:

- `beluna.schema.json`

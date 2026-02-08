# Configuration

Gateway config is nested under top-level `ai_gateway`.

## Required Fields

- `default_backend`
- `backends[]`

Each backend profile includes:

- `id`
- `dialect`
- `credential`
- `default_model`
- optional `endpoint`
- optional `capabilities`
- optional `copilot` (for `github_copilot_sdk`)

## Credential Shapes

- `{ "type": "env", "var": "..." }`
- `{ "type": "inline_token", "token": "..." }`
- `{ "type": "none" }`

## Policy Sections

- `reliability`
- `budget`

Schema source:

- `beluna.schema.json`

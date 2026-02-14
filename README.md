# Beluna

Beluna is an agent.

Read product documents in `docs/*`.

## Monorepo

- Core: `./core`
- CLI: `./cli`
- Apple Universal App: `./apple-universal`

## Run Core

```bash
cd ./core
cargo run -- --config ../beluna.jsonc
```

## Run CLI Body Endpoint

`cli` is a standalone Rust app. It does not boot core; it connects to core's Unix socket (NDJSON), registers capability `present.plain_text`, and emits `user_message` senses from stdin.

```bash
cd ./cli
cargo run -- --socket-path ../beluna.sock
```

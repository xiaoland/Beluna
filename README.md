# Beluna 蓝汐

Beluna is a life.

Read documents in `docs/*`.

## Monorepo

- Beluna Core: `./core`
- Beluna CLI: `./cli`
- Apple Universal App: `./apple-universal`

## Run Beluna

```bash
cd ./core
cargo run -- --config ../beluna.jsonc
```

## Run CLI Body Endpoint

`beluna-cli` is a standalone Rust app. It does not boot core; it connects to core's Unix socket (NDJSON), registers capability `present.plain_text`, and emits `user_message` senses from stdin.

```bash
cd ./cli
cargo run -- --socket-path ../beluna.sock
```

## Run Apple Universal Body Endpoint

Use XCode.

## AI Providers

- 阿里云百练 <https://help.aliyun.com/zh/model-studio/models>

# Beluna 蓝汐

Beluna is a life.

Read documents in `docs/*`.

## Usage

### Run Beluna

```bash
cd ./core
cargo run -- --config ../beluna.jsonc
```

## Start Body Endpoints you want Beluna has

We have built-in std bodies that bundled with Beluna core:

- `web`
- `shell`

And also following bodies in the monorepo:

- `cli`: a standalone Rust app. Registers NS `act/present.plain_text`, `sense/user_message` from stdin.

    ```bash
    cd ./cli
    cargo run -- --socket-path ../beluna.sock
    ```

- `apple-universal`: a SwiftUI Apple Universal app.

## Inspect

The core supports OTLP Metrics, Logs, and Traces. You need an OTLP collector/backend + UI to inspect Beluna activity.

I recommend you use the <https://github.com/jaegertracing/jaeger/> which is a single binary jaeger:

```bash
mkdir .o11y && cd .o11y
wget https://github.com/jaegertracing/jaeger/releases/download/v2.16.0/jaeger-2.16.0-darwin-arm64.tar.gz  # MacOS Apple Silicon, pick yours on release page
tar -xzvf jaeger-2.16.0-darwin-arm64.tar.gz && cd jaeger-2.16.0-darwin-arm64
./jaeger  # allows it in Settings > Privacy and Security
```

And then you will have:

- `http://localhost:4318`: OTLP over HTTP receiver
- `http://localhost:16686`: Jaeger UI

OTLP config is signal-scoped:

- `observability.otlp.defaults.timeout_ms`
- `observability.otlp.signals.<metrics|logs|traces>.{enabled,protocol,endpoint,...}`

When a signal is enabled, `endpoint` must be set explicitly. `protocol` supports `http` and `grpc` (default `grpc`).

Quickwit local setup tip: use gRPC endpoint `http://127.0.0.1:7281` with `protocol: "grpc"` for enabled signals.

## AI Providers

- 阿里云百练 <https://help.aliyun.com/zh/model-studio/models>

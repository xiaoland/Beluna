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

## AI Providers

- 阿里云百练 <https://help.aliyun.com/zh/model-studio/models>

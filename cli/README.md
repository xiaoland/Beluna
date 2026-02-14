# Beluna CLI

`beluna-cli` is a standalone Beluna body endpoint client.

- Connects to a running Beluna Core over Unix socket NDJSON.
- Registers capability `present.plain_text`.
- Sends stdin lines as `user_message` senses.
- Prints plain-text acts from `normalized_payload.text`.

## Run

```bash
cargo run -- --socket-path /path/to/beluna.sock
```

Optional:

```bash
cargo run -- --socket-path /path/to/beluna.sock --endpoint-id body.cli.local
```

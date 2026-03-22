# CLI Design

## Responsibility

1. Connect to Core over Unix socket NDJSON.
2. Register endpoint identity and capability.
3. Convert stdin text into outbound sense messages.
4. Print plain-text act outputs.

## Non-Responsibility

1. No Core domain logic replication.
2. No runtime observability ownership.

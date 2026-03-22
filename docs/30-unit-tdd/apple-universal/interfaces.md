# Apple Universal Interfaces

## Runtime Protocol

1. Unix socket NDJSON client to Core endpoint.
2. Endpoint identity and descriptor registration for chat presentation.
3. Correlated result senses include `act_instance_id` metadata/body field as defined by protocol contract.

## User Interface Surface

1. Connection controls (socket path, connect/disconnect, retry).
2. Chat history controls and bounded in-memory buffering.
3. Local persistence for sense/act history.
